use std::convert::Infallible;

use log::debug;
use log::trace;

use merc_syntax::ConstructorDecl;
use merc_syntax::ConstructorId;
use merc_syntax::IdDecl;
use merc_syntax::MapId;
use merc_syntax::Sort;
use merc_syntax::SortDecl;
use merc_syntax::SortExpression;
use merc_syntax::Span;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;

/// Hoists every anonymous structured sort — a `struct` occurring inside another
/// sort expression rather than as the body of a sort declaration — into a fresh
/// `@struct<n>` sort declaration, replacing the occurrence by a reference to it.
///
/// Structurally identical structs denote the same sort in mCRL2, so identical
/// occurrences share one declaration, and an anonymous struct that matches an
/// already-seen named struct alias reuses the user's name.
///
/// Runs before name resolution so the generated declarations are resolved and
/// checked exactly like user-written ones, after which
/// [`desugar_structured_sorts`] only encounters named structs.
pub(crate) fn hoist_anonymous_structs(spec: &mut UntypedDataSpecification) {
    let mut hoister = Hoister {
        table: Vec::new(),
        fresh: Vec::new(),
    };

    for declaration in &mut spec.sort_declarations {
        match &mut declaration.expr {
            // A top-level struct is the named struct itself and stays; only
            // structs nested inside its constructor arguments are hoisted.
            Some(SortExpression::Struct { inner }) => {
                for constructor in inner.iter_mut() {
                    for (_, sort) in &mut constructor.args {
                        *sort = hoister.hoist(sort.clone());
                    }
                }
                hoister.table.push((
                    SortExpression::Struct { inner: inner.clone() },
                    declaration.identifier.clone(),
                ));
            }
            Some(expr) => *expr = hoister.hoist(expr.clone()),
            None => {}
        }
    }

    for constructor in &mut spec.constructor_declarations {
        constructor.sort = hoister.hoist(constructor.sort.clone());
    }
    for map in &mut spec.map_declarations {
        map.sort = hoister.hoist(map.sort.clone());
    }
    for equation in &mut spec.equation_declarations {
        for variable in &mut equation.variables {
            variable.sort = hoister.hoist(variable.sort.clone());
        }
    }

    spec.sort_declarations.append(&mut hoister.fresh);
}

struct Hoister {
    /// Struct bodies that are already available under a name, so structurally
    /// identical occurrences resolve to the same sort.
    table: Vec<(SortExpression, String)>,
    /// The generated `@struct<n>` declarations.
    fresh: Vec<SortDecl>,
}

impl Hoister {
    /// Replaces every anonymous struct in `sort` by a reference to its (fresh
    /// or reused) named declaration.
    fn hoist(&mut self, sort: SortExpression) -> SortExpression {
        apply_sort_expression(sort, |expr| -> Result<Option<SortExpression>, Infallible> {
            if let SortExpression::Struct { inner } = expr {
                // Hoist the constructor arguments first, so identical structs
                // have identical bodies regardless of nesting.
                let mut inner = inner.clone();
                for constructor in &mut inner {
                    for (_, sort) in &mut constructor.args {
                        *sort = self.hoist(sort.clone());
                    }
                }

                return Ok(Some(SortExpression::Reference(
                    self.name_for(SortExpression::Struct { inner }),
                )));
            }

            Ok(None)
        })
        .expect("The inner function never fails")
    }

    /// The name declaring `body`, generating a fresh `@struct<n>` declaration
    /// when it has not been seen before.
    fn name_for(&mut self, body: SortExpression) -> String {
        if let Some((_, name)) = self.table.iter().find(|(existing, _)| *existing == body) {
            return name.clone();
        }

        let name = format!("@struct{}", self.fresh.len());
        debug!("desugar: hoisted anonymous struct '{body}' as sort '{name}'");
        self.table.push((body.clone(), name.clone()));
        self.fresh.push(SortDecl {
            identifier: name.clone(),
            expr: Some(body),
            span: Span::default(),
            id: None,
        });
        name
    }
}

