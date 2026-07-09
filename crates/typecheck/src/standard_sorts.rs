use std::convert::Infallible;
use std::fmt::Write;

use indoc::formatdoc;

use merc_syntax::ComplexSort;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;
use merc_utilities::MercError;

/// Returns a standard data specification containing the standard sorts and their associated constructors, mappings, and equations.
pub fn basic_sort_data_specification() -> Result<UntypedDataSpecification, MercError> {
    let mut result = UntypedDataSpecification::default();

    // Append the relevant specifications for the sorts that are present in the specification.
    result.merge(&UntypedDataSpecification::parse(include_str!(
        "../../syntax/spec/bool.mcrl2"
    ))?);
    result.merge(&UntypedDataSpecification::parse(include_str!(
        "../../syntax/spec/pos.mcrl2"
    ))?);
    result.merge(&UntypedDataSpecification::parse(include_str!(
        "../../syntax/spec/int.mcrl2"
    ))?);
    result.merge(&UntypedDataSpecification::parse(include_str!(
        "../../syntax/spec/nat.mcrl2"
    ))?);
    result.merge(&UntypedDataSpecification::parse(include_str!(
        "../../syntax/spec/real.mcrl2"
    ))?);

    Ok(result)
}

/// Constructs a data specification for a standard sort;
pub fn standard_sort(sort: &SortExpression) -> Result<UntypedDataSpecification, MercError> {
    if let SortExpression::Complex(complex, sort) = sort {
        let text = match complex {
            ComplexSort::List => include_str!("../../syntax/spec/list.mcrl2"),
            ComplexSort::Set => include_str!("../../syntax/spec/set.mcrl2"),
            ComplexSort::FSet => include_str!("../../syntax/spec/fset.mcrl2"),
            ComplexSort::Bag => include_str!("../../syntax/spec/bag.mcrl2"),
            ComplexSort::FBag => include_str!("../../syntax/spec/fbag.mcrl2"),
        };

        let spec = UntypedDataSpecification::parse(text)?;

        Ok(replace_sort(&spec, "S", sort))
    } else if let SortExpression::Function { domain, range } = sort {
        let text = include_str!("../../syntax/spec/function_update.mcrl2");

        let spec = UntypedDataSpecification::parse(text)?;

        // In the specification we define the function S -> T.
        let spec = replace_sort(&spec, "S", domain);
        let spec = replace_sort(&spec, "T", range);

        Ok(spec)
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
/// a specification for `List(Nat)`.
fn replace_sort(spec: &UntypedDataSpecification, identifier: &str, sort: &SortExpression) -> UntypedDataSpecification {
    let mut result = spec.clone();

    for constructor in &mut result.constructor_declarations {
        constructor.sort = replace_sort_expression(&constructor.sort, identifier, sort);
    }

    for map in &mut result.map_declarations {
        map.sort = replace_sort_expression(&map.sort, identifier, sort);
    }

    for equation in &mut result.equation_declarations {
        for var in &mut equation.variables {
            var.sort = replace_sort_expression(&var.sort, identifier, sort);
        }
    }

    result
}

/// Replaces sort references of `identifier` in `sort` by the given `result_sort`.
fn replace_sort_expression(sort: &SortExpression, identifier: &str, result_sort: &SortExpression) -> SortExpression {
    apply_sort_expression(sort.clone(), |expr| -> Result<Option<SortExpression>, Infallible> {
        if let SortExpression::Reference(id) = expr {
            if id == identifier {
                return Ok(Some(result_sort.clone()));
            }
        }

        Ok(None)
    })
    .unwrap()
}

/// Generate a data specification for any sort based on the rules in Appendix `B`.
pub fn basic_spec(sort: &str) -> Result<UntypedDataSpecification, MercError> {
    Ok(UntypedDataSpecification::parse(&formatdoc! {"
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
    "})?)
}

/// Generates a data specification for a structured sort, the rules are given in Appendix `B.10`.
///
/// # Details
///
/// Given a structured sort with constructors `c_1, ..., c_n`, where every
/// constructor `c_i` has arguments of sorts `A_{i,1}, ..., A_{i,k_i}`, this
/// generates the sort `Struct<name>` together with:
///
///   - the constructor `c_i : A_{i,1} # ... # A_{i,k_i} -> Struct<name>`;
///   - the recogniser `isC_i : Struct<name> -> Bool` (only when a recogniser is
///     declared for `c_i`);
///   - the projection `pr_{i,j} : Struct<name> -> A_{i,j}` (only when a
///     projection name is declared for the argument);
///   - the equations defining the recognisers, the projections, and the
///     comparison operators `==`, `<` and `<=` over the constructors.
pub fn structured_sort_spec(
    name: &str,
    structured_sort: &SortExpression,
) -> Result<UntypedDataSpecification, MercError> {
    let SortExpression::Struct { inner: constructors } = structured_sort else {
        unreachable!("structured_sort_spec should only be called on structured sorts");
    };

    let name = format!("Struct{}", name);

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

    // sort Struct@n;
    writeln!(spec, "sort {name};\n").unwrap();

    // cons c_i : A_{i,1} # ... # A_{i,k_i} -> Struct@n;
    writeln!(spec, "cons").unwrap();
    for constructor in constructors {
        if constructor.args.is_empty() {
            writeln!(spec, "    {}: {name};", constructor.name).unwrap();
        } else {
            let domain = constructor
                .args
                .iter()
                .map(|(_, sort)| sort.to_string())
                .collect::<Vec<_>>()
                .join(" # ");
            writeln!(spec, "    {}: {domain} -> {name};", constructor.name).unwrap();
        }
    }
    writeln!(spec).unwrap();

    // map: recognisers and projections (only those that are declared explicitly).
    let mut maps = String::new();
    for constructor in constructors {
        if let Some(recogniser) = &constructor.projection {
            writeln!(maps, "    {recogniser}: {name} -> Bool;").unwrap();
        }
    }
    for constructor in constructors {
        for (projection, sort) in &constructor.args {
            if let Some(projection) = projection {
                writeln!(maps, "    {projection}: {name} -> {sort};").unwrap();
            }
        }
    }
    if !maps.is_empty() {
        write!(spec, "map\n{maps}\n").unwrap();
    }

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
        writeln!(eqns, "    {} == {} = {equal};", application(i, "x"), application(i, "y")).unwrap();
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
        writeln!(eqns, "    {} <= {} = {less_equal};", application(i, "x"), application(i, "y")).unwrap();
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
    use super::{ComplexSort, Infallible, MercError, SortExpression, UntypedDataSpecification, Write, apply_sort_expression, formatdoc, replace_sort, replace_sort_expression, structured_sort_spec};

    /// Extracts the structured sort expression from `sort <ident> = <struct>;`.
    fn struct_sort(spec: &str) -> SortExpression {
        let spec = UntypedDataSpecification::parse(spec).unwrap();
        spec.sort_declarations
            .into_iter()
            .find_map(|decl| decl.expr)
            .expect("expected a sort alias with a structured sort")
    }

    #[test]
    fn structured_sort_spec_generates_a_parseable_specification() {
        let sort = struct_sort("sort D = struct c1(pr1: Nat, pr2: Bool)?is_c1 | c2?is_c2 | c3(Nat);");

        // The generated specification should be well-formed and parseable.
        let generated = structured_sort_spec("D", &sort).unwrap();

        // The sort itself and every constructor are declared.
        assert_eq!(generated.sort_declarations.len(), 1);
        assert_eq!(generated.sort_declarations[0].identifier, "StructD");

        let constructors: Vec<_> = generated
            .constructor_declarations
            .iter()
            .map(|c| c.identifier.as_str())
            .collect();
        assert!(constructors.contains(&"c1"));
        assert!(constructors.contains(&"c2"));
        assert!(constructors.contains(&"c3"));

        // Recognisers and the declared projections appear as mappings.
        let maps: Vec<_> = generated
            .map_declarations
            .iter()
            .map(|m| m.identifier.as_str())
            .collect();
        assert!(maps.contains(&"is_c1"));
        assert!(maps.contains(&"is_c2"));
        assert!(maps.contains(&"pr1"));
        assert!(maps.contains(&"pr2"));

        // c3 has no recogniser and no projection, so neither should be generated.
        assert!(!maps.contains(&"is_c3"));
    }

    #[test]
    fn structured_sort_spec_supports_only_constant_constructors() {
        // A structured sort where no constructor has arguments generates no
        // variables, so the `eqn` block must be emitted without a `var` block.
        let sort = struct_sort("sort E = struct red | green | blue;");
        let generated = structured_sort_spec("E", &sort).unwrap();

        assert_eq!(generated.constructor_declarations.len(), 3);
    }
}
