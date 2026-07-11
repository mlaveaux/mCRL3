use std::convert::Infallible;
use std::rc::Rc;

use merc_collections::IndexedSet;
use merc_syntax::DefId;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;

use crate::AliasError;
use crate::DeclarationSorts;
use crate::EquationTyping;
use crate::Signature;
use crate::SystemSortNames;
use crate::TypeckContext;
use crate::WellTypedError;
use crate::build_system_defined_specification;
use crate::check_aliases;
use crate::check_equations;
use crate::desugar_structured_sorts;
use crate::hoist_anonymous_structs;
use crate::is_well_typed;
use crate::lower_data_expressions;
use crate::map_sorts_in_spec;
use crate::normalize_sorts;
use crate::query_signature;
use crate::resolve_declaration_sorts;
use crate::resolve_names;
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
    declaration_sorts: DeclarationSorts,
    /// The display names of the system-internal sorts (`@NatPair`, ...).
    // Consumed by the sort rendering of inference errors (docs/typecheck.md §9).
    #[allow(dead_code)]
    system_sort_names: SystemSortNames,
    equation_typings: Vec<Vec<Rc<EquationTyping>>>,
}

impl DataSpecification {
    /// Create a completed well-typed data specification from an untyped data specification.
    pub fn from_untyped(mut spec: UntypedDataSpecification) -> Result<Self, WellTypedError> {
        // Hoist anonymous structured sorts into fresh named declarations, so
        // name resolution, the alias checks and the desugaring below only ever
        // see named structs.
        hoist_anonymous_structs(&mut spec);

        map_sorts_in_spec(&mut spec, |sort| -> Result<_, Infallible> {
            Ok(flatten_function_sorts(sort))
        })
        .expect("The inner function never fails");

        let sorts = resolve_names(&mut spec)?;

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
        // recognisers and projections. This runs after the alias checks so those
        // still see the `struct` form (a struct may recurse illegally through a
        // function sort), and before the checks below so the constructors it
        // introduces participate in them.
        let structs = desugar_structured_sorts(&mut spec);

        // Compute the (S, C, M) signature and run the signature-layer checks of
        // 15.1.7 (docs/typecheck.md §5 stage 2). This runs before alias
        // expansion so the errors refer to sorts as the user wrote them; the
        // semantic facts come from the interned sort lattice, which expands
        // alias indirection lazily.
        let mut context = TypeckContext::new();
        query_signature(&mut context, &spec)?;

        // Expand aliases to a canonical form now that they are known to be
        // acyclic, so the well-typedness check and the stored spec see sorts
        // without alias indirection.
        normalize_sorts(&mut spec);

        // Safety net over the normalized spec: it repeats the constructor
        // target and symbol-disjointness checks syntactically, and additionally
        // covers equation-variable sorts and the sort-emptiness check, which
        // the signature query does not.
        is_well_typed(&spec)?;

        // Lower the built-in operator nodes in the user equations to named
        // applications (docs/typecheck.md §5 stage 1), so Phase-3 inference
        // only sees a single application form. The sort passes above never
        // touch data expressions, so after this point the stored spec is both
        // normalized and fully lowered.
        lower_data_expressions(&mut spec);

        // Collect the Appendix-B definitions for the basic and container sorts
        // that the specification uses.
        let mut system = build_system_defined_specification(&spec).map_err(WellTypedError::Custom)?;

        // The defining equations of each structured sort (Appendix B.10) join
        // the system-defined part: they use the `==`/`<`/`<=` operators that
        // only exist there, and are trusted content like the rest of it.
        for constructors in &structs {
            system.merge(&structured_sort_equations(constructors).map_err(WellTypedError::Custom)?);
        }

        // The system equations parse with the same operator nodes (`b && true`,
        // `d |> s`), so they are lowered like the user equations.
        lower_data_expressions(&mut system);

        // Resolve the declaration-level sorts of the user specification onto
        // the interned sort lattice (docs/typecheck.md §5 stage 3). The system
        // specification is still unresolved content and is not covered (G3).
        let declaration_sorts = resolve_declaration_sorts(&mut context, &spec);

        // Resolve the system-defined declarations onto the same lattice, so
        // Phase-3 inference sees the overload sets of the built-in operators.
        let system_sort_names = resolve_system_signature(&mut context, &spec, &system)?;

        // Phase-3 core inference over the user equations (docs/typecheck.md
        // §9); equations using constructs it does not cover yet are skipped.
        let equation_typings = check_equations(&mut context, &spec, &declaration_sorts)?;

        Ok(Self {
            spec,
            sorts,
            system,
            context,
            declaration_sorts,
            system_sort_names,
            equation_typings,
        })
    }

    /// The resolved data specification. The declaration-level sorts (on `sort`,
    /// `cons`, `map` declarations and equation variable lists) have their names
    /// resolved to a [`DefId`]; sorts on binders inside equation bodies
    /// (`forall`/`exists`/`lambda`/comprehensions) are resolved later, as part
    /// of data-expression type checking. All equation expressions are lowered:
    /// built-in operators appear as named applications (`==(x, y)`).
    pub fn data_specification(&self) -> &UntypedDataSpecification {
        &self.spec
    }

