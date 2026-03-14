use merc_collections::IndexedSet;
use merc_syntax::DefId;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;

use crate::WellTypedError;

/// Ensure that all DefIds in the data specification are resolved. Returns an
/// indexed set that indicates the mapping from sort identifiers to their
/// DefIds.
pub fn resolve_names(spec: &mut UntypedDataSpecification) -> Result<IndexedSet<String>, WellTypedError> {
    // Every sort declaration should have a unique name.
    let mut sorts = IndexedSet::new();

    // Assign unique IDs to all sort declarations
    for (i, sort) in spec.sort_declarations.iter_mut().enumerate() {
        sort.id = Some(DefId::new(i));

        if !sorts.insert(sort.identifier.clone()).1 {
            return Err(WellTypedError::DuplicateSortDeclaration {
                sort: sort.identifier.clone(),
            });
        }
    }

    // Resolve all sorts.
    map_sorts_in_spec(spec, |sort| resolve_sort_id(sort, &sorts))?;

    Ok(sorts)
}

// Apply a mapping function to every sort in the specification.
pub fn map_sorts_in_spec<E, F>(spec: &mut UntypedDataSpecification, mut f: F) -> Result<(), E>
where
    F: FnMut(&SortExpression) -> Result<SortExpression, E>,
{
    for sort in &mut spec.sort_declarations {
        if let Some(expr) = &sort.expr {
            // Only apply to type aliases.
             sort.expr = Some(f(expr)?);
        }
    }

    for constructor in &mut spec.constructor_declarations {
        constructor.sort = f(&constructor.sort)?;
    }

    for map in &mut spec.map_declarations {
        map.sort = f(&map.sort)?;
    }

    for equation in &mut spec.equation_declarations {
        for var in &mut equation.variables {
            var.sort = f(&var.sort)?;
        }
    }

    Ok(())
}

/// Replaces sort references of `identifier` in `sort` by the given `result_sort`.
fn resolve_sort_id(sort: &SortExpression, resolved: &IndexedSet<String>) -> Result<SortExpression, WellTypedError> {
    apply_sort_expression(sort.clone(), |expr| {
        if let SortExpression::Reference(name) = expr {
            if let Some(id) = resolved.index(name) {
                return Ok(Some(SortExpression::Resolved(name.clone(), DefId::new(*id))));
            } 

            return Err(WellTypedError::UndefinedSort { sort: name.clone() });
        }

        Ok(None)
    })
}