/// Desugars every named structured-sort declaration into an abstract sort plus
/// the constructors, recognisers and projections it introduces.
///
/// `sort D = struct c1(p: A)?is_c1 | c2;` becomes
///
/// ```text
/// sort D;
/// cons c1: A -> D;  c2: D;
/// map  is_c1: D -> Bool;   % only for constructors with a recogniser
///      p: D -> A;          % only for named projections (deduplicated)
/// ```
///
/// Returns the constructor list of every desugared structured sort, from which
/// `structured_sort_equations` generates the defining equations (Appendix
/// B.10) for the system-defined specification. Anonymous structured sorts have
/// already been hoisted into named declarations by
/// [`hoist_anonymous_structs`], so every struct encountered here is named.
///
/// Runs after name resolution, so the generated sorts are already resolved and
/// flattened, and the structured sort keeps its `DefId`.
pub(crate) fn desugar_structured_sorts(spec: &mut UntypedDataSpecification) -> Vec<Vec<ConstructorDecl>> {
    let mut constructors: Vec<IdDecl<ConstructorId>> = Vec::new();
    let mut mappings: Vec<IdDecl<MapId>> = Vec::new();
    let mut structs = Vec::new();

    for declaration in &mut spec.sort_declarations {
        let inner = match &declaration.expr {
            Some(SortExpression::Struct { inner }) => inner.clone(),
            _ => continue,
        };

        let id = declaration.id.expect("Name must have been resolved");
        let sort = SortExpression::Resolved(declaration.identifier.clone(), id);
        // The structured sort becomes an abstract sort carrying its constructors.
        declaration.expr = None;
        debug!(
            "desugar: struct '{}' desugared into {} constructor(s)",
            declaration.identifier,
            inner.len()
        );

        for constructor in &inner {
            // cons c: A_1 # ... # A_n -> D  (or c: D when it has no arguments).
            let domain = constructor.args.iter().map(|(_, sort)| sort.clone()).collect();
            let constructor_sort = function_sort(domain, sort.clone());
            trace!("desugar:   cons {}: {constructor_sort}", constructor.name);
            constructors.push(IdDecl::new(constructor.name.clone(), constructor_sort, Span::default()));

            // map is_c: D -> Bool  (recogniser), when one is declared.
            if let Some(recogniser) = &constructor.projection {
                let recogniser_sort = function_sort(vec![sort.clone()], SortExpression::Simple(Sort::Bool));
                push_unique(
                    &mut mappings,
                    IdDecl::new(recogniser.clone(), recogniser_sort, Span::default()),
                );
            }

            // map p: D -> A  (projection), when a name is declared for the argument.
            for (projection, argument_sort) in &constructor.args {
                if let Some(projection) = projection {
                    let projection_sort = function_sort(vec![sort.clone()], argument_sort.clone());
                    push_unique(
                        &mut mappings,
                        IdDecl::new(projection.clone(), projection_sort, Span::default()),
                    );
                }
            }
        }

        structs.push(inner);
    }

    spec.constructor_declarations.append(&mut constructors);
    spec.map_declarations.append(&mut mappings);

    structs
}

/// Builds `domain_0 # ... # domain_n -> range` as an already-flattened function
/// sort, or just `range` when there are no arguments.
fn function_sort(domain: Vec<SortExpression>, range: SortExpression) -> SortExpression {
    if domain.is_empty() {
        range
    } else {
        SortExpression::FlattenedFunction {
            domain,
            range: Box::new(range),
        }
    }
}

/// Appends `mapping` unless an identical declaration is already present, so a
/// projection shared by several constructors is generated only once.
fn push_unique(mappings: &mut Vec<IdDecl<MapId>>, mapping: IdDecl<MapId>) {
    if !mappings.contains(&mapping) {
        trace!("desugar:   map {}: {}", mapping.identifier, mapping.sort);
        mappings.push(mapping);
    }
}

#[cfg(test)]
mod tests {
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;

