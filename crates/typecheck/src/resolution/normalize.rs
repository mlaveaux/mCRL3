use std::collections::HashMap;
use std::convert::Infallible;

use log::debug;

use merc_syntax::DefId;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::apply_sort_expression;

use crate::map_sorts_in_spec;

/// Normalizes every sort in `spec` to a canonical form by expanding aliases.
///
/// A non-structured alias (`sort D = Nat;`, `sort L = List(D);`) is replaced by
/// its recursively normalized definition, so an alias and the sort it stands for
/// become indistinguishable and sort equality is structural. A structured-sort
/// alias is instead its own representative and keeps its name, because
/// structured sorts are identified by name and because expanding a recursive
/// `struct` would not terminate.
///
/// Terminates on every specification that
/// [`check_aliases`](crate::alias::check_aliases) accepts. The `visited` stack
/// keeps any alias reached again during its own expansion as a named
/// representative, so a cycle is never unfolded — including a cycle that closes
/// through an inline `struct`, which `check_aliases` permits (recursion through
/// a constructor is well-defined) but which would otherwise diverge here.
pub(crate) fn normalize_sorts(spec: &mut UntypedDataSpecification) {
    // Clone the alias right-hand sides so the rewrite can borrow `spec` mutably
    // while still consulting the alias map.
    let alias_map: HashMap<DefId, SortExpression> = spec
        .sort_declarations
        .iter()
        .filter_map(|decl| Some((decl.id.expect("Name must have been resolved"), decl.expr.clone()?)))
        .collect();

    map_sorts_in_spec(spec, |sort| -> Result<_, Infallible> {
        let result = normalize_sort(sort, &alias_map, &mut Vec::new());
        if result != *sort {
            debug!("normalize: sort '{sort}' expanded to '{result}'");
        }
        Ok(result)
    })
    .expect("normalization never fails");
}

/// Recursively normalizes a single sort against the alias map. `visited` holds
/// the aliases currently being expanded, so an alias reached again is kept as a
/// named representative instead of being unfolded forever.
fn normalize_sort(
    sort: &SortExpression,
    alias_map: &HashMap<DefId, SortExpression>,
    visited: &mut Vec<DefId>,
) -> SortExpression {
    apply_sort_expression(sort.clone(), |expr| -> Result<_, Infallible> {
        let SortExpression::Resolved(_, id) = expr else {
            return Ok(None);
        };

        // A structured-sort alias, an abstract sort, or an alias reached again
        // while it is being expanded, is a named representative: keep the name
        // and do not recurse, so recursion through a `struct` terminates.
        if visited.contains(id) {
            return Ok(None);
        }
        match alias_map.get(id) {
            Some(SortExpression::Struct { .. }) | None => Ok(None),
            Some(alias) => {
                visited.push(*id);
                let result = normalize_sort(alias, alias_map, visited);
                visited.pop();
                Ok(Some(result))
            }
        }
    })
    .expect("normalization never fails")
}

#[cfg(test)]
mod tests {
    use merc_syntax::Sort;
    use merc_syntax::SortExpression;
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;

    /// Type checks `text` and returns the (normalized) sort of the map `name`.
    fn map_sort(text: &str, name: &str) -> SortExpression {
        let spec = DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()).unwrap();
        spec.data_specification()
            .map_declarations
            .iter()
            .find(|map| map.identifier == name)
            .unwrap_or_else(|| panic!("map {name} should be declared"))
            .sort
            .clone()
    }

    #[test]
    fn test_alias_to_basic_sort_is_expanded() {
        // `D` aliases `Nat`, so `f: D` normalizes to the built-in `Nat` sort.
        let sort = map_sort("sort D = Nat; map f: D;", "f");
        assert_eq!(sort, SortExpression::Simple(Sort::Nat));
    }

    #[test]
    fn test_alias_chain_is_expanded() {
        let sort = map_sort("sort D = Nat; E = D; map f: E;", "f");
        assert_eq!(sort, SortExpression::Simple(Sort::Nat));
    }

    #[test]
    fn test_alias_inside_container_is_expanded() {
        // `f: List(D)` with `D = Nat` normalizes to `List(Nat)`.
        let sort = map_sort("sort D = Nat; map f: List(D);", "f");
        let SortExpression::Complex(op, subsort) = sort else {
            panic!("expected a container sort, got {sort:?}");
        };
        assert_eq!(op, merc_syntax::ComplexSort::List);
        assert_eq!(*subsort, SortExpression::Simple(Sort::Nat));
    }

    #[test]
    fn test_structured_alias_keeps_its_name() {
        // A structured sort is its own representative, so `f: D` stays `D`
        // rather than being replaced by the (recursive) struct body.
        let sort = map_sort("sort D = struct a | b; map f: D;", "f");
        let SortExpression::Resolved(name, _) = sort else {
            panic!("expected a resolved nominal sort, got {sort:?}");
        };
        assert_eq!(name, "D");
    }

    #[test]
    fn test_chained_struct_alias_shares_representative() {
        // `A = B` chains to the structured sort `B`, so `f: A` and `g: B`
        // normalize to the same named representative rather than diverging.
        let text = "sort A = B; B = struct c; map f: A; g: B;";
        let a = map_sort(text, "f");
        let b = map_sort(text, "g");
        assert_eq!(a, b);
        let SortExpression::Resolved(name, _) = a else {
            panic!("expected a resolved nominal sort, got {a:?}");
        };
        assert_eq!(name, "B");
    }

    #[test]
    fn test_recursive_alias_through_inline_struct_terminates() {
        // `D` recurses into itself through an inline `struct`, which
        // check_aliases permits (it stops at every struct); normalization must
        // keep the back-reference named rather than unfold it forever.
        let sort = map_sort("sort D = List(struct f(D)); map g: D;", "g");
        let SortExpression::Complex(op, _) = sort else {
            panic!("expected a List container, got {sort:?}");
        };
        assert_eq!(op, merc_syntax::ComplexSort::List);
    }
}
