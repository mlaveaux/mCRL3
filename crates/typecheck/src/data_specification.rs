use std::convert::Infallible;
use std::rc::Rc;

use log::debug;

use merc_collections::IndexedSet;
use merc_data::Mcrl2DataSpecification;
use merc_syntax::ConstructorId;
use merc_syntax::DefId;
use merc_syntax::EqnSpecId;
use merc_syntax::EqnVarId;
use merc_syntax::MapId;
use merc_syntax::SortExpression;
use merc_syntax::SortExpressionKind;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;

use crate::AliasError;
use crate::EquationTyping;
use crate::NumberEncoding;
use crate::Signature;
use crate::TypeckContext;
use crate::WellTypedError;
use crate::apply_sorts_in_spec;
use crate::assign_declaration_ids;
use crate::basic_sort_data_specification;
use crate::build_signature;
use crate::build_system_defined_specification;
use crate::check_aliases;
use crate::check_equations;
use crate::check_no_system_function_redeclaration;
use crate::check_system_specification;
use crate::desugar_structured_sorts;
use crate::hoist_anonymous_structs;
use crate::is_well_typed;
use crate::lower_data_expressions;
use crate::lower_data_specification;
use crate::normalize_sorts;
use crate::resolve_sort_ids;
use crate::resolve_system_signature;
use crate::structured_sort_equations;

/// A type checked and well-typed data specification.
///
/// Holds the resolved user declarations (see [`Self::data_specification`] for
/// exactly which sorts are resolved), the sort-name → [`DefId`] map assigned
/// during name resolution, and the system-defined (Appendix-B) declarations for
/// the sorts that occur (see [`Self::system_defined_specification`]).
pub struct DataSpecification {
    spec: UntypedDataSpecification,
    sorts: IndexedSet<String>,
    system: UntypedDataSpecification,
    context: TypeckContext,
    equation_typings: Vec<Vec<Rc<EquationTyping>>>,
    encoding: NumberEncoding,
}

impl DataSpecification {
    /// Create a completed well-typed data specification from an untyped data
    /// specification, using the default number encoding.
    pub fn from_untyped(spec: UntypedDataSpecification) -> Result<Self, WellTypedError> {
        Self::from_untyped_with(spec, NumberEncoding::default())
    }

