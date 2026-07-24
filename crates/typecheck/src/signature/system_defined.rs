use std::collections::HashSet;
use std::ops::ControlFlow;

use merc_syntax::ComplexSort;
use merc_syntax::DataExpr;
use merc_syntax::DataExprKind;
use merc_syntax::SortExpression;
use merc_syntax::SortExpressionKind;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::visit_data_expr;
use merc_syntax::visit_sort_expr;

use crate::NumberEncoding;
use crate::POLYMORPHIC_SIGNATURE;
use crate::WellTypedError;
use crate::is_supported_binder_sort;
use crate::standard_sort;

/// Builds the system-defined part of a specification: the Appendix-B
/// definitions (constructors, mappings and equations) for every basic sort,
/// container sort and single-argument function sort that occurs in `spec`.
///
/// The five basic sorts are always included. A container sort pulls in the
/// containers it is defined in terms of — a `Set(S)` needs `FSet(S)`, a `Bag(S)`
/// needs `FBag(S)`, `FSet(S)` and `Set(S)` — which the fixpoint below discovers
/// by re-scanning each generated specification. A function sort
/// `D_0 # ... # D_{n-1} -> T` contributes the function-update operators for
/// its declared arity — the bundled single-argument template when `n == 1`,
/// otherwise [standard_sort] generalizes it to the flattened domain.
/// Structured-sort equations are generated separately from the desugared
/// declarations and merged in by `DataSpecification::from_untyped`.
///
/// The result is deliberately left unresolved: it uses the built-in `Simple`
/// sorts and the Appendix-B operator names, and is not re-checked against the
/// user-oriented well-typedness rules.
///
/// `basics` is the [basic_sort_data_specification], passed in because the
/// caller also needs it separately for the system signature.
pub(crate) fn build_system_defined_specification(
    spec: &UntypedDataSpecification,
    basics: UntypedDataSpecification,
    encoding: NumberEncoding,
) -> UntypedDataSpecification {
    let mut result = basics;

    let mut worklist = Vec::new();
    // Seed from the user specification, including its function sorts.
    collect_system_sorts_in_spec(spec, &mut worklist, true);

    let mut seen: HashSet<SortExpression> = HashSet::new();
    while let Some(sort) = worklist.pop() {
        if !seen.insert(sort.clone()) {
            continue;
        }

        let generated = standard_sort(&sort, encoding);
        // A container is defined in terms of other containers, so re-scan the
        // generated specification for those. Function sorts are collected from
        // the user specification only: the function-update operators introduce
        // ever-larger function sorts (`@is_not_an_update: (S -> T) -> Bool`),
        // which the user did not ask for and which would not terminate here.
        collect_system_sorts_in_spec(&generated, &mut worklist, false);
        result.merge(&generated);
    }

    result
}

