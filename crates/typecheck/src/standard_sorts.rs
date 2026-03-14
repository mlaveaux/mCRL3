use std::convert::Infallible;

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
    }).unwrap()
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
pub fn structured_sort_spec(
    name: &str,
    structured_sort: &SortExpression,
) -> Result<UntypedDataSpecification, MercError> {
    if let SortExpression::Struct { inner: _ } = structured_sort {
        let name = format!("Struct{}", name);

        let spec = formatdoc!(
            "sort {name}
        
        "
        );

        UntypedDataSpecification::parse(&spec)
    } else {
        unreachable!("structure_sort_spec should only be called on structured sorts");
    }
}
