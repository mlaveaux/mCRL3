use std::collections::HashMap;
use std::convert::Infallible;

use merc_syntax::DefId;
use merc_syntax::EqnSpec;
use merc_syntax::IdDecl;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;

use crate::WellTypedError;
use crate::basic_sort_data_specification;
use crate::has_alias_cycle;
use crate::is_well_typed;
use crate::map_sorts_in_spec;
use crate::resolve_names;

/// A type checked and well-typed data specification.
pub struct DataSpecification {
    pub sort_declarations: HashMap<DefId, (Vec<IdDecl>, Vec<EqnSpec>)>,
}

impl DataSpecification {
    /// Create a completed well-typed data specification from an untyped data specification.
    pub fn from_untyped(mut spec: UntypedDataSpecification) -> Result<Self, WellTypedError> {
        map_sorts_in_spec(&mut spec, |sort| -> Result<_, Infallible> {
            Ok(flatten_function_sorts(sort))
        })
        .expect("The inner function never fails");

        let sorts = resolve_names(&mut spec)?;

        has_alias_cycle(&spec).map_err(|cycle| WellTypedError::AliasCycle {
            sorts: cycle
                .iter()
                .map(|id| sorts.get_by_index(**id).expect("The sort should be declared").clone())
                .collect(),
        })?;

        is_well_typed(&spec)?;

        // Obtain the definitions for all basic sort.
        let _basic_sort_spec = basic_sort_data_specification().map_err(WellTypedError::Custom)?;

        // Add all the occurring standard sorts to the specification.

        Ok(Self {
            sort_declarations: HashMap::new(), // TODO: Fill this in with the actual sort declarations.
        })
    }
}

/// Returns the target sort of a sort expression, i.e. the range of a function
/// sort, or the sort itself if it is not a function sort.
pub fn target_sort(sort: &SortExpression) -> &SortExpression {
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

/// Returns the arguments of a function sort, requires that function sorts are flattened.
pub fn argument_sorts(sort: &SortExpression) -> &Vec<SortExpression> {
    if let SortExpression::FlattenedFunction { domain, range: _ } = sort {
        domain
    } else {
        unreachable!("argument_sorts should only be called on function sorts")
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