/// Any user `cons`/`map` declaration whose name collides with a system-defined
/// function is rejected, regardless of the user's declared sort.
pub(crate) fn check_no_system_function_redeclaration(
    spec: &UntypedDataSpecification,
    basics: &UntypedDataSpecification,
) -> Result<(), WellTypedError> {
    let mut reserved: HashSet<&str> = HashSet::new();
    reserved.extend(
        basics
            .constructor_declarations
            .iter()
            .map(|decl| decl.identifier.as_str()),
    );
    reserved.extend(basics.map_declarations.iter().map(|decl| decl.identifier.as_str()));
    // The container/function-update operations *and* the comparison operators
    // and `if` are all polymorphic built-ins, so they share one table.
    reserved.extend(POLYMORPHIC_SIGNATURE.ops.keys().map(String::as_str));

    for decl in &spec.constructor_declarations {
        if reserved.contains(decl.identifier.as_str()) {
            return Err(WellTypedError::SystemFunctionRedeclared {
                name: decl.identifier.clone(),
            });
        }
    }
    for decl in &spec.map_declarations {
        if reserved.contains(decl.identifier.as_str()) {
            return Err(WellTypedError::SystemFunctionRedeclared {
                name: decl.identifier.clone(),
            });
        }
    }
    Ok(())
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
/// Binder sorts that are not valid variable sorts (see
/// [is_supported_binder_sort]) are skipped: inference rejects the constructs
/// that bind them, so their operators are never looked up.
fn collect_system_sorts_in_expr(expr: &DataExpr, out: &mut Vec<SortExpression>, include_functions: bool) {
    visit_data_expr::<(), _>(expr, |expr| {
        match &expr.node {
            DataExprKind::SetBagComp { variable, predicate: _ } => {
                if is_supported_binder_sort(&variable.sort) {
                    collect_system_sorts(&variable.sort, out, include_functions);
                    out.push(SortExpressionKind::Complex(ComplexSort::Set, Box::new(variable.sort.clone())).into());
                    out.push(SortExpressionKind::Complex(ComplexSort::Bag, Box::new(variable.sort.clone())).into());
                }
            }
            DataExprKind::Lambda { variables, body: _ }
            | DataExprKind::Quantifier {
                op: _,
                variables,
                body: _,
            } => {
                for variable in variables {
                    if is_supported_binder_sort(&variable.sort) {
                        collect_system_sorts(&variable.sort, out, include_functions);
                    }
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    });
}

/// Collects the system-defined sorts in a single sort expression, recursing
/// through element, function, product and structured sorts.
///
/// Container sorts are always collected. Function sorts of any arity are
/// collected only when `include_functions` — see the call in
/// [`build_system_defined_specification`] for why generated specifications are
/// scanned without them. A single-argument domain is converted to the nested
/// `Function` form [`standard_sort`]'s single-argument branch expects; a
/// multi-argument domain is passed through as `FlattenedFunction`, which
/// `standard_sort`'s multi-argument branch consumes directly.
fn collect_system_sorts(sort: &SortExpression, out: &mut Vec<SortExpression>, include_functions: bool) {
    visit_sort_expr::<(), _>(sort, |expr| {
        match &expr.node {
            SortExpressionKind::Complex(_, _) => out.push(expr.clone()),
            // A user specification carries flattened function sorts; the
            // generated Appendix-B specifications carry the un-flattened
            // `Function` form.
            SortExpressionKind::Function { domain, .. } => {
                if include_functions && !matches!(domain.node, SortExpressionKind::Product { .. }) {
                    out.push(expr.clone());
                }
            }
            SortExpressionKind::FlattenedFunction { domain, range } if include_functions => {
                if let [single] = domain.as_slice() {
                    out.push(
                        SortExpressionKind::Function {
                            domain: Box::new(single.clone()),
                            range: range.clone(),
                        }
                        .into(),
                    );
                } else {
                    out.push(expr.clone());
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
    use merc_syntax::SortExpressionKind;
    use merc_syntax::UntypedDataSpecification;

    use super::build_system_defined_specification;
    use super::collect_system_sorts_in_spec;
    use crate::DataSpecification;
    use crate::NumberEncoding;
    use crate::basic_sort_data_specification;

    /// The distinct container constructors that occur in a specification.
    fn container_ops(spec: &UntypedDataSpecification) -> Vec<ComplexSort> {
        let mut sorts = Vec::new();
        collect_system_sorts_in_spec(spec, &mut sorts, true);
        let mut ops: Vec<ComplexSort> = sorts
            .into_iter()
            .filter_map(|sort| match sort.node {
                SortExpressionKind::Complex(op, _) => Some(op),
                _ => None,
            })
            .collect();
        ops.sort();
        ops.dedup();
        ops
    }

    fn system_spec(text: &str) -> UntypedDataSpecification {
        let basics = basic_sort_data_specification(NumberEncoding::Binary);
        build_system_defined_specification(
            &UntypedDataSpecification::parse(text).unwrap(),
            basics,
            NumberEncoding::Binary,
        )
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
    fn test_multi_argument_function_gets_update_operators() {
        // `Nat # Bool -> Nat` has a product domain; `standard_sort` generalizes
        // the Appendix-B template to it instead of deferring it.
        assert!(has_function_update("map f: Nat # Bool -> Nat;"));
    }

    #[test]
    fn test_function_over_containers_terminates() {
        // Regression: re-scanning generated function-update specs for further
        // function sorts diverged, because `@is_not_an_update: (S -> T) -> Bool`
        // is itself a single-argument function, growing the sort without bound.
        assert!(has_function_update("map f: List(Nat) -> List(Nat);"));
    }

    #[test]
    fn test_multi_argument_function_over_containers_terminates() {
        // The same regression as above, but seeded from a multi-argument
        // function so the fixpoint also terminates when it re-scans a
        // generated multi-argument `@func_update`/`@is_not_an_update`/
        // `@if_always_else` specification.
        assert!(has_function_update("map f: List(Nat) # Bool -> List(Nat);"));
    }
}
