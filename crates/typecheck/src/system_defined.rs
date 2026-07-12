use std::collections::HashSet;
use std::ops::ControlFlow;

use merc_syntax::ComplexSort;
use merc_syntax::DataExpr;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::visit_sort_expr;
use merc_utilities::MercError;

use crate::is_supported_binder_sort;
use crate::standard_sort;

/// Builds the system-defined part of a specification: the Appendix-B
/// definitions (constructors, mappings and equations) for every basic sort,
/// container sort and single-argument function sort that occurs in `spec`,
/// mirroring mCRL2's `initialise_system_defined_functions`.
///
/// The five basic sorts are always included. A container sort pulls in the
/// containers it is defined in terms of — a `Set(S)` needs `FSet(S)`, a `Bag(S)`
/// needs `FBag(S)`, `FSet(S)` and `Set(S)` — which the fixpoint below discovers
/// by re-scanning each generated specification. A single-argument function sort
/// `S -> T` contributes the function-update operators; multi-argument function
/// sorts are deferred (their `S` would be a product, which the Appendix-B
/// template cannot take as a stand-alone argument). Structured-sort equations
/// are generated separately from the desugared declarations and merged in by
/// `DataSpecification::from_untyped`.
///
/// The result is deliberately left unresolved: it uses the built-in `Simple`
/// sorts and the Appendix-B operator names, and is trusted content rather than
/// something re-checked against the user-oriented well-typedness rules.
///
/// `basics` is the [basic_sort_data_specification], passed in because the
/// caller also needs it separately (for the system signature).
pub(crate) fn build_system_defined_specification(
    spec: &UntypedDataSpecification,
    basics: UntypedDataSpecification,
) -> Result<UntypedDataSpecification, MercError> {
    let mut result = basics;

    let mut worklist = Vec::new();
    // Seed from the user specification, including its function sorts.
    collect_system_sorts_in_spec(spec, &mut worklist, true);

    let mut seen: HashSet<SortExpression> = HashSet::new();
    while let Some(sort) = worklist.pop() {
        if !seen.insert(sort.clone()) {
            continue;
        }

        let generated = standard_sort(&sort)?;
        // A container is defined in terms of other containers, so re-scan the
        // generated specification for those. Function sorts are collected from
        // the user specification only: the function-update operators introduce
        // ever-larger function sorts (`@is_not_an_update: (S -> T) -> Bool`),
        // which the user did not ask for and which would not terminate here.
        collect_system_sorts_in_spec(&generated, &mut worklist, false);
        result.merge(&generated);
    }

    Ok(result)
}

/// Collects every container sort — and, when `include_functions`, every
/// single-argument function sort — occurring in the specification into `out`,
/// including the sorts on binders inside the equation expressions.
fn collect_system_sorts_in_spec(
    spec: &UntypedDataSpecification,
    out: &mut Vec<SortExpression>,
    include_functions: bool,
) {
    for declaration in &spec.sort_declarations {
        if let Some(expr) = &declaration.expr {
            collect_system_sorts(expr, out, include_functions);
        }
    }
    for constructor in &spec.constructor_declarations {
        collect_system_sorts(&constructor.sort, out, include_functions);
    }
    for map in &spec.map_declarations {
        collect_system_sorts(&map.sort, out, include_functions);
    }
    for equation in &spec.equation_declarations {
        for variable in &equation.variables {
            collect_system_sorts(&variable.sort, out, include_functions);
        }
        for eqn in &equation.equations {
            if let Some(condition) = &eqn.condition {
                collect_system_sorts_in_expr(condition, out, include_functions);
            }
            collect_system_sorts_in_expr(&eqn.lhs, out, include_functions);
            collect_system_sorts_in_expr(&eqn.rhs, out, include_functions);
        }
    }
}

