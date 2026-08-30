//! A semantic-aware reparse pass that runs *before* type checking (see
//! [`reparse_process_specification`]): mCRL2's concrete grammar shares tokens between the
//! process algebra and the data language — most notably `.` (process sequential composition vs.
//! the data "at"/indexing operator) and `+` (process choice vs. data addition) — and a
//! context-free parser always prefers the greedier data-expression reading. The only place this
//! actually bites is a [`ProcessExprKind::Condition`]'s `condition` field: it is the one
//! `DataExpr` slot in the process grammar with no delimiter (no brackets, no `[...]`) bounding
//! how far it can extend, so `merc_syntax`'s grammar happily keeps consuming `.`/`+` as data
//! operators straight past the point where the concrete syntax actually meant to hand control
//! back to the process algebra.
//!
//! Concretely, `act(args) . cond -> P <> Q` parses `act(args) . cond` as one `DataExprAt` chain
//! rather than an action step followed by the real condition, and `cond1 -> P + cond2 -> Q`
//! parses as one `Condition` nested inside another, with the second `->`'s condition folded into
//! the first's as `Binary { op: Add, lhs: P, rhs: cond2 }` — see this module's tests for the
//! exact shapes.
//!
//! Recovering the intended parse needs to know which identifiers are declared as actions or
//! processes — genuinely semantic information, but only *name* resolution, not full type
//! checking: mCRL2 itself resolves this ambiguity the same way, before type checking runs at
//! all (the same grammar sharing exists in `mCRL2`'s own concrete syntax). So this pass runs
//! first, using nothing but the declared action/process *names* (arity and sort don't matter —
//! even an overloaded name is unambiguous as a name, and overload resolution itself still
//! happens later, during type checking in [`super::check`]), and rewrites every misparsed
//! `Condition` it finds back into the `ProcessExpr` shape — `Sequence`/`Choice`, `Action` steps —
//! it should have parsed as in the first place. [`super::check`]'s walk can then assume every
//! `Condition` it sees is already correctly shaped, and needs no error-driven recovery of its
//! own.
//!
//! [`merc_syntax::Traverse::apply_mut`] drives the walk (see [`reparse_mut`]): only [`Condition`]
//! is special-cased, in [`fix_swallow`]; every other `ProcessExprKind` variant's own recursion
//! into its children comes for free from `Traverse`'s generated per-type implementation, rather
//! than a hand-matched arm per variant here.
//!
//! [`Condition`]: ProcessExprKind::Condition

use std::collections::HashSet;
use std::convert::Infallible;

use merc_syntax::DataExpr;
use merc_syntax::DataExprBinaryOp;
use merc_syntax::DataExprKind;
use merc_syntax::ProcExprBinaryOp;
use merc_syntax::ProcessExpr;
use merc_syntax::ProcessExprKind;
use merc_syntax::Span;
use merc_syntax::Traverse;
use merc_syntax::UntypedProcessSpecification;

/// Every name declared as an action or a process, gathered once up front.
struct Names(HashSet<String>);

impl Names {
    fn build(spec: &UntypedProcessSpecification) -> Self {
        let mut names = HashSet::new();
        names.extend(spec.action_declarations.iter().map(|decl| decl.identifier.clone()));
        names.extend(spec.process_declarations.iter().map(|decl| decl.identifier.clone()));
        Names(names)
    }

    fn contains(&self, name: &str) -> bool {
        self.0.contains(name)
    }
}

/// Rewrites every `proc` body and `init` in `spec` in place, fixing every misparsed `Condition`
/// this module's doc comment describes. Idempotent: running it again on an already-fixed
/// specification finds nothing left to rewrite.
pub(super) fn reparse_process_specification(spec: &mut UntypedProcessSpecification) {
    let names = Names::build(spec);
    for decl in &mut spec.process_declarations {
        reparse_mut(&names, &mut decl.body);
    }
    if let Some(init) = &mut spec.init {
        reparse_mut(&names, init);
    }
}