    /// Create a completed well-typed data specification from an untyped data
    /// specification.
    ///
    /// For the encoding see [`NumberEncoding`].
    pub fn from_untyped_with(
        mut spec: UntypedDataSpecification,
        encoding: NumberEncoding,
    ) -> Result<Self, WellTypedError> {
        debug!(
            "typecheck: starting on {} sort, {} constructor, {} map and {} equation declaration(s)",
            spec.sort_declarations.len(),
            spec.constructor_declarations.len(),
            spec.map_declarations.len(),
            spec.equation_declarations.len()
        );

        // Hoist anonymous structured sorts into fresh named declarations, so
        // name resolution, the alias checks and the desugaring below only ever
        // see named structs.
        hoist_anonymous_structs(&mut spec);
        debug!(
            "typecheck: hoisted anonymous structs; {} sort declaration(s) remain",
            spec.sort_declarations.len()
        );

        apply_sorts_in_spec(&mut spec, |sort| -> Result<_, Infallible> {
            Ok(flatten_function_sorts(sort))
        })
        .expect("The inner function never fails");

        let sorts = resolve_sort_ids(&mut spec)?;
        debug!("typecheck: resolved {} sort name(s)", sorts.len());

        check_aliases(&spec).map_err(|err| {
            let name = |id: &DefId| sorts.get_by_index(**id).expect("The sort should be declared").clone();
            match err {
                AliasError::Circular { cycle } => WellTypedError::AliasCycle {
                    sorts: cycle.iter().map(name).collect(),
                },
                AliasError::ThroughFunctionSort { sort } => {
                    WellTypedError::RecursiveAliasThroughFunctionSort { sort: name(&sort) }
                }
            }
        })?;

        // Desugar structured sorts into abstract sorts plus their constructors,
        // recognisers and projections. Alias checks still need to see the
        // structured sorts.
        let structs = desugar_structured_sorts(&mut spec);
        debug!(
            "typecheck: desugared {} structured sort(s) into {} constructor(s)",
            structs.len(),
            structs.iter().map(Vec::len).sum::<usize>()
        );

        // Assign ids to desugared declarations.
        assign_declaration_ids(&mut spec);

        // Compute the (S, C, M) signature and run the signature-layer checks of
        // definition 15.1.7. This runs before alias expansion so the errors refer to sorts
        // as the user wrote them.
        let mut context = TypeckContext::new();
        build_signature(&mut context, &spec)?;
        debug!("typecheck: signature checks passed");

        // Expand aliases to a canonical form now that they are known to be
        // acyclic.
        normalize_sorts(&mut spec);
        debug!("typecheck: normalized alias indirection");

        // Safety net over the normalized spec:.
        is_well_typed(&spec)?;
        debug!("typecheck: well-typedness checks passed");

        // Lower the built-in operator nodes in the user equations to named
        // applications.
        lower_data_expressions(&mut spec);
        debug!("typecheck: lowered the user equations");

        // Collect the Appendix-B definitions for the basic and container sorts
        // that the specification uses. The container sorts are deliberately
        // excluded such that type checking can be done on their polymorphic
        // definitions.
        let basics = basic_sort_data_specification(encoding);
        check_no_system_function_redeclaration(&spec, &basics)?;
        debug!("typecheck: no user declaration redeclares a system function");

        let mut system = build_system_defined_specification(&spec, basics.clone(), encoding);

        // The defining equations of each structured sort (Appendix B.10) join
        // the system-defined part.
        for constructors in &structs {
            system.merge(&structured_sort_equations(constructors).map_err(WellTypedError::Custom)?);
        }

        // The system equations parse with the same operator nodes, so they are
        // lowered like the user equations.
        lower_data_expressions(&mut system);

        // Perform some basic sanity checks on the system-defined specification.
        if cfg!(debug_assertions)
            && let Err(error) = check_system_specification(&spec, &system)
        {
            panic!("the generated system-defined specification is malformed: {error}");
        }
        debug!(
            "typecheck: built the system-defined specification with {} sort, {} map and {} equation declaration(s)",
            system.sort_declarations.len(),
            system.map_declarations.len(),
            system.equation_declarations.len()
        );

        // Resolve the system-defined declarations of the *basic* sorts onto
        // the same lattice, so Phase-3 inference sees the overload sets of the
        // built-in operators.
        resolve_system_signature(&mut context, &spec, &basics)?;
        debug!("typecheck: resolved the system signature");

        // Inference over every user equation; an equation binding
        // a variable through an invalid sort (a bare product) is rejected here.
        let equation_typings = check_equations(&mut context, &spec)?;
        debug!("typecheck: inference finished; the specification is well-typed");

        // Warm the constructor and map sort caches eagerly so that callers can
        // access sorts without needing a `&mut TypeckContext`.
        for decl in &spec.constructor_declarations {
            if let Some(id) = decl.id {
                crate::query_sort_of_constructor(&mut context, &spec, id);
            }
        }
        for decl in &spec.map_declarations {
            if let Some(id) = decl.id {
                crate::query_sort_of_map(&mut context, &spec, id);
            }
        }

        Ok(Self {
            spec,
            sorts,
            system,
            context,
            equation_typings,
            encoding,
        })
    }

    /// The number encoding this specification was built with.
    pub fn number_encoding(&self) -> NumberEncoding {
        self.encoding
    }

    /// The resolved data specification.
    pub fn data_specification(&self) -> &UntypedDataSpecification {
        &self.spec
    }

    /// Maps each declared sort name to the [`DefId`] assigned during name
    /// resolution.
    pub fn sorts(&self) -> &IndexedSet<String> {
        &self.sorts
    }

    /// The system-defined (Appendix-B) declarations for the basic and container
    /// sorts that occur in the specification, plus the defining equations of
    /// the desugared structured sorts (Appendix B.10). This is generated
    /// content with unresolved sorts but lowered equation expressions, verified
    /// in debug builds by `check_system_specification`; multi-argument function
    /// updates are not included yet.
    pub fn system_defined_specification(&self) -> &UntypedDataSpecification {
        &self.system
    }

    /// The query context holding the interned sorts referenced by the
    /// declaration-sort queries ([`crate::query_sort_of_constructor`],
    /// [`crate::query_sort_of_map`], [`crate::query_sort_of_equation_var`]).
    // Currently exercised by tests only.
    #[allow(dead_code)]
    pub(crate) fn context(&self) -> &TypeckContext {
        &self.context
    }

    /// The resolved sort of the constructor declaration with the given
    /// [ConstructorId]. Requires `id` to be a valid constructor id from this
    /// specification; panics if called before `from_untyped` has completed.
    // Currently exercised by tests only.
    #[allow(dead_code)]
    pub(crate) fn sort_of_constructor(&self, id: ConstructorId) -> crate::ResolvedSortId {
        self.context
            .sort_of_constructor
            .get(&id)
            .copied()
            .expect("constructor sorts are all resolved during from_untyped")
    }