/// Collects the system-defined sorts mentioned syntactically inside a data
/// expression: the sorts on binders, and around a set/bag comprehension's
/// element sort also `Set(S)` and `Bag(S)` — the comprehension denotes one of
/// the two, which reading applies is only decided by sort inference, so the
/// operators of both are provided. The element sorts of enumeration literals
/// (`{1, 2}`) are not syntactically apparent and are not collected.
///
/// Binder sorts the pipeline cannot resolve (see [is_supported_binder_sort])
/// are skipped: inference defers the constructs that bind them, so their
/// operators are never looked up.
fn collect_system_sorts_in_expr(expr: &DataExpr, out: &mut Vec<SortExpression>, include_functions: bool) {
    match expr {
        DataExpr::SetBagComp { variable, predicate } => {
            if is_supported_binder_sort(&variable.sort) {
                collect_system_sorts(&variable.sort, out, include_functions);
                out.push(SortExpression::Complex(
                    ComplexSort::Set,
                    Box::new(variable.sort.clone()),
                ));
                out.push(SortExpression::Complex(
                    ComplexSort::Bag,
                    Box::new(variable.sort.clone()),
                ));
            }
            collect_system_sorts_in_expr(predicate, out, include_functions);
        }
        DataExpr::Lambda { variables, body } | DataExpr::Quantifier { op: _, variables, body } => {
            for variable in variables {
                if is_supported_binder_sort(&variable.sort) {
                    collect_system_sorts(&variable.sort, out, include_functions);
                }
            }
            collect_system_sorts_in_expr(body, out, include_functions);
        }
        DataExpr::Application { function, arguments } => {
            collect_system_sorts_in_expr(function, out, include_functions);
            for argument in arguments {
                collect_system_sorts_in_expr(argument, out, include_functions);
            }
        }
        DataExpr::Unary { op: _, expr } => collect_system_sorts_in_expr(expr, out, include_functions),
        DataExpr::Binary { op: _, lhs, rhs } => {
            collect_system_sorts_in_expr(lhs, out, include_functions);
            collect_system_sorts_in_expr(rhs, out, include_functions);
        }
        DataExpr::List(elements) | DataExpr::Set(elements) => {
            for element in elements {
                collect_system_sorts_in_expr(element, out, include_functions);
            }
        }
        DataExpr::Bag(elements) => {
            for element in elements {
                collect_system_sorts_in_expr(&element.expr, out, include_functions);
                collect_system_sorts_in_expr(&element.multiplicity, out, include_functions);
            }
        }
        DataExpr::FunctionUpdate { expr, update } => {
            collect_system_sorts_in_expr(expr, out, include_functions);
            collect_system_sorts_in_expr(&update.expr, out, include_functions);
            collect_system_sorts_in_expr(&update.update, out, include_functions);
        }
        DataExpr::Whr { expr, assignments } => {
            collect_system_sorts_in_expr(expr, out, include_functions);
            for assignment in assignments {
                collect_system_sorts_in_expr(&assignment.expr, out, include_functions);
            }
        }
        DataExpr::Id(_)
        | DataExpr::Number(_)
        | DataExpr::Bool(_)
        | DataExpr::EmptyList
        | DataExpr::EmptySet
        | DataExpr::EmptyBag => {}
    }
}

