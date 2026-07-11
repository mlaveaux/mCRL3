use std::collections::HashSet;

use merc_collections::IndexedSet;
use merc_syntax::DefId;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;

use crate::WellTypedError;

/// Ensure that all DefIds in the data specification are resolved. Returns an
/// indexed set that indicates the mapping from sort identifiers to their
/// DefIds.
pub(crate) fn resolve_names(spec: &mut UntypedDataSpecification) -> Result<IndexedSet<String>, WellTypedError> {
    // mCRL2 silently deduplicates byte-identical sort declarations
    // (sort_specification::add_alias), so repeated identical declarations are
    // accepted; conflicting redeclarations still fail below.
    let mut seen = HashSet::new();
    spec.sort_declarations
        .retain(|decl| seen.insert((decl.identifier.clone(), decl.expr.clone())));

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
pub(crate) fn map_sorts_in_spec<E, F>(spec: &mut UntypedDataSpecification, mut f: F) -> Result<(), E>
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

#[cfg(test)]
mod tests {
    use merc_syntax::DataExpr;
    use merc_syntax::SortExpression;
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::WellTypedError;

    #[test]
    fn test_identical_duplicate_sort_is_deduplicated() {
        let spec = UntypedDataSpecification::parse(
            "
            sort D = List(Bool);
            sort D = List(Bool);
        ",
        )
        .unwrap();

        DataSpecification::from_untyped(spec).expect("identical redeclarations are deduplicated as in mCRL2");
    }

    #[test]
    fn test_conflicting_duplicate_sort_is_rejected() {
        let spec = UntypedDataSpecification::parse(
            "
            sort D = List(Bool);
            sort D = List(Nat);
        ",
        )
        .unwrap();

        match DataSpecification::from_untyped(spec) {
            Err(WellTypedError::DuplicateSortDeclaration { sort }) if sort == "D" => {}
            Err(other) => panic!("Unexpected error {:?}", other),
            _ => panic!("Expected from_untyped to fail"),
        }
    }

    /// Locks the resolution boundary documented on
    /// `DataSpecification::data_specification`: name resolution rewrites the
    /// declaration-level sorts, but leaves sorts on binders inside equation
    /// bodies as `Reference`s (they are resolved later during data-expression
    /// type checking).
    #[test]
    fn test_equation_body_binder_sorts_are_not_resolved() {
        let spec = DataSpecification::from_untyped(
            UntypedDataSpecification::parse(
                "sort D = struct d1 | d2;
                 map f: D -> Bool;
                 var x: D;
                 eqn f(x) = forall y: D. y == x;",
            )
            .unwrap(),
        )
        .unwrap();

        let equation = &spec.data_specification().equation_declarations[0];

        // The declaration-level variable `x: D` is resolved.
        assert!(matches!(equation.variables[0].sort, SortExpression::Resolved(_, _)));

        // The quantifier binder `y: D` in the body is still an unresolved reference.
        let DataExpr::Quantifier { variables, .. } = &equation.equations[0].rhs else {
            panic!("expected a quantifier body, got {:?}", equation.equations[0].rhs);
        };
        assert!(matches!(variables[0].sort, SortExpression::Reference(_)));
    }
}