/// Recursively rewrites `expr` in place: [`merc_syntax::Traverse::apply_mut`] provides the
/// descent into every non-`Condition` `ProcessExpr` variant's own children (`Sum`/`Dist`'s
/// `operand`, `Binary`'s `lhs`/`rhs`, …) generically — [`fix_swallow`] only has to know about
/// `Condition` — top-down, so it always sees a not-yet-rewritten `then`/`else_` in its original,
/// as-parsed shape, which the swallow it looks for needs (see [`fix_condition`]'s doc comment for
/// why: this can't be a bottom-up rewrite like [`merc_syntax::Traverse::transform`], since a
/// child already rewritten in isolation no longer has the raw shape the swallow lives in).
fn reparse_mut(names: &Names, expr: &mut ProcessExpr) {
    match expr.apply_mut::<Infallible, _>(|node| Ok(fix_swallow(names, node))) {
        Ok(()) => {}
        Err(error) => match error {},
    }
}

/// See [`reparse_mut`]; for call sites (inside [`fix_condition`] and friends) that have an owned
/// `ProcessExpr` rather than a `&mut` one already in hand.
fn reparse(names: &Names, mut expr: ProcessExpr) -> ProcessExpr {
    reparse_mut(names, &mut expr);
    expr
}

/// The [`Traverse::apply_mut`] callback [`reparse_mut`] drives: `None` for every `ProcessExpr`
/// variant other than `Condition`, letting `Traverse`'s own generated recursion continue into
/// that node's children unchanged. For a `Condition`, always clones and fully re-derives it via
/// [`fix_condition`] — cheaper to reason about (one path, matching [`fix_condition`]'s logic by
/// construction) than a hand-rolled "does this actually need fixing" fast path that risks quietly
/// drifting out of sync with it; a condition is rarely a large subtree, so the clone is cheap in
/// practice. The replacement is not itself re-descended into by `Traverse` (see its doc comment),
/// which is correct here: [`fix_condition`] already resolves the whole thing recursively itself.
fn fix_swallow(names: &Names, node: &ProcessExpr) -> Option<ProcessExpr> {
    let ProcessExprKind::Condition { condition, then, else_ } = &node.node else {
        return None;
    };
    Some(fix_condition(names, condition.clone(), (**then).clone(), else_.clone()))
}

/// Fixes one `Condition` node. `then` is not yet reparsed — this function recurses into it
/// itself, since the `+` swallow (see the module doc comment) needs to see it in its raw,
/// as-parsed shape to recognize it: a bare `cond -> then` (no `<>`) leaves `then` unrestricted, so
/// a further `+ cond2 -> ...` can swallow straight into it (`take_swallow`'s job). `else_`, when
/// present, is reparsed normally and re-attached to whichever clause `condition`/`then` end up
/// belonging to — it is *not* itself a place this swallow can start (an explicit `<>` always
/// closes the construct it belongs to), but it must never be dropped, including when `then` did
/// hide a swallow: a nested `if` inside `then` can have its own, unrelated swallow regardless of
/// whether the outer construct has an `else_` of its own.
///
/// A leading `.`-prefix peeled off `condition` (see [`peel_dot_prefix`]) sequences *before* the
/// whole (possibly `+`-split) construct, not inside its `then`: `a(1) . cond -> P` means
/// `a(1) . (cond -> P)` — the action happens unconditionally, then `cond -> P` follows — not `cond
/// -> (a(1) . P)`.
///
/// [`Choice`]: ProcExprBinaryOp::Choice
fn fix_condition(names: &Names, condition: DataExpr, then: ProcessExpr, else_: Option<Box<ProcessExpr>>) -> ProcessExpr {
    // Peel `condition` itself: any leading, fully-unconditional `+`-branches first (a whole run
    // with no `->` of its own at all, up to the *one* `->` this construct actually has — see
    // `peel_condition`), then a leading `.`-prefix off whatever's left (the plain `.` case).
    let (branches, prefix, condition) = peel_condition(names, condition);
    let else_ = else_.map(|boxed| Box::new(reparse(names, *boxed)));

    let result = match take_swallow(names, then) {
        Ok((this_then, rest)) => {
            // `then` hid a further `+`-separated clause (see `take_swallow`). `else_` belongs to
            // *this* clause, not `rest` — an explicit `<>` always closes the construct it was
            // written on, never a clause a swallow only revealed after the fact.
            let this_span = Span { start: condition.span.start, end: this_then.span.end };
            let this_clause =
                ProcessExprKind::Condition { condition, then: Box::new(this_then), else_ }.spanned(this_span.clone());
            let choice_span = Span { start: this_span.start, end: rest.span.end };
            ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, lhs: Box::new(this_clause), rhs: Box::new(rest) }.spanned(choice_span)
        }
        Err(then) => {
            let this_then = reparse(names, *then);
            let this_span = Span { start: condition.span.start, end: this_then.span.end };
            let this_span = match &else_ {
                Some(else_) => Span { end: else_.span.end, ..this_span },
                None => this_span,
            };
            ProcessExprKind::Condition { condition, then: Box::new(this_then), else_ }.spanned(this_span)
        }
    };

    let result = prepend_seq(prefix, result);

    // Re-attach the leading unconditional branches peeled off above, each `Choice`d in front —
    // `r_assign_turnA.Turn(A) + r_assign_turnB.Turn(B) + (t == A) -> ...` (`mutex.mcrl2`'s `Turn`)
    // has no `->` at all before its first two `+`s, so its *whole* run up to the process's one
    // and only `->` swallows into this single `Condition`'s `condition` in one shot, rather than
    // cascading through nested `Condition`s the way `take_swallow`'s two shapes do.
    branches.into_iter().rev().fold(result, |acc, branch| {
        let span = Span { start: branch.span.start, end: acc.span.end };
        ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, lhs: Box::new(branch), rhs: Box::new(acc) }.spanned(span)
    })
}

