use ahash::AHashSet;
use mcrl2::aterm::ATerm;
use mcrl2::aterm::ATermRef;
use mcrl2::aterm::Protected;
use mcrl2::aterm::TermBuilder;
use mcrl2::aterm::TermPool;
use mcrl2::aterm::Yield;
use mcrl2::data::DataExpression;
use mcrl2::data::DataFunctionSymbol;
use mcrl2::data::DataVariable;

pub type SubstitutionBuilder = Protected<Vec<ATermRef<'static>>>;

/// Creates a new term where a subterm is replaced with another term.
///
/// # Parameters
/// 't'             -   The original term
/// 'new_subterm'   -   The subterm that will be injected
/// 'p'             -   The place in 't' on which 'new_subterm' will be placed,
///                     given as a slice of position indexes
///
/// # Example
///
/// The term is constructed bottom up. As an an example take the term s(s(a)).
/// Lets say we want to replace the a with the term 0. Then we traverse the term
/// until we have arrived at a and replace it with 0. We then construct s(0)
/// and then construct s(s(0)).
pub fn substitute(tp: &mut TermPool, t: &ATermRef<'_>, new_subterm: ATerm, p: &[usize]) -> ATerm {
    let mut args = Protected::new(vec![]);
    substitute_rec(tp, t, new_subterm, p, &mut args, 0)
}

pub fn substitute_with(
    builder: &mut SubstitutionBuilder,
    tp: &mut TermPool,
    t: &ATermRef<'_>,
    new_subterm: ATerm,
    p: &[usize],
) -> ATerm {
    substitute_rec(tp, t, new_subterm, p, builder, 0)
}

/// The recursive implementation for subsitute
///
/// 'depth'         -   Used to keep track of the depth in 't'. Function should be called with
///                     'depth' = 0.
fn substitute_rec(
    tp: &mut TermPool,
    t: &ATermRef<'_>,
    new_subterm: ATerm,
    p: &[usize],
    args: &mut Protected<Vec<ATermRef<'static>>>,
    depth: usize,
) -> ATerm {
    if p.len() == depth {
        // in this case we have arrived at the place where 'new_subterm' needs to be injected
        new_subterm
    } else {
        // else recurse deeper into 't'
        let new_child_index = p[depth] - 1;
        let new_child = substitute_rec(tp, &t.arg(new_child_index), new_subterm, p, args, depth + 1);

        let mut write_args = args.write();
        for (index, arg) in t.arguments().enumerate() {
            if index == new_child_index {
                let t = write_args.protect(&new_child);
                write_args.push(t);
            } else {
                let t = write_args.protect(&arg);
                write_args.push(t);
            }
        }

        let result = tp.create(&t.get_head_symbol(), &write_args);
        drop(write_args);

        // TODO: When write is dropped we check whether all terms where inserted, but this clear violates that assumption.
        args.write().clear();
        result
    }
}

/// Converts an [ATerm] to an untyped data expression.
pub fn to_untyped_data_expression(tp: &mut TermPool, t: &ATerm, variables: &AHashSet<String>) -> DataExpression {
    let mut builder = TermBuilder::<ATerm, ATerm>::new();

    builder
        .evaluate(
            tp,
            t.clone(),
            |tp, args, t| {
                debug_assert!(!t.is_int(), "Term cannot be an aterm_int, although not sure why");

                if variables.contains(t.get_head_symbol().name()) {
                    // Convert a constant variable, for example 'x', into an untyped variable.
                    Ok(Yield::Term(DataVariable::new(tp, t.get_head_symbol().name()).into()))
                } else if t.get_head_symbol().arity() == 0 {
                    Ok(Yield::Term(
                        DataFunctionSymbol::new(tp, t.get_head_symbol().name()).into(),
                    ))
                } else {
                    // This is a function symbol applied to a number of arguments (higher order terms not allowed)
                    let head = DataFunctionSymbol::new(tp, t.get_head_symbol().name());

                    for arg in t.arguments() {
                        args.push(arg.protect());
                    }

                    Ok(Yield::Construct(head.into()))
                }
            },
            |tp, input, args| Ok(tp.create_data_application(&input, args)),
        )
        .unwrap()
        .into()
}

#[cfg(test)]
mod tests {
    use crate::utilities::ExplicitPosition;
    use crate::utilities::PositionIndexed;

    use super::*;

    #[test]
    fn test_substitute() {
        let mut term_pool = TermPool::new();

        let t = term_pool.from_string("s(s(a))").unwrap();
        let t0 = term_pool.from_string("0").unwrap();

        // substitute the a for 0 in the term s(s(a))
        let result = substitute(&mut term_pool, &t, t0.clone(), &vec![1, 1]);

        // Check that indeed the new term as a 0 at position 1.1.
        assert_eq!(t0, result.get_position(&ExplicitPosition::new(&vec![1, 1])).protect());
    }

    #[test]
    fn test_to_data_expression() {
        let mut term_pool = TermPool::new();

        let t = term_pool.from_string("s(s(a))").unwrap();

        let _expression = to_untyped_data_expression(&mut term_pool, &t, &AHashSet::from_iter(["a".to_string()]));
    }
}