    /// The resolved sort of the map declaration with the given [MapId].
    /// Requires `id` to be a valid map id from this specification; panics if
    /// called before `from_untyped` has completed.
    // Currently exercised by tests only.
    #[allow(dead_code)]
    pub(crate) fn sort_of_map(&self, id: MapId) -> crate::ResolvedSortId {
        self.context
            .sort_of_map
            .get(&id)
            .copied()
            .expect("map sorts are all resolved during from_untyped")
    }

    /// The resolved sort of the `var_id`-th variable in the equation block
    /// identified by `eqn_spec_id`. Requires both ids to be valid from this
    /// specification; panics if called before `from_untyped` has completed.
    // Currently exercised by tests only.
    #[allow(dead_code)]
    pub(crate) fn sort_of_equation_var(&self, eqn_spec_id: EqnSpecId, var_id: EqnVarId) -> crate::ResolvedSortId {
        self.context
            .sort_of_equation_var
            .get(&(eqn_spec_id, var_id))
            .copied()
            .expect("equation variable sorts are all resolved during from_untyped")
    }

    /// The (S, C, M) signature: the resolved overload sets of every constructor
    /// and mapping name.
    // Currently exercised by tests only.
    #[allow(dead_code)]
    pub(crate) fn signature(&self) -> &Signature {
        self.context
            .signature
            .as_deref()
            .expect("build_signature ran in from_untyped")
    }

    /// The Phase-3 typing of every user equation, positionally parallel to
    /// `equation_declarations` (outer) and each equation list (inner).
    // Currently exercised by tests only.
    #[allow(dead_code)]
    pub(crate) fn equation_typings(&self) -> &[Vec<Rc<EquationTyping>>] {
        &self.equation_typings
    }

    /// Assembles and returns the fully typed mCRL2 data specification in the
    /// binary aterm format, ready for downstream consumption by `merc_sabre`
    /// and `merc_explore`.
    ///
    /// Includes the user sort declarations, aliases, constructors, mappings,
    /// and equations (those whose expression tree is fully supported by
    /// Phase-4 lowering), followed by all system (Appendix-B) declarations and
    /// the system equations, which are resolved structurally — empty-container
    /// and `Number` literals are resolved against their context, so only
    /// equations that still need full inference (binders, set/bag
    /// enumerations) are skipped.
    ///
    /// Call this once after [`Self::from_untyped`] when the typed specification
    /// is needed; it may be called more than once (results are identical since
    /// the underlying caches are warm after the first call).
    pub fn lower_data_specification(&mut self) -> Mcrl2DataSpecification {
        lower_data_specification(
            &mut self.context,
            &self.spec,
            &self.system,
            &self.equation_typings,
            self.encoding,
        )
    }
}

/// Returns the target sort of a sort expression, i.e. the range of a function
/// sort, or the sort itself if it is not a function sort.
pub(crate) fn target_sort(sort: &SortExpression) -> &SortExpression {
    debug_assert!(
        !matches!(sort.node, SortExpressionKind::Function { .. }),
        "target_sort should only be called on non-function sorts or flattened function sorts"
    );

    if let SortExpressionKind::FlattenedFunction { domain: _, range } = &sort.node {
        range
    } else {
        sort
    }
}

/// Returns the argument sorts of a (flattened) function sort, or an empty slice
/// for a non-function sort — such as the target sort of a constant constructor
/// like `cons c: S;`, which takes no arguments.
pub(crate) fn argument_sorts(sort: &SortExpression) -> &[SortExpression] {
    if let SortExpressionKind::FlattenedFunction { domain, range: _ } = &sort.node {
        domain
    } else {
        &[]
    }
}

/// Rewrites every `Function` node of `sort` into a `FlattenedFunction` whose
/// domain is the flattened `Product` spine (`(A#B)->C` becomes `A#B->C`).
fn flatten_function_sorts(sort: &SortExpression) -> SortExpression {
    apply_sort_expression(sort.clone(), |expr| -> Result<_, Infallible> {
        if let SortExpressionKind::Function { domain, range } = &expr.node {
            let mut flattened_domain = Vec::new();
            flatten_function_domain_rec(domain, &mut flattened_domain);

            return Ok(Some(
                SortExpressionKind::FlattenedFunction {
                    domain: flattened_domain,
                    range: range.clone(),
                }
                .into(),
            ));
        }

        Ok(None)
    })
    .expect("flatten_function_sorts should not fail")
}