/// Peels a leading run of fully-unconditional `+`-branches off `condition` (see the comment on
/// [`fix_condition`]'s call site), then — per [`peel_dot_prefix`] — a leading `.`-prefix off
/// whatever's left. Returns `(branches, dot_prefix, real_condition)`; `branches` is empty in the
/// overwhelmingly common case (nothing to peel at this level at all).
fn peel_condition(names: &Names, condition: DataExpr) -> (Vec<ProcessExpr>, Vec<DataExpr>, DataExpr) {
    let mut branches = Vec::new();
    let mut current = condition;
    loop {
        let is_branch = matches!(&current.node,
            DataExprKind::Binary { op: DataExprBinaryOp::Add, lhs, .. } if is_fully_process_content(names, lhs));
        if !is_branch {
            break;
        }
        let DataExprKind::Binary { op: DataExprBinaryOp::Add, lhs, rhs } = current.node else {
            unreachable!("just matched this shape above");
        };
        branches.push(reinterpret_as_process(*lhs));
        current = *rhs;
    }
    let (prefix, remaining) = peel_dot_prefix(names, current);
    (branches, prefix, remaining)
}

/// If `node` is `Condition { condition: Binary { op: Add, lhs, rhs }, then: inner_then, else_:
/// inner_else }` with `lhs` pure declared process content (see [`is_fully_process_content`]),
/// fixes and extracts it as `Ok((this_clause_content, rest))` — `rest` fully reparsed and ready to
/// use as-is. Otherwise hands `node` straight back, boxed, as `Err` (boxed only to keep this
/// `Result` from ballooning to `ProcessExprKind`'s own size — clippy's `result_large_err`).
///
/// This is the one shape the swallow (see the module doc comment) can actually produce as `then`:
/// the second `->` in `cond -> lhs + rhs -> inner_then <...>` was swallowed *inside* the
/// shared-token `DataExpr` grammar, which only ever happens through `condition`. A bare
/// `Binary { op: Choice, .. }` sitting directly as `then` is deliberately **not** treated as a
/// swallow here, even though it looks superficially similar: `cond -> a + b` and `cond -> (a + b)`
/// produce byte-identical trees (`ProcExprBrackets` adds no node of its own), so nothing here can
/// tell a user's own, already-correct `a + b` apart from one that supposedly needs splitting —
/// treating every such `Choice` as swallowed silently discarded a trailing `<>` branch and
/// rewrote deliberately parenthesized choices into a different program.
fn take_swallow(names: &Names, node: ProcessExpr) -> Result<(ProcessExpr, ProcessExpr), Box<ProcessExpr>> {
    let is_add_swallow = if let ProcessExprKind::Condition { condition, .. } = &node.node
        && let DataExprKind::Binary { op: DataExprBinaryOp::Add, lhs, .. } = &condition.node
    {
        is_fully_process_content(names, lhs)
    } else {
        false
    };
    if !is_add_swallow {
        return Err(Box::new(node));
    }

    let ProcessExprKind::Condition { condition, then: inner_then, else_: inner_else } = node.node else {
        unreachable!("just matched this shape above");
    };
    let DataExprKind::Binary { op: DataExprBinaryOp::Add, lhs, rhs } = condition.node else {
        unreachable!("just matched this shape above");
    };
    let this = reinterpret_as_process(*lhs);
    let rest = fix_condition(names, *rhs, *inner_then, inner_else);
    Ok((this, rest))
}