    /// Returns the constructor and mapping names of the type-checked spec.
    fn constructors_and_mappings(text: &str) -> (Vec<String>, Vec<String>) {
        let checked = DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()).unwrap();
        let spec = checked.data_specification();
        let constructors = spec
            .constructor_declarations
            .iter()
            .map(|declaration| declaration.identifier.clone())
            .collect();
        let mappings = spec
            .map_declarations
            .iter()
            .map(|declaration| declaration.identifier.clone())
            .collect();
        (constructors, mappings)
    }

    #[test]
    fn test_struct_desugars_to_constructors() {
        let (constructors, mappings) = constructors_and_mappings("sort D = struct c1(p1: Bool)?is_c1 | c2;");
        assert!(constructors.contains(&"c1".to_string()));
        assert!(constructors.contains(&"c2".to_string()));
        assert!(mappings.contains(&"is_c1".to_string()));
        assert!(mappings.contains(&"p1".to_string()));
    }

    #[test]
    fn test_reused_projection_is_generated_once() {
        // `p` is shared by `c` and `d` with the same sort, so only one mapping
        // is generated for it.
        let (_, mappings) = constructors_and_mappings("sort S = struct c(p: Bool) | d(p: Bool, q: S);");
        assert_eq!(mappings.iter().filter(|name| *name == "p").count(), 1);
        assert!(mappings.contains(&"q".to_string()));
    }

    #[test]
    fn test_struct_equations_are_in_system_spec() {
        let checked = DataSpecification::from_untyped(
            UntypedDataSpecification::parse("sort D = struct c1(p1: Bool)?is_c1 | c2;").unwrap(),
        )
        .unwrap();

        let equations: Vec<String> = checked
            .system_defined_specification()
            .equation_declarations
            .iter()
            .flat_map(|eqn_spec| &eqn_spec.equations)
            .map(|eqn| format!("{} = {}", eqn.lhs, eqn.rhs))
            .collect();

        // The recogniser, projection and comparison equations are generated
        // (operators are lowered to named applications by then).
        assert!(equations.iter().any(|eqn| eqn.starts_with("is_c1(c1(")));
        assert!(equations.iter().any(|eqn| eqn.starts_with("p1(c1(")));
        assert!(equations.iter().any(|eqn| eqn.contains("==(c2, c2)")));
    }

    #[test]
    fn test_anonymous_struct_in_mapping_is_desugared() {
        // An anonymous struct in a mapping declaration is hoisted to a fresh
        // named sort, whose constructors are then desugared as usual.
        let (constructors, _) = constructors_and_mappings("map f: struct c | d;");
        assert!(constructors.contains(&"c".to_string()));
        assert!(constructors.contains(&"d".to_string()));
    }

    #[test]
    fn test_nested_anonymous_struct_is_desugared() {
        // The struct nested inside `t`'s argument is hoisted and desugared, so
        // its constructor `e` is declared too.
        let (constructors, _) = constructors_and_mappings("sort S = struct t(struct e(Nat));");
        assert!(constructors.contains(&"t".to_string()));
        assert!(constructors.contains(&"e".to_string()));
    }

    #[test]
    fn test_identical_anonymous_structs_share_a_declaration() {
        // Structurally identical structs are the same sort, so `c` is declared
        // only once.
        let (constructors, _) = constructors_and_mappings("map f: struct c;\n    g: struct c;");
        assert_eq!(constructors.iter().filter(|name| *name == "c").count(), 1);
    }

    #[test]
    fn test_anonymous_struct_reuses_named_alias() {
        // An anonymous struct that matches a named struct alias is the same
        // sort as the alias, so no second declaration (and constructor) is
        // generated.
        let (constructors, _) = constructors_and_mappings("sort D = struct c;\nmap f: struct c;");
        assert_eq!(constructors.iter().filter(|name| *name == "c").count(), 1);
    }

    #[test]
    fn test_recursive_struct_is_non_empty() {
        // A recursive structured sort with a base constructor is non-empty and
        // type checks after desugaring.
        DataSpecification::from_untyped(
            UntypedDataSpecification::parse("sort Tree = struct leaf | node(Tree, Tree);").unwrap(),
        )
        .expect("a recursive struct with a base case is non-empty");
    }

    #[test]
    fn test_struct_over_abstract_arguments_is_non_empty() {
        // A struct whose constructors take abstract-sort arguments is non-empty
        // (the abstract arguments are assumed non-empty).
        DataSpecification::from_untyped(
            UntypedDataSpecification::parse("sort A;\n     B;\nsort S = struct c(A) | d(B);").unwrap(),
        )
        .expect("a struct over abstract argument sorts is non-empty");
    }
}
