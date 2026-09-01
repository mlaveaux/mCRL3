use merc_aterm::ATerm;
use merc_aterm::ATermRef;
use merc_aterm::Protected;
use merc_aterm::Term;
use merc_aterm::storage::ThreadTermPool;

pub type SubstitutionBuilder = Protected<Vec<ATermRef<'static>>>;

/// Creates a new term with the subterm at position `p` (a slice of 1-indexed indices) replaced by
/// `new_subterm`.
///
/// The term is rebuilt bottom-up: e.g. replacing `a` with `0` in `s(s(a))` at position `[1, 1]`
/// first constructs `s(0)`, then `s(s(0))`.
pub fn substitute<'a, 'b, T: Term<'a, 'b>>(tp: &ThreadTermPool, t: &'b T, new_subterm: ATerm, p: &[usize]) -> ATerm {
    let mut args = Protected::new(vec![]);
    substitute_rec(tp, t, new_subterm, p, &mut args, 0)
}

pub fn substitute_with<'a, 'b, T: Term<'a, 'b>>(
    builder: &mut SubstitutionBuilder,
    tp: &ThreadTermPool,
    t: &'b T,
    new_subterm: ATerm,
    p: &[usize],
) -> ATerm {
    substitute_rec(tp, t, new_subterm, p, builder, 0)
}

/// The recursive implementation for [substitute] and [substitute_with]. `depth` tracks the depth
/// reached in `t` so far and must be `0` on the initial call.
fn substitute_rec<'a, 'b, T: Term<'a, 'b>>(
    tp: &ThreadTermPool,
    t: &'b T,
    new_subterm: ATerm,
    p: &[usize],
    args: &mut SubstitutionBuilder,
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
                // Safety: t is pushed into the container on the next line.
                let t = unsafe { write_args.protect(&new_child) };
                write_args.push(t);
            } else {
                // Safety: t is pushed into the container on the next line.
                let t = unsafe { write_args.protect(&arg) };
                write_args.push(t);
            }
        }

        let result = tp.create_term(&t.get_head_symbol(), &write_args);
        drop(write_args);

        // Clear the args buffer for reuse.
        args.write().clear();
        result.protect()
    }
}

#[cfg(test)]
mod tests {
    use merc_aterm::ATerm;
    use merc_aterm::Term;
    use merc_aterm::storage::THREAD_TERM_POOL;

    use crate::utilities::ExplicitPosition;
    use crate::utilities::PositionIndexed;

    use super::substitute;

    #[test]
    fn test_substitute() {
        let t = ATerm::from_string("s(s(a))").unwrap();
        let t0 = ATerm::from_string("0").unwrap();

        // substitute the a for 0 in the term s(s(a))
        let result = THREAD_TERM_POOL.with(|tp| substitute(tp, &t, t0.clone(), &[1, 1]));

        // Check that indeed the new term as a 0 at position 1.1.
        assert_eq!(t0, result.get_position(&ExplicitPosition::new(&[1, 1])).protect());
    }
}