/// Peels a leading run of declared-action/process steps off `condition`'s left-associative `.`
/// chain, returning `(steps, remaining_condition)` — an empty `steps` and `condition` unchanged
/// if no leading step is found.
fn peel_dot_prefix(names: &Names, condition: DataExpr) -> (Vec<DataExpr>, DataExpr) {
    let mut pieces = flatten_at_chain(condition).into_iter();
    let mut prefix = Vec::new();
    for piece in pieces.by_ref() {
        if is_declared_step(names, &piece) {
            prefix.push(piece);
        } else {
            let rest: Vec<DataExpr> = std::iter::once(piece).chain(pieces).collect();
            return (prefix, rebuild_at_chain(rest));
        }
    }
    // Every piece looked like a declared step: there's no real boolean condition left at all.
    // Pathological (the resulting specification won't type check either way) — leave it alone.
    (Vec::new(), rebuild_at_chain(prefix))
}

/// Flattens a left-associative `Binary { op: At, lhs, rhs }` chain (`a . b . c` parses as
/// `(a . b) . c`) back into its original left-to-right pieces `[a, b, c]`. An `expr` whose
/// top-level node isn't `At` at all flattens to the single piece `[expr]`.
fn flatten_at_chain(expr: DataExpr) -> Vec<DataExpr> {
    let mut parts = Vec::new();
    let mut current = expr;
    loop {
        match current.node {
            DataExprKind::Binary { op: DataExprBinaryOp::At, lhs, rhs } => {
                parts.push(*rhs);
                current = *lhs;
            }
            _ => {
                parts.push(current);
                break;
            }
        }
    }
    parts.reverse();
    parts
}

/// The inverse of [`flatten_at_chain`]: left-folds `pieces` back into one left-associative `At`
/// chain. Panics on an empty `pieces` — never called with one.
fn rebuild_at_chain(pieces: Vec<DataExpr>) -> DataExpr {
    let mut pieces = pieces.into_iter();
    let mut result = pieces.next().expect("rebuild_at_chain is never called with an empty piece list");
    for piece in pieces {
        let span = Span { start: result.span.start, end: piece.span.end };
        result = DataExprKind::Binary { op: DataExprBinaryOp::At, lhs: Box::new(result), rhs: Box::new(piece) }.spanned(span);
    }
    result
}

/// Whether `piece` is a bare identifier or application naming a declared action or process, or
/// `delta`/`tau` (the two nullary process constants — reserved words in the process algebra, but
/// not excluded from the data grammar's own `Id` token, so a swallowed `delta`/`tau` parses as an
/// ordinary [`DataExprKind::Id`] rather than [`merc_syntax::ProcessExprKind::Delta`]/`Tau`) — the
/// leaf shape [`reinterpret_as_process`] turns into a process step.
fn is_declared_step(names: &Names, piece: &DataExpr) -> bool {
    match &piece.node {
        DataExprKind::Id(name) => names.contains(name) || is_process_constant(name),
        DataExprKind::Application { function, .. } => {
            matches!(&function.node, DataExprKind::Id(name) if names.contains(name))
        }
        _ => false,
    }
}

/// Whether `name` is one of mCRL2's nullary process constants — see [`is_declared_step`].
fn is_process_constant(name: &str) -> bool {
    name == "delta" || name == "tau"
}

/// Whether every leaf of `expr`'s `.`/`+` chain is a declared action/process step — the
/// precondition [`reinterpret_as_process`] relies on.
fn is_fully_process_content(names: &Names, expr: &DataExpr) -> bool {
    match &expr.node {
        DataExprKind::Binary { op: DataExprBinaryOp::At | DataExprBinaryOp::Add, lhs, rhs } => {
            is_fully_process_content(names, lhs) && is_fully_process_content(names, rhs)
        }
        _ => is_declared_step(names, expr),
    }
}