    /// Maps each declared sort name to the [`DefId`] assigned during name
    /// resolution (`sorts().index(name)` yields the index behind that `DefId`).
    pub fn sorts(&self) -> &IndexedSet<String> {
        &self.sorts
    }

    /// The system-defined (Appendix-B) declarations for the basic and container
    /// sorts that occur in the specification, plus the defining equations of
    /// the desugared structured sorts (Appendix B.10). This is trusted content
    /// with unresolved sorts but lowered equation expressions; multi-argument
    /// function updates are not included yet (see G3 in `docs/typecheck.md`).
    pub fn system_defined_specification(&self) -> &UntypedDataSpecification {
        &self.system
    }

    /// The query context holding the interned sorts referenced by
    /// [`Self::declaration_sorts`].
    // Consumed by Phase-3 inference (docs/typecheck.md §9); exercised by tests only until then.
    #[allow(dead_code)]
    pub(crate) fn context(&self) -> &TypeckContext {
        &self.context
    }

    /// The resolved sorts of the user declarations, positionally parallel to
    /// the declaration lists of [`Self::data_specification`].
    // Consumed by Phase-3 inference (docs/typecheck.md §9); exercised by tests only until then.
    #[allow(dead_code)]
    pub(crate) fn declaration_sorts(&self) -> &DeclarationSorts {
        &self.declaration_sorts
    }

    /// The (S, C, M) signature: the resolved overload sets of every constructor
    /// and mapping name.
    // Consumed by Phase-3 overload resolution (docs/typecheck.md §9); exercised by tests only until then.
    #[allow(dead_code)]
    pub(crate) fn signature(&self) -> &Signature {
        self.context
            .signature
            .as_ref()
            .expect("query_signature ran in from_untyped")
    }

    /// The Phase-3 typing of every user equation, positionally parallel to
    /// `equation_declarations` (outer) and each equation list (inner).
    // Consumed by Phase-4 lowering (docs/typecheck.md §9); exercised by tests only until then.
    #[allow(dead_code)]
    pub(crate) fn equation_typings(&self) -> &[Vec<Rc<EquationTyping>>] {
        &self.equation_typings
    }
}

/// Returns the target sort of a sort expression, i.e. the range of a function
/// sort, or the sort itself if it is not a function sort.
pub(crate) fn target_sort(sort: &SortExpression) -> &SortExpression {
    debug_assert!(
        !matches!(sort, SortExpression::Function { .. }),
        "target_sort should only be called on non-function sorts or flattened function sorts"
    );

    if let SortExpression::FlattenedFunction { domain: _, range } = sort {
        range
    } else {
        sort
    }
}

/// Returns the argument sorts of a (flattened) function sort, or an empty slice
/// for a non-function sort — such as the target sort of a constant constructor
/// like `cons c: S;`, which takes no arguments.
pub(crate) fn argument_sorts(sort: &SortExpression) -> &[SortExpression] {
    if let SortExpression::FlattenedFunction { domain, range: _ } = sort {
        domain
    } else {
        &[]
    }
}

/// Replaces sort references of `identifier` in `sort` by the given `result_sort`.
fn flatten_function_sorts(sort: &SortExpression) -> SortExpression {
    apply_sort_expression(sort.clone(), |expr| -> Result<_, Infallible> {
        if let SortExpression::Function { domain, range } = expr {
            let mut flattened_domain = Vec::new();
            flatten_function_domain_rec(domain, &mut flattened_domain);

            return Ok(Some(SortExpression::FlattenedFunction {
                domain: flattened_domain,
                range: range.clone(),
            }));
        }

        Ok(None)
    })
    .expect("flatten_function_sorts should not fail")
}

/// Flattens a function sort of the form ((A_0 # A_1) # ... # A_n) -> B into a
/// sort of the form A_0 # A_1 # ... # A_n -> B, where B is the original range
/// of the function.
fn flatten_function_domain_rec(sort: &SortExpression, domain: &mut Vec<SortExpression>) {
    match sort {
        SortExpression::Product { lhs, rhs } => {
            flatten_function_domain_rec(lhs, domain);
            flatten_function_domain_rec(rhs, domain);
        }
        _ => domain.push(sort.clone()),
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::query_equation_typing;

    #[test]
    fn test_equation_typing_is_memoized() {
        let spec = UntypedDataSpecification::parse("map f: Nat; eqn f = 1;").unwrap();
        let mut checked = DataSpecification::from_untyped(spec).unwrap();

        let first = Rc::clone(&checked.equation_typings[0][0]);
        let again =
            query_equation_typing(&mut checked.context, &checked.spec, &checked.declaration_sorts, (0, 0)).unwrap();
        assert!(Rc::ptr_eq(&first, &again));
    }
}
