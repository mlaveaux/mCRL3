use std::convert::Infallible;
use std::fmt::Write;
use std::sync::LazyLock;

use indoc::formatdoc;

use merc_syntax::ComplexSort;
use merc_syntax::ConstructorDecl;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;
use merc_utilities::MercError;

use crate::map_sorts_in_spec;

/// Parses a bundled `spec/*.mcrl2` file. The templates are compiled in, so a
/// parse failure is a build defect, not a runtime condition — the statics
/// below panic instead of threading a `Result` through every caller.
fn parse_template(text: &str) -> UntypedDataSpecification {
    UntypedDataSpecification::parse(text).expect("the bundled templates parse")
}

/// The merged specifications of the five basic sorts (Appendix B.1–B.7),
/// parsed once like the Pratt parsers of `merc_syntax`.
static BASIC_SORTS: LazyLock<UntypedDataSpecification> = LazyLock::new(|| {
    let mut result = UntypedDataSpecification::default();
    result.merge(&parse_template(include_str!("../../../syntax/spec/bool.mcrl2")));
    result.merge(&parse_template(include_str!("../../../syntax/spec/pos.mcrl2")));
    result.merge(&parse_template(include_str!("../../../syntax/spec/int.mcrl2")));
    result.merge(&parse_template(include_str!("../../../syntax/spec/nat.mcrl2")));
    result.merge(&parse_template(include_str!("../../../syntax/spec/real.mcrl2")));
    result
});

/// The raw, uninstantiated container and function-update templates, parsed
/// once. The sort names `S` and `T` are the templates' sort variables: they
/// remain unresolved `Reference` nodes, to be substituted ([standard_sort]) or
/// instantiated with fresh unification variables (`POLYMORPHIC_SIGNATURE`).
pub(crate) struct ContainerTemplates {
    list: UntypedDataSpecification,
    set: UntypedDataSpecification,
    fset: UntypedDataSpecification,
    bag: UntypedDataSpecification,
    fbag: UntypedDataSpecification,
    function_update: UntypedDataSpecification,
}

impl ContainerTemplates {
    /// All templates, for building the polymorphic signature.
    pub(crate) fn all(&self) -> [&UntypedDataSpecification; 6] {
        [
            &self.list,
            &self.set,
            &self.fset,
            &self.bag,
            &self.fbag,
            &self.function_update,
        ]
    }
}

pub(crate) static CONTAINER_TEMPLATES: LazyLock<ContainerTemplates> = LazyLock::new(|| ContainerTemplates {
    list: parse_template(include_str!("../../../syntax/spec/list.mcrl2")),
    set: parse_template(include_str!("../../../syntax/spec/set.mcrl2")),
    fset: parse_template(include_str!("../../../syntax/spec/fset.mcrl2")),
    bag: parse_template(include_str!("../../../syntax/spec/bag.mcrl2")),
    fbag: parse_template(include_str!("../../../syntax/spec/fbag.mcrl2")),
    function_update: parse_template(include_str!("../../../syntax/spec/function_update.mcrl2")),
});

/// Returns a standard data specification containing the standard sorts and their associated constructors, mappings, and equations.
pub(crate) fn basic_sort_data_specification() -> UntypedDataSpecification {
    BASIC_SORTS.clone()
}

/// Constructs a data specification for a standard sort;
pub(crate) fn standard_sort(sort: &SortExpression) -> UntypedDataSpecification {
    if let SortExpression::Complex(complex, sort) = sort {
        let template = match complex {
            ComplexSort::List => &CONTAINER_TEMPLATES.list,
            ComplexSort::Set => &CONTAINER_TEMPLATES.set,
            ComplexSort::FSet => &CONTAINER_TEMPLATES.fset,
            ComplexSort::Bag => &CONTAINER_TEMPLATES.bag,
            ComplexSort::FBag => &CONTAINER_TEMPLATES.fbag,
        };

        replace_sort(template, "S", sort)
    } else if let SortExpression::Function { domain, range } = sort {
        // In the specification we define the function S -> T.
        let spec = replace_sort(&CONTAINER_TEMPLATES.function_update, "S", domain);
        replace_sort(&spec, "T", range)
    } else {
        unreachable!("The given sort {} is not a standard sort", sort);
    }
}