/// Converts `expr` — confirmed by [`is_fully_process_content`]/[`is_declared_step`] to be pure
/// process content — into the `ProcessExpr` it should have parsed as: `.`/`+` become
/// `Sequence`/`Choice`, and each `Id`/`Application` leaf becomes an `Action` step. The grammar
/// hands out `ProcessExprKind::Action` for both an action instance and a positional process
/// instantiation (see the crate README); [`super::check`] resolves which table it belongs to.
fn reinterpret_as_process(expr: DataExpr) -> ProcessExpr {
    let span = expr.span.clone();
    match expr.node {
        DataExprKind::Binary { op: DataExprBinaryOp::At, lhs, rhs } => ProcessExprKind::Binary {
            op: ProcExprBinaryOp::Sequence,
            lhs: Box::new(reinterpret_as_process(*lhs)),
            rhs: Box::new(reinterpret_as_process(*rhs)),
        }
        .spanned(span),
        DataExprKind::Binary { op: DataExprBinaryOp::Add, lhs, rhs } => ProcessExprKind::Binary {
            op: ProcExprBinaryOp::Choice,
            lhs: Box::new(reinterpret_as_process(*lhs)),
            rhs: Box::new(reinterpret_as_process(*rhs)),
        }
        .spanned(span),
        DataExprKind::Application { function, arguments } => {
            let DataExprKind::Id(name) = function.node else {
                unreachable!("is_fully_process_content only accepts an Id-headed application");
            };
            ProcessExprKind::Action(name, arguments).spanned(span)
        }
        DataExprKind::Id(name) if name == "delta" => ProcessExprKind::Delta.spanned(span),
        DataExprKind::Id(name) if name == "tau" => ProcessExprKind::Tau.spanned(span),
        DataExprKind::Id(name) => ProcessExprKind::Action(name, Vec::new()).spanned(span),
        _ => unreachable!("is_fully_process_content only accepts At/Add chains of Id/Application leaves"),
    }
}

/// Sequences `prefix` (in original left-to-right order) before `tail`; an empty `prefix` folds
/// away to just `tail`.
fn prepend_seq(prefix: Vec<DataExpr>, tail: ProcessExpr) -> ProcessExpr {
    prefix.into_iter().rev().fold(tail, |acc, piece| {
        let step = reinterpret_as_process(piece);
        let span = Span { start: step.span.start, end: acc.span.end };
        ProcessExprKind::Binary { op: ProcExprBinaryOp::Sequence, lhs: Box::new(step), rhs: Box::new(acc) }.spanned(span)
    })
}

#[cfg(test)]
mod tests {
    use merc_syntax::UntypedProcessSpecification;

    use super::*;

    fn reparsed_body(text: &str) -> ProcessExpr {
        let mut spec = UntypedProcessSpecification::parse(text).expect("the fixture should parse");
        reparse_process_specification(&mut spec);
        spec.init.take().expect("the fixture always has an init")
    }

    fn reparsed_proc_body(text: &str) -> ProcessExpr {
        let mut spec = UntypedProcessSpecification::parse(text).expect("the fixture should parse");
        reparse_process_specification(&mut spec);
        let decl = spec.process_declarations.pop().expect("the fixture always has a proc");
        decl.body
    }

    /// A single leading action step swallowed into the condition via `.` — the classic
    /// `act(args) . cond -> P <> Q` shape.
    #[test]
    fn dot_swallowed_action_prefix_is_recovered() {
        let body = reparsed_body("act a: Nat; init a(1) . true -> delta;");
        let ProcessExprKind::Binary { op: ProcExprBinaryOp::Sequence, lhs, rhs } = &body.node else {
            panic!("expected a Sequence, got {body:?}");
        };
        assert!(matches!(&lhs.node, ProcessExprKind::Action(name, _) if name == "a"));
        assert!(matches!(&rhs.node, ProcessExprKind::Condition { .. }));
    }