/// Flattens a function sort of the form ((A_0 # A_1) # ... # A_n) -> B into a
/// sort of the form A_0 # A_1 # ... # A_n -> B, where B is the original range
/// of the function.
fn flatten_function_domain_rec(sort: &SortExpression, domain: &mut Vec<SortExpression>) {
    match &sort.node {
        SortExpressionKind::Product { lhs, rhs } => {
            flatten_function_domain_rec(lhs, domain);
            flatten_function_domain_rec(rhs, domain);
        }
        _ => domain.push(sort.clone()),
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use merc_syntax::EqnSpecId;
    use merc_syntax::EquationId;
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::query_equation_typing;

    #[test]
    fn test_equation_typing_is_memoized() {
        let spec = UntypedDataSpecification::parse("map f: Nat; eqn f = 1;").unwrap();
        let mut checked = DataSpecification::from_untyped(spec).unwrap();

        let first = Rc::clone(&checked.equation_typings[0][0]);
        let again = query_equation_typing(
            &mut checked.context,
            &checked.spec,
            (EqnSpecId::new(0), EquationId::new(0)),
        )
        .unwrap();
        assert!(Rc::ptr_eq(&first, &again));
    }

    #[test]
    fn test_mcrl2_data_specification_sections_populated() {
        let mut spec = DataSpecification::from_untyped(
            UntypedDataSpecification::parse(
                "sort D; \
                 sort A = Nat; \
                 cons c: D; \
                 map f: D -> Bool; \
                 var d: D; \
                 eqn f(d) = true;",
            )
            .unwrap(),
        )
        .unwrap();
        let mcrl2 = spec.lower_data_specification();

        // User abstract sort `D` → sorts section.
        assert!(mcrl2.sorts().iter().any(|s| s.name() == "D"), "D must appear in sorts");
        // User alias `A = Nat` → aliases section.
        assert!(
            mcrl2.aliases().iter().any(|a| a.name().name() == "A"),
            "A must appear in aliases"
        );
        // User constructor `c` → constructors section.
        assert!(
            mcrl2.constructors().iter().any(|c| c.name() == "c"),
            "c must appear in constructors"
        );
        // User mapping `f` → mappings section.
        assert!(
            mcrl2.mappings().iter().any(|m| m.name() == "f"),
            "f must appear in mappings"
        );
        // User equation `f(d) = true` → equations section.
        assert!(
            !mcrl2.equations().is_empty(),
            "at least one user equation must be lowered"
        );
        assert_eq!(mcrl2.equations()[0].lhs().to_string(), "f(d)");
        assert_eq!(mcrl2.equations()[0].rhs().to_string(), "true");
    }

    #[test]
    fn test_mcrl2_data_specification_system_constructors_present() {
        // `Bool` always pulls in its system constructors; at least `true`/`false` must appear.
        let mut spec =
            DataSpecification::from_untyped(UntypedDataSpecification::parse("map f: Bool;").unwrap()).unwrap();
        let mcrl2 = spec.lower_data_specification();
        assert!(
            mcrl2.constructors().iter().any(|c| c.name() == "true"),
            "system Bool constructor `true` must appear in constructors"
        );
    }

    #[test]
    fn test_mcrl2_data_specification_system_equations_present() {
        // System Bool equations (e.g. `!true = false`) must appear now that
        // `lower_data_specification` includes structurally-lowerable system equations.
        let mut spec =
            DataSpecification::from_untyped(UntypedDataSpecification::parse("map f: Bool;").unwrap()).unwrap();
        let mcrl2 = spec.lower_data_specification();
        // `!true = false` should be among the system Bool equations.
        let found = mcrl2
            .equations()
            .iter()
            .any(|e| e.lhs().to_string().contains("!(true)") && e.rhs().to_string() == "false");
        assert!(found, "system Bool equation `!(true) = false` must be present");
    }

    #[test]
    fn test_mcrl2_data_specification_empty_container_equations_present() {
        // A `List(D)` sort pulls in the Appendix-B list equations, several of
        // which mention the empty-list literal `[]` (`in(d, []) = false`,
        // `#[] = @c0`). These are now lowered structurally via expected-sort
        // propagation rather than skipped.
        let mut spec = DataSpecification::from_untyped(
            UntypedDataSpecification::parse("sort D; map f: List(D) -> Bool;").unwrap(),
        )
        .unwrap();
        let mcrl2 = spec.lower_data_specification();

        // `in(d, []) = false` — the empty list as a `List(D)` argument.
        let in_empty = mcrl2
            .equations()
            .iter()
            .any(|e| e.lhs().to_string() == "in(d, [])" && e.rhs().to_string() == "false");
        assert!(in_empty, "system list equation `in(d, []) = false` must be present");

        // `#[] = @c0` — the empty list under the length operator.
        let length_empty = mcrl2
            .equations()
            .iter()
            .any(|e| e.lhs().to_string() == "#([])" && e.rhs().to_string() == "@c0");
        assert!(length_empty, "system list equation `#[] = @c0` must be present");
    }
}