/// Replaces the given identifier by the given sort expression in the given data
/// specification.
///
/// # Details
///
/// This function can be used to instantiate polymorphic types, for example,
/// replacing identifier `S` in the specification for `List(S)` by `Nat` to get
/// a specification for `List(Nat)`. The substitution covers every sort in the
/// specification, including the binder sorts inside equations (`forall c:S.`
/// in the set/bag templates).
fn replace_sort(spec: &UntypedDataSpecification, identifier: &str, sort: &SortExpression) -> UntypedDataSpecification {
    let mut result = spec.clone();

    map_sorts_in_spec(&mut result, |expr| -> Result<_, Infallible> {
        Ok(replace_sort_expression(expr, identifier, sort))
    })
    .expect("substitution never fails");

    result
}

/// Replaces sort references of `identifier` in `sort` by the given `result_sort`.
fn replace_sort_expression(sort: &SortExpression, identifier: &str, result_sort: &SortExpression) -> SortExpression {
    apply_sort_expression(sort.clone(), |expr| -> Result<Option<SortExpression>, Infallible> {
        if let SortExpression::Reference(id) = expr
            && id == identifier
        {
            return Ok(Some(result_sort.clone()));
        if let SortExpression::Reference(id) = expr
            && id == identifier
        {
            return Ok(Some(result_sort.clone()));
        }

        Ok(None)
    })
    .unwrap()
}

/// Generate a data specification for any sort based on the rules in Appendix `B`.
pub fn basic_spec(sort: &str) -> Result<UntypedDataSpecification, MercError> {
    UntypedDataSpecification::parse(&formatdoc! {"
// Reserved for wiring the comparison/`if` operators of each sort (docs/typecheck.md G3).
#[allow(dead_code)]
pub(crate) fn basic_spec(sort: &str) -> Result<UntypedDataSpecification, MercError> {
    UntypedDataSpecification::parse(&formatdoc! {"
        map ==, !=, <, <=, >=, >: {sort} # {sort} -> Bool;
            if: Bool # {sort} # {sort} -> {sort};

        var x, y: {sort};
            b: Bool;

        eqm x == x = true;
            x != y = !(x == y);
            if(true, x, y)  = x;
            if(false, x, y) = y;
            if(b, x, y)      = if (b, x, y);
            if(x == y, x, y) = y;
            x < x  = false;
            x <= x = true;
            x > y  = y < x;
            x >= y = y <= x;
    "})
    "})
}

/// Generates the defining equations of a structured sort, following Appendix `B.10`.
///
/// # Details
///
/// Given the constructors `c_1, ..., c_n` of a structured sort, where every
/// constructor `c_i` has arguments of sorts `A_{i,1}, ..., A_{i,k_i}`, this
/// generates the equations defining the recognisers, the projections, and the
/// comparison operators `==`, `<` and `<=` over the constructors.
///
/// Only equations are generated; the abstract sort and the constructor,
/// recogniser and projection declarations are introduced by
/// `desugar_structured_sorts`, which also yields the `constructors` passed
/// here. The result joins the system-defined specification, like the other
/// Appendix-B content.
pub(crate) fn structured_sort_equations(
    constructors: &[ConstructorDecl],
) -> Result<UntypedDataSpecification, MercError> {
    // Builds the term `c_i(<prefix>i_0, ..., <prefix>i_{k_i - 1})`, using the
    // bare constructor name when `c_i` takes no arguments.
    let application = |i: usize, prefix: &str| -> String {
        let constructor = &constructors[i];
        if constructor.args.is_empty() {
            constructor.name.clone()
        } else {
            let arguments = (0..constructor.args.len())
                .map(|j| format!("{prefix}{i}_{j}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({arguments})", constructor.name)
        }
    };

    // Builds the right-hand side of the `<` or `<=` equation between two equal
    // constructors, i.e. the lexicographic comparison of their arguments where
    // the final argument is compared using `last_op` (`<` or `<=`):
    //   x0 < y0 || (x0 == y0 && (... || (x_{k-2} == y_{k-2} && (x_{k-1} OP y_{k-1}))...))
    let lexicographic = |i: usize, arity: usize, last_op: &str| -> String {
        let mut expr = format!("x{i}_{last} {last_op} y{i}_{last}", last = arity - 1);
        for j in (0..arity - 1).rev() {
            expr = format!("x{i}_{j} < y{i}_{j} || (x{i}_{j} == y{i}_{j} && ({expr}))");
        }
        expr
    };

    let mut spec = String::new();

    // var: one x/y pair per constructor argument.
    let mut vars = String::new();
    for (i, constructor) in constructors.iter().enumerate() {
        for (j, (_, sort)) in constructor.args.iter().enumerate() {
            writeln!(vars, "    x{i}_{j}, y{i}_{j}: {sort};").unwrap();
        }
    }

    // eqn: recogniser, projection and comparison equations.
    let mut eqns = String::new();

    // Recognisers: isC_i(c_i(..)) = true; isC_i(c_j(..)) = false for j != i.
    for (i, constructor) in constructors.iter().enumerate() {
        if let Some(recogniser) = &constructor.projection {
            writeln!(eqns, "    {recogniser}({}) = true;", application(i, "x")).unwrap();
            for j in 0..constructors.len() {
                if j != i {
                    writeln!(eqns, "    {recogniser}({}) = false;", application(j, "x")).unwrap();
                }
            }
        }
    }

    // Projections: pr_{i,j}(c_i(..)) = x_{i,j}.
    for (i, constructor) in constructors.iter().enumerate() {
        for (j, (projection, _)) in constructor.args.iter().enumerate() {
            if let Some(projection) = projection {
                writeln!(eqns, "    {projection}({}) = x{i}_{j};", application(i, "x")).unwrap();
            }
        }
    }

    // Equality: componentwise on equal constructors, false on distinct ones.
    for (i, constructor) in constructors.iter().enumerate() {
        let equal = if constructor.args.is_empty() {
            "true".to_string()
        } else {
            (0..constructor.args.len())
                .map(|j| format!("x{i}_{j} == y{i}_{j}"))
                .collect::<Vec<_>>()
                .join(" && ")
        };
        writeln!(
            eqns,
            "    {} == {} = {equal};",
            application(i, "x"),
            application(i, "y")
        )
        .unwrap();
        for j in 0..constructors.len() {
            if j != i {
                writeln!(eqns, "    {} == {} = false;", application(i, "x"), application(j, "y")).unwrap();
            }
        }
    }

    // Less-than: lexicographic on equal constructors, by constructor index otherwise.
    for (i, constructor) in constructors.iter().enumerate() {
        let less = if constructor.args.is_empty() {
            "false".to_string()
        } else {
            lexicographic(i, constructor.args.len(), "<")
        };
        writeln!(eqns, "    {} < {} = {less};", application(i, "x"), application(i, "y")).unwrap();
        for j in 0..constructors.len() {
            if i < j {
                writeln!(eqns, "    {} < {} = true;", application(i, "x"), application(j, "y")).unwrap();
            } else if i > j {
                writeln!(eqns, "    {} < {} = false;", application(i, "x"), application(j, "y")).unwrap();
            }
        }
    }

    // Less-than-or-equal: as `<`, but the last argument is compared with `<=`.
    for (i, constructor) in constructors.iter().enumerate() {
        let less_equal = if constructor.args.is_empty() {
            "true".to_string()
        } else {
            lexicographic(i, constructor.args.len(), "<=")
        };
        writeln!(
            eqns,
            "    {} <= {} = {less_equal};",
            application(i, "x"),
            application(i, "y")
        )
        .unwrap();
        for j in 0..constructors.len() {
            if i < j {
                writeln!(eqns, "    {} <= {} = true;", application(i, "x"), application(j, "y")).unwrap();
            } else if i > j {
                writeln!(eqns, "    {} <= {} = false;", application(i, "x"), application(j, "y")).unwrap();
            }
        }
    }

    if vars.is_empty() {
        write!(spec, "eqn\n{eqns}").unwrap();
    } else {
        write!(spec, "var\n{vars}eqn\n{eqns}").unwrap();
    }

    UntypedDataSpecification::parse(&spec)
}

#[cfg(test)]
mod tests {
    use merc_syntax::ConstructorDecl;

    use super::SortExpression;
    use super::UntypedDataSpecification;
    use super::standard_sort;
    use super::structured_sort_equations;

    #[test]
    fn test_standard_sort_substitutes_binder_sorts() {
        // The set template's `==` equation quantifies over the element sort
        // (`forall c:S.`); instantiation must substitute binder sorts like any
        // declaration sort, or the generated equation would reference the
        // undeclared `S`.
        let spec = UntypedDataSpecification::parse("map f: Set(Nat);").unwrap();
        let generated = standard_sort(&spec.map_declarations[0].sort);

        let equations: Vec<String> = generated
            .equation_declarations
            .iter()
            .flat_map(|eqn_spec| &eqn_spec.equations)
            .map(|eqn| eqn.to_string())
            .collect();
        assert!(
            equations.iter().any(|eqn| eqn.contains("forall c: Nat")),
            "the quantifier's binder sort should be instantiated: {equations:#?}"
        );
    }

    /// Extracts the constructors of the structured sort in `sort <ident> = <struct>;`.
    fn struct_constructors(spec: &str) -> Vec<ConstructorDecl> {
        let spec = UntypedDataSpecification::parse(spec).unwrap();
        let expr = spec
            .sort_declarations
            .into_iter()
            .find_map(|decl| decl.expr)
            .expect("expected a sort alias with a structured sort");
        let SortExpression::Struct { inner } = expr else {
            panic!("expected a structured sort");
        };
        inner
    }

    #[test]
    fn structured_sort_equations_generates_a_parseable_specification() {
        let constructors = struct_constructors("sort D = struct c1(pr1: Nat, pr2: Bool)?is_c1 | c2?is_c2 | c3(Nat);");

        // The generated specification should be well-formed and parseable, and
        // contain only equations; the declarations come from desugaring.
        let generated = structured_sort_equations(&constructors).unwrap();
        assert!(generated.sort_declarations.is_empty());
        assert!(generated.constructor_declarations.is_empty());
        assert!(generated.map_declarations.is_empty());

        let equations = generated
            .equation_declarations
            .iter()
            .flat_map(|eqn_spec| &eqn_spec.equations)
            .map(|eqn| format!("{} = {}", eqn.lhs, eqn.rhs))
            .collect::<Vec<_>>();

        // Recogniser and projection equations for the declared names.
        assert!(
            equations
                .iter()
                .any(|eqn| eqn.contains("is_c1") && eqn.contains("true"))
        );
        assert!(
            equations
                .iter()
                .any(|eqn| eqn.contains("is_c1") && eqn.contains("false"))
        );
        assert!(equations.iter().any(|eqn| eqn.contains("pr1")));

        // c3 has no recogniser, so no equation defines one for it.
        assert!(!equations.iter().any(|eqn| eqn.contains("is_c3")));
    }

    #[test]
    fn structured_sort_equations_supports_only_constant_constructors() {
        // A structured sort where no constructor has arguments generates no
        // variables, so the `eqn` block must be emitted without a `var` block.
        let constructors = struct_constructors("sort E = struct red | green | blue;");
        let generated = structured_sort_equations(&constructors).unwrap();

        assert!(!generated.equation_declarations.is_empty());
    }
}