    /// A `+`-separated chain of guarded actions, each swallowed into the previous clause's
    /// condition — the shape [`super::check`]'s old error-driven recovery could not fix.
    #[test]
    fn choice_swallowed_guarded_actions_are_recovered() {
        let body = reparsed_body("act a: Nat; b: Nat; init (true) -> a(1) + (false) -> b(2);");
        let ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, lhs, rhs } = &body.node else {
            panic!("expected a Choice, got {body:?}");
        };
        let ProcessExprKind::Condition { then: lhs_then, .. } = &lhs.node else {
            panic!("expected a Condition, got {lhs:?}");
        };
        assert!(matches!(&lhs_then.node, ProcessExprKind::Action(name, _) if name == "a"));
        let ProcessExprKind::Condition { then: rhs_then, .. } = &rhs.node else {
            panic!("expected a Condition, got {rhs:?}");
        };
        assert!(matches!(&rhs_then.node, ProcessExprKind::Action(name, _) if name == "b"));
    }

    /// A three-way `+`-chain, each clause's action itself preceded by a `.`-sequence — combines
    /// both swallow shapes at once.
    #[test]
    fn three_way_choice_with_sequenced_actions_is_recovered() {
        let body = reparsed_body(
            "act a: Nat; b: Nat; c: Nat; init (true) -> a(1) . b(1) + (false) -> a(2) . b(2) + (true) -> c(3);",
        );
        let ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, lhs, rhs } = &body.node else {
            panic!("expected a Choice, got {body:?}");
        };
        let ProcessExprKind::Condition { then: first_then, .. } = &lhs.node else {
            panic!("expected a Condition, got {lhs:?}");
        };
        assert!(matches!(&first_then.node, ProcessExprKind::Binary { op: ProcExprBinaryOp::Sequence, .. }));

        let ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, lhs: second, rhs: third } = &rhs.node else {
            panic!("expected a nested Choice, got {rhs:?}");
        };
        let ProcessExprKind::Condition { then: second_then, .. } = &second.node else {
            panic!("expected a Condition, got {second:?}");
        };
        assert!(matches!(&second_then.node, ProcessExprKind::Binary { op: ProcExprBinaryOp::Sequence, .. }));
        let ProcessExprKind::Condition { then: third_then, .. } = &third.node else {
            panic!("expected a Condition, got {third:?}");
        };
        assert!(matches!(&third_then.node, ProcessExprKind::Action(name, _) if name == "c"));
    }

    /// A `+`-separated chain in `else_` position — `cond -> then <> a + cond2 -> then2 <> else2`,
    /// mirroring `knights.mcrl2`'s `(...) -> jump . X(...) <> delta + (f==finalBoard) -> ready .
    /// delta <> delta`. This module used to (incorrectly) split constructs shaped like this: a
    /// bare `Binary { op: Choice, .. }` sitting directly in `then`/`else_` position looks the same
    /// whether it is a genuine, deliberate `a + b` the user wrote (`ProcExprBrackets` adds no node
    /// of its own, so `cond -> (a + b)` and `cond -> a + b` parse identically) or a swallowed
    /// clause — nothing in the tree distinguishes the two, so "splitting" a deliberate `a + b`
    /// silently rewrote it into a different program and dropped a trailing `<>` branch entirely.
    /// This case is now deliberately left alone — see [`take_swallow`]'s doc comment.
    #[test]
    fn choice_directly_in_else_position_is_left_alone() {
        let body = reparsed_proc_body(
            "act jump, ready; proc X(f: Bool) = (f) -> jump . X(f) <> delta + (!f) -> ready . delta <> delta; init X(true);",
        );
        let ProcessExprKind::Condition { then, else_, .. } = &body.node else {
            panic!("expected a Condition, got {body:?}");
        };
        assert!(matches!(&then.node, ProcessExprKind::Binary { op: ProcExprBinaryOp::Sequence, .. }));
        let Some(else_) = else_ else {
            panic!("expected an else_ branch, got None");
        };
        assert!(matches!(&else_.node, ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, .. }));
    }

    /// Minimal form of the same bug: `then` hiding an unrelated (`Add`-shaped) swallow must never
    /// cost the current clause its own `<>`. `(ps==1) -> a(1) + (ps==2) -> a(2) <> b(9)` puts the
    /// swallow (`a(1) + (ps==2) -> ...`) one level down from the outermost condition, and the
    /// grammar attaches `<>`'s `b(9)` to the *innermost* reached `->` (`(ps==2)`, not `(ps==1)`) —
    /// empirically confirmed, not derived from the grammar file, since this corner of it is easy
    /// to get wrong by inspection alone (see the module's git history). `else_` still must not be
    /// dropped by `take_swallow`'s `Ok` arm, wherever the grammar actually attaches it.
    #[test]
    fn else_branch_survives_a_swallow_one_level_down() {
        let body = reparsed_body("act a: Nat; b: Nat; init (true) -> a(1) + (false) -> a(2) <> b(9);");
        let ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, lhs: first, rhs: second } = &body.node else {
            panic!("expected a Choice, got {body:?}");
        };
        let ProcessExprKind::Condition { else_: first_else, .. } = &first.node else {
            panic!("expected a Condition, got {first:?}");
        };
        assert!(first_else.is_none(), "the swallow consumes the only `<>` in this input");
        let ProcessExprKind::Condition { else_: second_else, .. } = &second.node else {
            panic!("expected a Condition, got {second:?}");
        };
        let Some(second_else) = second_else else {
            panic!("expected `<> b(9)` to survive on the clause the grammar actually attaches it to");
        };
        assert!(matches!(&second_else.node, ProcessExprKind::Action(name, _) if name == "b"));
    }

    /// The minimal shape of the same bug class as [`choice_directly_in_else_position_is_left_alone`],
    /// with `then` (not `else_`) directly `a + b`: `cond -> a + b <> c` must keep `b` unconditional
    /// (not folded into `cond`'s own guard) *and* keep `c` as the `else_`.
    #[test]
    fn choice_directly_in_then_position_keeps_its_own_else() {
        let body = reparsed_body("act a: Nat; b: Nat; c: Nat; init (true) -> a(1) + b(2) <> c(3);");
        let ProcessExprKind::Condition { condition, then, else_ } = &body.node else {
            panic!("expected a Condition, got {body:?}");
        };
        assert!(matches!(&condition.node, DataExprKind::Bool(true)));
        assert!(matches!(&then.node, ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, .. }), "a + b must stay intact, got {then:?}");
        let Some(else_) = else_ else {
            panic!("expected `<> c(3)` to survive, got None");
        };
        assert!(matches!(&else_.node, ProcessExprKind::Action(name, _) if name == "c"));
    }

    /// Leading, wholly *unconditional* `+`-branches (no `->` of their own at all) swallowed
    /// straight into the one and only `Condition`'s `condition` field — the shape produced when a
    /// process body has just one `->` in it, so the *whole* run up to that point is one legal
    /// `DataExpr`. Mirrors `mutex.mcrl2`'s `Turn`: `r_assign_turnA.Turn(A) + r_assign_turnB.Turn(B)
    /// + (t==A) -> s_read_turnA|label(true).Turn(A)`.
    #[test]
    fn unconditional_leading_branches_are_recovered() {
        let body = reparsed_proc_body("act a, b, c; proc P = a.P + b.P + (true) -> c.P; init P;");
        // The two unconditional branches (`a.P`, `b.P`) peel off together, in one `Add`-chain
        // piece, as a nested `Choice` of their own; the real, guarded clause (`c.P`) is the
        // right-hand side.
        let ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, lhs: branches, rhs: guarded } = &body.node else {
            panic!("expected a Choice, got {body:?}");
        };
        let ProcessExprKind::Binary { op: ProcExprBinaryOp::Choice, lhs: first, rhs: second } = &branches.node else {
            panic!("expected a nested Choice, got {branches:?}");
        };
        assert!(matches!(&first.node, ProcessExprKind::Binary { op: ProcExprBinaryOp::Sequence, .. }));
        assert!(matches!(&second.node, ProcessExprKind::Binary { op: ProcExprBinaryOp::Sequence, .. }));
        let ProcessExprKind::Condition { condition, then, else_ } = &guarded.node else {
            panic!("expected a Condition, got {guarded:?}");
        };
        assert!(matches!(&condition.node, DataExprKind::Bool(true)));
        assert!(matches!(&then.node, ProcessExprKind::Binary { op: ProcExprBinaryOp::Sequence, .. }));
        assert!(else_.is_none());
    }

    /// A genuine data expression that merely happens to use `.`/`+` — nothing declared as an
    /// action or process appears in it — must be left untouched.
    #[test]
    fn a_genuine_condition_using_at_and_plus_is_left_alone() {
        let body = reparsed_body("sort L = List(Nat); init (([1,2,3] . 0) + 1 == 2) -> delta;");
        assert!(matches!(&body.node, ProcessExprKind::Condition { .. }));
    }

    /// Nothing to fix at all: a plain, already-well-formed condition is unchanged.
    #[test]
    fn an_ordinary_condition_is_left_alone() {
        let body = reparsed_body("init true -> delta;");
        let ProcessExprKind::Condition { condition, then, else_ } = &body.node else {
            panic!("expected a Condition, got {body:?}");
        };
        assert!(matches!(&condition.node, DataExprKind::Bool(true)));
        assert!(matches!(&then.node, ProcessExprKind::Delta));
        assert!(else_.is_none());
    }
}