/// Collects the system-defined sorts in a single sort expression, recursing
/// through element, function, product and structured sorts.
///
/// Container sorts are always collected. Single-argument function sorts are
/// collected only when `include_functions` — see the call in
/// [`build_system_defined_specification`] for why generated specifications are
/// scanned without them. A multi-argument function is never collected: its `S`
/// would be a product that [`standard_sort`] cannot turn into a valid
/// declaration.
fn collect_system_sorts(sort: &SortExpression, out: &mut Vec<SortExpression>, include_functions: bool) {
    visit_sort_expr::<(), _>(sort, |expr| {
        match expr {
            SortExpression::Complex(_, _) => out.push(expr.clone()),
            // A user specification carries flattened function sorts; the
            // generated Appendix-B specifications carry the un-flattened
            // `Function` form.
            SortExpression::Function { domain, .. } => {
                if include_functions && !matches!(**domain, SortExpression::Product { .. }) {
                    out.push(expr.clone());
                }
            }
            SortExpression::FlattenedFunction { domain, range } => {
                if include_functions && let [single] = domain.as_slice() {
                    out.push(SortExpression::Function {
                        domain: Box::new(single.clone()),
                        range: range.clone(),
                    });
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    });
}

#[cfg(test)]
mod tests {
    use merc_syntax::ComplexSort;
    use merc_syntax::SortExpression;
    use merc_syntax::UntypedDataSpecification;

    use super::build_system_defined_specification;
    use super::collect_system_sorts_in_spec;
    use crate::DataSpecification;
    use crate::basic_sort_data_specification;

    /// The distinct container constructors that occur in a specification.
    fn container_ops(spec: &UntypedDataSpecification) -> Vec<ComplexSort> {
        let mut sorts = Vec::new();
        collect_system_sorts_in_spec(spec, &mut sorts, true);
        let mut ops: Vec<ComplexSort> = sorts
            .into_iter()
            .filter_map(|sort| match sort {
                SortExpression::Complex(op, _) => Some(op),
                _ => None,
            })
            .collect();
        ops.sort();
        ops.dedup();
        ops
    }

    fn system_spec(text: &str) -> UntypedDataSpecification {
        let basics = basic_sort_data_specification().unwrap();
        build_system_defined_specification(&UntypedDataSpecification::parse(text).unwrap(), basics).unwrap()
    }

    #[test]
    fn test_basic_sorts_are_always_present() {
        let spec = system_spec("map f: Bool;");
        for basic in ["Bool", "Pos", "Nat", "Int", "Real"] {
            assert!(
                spec.sort_declarations.iter().any(|decl| decl.identifier == basic),
                "the basic sort {basic} should always be included"
            );
        }
    }

    #[test]
    fn test_set_pulls_in_finite_set() {
        // A `Set(S)` is defined in terms of `FSet(S)`, so both must be present.
        let ops = container_ops(&system_spec("map f: Set(Nat);"));
        assert!(ops.contains(&ComplexSort::Set));
        assert!(ops.contains(&ComplexSort::FSet));
    }

    #[test]
    fn test_comprehension_contributes_set_and_bag() {
        // A comprehension may denote a set or a bag; the equations of both are
        // provided for its element sort even though no declaration mentions a
        // container.
        let spec = UntypedDataSpecification::parse("map b: Bool; eqn b = 1 in { n: Pos | n < 3 };").unwrap();
        let ops = container_ops(&spec);
        for op in [ComplexSort::Set, ComplexSort::Bag] {
            assert!(ops.contains(&op), "a comprehension should contribute {op:?}");
        }
    }

    #[test]
    fn test_quantifier_binder_sort_is_collected() {
        // The `List(Nat)` mentioned only on the quantifier binder still gets
        // its Appendix-B equations.
        let spec = UntypedDataSpecification::parse("map b: Bool; eqn b = forall l: List(Nat). l == [];").unwrap();
        assert!(container_ops(&spec).contains(&ComplexSort::List));
    }

    #[test]
    fn test_bag_pulls_in_all_related_containers() {
        // A `Bag(S)` transitively needs `FBag(S)`, `FSet(S)` and `Set(S)`.
        let ops = container_ops(&system_spec("map f: Bag(Nat);"));
        for op in [ComplexSort::Bag, ComplexSort::FBag, ComplexSort::FSet, ComplexSort::Set] {
            assert!(ops.contains(&op), "using Bag should pull in {op:?}");
        }
    }

    #[test]
    fn test_nested_container_element_is_included() {
        // `List(Set(Nat))` needs both the list and the (transitive) set defs.
        let ops = container_ops(&system_spec("map f: List(Set(Nat));"));
        assert!(ops.contains(&ComplexSort::List));
        assert!(ops.contains(&ComplexSort::Set));
        assert!(ops.contains(&ComplexSort::FSet));
    }

    /// Whether the system-defined spec of `text` declares the function-update
    /// operators, checked through the full `from_untyped` path (which flattens
    /// function sorts).
    fn has_function_update(text: &str) -> bool {
        let spec = DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()).unwrap();
        spec.system_defined_specification()
            .map_declarations
            .iter()
            .any(|map| map.identifier.contains("func_update"))
    }

    #[test]
    fn test_single_argument_function_gets_update_operators() {
        assert!(has_function_update("map f: Nat -> Bool;"));
    }

    #[test]
    fn test_multi_argument_function_update_is_deferred() {
        // `Nat # Bool -> Nat` has a product domain, which the Appendix-B
        // template cannot take as a stand-alone argument, so it is skipped.
        assert!(!has_function_update("map f: Nat # Bool -> Nat;"));
    }

    #[test]
    fn test_function_over_containers_terminates() {
        // Regression: re-scanning generated function-update specs for further
        // function sorts diverged, because `@is_not_an_update: (S -> T) -> Bool`
        // is itself a single-argument function, growing the sort without bound.
        assert!(has_function_update("map f: List(Nat) -> List(Nat);"));
    }
}
