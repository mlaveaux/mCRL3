use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use log::debug;
use log::trace;

use merc_syntax::ComplexSort;
use merc_syntax::DataExpr;
use merc_syntax::DataExprKind;
use merc_syntax::EqnSpecId;
use merc_syntax::EquationId;
use merc_syntax::IdDecl;
use merc_syntax::Sort;
use merc_syntax::SortExpression;
use merc_syntax::Span;
use merc_syntax::UntypedDataSpecification;
use merc_utilities::TagIndex;

use crate::InferSort;
use crate::InferSortId;
use crate::POLYMORPHIC_SIGNATURE;
use crate::ResolvedSort;
use crate::ResolvedSortId;
use crate::Signature;
use crate::SortInterner;
use crate::TypeckContext;
use crate::Unifier;
use crate::display_sort;
use crate::is_lowered;
use crate::is_supported_binder_sort;
use crate::number_generality;
use crate::query_sort_of_equation_var;
use crate::resolve_sort;

/// A unique type for expression nodes within a single equation.
pub(crate) struct ExprTag;

/// Identifies an expression node of one equation.
///
/// Ids are assigned parents before children, and within an application the
/// arguments before the applied function (so the solver sees argument
/// constraints before the callee's overload disjunction), over the condition,
/// left-hand side and right-hand side in that order. Container literals number
/// their members in syntactic order (a bag member before its multiplicity); a
/// comprehension numbers only its predicate, and a `lambda`/`forall`/`exists`
/// only its body — the bound variables have no id, like the equation
/// variables. A `whr` numbers each assignment's right-hand side, in binding
/// order, before the body. Phase-4 lowering re-walks the same lowered AST, so
/// this numbering must stay deterministic.
pub(crate) type ExprId = TagIndex<usize, ExprTag>;

/// What a name (`Id` node) in an equation resolved to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameTarget {
    /// An equation variable of the enclosing `var` block.
    Variable,
    /// A declared constructor or mapping (user or system-defined) with the
    /// given overload sort.
    Op { sort: ResolvedSortId },
    /// A polymorphic built-in (`==`, `!=`, `<`, `<=`, `>`, `>=`, `if`), whose
    /// concrete sort follows from the inferred argument sorts.
    Builtin,
}

/// The Phase-3 typing result of a single equation. Every equation that type
/// checks is fully inferred: a binder over a sort inference cannot model (a
/// bare product sort, which is not a valid variable sort) is now a hard error
/// rather than a silently untyped equation.
#[derive(Debug)]
pub(crate) struct EquationTyping {
    /// The inferred sort of every expression node, indexed by [ExprId].
    pub(crate) sorts: Vec<ResolvedSortId>,
    /// The resolution of every name, keyed by the [ExprId] of its `Id` node.
    pub(crate) names: HashMap<ExprId, NameTarget>,
}

/// The errors of Phase-3 sort inference. `Clone` so a failure can be stored in
/// the query cache and reported again on later lookups.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum InferenceError {
    #[error("the name '{name}' is not declared")]
    UndeclaredName { name: String, span: Span },

    #[error("'{expr}' is applied to arguments, but cannot have a function sort")]
    NotAFunction { expr: String, span: Span },

    #[error("the condition '{condition}' cannot have sort Bool")]
    ConditionNotBool { condition: String, span: Span },

    #[error("the body '{body}' of a forall/exists must have sort Bool")]
    QuantifierNotBool { body: String, span: Span },

    #[error("the equation '{equation}' has no valid sort assignment")]
    NoTyping { equation: String, span: Span },

    #[error("the sorts in equation '{equation}' are ambiguous")]
    AmbiguousExpression { equation: String, span: Span },

    #[error("the sorts in equation '{equation}' are underdetermined")]
    UnderdeterminedSort { equation: String, span: Span },

    #[error("the binder sort '{sort}' in equation '{equation}' is not a valid variable sort")]
    InvalidBinderSort { sort: String, equation: String, span: Span },
}

impl InferenceError {
    /// The span of the offending sub-expression (or, for `NoTyping` /
    /// `AmbiguousExpression` / `UnderdeterminedSort`, the whole equation, since
    /// no narrower sub-expression can be blamed for those). Pair with
    /// [Span::render] to show a source snippet alongside the message.
    pub fn span(&self) -> &Span {
        match self {
            InferenceError::UndeclaredName { span, .. }
            | InferenceError::NotAFunction { span, .. }
            | InferenceError::ConditionNotBool { span, .. }
            | InferenceError::QuantifierNotBool { span, .. }
            | InferenceError::NoTyping { span, .. }
            | InferenceError::AmbiguousExpression { span, .. }
            | InferenceError::UnderdeterminedSort { span, .. }
            | InferenceError::InvalidBinderSort { span, .. } => span,
        }
    }
}

/// Returns the typing of one user equation, keyed by the id of its enclosing
/// equation specification block and its own id within that block (assigned by
/// [assign_declaration_ids](crate::assign_declaration_ids)). Memoized on
/// [TypeckContext::equation_typing].
pub(crate) fn query_equation_typing(
    ctx: &mut TypeckContext,
    spec: &UntypedDataSpecification,
    key: (EqnSpecId, EquationId),
) -> Result<Rc<EquationTyping>, InferenceError> {
    let (eqn_spec_id, equation_id) = key;

    // Checked before the cache lock: an out-of-range key would panic inside
    // `infer_equation` with the entry left `InProgress`, misreporting any
    // later identical query as a cyclic dependency.
    debug_assert!(
        spec.equation_declarations
            .get(*eqn_spec_id)
            .is_some_and(|eqn_spec| *equation_id < eqn_spec.equations.len()),
        "equation typing key {key:?} must index an equation of the specification"
    );

    match ctx
        .equation_typing
        .get_or_lock(key)
        .expect("equation typing does not depend on other equations")
    {
        Some(result) => result.clone(),
        None => {
            let result = infer_equation(ctx, spec, eqn_spec_id, equation_id).map(Rc::new);
            ctx.equation_typing.unlock(key, result).clone()
        }
    }
}

/// Infers the sorts of every user equation, positionally parallel to
/// `equation_declarations` (outer) and each equation list (inner). Phase-3
/// (constraint-based) inference does not run over the system-defined
/// equations — they are checked separately and more cheaply, by
/// `check_system_specification`'s structural well-formedness pass (debug
/// builds only) and by `lower_system_equations`'s own per-equation sort
/// propagation during lowering. Neither of those currently covers every
/// construct Phase-3 does (see `lower_system_equations`'s doc comment), so a
/// system equation using an unsupported construct is silently dropped from
/// the lowered output rather than rejected — this is a known gap, not an
/// intentional trust boundary, and needs a real fix (extend structural
/// lowering, or run Phase-3 over the system spec too).
pub(crate) fn check_equations(
    ctx: &mut TypeckContext,
    spec: &UntypedDataSpecification,
) -> Result<Vec<Vec<Rc<EquationTyping>>>, InferenceError> {
    let mut typings = Vec::with_capacity(spec.equation_declarations.len());
    for eqn_spec in &spec.equation_declarations {
        let eqn_spec_id = eqn_spec.id.expect("assign_declaration_ids ran before check_equations");
        let mut spec_typings = Vec::with_capacity(eqn_spec.equations.len());
        for equation in &eqn_spec.equations {
            let equation_id = equation.id.expect("assign_declaration_ids ran before check_equations");
            spec_typings.push(query_equation_typing(ctx, spec, (eqn_spec_id, equation_id))?);
        }
        typings.push(spec_typings);
    }
    Ok(typings)
}

/// Infers the sorts of a single equation: generates constraints over the
/// condition, left-hand side and right-hand side, solves them by ranked
/// backtracking, and extracts the sorts of the best solution.
///
/// The two sides need not have equal sorts, only a common supersort (either
/// side may be upcast, e.g. `eqn f = 1;` with `f: Nat`), so each side gets a
/// `Sub` constraint against a shared fresh variable.
fn infer_equation(
    ctx: &mut TypeckContext,
    spec: &UntypedDataSpecification,
    eqn_spec_id: EqnSpecId,
    equation_id: EquationId,
) -> Result<EquationTyping, InferenceError> {
    let eqn_spec = &spec.equation_declarations[eqn_spec_id];
    let equation = &eqn_spec.equations[equation_id];
    let equation_text = || format!("{} = {}", equation.lhs, equation.rhs);
    debug!("inference: typing equation '{}'", equation_text());

    let mut unifier = Unifier::new();

    // The equation variables shadow constructors and mappings on lookup; their
    // declared sorts are concrete, so all uses of a variable share one node.
    let mut variables = HashMap::new();
    for var in &eqn_spec.variables {
        let var_id = var.id.expect("assign_declaration_ids ran before check_equations");
        let sort = query_sort_of_equation_var(ctx, spec, eqn_spec_id, var_id);
        let node = unifier.resolved_node(sort);
        variables.insert(var.identifier.as_str(), node);
    }

    // The signatures are cloned out of the context (cheaply, behind `Rc`)
    // because the generator needs the context mutably: resolving a
    // comprehension's binder sort interns sorts and fills the sort-of-def
    // cache mid-walk.
    let signature = Rc::clone(ctx.signature.as_ref().expect("build_signature ran before inference"));
    let system_signature = Rc::clone(
        ctx.system_signature
            .as_ref()
            .expect("resolve_system_signature ran before inference"),
    );

    let mut generator = ConstraintGenerator {
        ctx: &mut *ctx,
        spec,
        signature,
        system_signature,
        variables,
        unifier: &mut unifier,
        expr_sorts: Vec::new(),
        expr_texts: Vec::new(),
        log_texts: log::log_enabled!(log::Level::Debug),
        names: HashMap::new(),
        constraints: Vec::new(),
    };

    match generator.generate(equation.condition.as_ref(), &equation.lhs, &equation.rhs) {
        Ok(()) => {}
        Err(GenFailure::InvalidBinderSort(sort, span)) => {
            debug!(
                "inference: rejected '{}', its binder sort '{sort}' is not a valid variable sort",
                equation_text()
            );
            return Err(InferenceError::InvalidBinderSort {
                sort,
                equation: equation_text(),
                span,
            });
        }
        Err(GenFailure::Error(error)) => {
            debug!(
                "inference: constraint generation failed for '{}': {error}",
                equation_text()
            );
            return Err(error);
        }
    }

    // Drop the generator's borrow of the context; the solver needs the
    // interner mutably to intern widened and extracted sorts.
    let ConstraintGenerator {
        expr_sorts,
        expr_texts,
        names,
        constraints,
        ..
    } = generator;

    // Merge the `Sub`s that widen into a shared free variable into one `Join`,
    // so their common supersort is computed in one step instead of order-
    // sensitively.
    let constraints = merge_shared_subs(constraints, &mut unifier);
    trace!(
        "inference: generated {} constraint(s) over {} expression node(s)",
        constraints.len(),
        expr_sorts.len()
    );

    let mut solver = Solver {
        sorts: &mut ctx.sorts,
        unifier: &mut unifier,
        constraints: &constraints,
        expr_sorts: &expr_sorts,
        base_names: &names,
        choices: Vec::new(),
        measure: Vec::new(),
        best: None,
    };
    solver.solve(0);

    // A push/pop mismatch on a dead-end branch never reaches `leaf()`'s
    // balance check, yet would corrupt the measure prefix of every later
    // branch — a silently wrong "best" typing rather than a crash.
    debug_assert!(
        solver.measure.is_empty() && solver.choices.is_empty(),
        "the solver unwinds its measure and choice stacks"
    );

    match solver.best {
        None => {
            debug!("inference: no valid sort assignment for '{}'", equation_text());
            Err(InferenceError::NoTyping {
                equation: equation_text(),
                span: equation.span.clone(),
            })
        }
        Some(best) if best.duplicate => {
            debug!(
                "inference: two solutions tie at measure {:?} for '{}'",
                best.measure,
                equation_text()
            );
            Err(InferenceError::AmbiguousExpression {
                equation: equation_text(),
                span: equation.span.clone(),
            })
        }
        Some(best) => match best.typing {
            None => {
                debug!(
                    "inference: the best solution leaves a sort free in '{}'",
                    equation_text()
                );
                Err(InferenceError::UnderdeterminedSort {
                    equation: equation_text(),
                    span: equation.span.clone(),
                })
            }
            Some((sorts, names)) => {
                // The contract of the Phase-4 side tables: one sort per
                // expression node, and name targets only for existing nodes.
                debug_assert_eq!(sorts.len(), expr_sorts.len(), "one inferred sort per expression node");
                debug_assert!(
                    names.keys().all(|id| **id < sorts.len()),
                    "every name target keys an expression node"
                );
                debug_assert!(
                    expr_texts.is_empty() || expr_texts.len() == sorts.len(),
                    "expr_texts is parallel to the expression nodes when filled"
                );

                debug!("inference: solved '{}' at measure {:?}", equation_text(), best.measure);
                if log::log_enabled!(log::Level::Debug) {
                    for var in &eqn_spec.variables {
                        let var_id = var.id.expect("assign_declaration_ids ran before check_equations");
                        // The cache was populated in the variables-binding loop above.
                        let sort = ctx
                            .sort_of_equation_var
                            .get(&(eqn_spec_id, var_id))
                            .copied()
                            .expect("equation variable sort was resolved above");
                        trace!(
                            "inference:   variable {}: {}",
                            var.identifier,
                            display_sort(ctx, spec, sort)
                        );
                    }
                    for (&sort, text) in sorts.iter().zip(&expr_texts) {
                        trace!("inference:   '{text}': {}", display_sort(ctx, spec, sort));
                    }
                }
                Ok(EquationTyping { sorts, names })
            }
        },
    }
}

/// Merges every group of `Sub` constraints that widen into the same free
/// variable into a single [Join] at the position of the group's last member.
/// A free variable shared by two or more `Sub` targets is an eagerly-unified
/// parameter — a scheme operand (`==`/`!=`/`<`/…/`if`), a set/bag element, or
/// the equation's LHS/RHS join — whose sequential greedy widening is
/// order-sensitive and can force a fruitless re-exploration. The join computes
/// their least common supersort in one step instead. A
/// disjunction overload's parameters are *distinct* fresh variables (the
/// overload is not committed until solving), so they are never grouped and the
/// argument-before-callee pruning of [Constraint] is preserved.
fn merge_shared_subs(constraints: Vec<Constraint>, unifier: &mut Unifier) -> Vec<Constraint> {
    // Group the `Sub` indices by the union-find root of their free-variable
    // target. A bound (concrete) target has no root and is never grouped.
    let mut groups: HashMap<u32, Vec<usize>> = HashMap::new();
    for (index, constraint) in constraints.iter().enumerate() {
        if let Constraint::Sub(sub) = constraint
            && let Some(root) = unifier.free_root(sub.rhs)
        {
            groups.entry(root).or_default().push(index);
        }
    }

    // Build one `Join` per group of two or more, placed at the last member's
    // position (by then every source's own sort is determined); the earlier
    // members are dropped.
    let mut joins: HashMap<usize, Join> = HashMap::new();
    let mut dropped = vec![false; constraints.len()];
    for indices in groups.into_values() {
        if indices.len() < 2 {
            continue;
        }
        let sources = indices
            .iter()
            .map(|&index| match &constraints[index] {
                Constraint::Sub(sub) => sub.lhs,
                _ => unreachable!("only Sub indices are grouped"),
            })
            .collect();
        let last = *indices.last().expect("a merged group has at least two members");
        // Every member's target denotes the same shared class; take one.
        let target = match &constraints[last] {
            Constraint::Sub(sub) => sub.rhs,
            _ => unreachable!("only Sub indices are grouped"),
        };
        for &index in &indices {
            dropped[index] = true;
        }
        joins.insert(last, Join { sources, target });
    }

    if joins.is_empty() {
        return constraints;
    }

    // Rebuild in the original order: a grouped `Sub` becomes its group's `Join`
    // at the last member and vanishes at the earlier members; everything else
    // is untouched.
    let mut result = Vec::with_capacity(constraints.len());
    for (index, constraint) in constraints.into_iter().enumerate() {
        if let Some(join) = joins.remove(&index) {
            result.push(Constraint::Join(join));
        } else if !dropped[index] {
            result.push(constraint);
        }
    }
    result
}

/// The kind of a number literal: `0` is natural, every other literal positive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LitKind {
    Positive,
    Natural,
}

/// Requires the sort of `lhs` to be a subsort of `rhs`, modelling the implicit
/// upcasts that mCRL2 inserts (a `Nat` argument may be passed where `Int` is
/// expected).
struct SubConstraint {
    lhs: InferSortId,
    rhs: InferSortId,
}

/// Requires `sort` to be a number sort admitting the literal kind.
struct LitConstraint {
    sort: InferSortId,
    kind: LitKind,
}

/// The overload set of a name with several candidates; the solver commits to
/// exactly one disjunct per solution.
struct Disjunction {
    /// The `Id` node whose chosen [NameTarget] is recorded per solution.
    expr: ExprId,
    /// The sort node of that `Id` node.
    sort: InferSortId,
    disjuncts: Vec<(NameTarget, InferSortId)>,
}

/// The two readings of a set/bag comprehension `{ x: S | e }`: a `Bool` body
/// denotes a `Set(S)`, a `Nat` or `Pos` body the multiplicities of a `Bag(S)`.
/// Which reading applies follows from the solved body sort; Phase-4 lowering
/// derives the binder kind from the node's sort and inserts the `Pos` → `Nat`
/// coercion on a positive body.
struct Comprehension {
    /// The sort node of the predicate (or count) body.
    body: InferSortId,
    /// The sort node of the comprehension expression itself.
    node: InferSortId,
    /// The resolved sort of the bound variable.
    element: ResolvedSortId,
}

/// A name of the arithmetic family (`+`, `-`, `*`, `/`, `div`, `mod`, `exp`,
/// `max`, `min`) with no user-declared overload:
/// solved by an O(1) lookup against the argument sorts, already bound by the
/// time this constraint is reached (via the `Sub` constraints generated for
/// the application's arguments), instead of a [Disjunction] over every
/// system-defined overload. The system's overloads for one of these names
/// never overlap on argument sort, so at most one can ever match a
/// fully-bound argument tuple — enumerating them as a `Disjunction` instead
/// tries (and, to detect ties, fully explores) every candidate at every
/// occurrence, which is what made repeated arithmetic sub-expressions
/// exponential.
struct Numeric {
    /// The sort node of the applied name's `Id` node (its `NameTarget::Builtin`
    /// is recorded eagerly at generation time, since every candidate here is a
    /// `Builtin`): already unified, structurally, to
    /// `Function { domain: <argument parameter nodes>, range: <the
    /// application's own node> }` by the eager unify in the `Application`
    /// case of [ConstraintGenerator::visit], before this constraint is ever
    /// solved.
    sort: InferSortId,
    /// The system-defined overloads of this name (`system_signature.mappings[name]`
    /// at generation time); their domains are pairwise distinct, so at most
    /// one can match the bound argument sorts.
    candidates: Vec<ResolvedSortId>,
}

/// A least-upper-bound constraint: the `target` sort is the lattice join of
/// the `sources` (their least common supersort). It replaces a group of
/// individual `Sub` constraints that all widen into the *same* free variable —
/// the eagerly-shared parameter of a scheme (`==`/`!=`/`<`/…/`if`), a set or
/// bag element, or the equation's LHS/RHS join. Sequential `Sub`s are
/// order-sensitive: whichever source is solved first binds the shared variable
/// by equality, so a finite-container source (a set literal, `FSet`) fixes the
/// result to `FSet` before another source (a comprehension, `Set`) is typed,
/// which then cannot satisfy its own `Sub` and forces the whole comprehension
/// body to be re-explored fruitlessly. Computing the join directly picks the
/// common supersort in one step, with no premature commitment, while
/// contributing the same per-source widening measure as the `Sub`s did — so
/// the ranking is unchanged. Built by [merge_shared_subs] after constraint
/// generation.
struct Join {
    /// The branch sort nodes joined into `target`, in generation order.
    sources: Vec<InferSortId>,
    /// The shared variable the sources widen into (e.g. an `if`'s result).
    target: InferSortId,
}

/// One constraint of an equation, solved in generation order. Interleaving the
/// kinds (rather than deciding all disjunctions first) is what keeps the
/// search tractable: the arguments of an application are generated before its
/// callee, so by the time a callee's overload disjunction is tried, the
/// argument sorts are already bound and most disjuncts fail to unify at once.
enum Constraint {
    Sub(SubConstraint),
    Lit(LitConstraint),
    Disjunction(Disjunction),
    Comprehension(Comprehension),
    Numeric(Numeric),
    Join(Join),
}

/// Names resolved as arithmetic promotions ([Numeric]) rather than general
/// overload disjunction, when the name has no user-declared overload.
fn is_numeric_family(name: &str) -> bool {
    matches!(name, "+" | "-" | "*" | "/" | "div" | "mod" | "exp" | "max" | "min")
}

/// Why constraint generation stopped early.
enum GenFailure {
    /// A binder in the equation declares a sort that is not a valid variable
    /// sort (a bare product; see [is_supported_binder_sort]). The equation is
    /// rejected rather than left untyped. Carries the offending sort's text
    /// and the span of the binder that declares it.
    InvalidBinderSort(String, Span),
    Error(InferenceError),
}

/// Walks the lowered expressions of one equation, assigning [ExprId]s and
/// emitting the constraints. Structural facts that must hold in every solution
/// (a callee has a function sort, the condition is boolean) are unified
/// eagerly, so their failure is a direct error rather than a solver miss.
struct ConstraintGenerator<'a> {
    /// Mutable so a comprehension's binder sort can be resolved (interned)
    /// mid-walk; the signatures below are `Rc` clones out of this same context.
    ctx: &'a mut TypeckContext,
    spec: &'a UntypedDataSpecification,
    signature: Rc<Signature>,
    system_signature: Rc<Signature>,
    variables: HashMap<&'a str, InferSortId>,
    unifier: &'a mut Unifier,
    /// The sort node of every expression, indexed by [ExprId].
    expr_sorts: Vec<InferSortId>,
    /// The display text of every expression, parallel to `expr_sorts`; only
    /// filled when [Self::log_texts], to report the solved typing.
    expr_texts: Vec<String>,
    /// Whether debug logging was enabled when generation started; sampled once
    /// so `expr_texts` stays parallel to `expr_sorts` even under a log filter
    /// that changes mid-equation.
    log_texts: bool,
    /// The targets of names resolved during generation (variables and
    /// single-candidate names); disjunction choices are added by the solver.
    names: HashMap<ExprId, NameTarget>,
    constraints: Vec<Constraint>,
}

impl<'a> ConstraintGenerator<'a> {
    fn generate(
        &mut self,
        condition: Option<&'a DataExpr>,
        lhs: &'a DataExpr,
        rhs: &'a DataExpr,
    ) -> Result<(), GenFailure> {
        // Checked once at the roots: the property is subtree-closed, and
        // re-checking per node in `visit` would be quadratic on the deep
        // expressions where solving is already expensive.
        debug_assert!(
            condition.is_none_or(is_lowered) && is_lowered(lhs) && is_lowered(rhs),
            "inference requires lowered expressions"
        );

        if let Some(condition) = condition {
            let sort = self.visit(condition)?;
            let bool_node = self.unifier.resolved_node(self.ctx.sorts.bool_sort());
            if !self.unifier.unify(&self.ctx.sorts, sort, bool_node) {
                return Err(GenFailure::Error(InferenceError::ConditionNotBool {
                    condition: condition.to_string(),
                    span: condition.span.clone(),
                }));
            }
        }

        let lhs_sort = self.visit(lhs)?;
        let rhs_sort = self.visit(rhs)?;
        let joined = self.unifier.fresh_var();
        self.constraints.push(Constraint::Sub(SubConstraint {
            lhs: lhs_sort,
            rhs: joined,
        }));
        self.constraints.push(Constraint::Sub(SubConstraint {
            lhs: rhs_sort,
            rhs: joined,
        }));
        Ok(())
    }

    /// Emits the constraints for `expr` and returns its sort node: a fresh
    /// variable constrained by the expression form.
    fn visit(&mut self, expr: &'a DataExpr) -> Result<InferSortId, GenFailure> {
        let node = self.unifier.fresh_var();
        let id = ExprId::new(self.expr_sorts.len());
        self.expr_sorts.push(node);
        if self.log_texts {
            self.expr_texts.push(expr.to_string());
        }

        match &expr.node {
            DataExprKind::Id(name) => self.gen_name(id, node, name, &expr.span)?,
            DataExprKind::Number(value) => {
                let kind = if value == "0" {
                    LitKind::Natural
                } else {
                    LitKind::Positive
                };
                self.constraints
                    .push(Constraint::Lit(LitConstraint { sort: node, kind }));
            }
            DataExprKind::Bool(_) => {
                let bool_node = self.unifier.resolved_node(self.ctx.sorts.bool_sort());
                self.bind_fresh(node, bool_node);
            }
            DataExprKind::EmptyList => {
                let element = self.unifier.fresh_var();
                let list = self.unifier.generic(ComplexSort::List, element);
                self.bind_fresh(node, list);
            }
            // The enumerated and empty set/bag literals take the *finite*
            // container sort; where a `Set`/`Bag` is expected, the sub-sort
            // constraints widen `FSet(S) <= Set(S)` (`FBag(S) <= Bag(S)`) at
            // the point of use, matching mCRL2's upcast of an enumeration.
            DataExprKind::EmptySet => {
                let element = self.unifier.fresh_var();
                let set = self.unifier.generic(ComplexSort::FSet, element);
                self.bind_fresh(node, set);
            }
            DataExprKind::EmptyBag => {
                let element = self.unifier.fresh_var();
                let bag = self.unifier.generic(ComplexSort::FBag, element);
                self.bind_fresh(node, bag);
            }
            DataExprKind::Set(members) => {
                // The members share one element node into which each may be
                // upcast, so the solved element sort is the least common
                // supersort of the member sorts.
                let element = self.unifier.fresh_var();
                for member in members {
                    let member_sort = self.visit(member)?;
                    self.constraints.push(Constraint::Sub(SubConstraint {
                        lhs: member_sort,
                        rhs: element,
                    }));
                }
                let set = self.unifier.generic(ComplexSort::FSet, element);
                self.bind_fresh(node, set);
            }
            DataExprKind::Bag(members) => {
                let element = self.unifier.fresh_var();
                let nat = self.unifier.resolved_node(self.ctx.sorts.nat_sort());
                for member in members {
                    let member_sort = self.visit(&member.expr)?;
                    self.constraints.push(Constraint::Sub(SubConstraint {
                        lhs: member_sort,
                        rhs: element,
                    }));
                    // A multiplicity is a natural number; a `Pos` count is
                    // upcast, anything else is an error.
                    let count_sort = self.visit(&member.multiplicity)?;
                    self.constraints.push(Constraint::Sub(SubConstraint {
                        lhs: count_sort,
                        rhs: nat,
                    }));
                }
                let bag = self.unifier.generic(ComplexSort::FBag, element);
                self.bind_fresh(node, bag);
            }
            DataExprKind::SetBagComp { variable, predicate } => {
                let element = self.binder_sort(&variable.sort, &variable.span)?;
                let element_node = self.unifier.resolved_node(element);

                // The bound variable shadows an equation variable of the same
                // name for the predicate only; it has no [ExprId] of its own,
                // like the equation variables.
                let name = variable.identifier.as_str();
                let shadowed = self.variables.insert(name, element_node);
                let body = self.visit(predicate)?;
                match shadowed {
                    Some(previous) => self.variables.insert(name, previous),
                    None => self.variables.remove(name),
                };

                self.constraints
                    .push(Constraint::Comprehension(Comprehension { body, node, element }));
            }
            DataExprKind::Application { function, arguments } => {
                // The arguments are visited (and hence constrained) before the
                // applied function, so the function's overload disjunction is
                // solved against already-bound argument sorts.
                let mut parameters = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    let argument_sort = self.visit(argument)?;
                    // The parameter is a fresh variable rather than the
                    // argument sort itself, so the argument may be upcast into
                    // the parameter the overload expects.
                    let parameter = self.unifier.fresh_var();
                    self.constraints.push(Constraint::Sub(SubConstraint {
                        lhs: argument_sort,
                        rhs: parameter,
                    }));
                    parameters.push(parameter);
                }
                let function_sort = self.visit(function)?;
                let expected = self.unifier.function(parameters, node);
                if !self.unifier.unify(&self.ctx.sorts, function_sort, expected) {
                    return Err(GenFailure::Error(InferenceError::NotAFunction {
                        expr: function.to_string(),
                        span: function.span.clone(),
                    }));
                }
            }
            DataExprKind::Lambda { variables, body } => {
                // The result is a function from the bound variables' declared
                // sorts to the body's sort.
                let function_sort = self.with_binder_scope(variables, |this, sorts| {
                    let body_sort = this.visit(body)?;
                    let parameters = sorts.iter().map(|&sort| this.unifier.resolved_node(sort)).collect();
                    Ok(this.unifier.function(parameters, body_sort))
                })?;
                self.bind_fresh(node, function_sort);
            }
            DataExprKind::Quantifier { op: _, variables, body } => {
                // A `forall`/`exists` is `Bool`, and requires its body to be
                // `Bool` too (mCRL2 checks both with `TypeMatchA(Bool, ..)`).
                let bool_node = self.unifier.resolved_node(self.ctx.sorts.bool_sort());
                self.with_binder_scope(variables, |this, _sorts| {
                    let body_sort = this.visit(body)?;
                    if !this.unifier.unify(&this.ctx.sorts, body_sort, bool_node) {
                        return Err(GenFailure::Error(InferenceError::QuantifierNotBool {
                            body: body.to_string(),
                            span: body.span.clone(),
                        }));
                    }
                    Ok(())
                })?;
                self.bind_fresh(node, bool_node);
            }
            DataExprKind::Whr { expr, assignments } => {
                // Each assignment's right-hand side is typed in the outer
                // scope — bindings do not see each other, only the body does
                // (every assignment is typed against the original declared
                // variables, the context being extended once, for the body).
                // So every right-hand side is visited first, and only
                // then are the names shadowed as a batch.
                // The bound variable's sort is the assignment's own inferred
                // sort node, so it has no [ExprId] and no declared sort to
                // resolve, unlike a comprehension/lambda/quantifier binder.
                let mut bindings = Vec::with_capacity(assignments.len());
                for assignment in assignments {
                    let value_node = self.visit(&assignment.expr)?;
                    bindings.push((assignment.identifier.as_str(), value_node));
                }
                let mut shadowed = Vec::with_capacity(bindings.len());
                for &(name, value_node) in &bindings {
                    shadowed.push((name, self.variables.insert(name, value_node)));
                }
                let body_sort = self.visit(expr)?;
                for (name, previous) in shadowed.into_iter().rev() {
                    match previous {
                        Some(previous) => self.variables.insert(name, previous),
                        None => self.variables.remove(name),
                    };
                }
                self.bind_fresh(node, body_sort);
            }
            DataExprKind::List(_)
            | DataExprKind::Unary { .. }
            | DataExprKind::Binary { .. }
            | DataExprKind::FunctionUpdate { .. } => {
                unreachable!("lowering rewrote this expression form")
            }
        }

        Ok(node)
    }

    /// Binds the fresh sort node of the current expression to `sort`;
    /// infallible because a fresh variable unifies with any sort.
    fn bind_fresh(&mut self, node: InferSortId, sort: InferSortId) {
        let unified = self.unifier.unify(&self.ctx.sorts, node, sort);
        debug_assert!(unified, "a fresh variable unifies with any sort");
    }

    /// Resolves the declared sort of each of `variables` (rejecting an invalid
    /// binder sort, see [Self::binder_sort]) and shadows it in
    /// `self.variables` for the scope of `f`, restoring the previous bindings
    /// (or removing them) afterwards — the multi-variable generalization of
    /// the shadowing done inline for a comprehension's single bound variable.
    /// Used by `lambda` and `forall`/`exists`, which declare their variables'
    /// sorts, unlike a `whr` binding whose sort follows from its right-hand side.
    fn with_binder_scope<T>(
        &mut self,
        variables: &'a [IdDecl],
        f: impl FnOnce(&mut Self, &[ResolvedSortId]) -> Result<T, GenFailure>,
    ) -> Result<T, GenFailure> {
        let mut sorts = Vec::with_capacity(variables.len());
        let mut shadowed = Vec::with_capacity(variables.len());
        for variable in variables {
            let sort = self.binder_sort(&variable.sort, &variable.span)?;
            let node = self.unifier.resolved_node(sort);
            let name = variable.identifier.as_str();
            shadowed.push((name, self.variables.insert(name, node)));
            sorts.push(sort);
        }

        let result = f(self, &sorts);

        for (name, previous) in shadowed.into_iter().rev() {
            match previous {
                Some(previous) => self.variables.insert(name, previous),
                None => self.variables.remove(name),
            };
        }

        result
    }

    /// Resolves the declared sort of a comprehension's bound variable onto the
    /// interned lattice, rejecting a sort that is not a valid variable sort
    /// (a bare product; see [is_supported_binder_sort]). `span` is the
    /// binder's declaration span, reported on rejection.
    fn binder_sort(&mut self, sort: &SortExpression, span: &Span) -> Result<ResolvedSortId, GenFailure> {
        if !is_supported_binder_sort(sort) {
            return Err(GenFailure::InvalidBinderSort(sort.to_string(), span.clone()));
        }
        Ok(resolve_sort(self.ctx, self.spec, sort))
    }

    /// Resolves the candidates of a name: the equation variables shadow
    /// everything, then the user overloads joined by either the built-in
    /// scheme (for the polymorphic comparison operators and `if`) or the
    /// system-defined overloads.
    fn gen_name(&mut self, id: ExprId, node: InferSortId, name: &'a str, span: &Span) -> Result<(), GenFailure> {
        if let Some(&sort) = self.variables.get(name) {
            self.names.insert(id, NameTarget::Variable);
            self.bind_fresh(node, sort);
            return Ok(());
        }

        let mut disjuncts: Vec<(NameTarget, InferSortId)> = Vec::new();
        let push_signature = |signature: &Signature, disjuncts: &mut Vec<_>, unifier: &mut Unifier| {
            for overloads in [signature.constructors.get(name), signature.mappings.get(name)]
                .into_iter()
                .flatten()
            {
                for &overload in overloads {
                    let target = NameTarget::Op { sort: overload };
                    // The user and system specifications may declare the same
                    // symbol; a duplicate disjunct would misreport ambiguity.
                    if !disjuncts.iter().any(|(existing, _)| *existing == target) {
                        disjuncts.push((target, unifier.resolved_node(overload)));
                    }
                }
            }
        };

        push_signature(&self.signature, &mut disjuncts, self.unifier);
        if let Some(instance) = self.scheme_instance(name) {
            // The scheme subsumes the per-sort declarations of the system
            // specification, so those are not added as candidates.
            disjuncts.push((NameTarget::Builtin, instance));
        } else if disjuncts.is_empty()
            && is_numeric_family(name)
            && self.system_signature.mappings.contains_key(name)
            && !POLYMORPHIC_SIGNATURE.ops.contains_key(name)
        {
            // No user overload shadows the name, and it has no container
            // meaning either (`+`/`-`/`*` are also Set/Bag union, difference
            // and intersection, via the polymorphic templates below): its
            // concrete promotion is picked by a direct lookup (`Numeric`)
            // instead of a disjunction over the system-defined overloads.
            self.names.insert(id, NameTarget::Builtin);
            self.constraints.push(Constraint::Numeric(Numeric {
                sort: node,
                candidates: self.system_signature.mappings[name].clone(),
            }));
            return Ok(());
        } else {
            push_signature(&self.system_signature, &mut disjuncts, self.unifier);

            // The container operations (`in`, `#`, `|>`, `head`, ...) exist
            // for every element sort, so they are further schemes: each
            // template overload is instantiated with fresh variables per
            // occurrence, mirroring mCRL2's polymorphic symbol table. Phase-4
            // lowering recovers the concrete operation from the name and the
            // inferred sort, as for the comparison schemes.
            for overload in POLYMORPHIC_SIGNATURE.ops.get(name).into_iter().flatten() {
                let instance = self.template_instance(overload);
                disjuncts.push((NameTarget::Builtin, instance));
            }
        }

        match disjuncts.as_slice() {
            [] => Err(GenFailure::Error(InferenceError::UndeclaredName {
                name: name.to_string(),
                span: span.clone(),
            })),
            [(target, sort)] => {
                self.names.insert(id, *target);
                self.bind_fresh(node, *sort);
                Ok(())
            }
            _ => {
                trace!("inference: name '{name}' has {} candidate(s)", disjuncts.len());
                self.constraints.push(Constraint::Disjunction(Disjunction {
                    expr: id,
                    sort: node,
                    disjuncts,
                }));
                Ok(())
            }
        }
    }

    /// A fresh instance of a template overload: every sort variable (a
    /// `Reference` node of the uninstantiated template, i.e. `S` or `T`)
    /// becomes one fresh unification variable, shared between its occurrences.
    fn template_instance(&mut self, sort: &SortExpression) -> InferSortId {
        let mut variables = HashMap::new();
        self.template_node(sort, &mut variables)
    }

    fn template_node(&mut self, sort: &SortExpression, variables: &mut HashMap<String, InferSortId>) -> InferSortId {
        match sort {
            SortExpression::Simple(sort) => {
                let resolved = self.ctx.sorts.primitive(*sort);
                self.unifier.resolved_node(resolved)
            }
            SortExpression::Complex(op, subsort) => {
                let subsort = self.template_node(subsort, variables);
                self.unifier.generic(*op, subsort)
            }
            SortExpression::Function { domain, range } => {
                let mut parameters = Vec::new();
                self.template_domain(domain, variables, &mut parameters);
                let range = self.template_node(range, variables);
                self.unifier.function(parameters, range)
            }
            SortExpression::FlattenedFunction { domain, range } => {
                let parameters = domain
                    .iter()
                    .map(|parameter| self.template_node(parameter, variables))
                    .collect();
                let range = self.template_node(range, variables);
                self.unifier.function(parameters, range)
            }
            SortExpression::Reference(name) => *variables
                .entry(name.clone())
                .or_insert_with(|| self.unifier.fresh_var()),
            SortExpression::Resolved(_, _) | SortExpression::Struct { .. } | SortExpression::Product { .. } => {
                unreachable!("the templates declare only primitive, container, function and variable sorts")
            }
        }
    }

    /// Instantiates the leaves of a `Product` domain spine in declaration
    /// order, the template counterpart of `resolve_function_domain`.
    fn template_domain(
        &mut self,
        sort: &SortExpression,
        variables: &mut HashMap<String, InferSortId>,
        domain: &mut Vec<InferSortId>,
    ) {
        match sort {
            SortExpression::Product { lhs, rhs } => {
                self.template_domain(lhs, variables, domain);
                self.template_domain(rhs, variables, domain);
            }
            _ => domain.push(self.template_node(sort, variables)),
        }
    }

    /// A fresh instance of the polymorphic built-in `name`, or `None` when the
    /// name is not one. The comparison operators and `if` exist for *every*
    /// sort, so they are typed as schemes (`?a # ?a -> Bool`) instantiated per
    /// occurrence instead of one overload per declared sort.
    fn scheme_instance(&mut self, name: &str) -> Option<InferSortId> {
        match name {
            "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                let element = self.unifier.fresh_var();
                let bool_node = self.unifier.resolved_node(self.ctx.sorts.bool_sort());
                Some(self.unifier.function(vec![element, element], bool_node))
            }
            "if" => {
                let element = self.unifier.fresh_var();
                let bool_node = self.unifier.resolved_node(self.ctx.sorts.bool_sort());
                Some(self.unifier.function(vec![bool_node, element, element], element))
            }
            _ => None,
        }
    }
}

/// A candidate solution: the measure ranks it against other leaves, and the
/// typing is extracted eagerly because backtracking destroys the variable
/// bindings it is read from.
struct Candidate {
    measure: Vec<u8>,
    /// Whether another leaf reached the same measure; an unbeaten duplicate
    /// means the equation is ambiguous.
    duplicate: bool,
    /// `None` when a free variable remained at this leaf, i.e. the sorts were
    /// underdetermined.
    typing: Option<(Vec<ResolvedSortId>, HashMap<ExprId, NameTarget>)>,
}

/// Solves the constraints by ranked backtracking, in generation order rather
/// than kind-grouped: a disjunction tries every overload, a sub-constraint
/// tries equality first and widening second, a literal takes its most specific
/// admissible number sort.
///
/// Each sub and literal constraint contributes one component to the measure
/// (in generation order, earlier constraints most significant — the arguments
/// of an application precede the equation-level join — and `0` best), so
/// solutions compare lexicographically and the minimum is the most specific
/// typing. Disjunctions contribute no component and are enumerated
/// exhaustively, so tied leaves through different overloads are still detected
/// as ambiguity.
struct Solver<'a> {
    sorts: &'a mut SortInterner,
    unifier: &'a mut Unifier,
    constraints: &'a [Constraint],
    expr_sorts: &'a [InferSortId],
    base_names: &'a HashMap<ExprId, NameTarget>,
    /// The disjunct chosen per disjunction on the current branch.
    choices: Vec<(ExprId, NameTarget)>,
    /// The measure components pushed on the current branch.
    measure: Vec<u8>,
    best: Option<Candidate>,
}

impl Solver<'_> {
    /// Solves the constraints from `index` onward; returns whether any leaf
    /// was reached below this point.
    fn solve(&mut self, index: usize) -> bool {
        if self.dominated() {
            return false;
        }
        let Some(constraint) = self.constraints.get(index) else {
            self.leaf();
            return true;
        };
        match constraint {
            Constraint::Disjunction(disjunction) => self.solve_disjunction(disjunction, index),
            Constraint::Sub(sub) => self.solve_sub(sub, index),
            Constraint::Lit(lit) => self.solve_lit(lit, index),
            Constraint::Comprehension(comprehension) => self.solve_comprehension(comprehension, index),
            Constraint::Numeric(numeric) => self.solve_numeric(numeric, index),
            Constraint::Join(join) => self.solve_join(join, index),
        }
    }

    /// Picks the one system-defined overload (if any) whose domain matches
    /// the already-bound argument sorts, and continues solving — no
    /// branching over candidates, unlike [Self::solve_disjunction], since at
    /// most one can ever match (see [Numeric]).
    fn solve_numeric(&mut self, numeric: &Numeric, index: usize) -> bool {
        let InferSort::Function { domain, range } = self.unifier.head(numeric.sort) else {
            unreachable!("the eager unify in the Application case always binds this to a function sort")
        };

        let mut arg_sorts = Vec::with_capacity(domain.len());
        for parameter in domain {
            match self.unifier.resolve(self.sorts, parameter) {
                Some(resolved) => arg_sorts.push(resolved),
                // An argument sort is still underdetermined; no promotion
                // table entry can be picked yet, and there is nothing left to
                // widen from this constraint's side.
                None => return false,
            }
        }

        for &candidate in &numeric.candidates {
            let ResolvedSort::Function {
                domain: candidate_domain,
                range: candidate_range,
            } = self.sorts.get(candidate)
            else {
                continue;
            };
            if *candidate_domain != arg_sorts {
                continue;
            }
            let candidate_range = *candidate_range;
            let range_node = self.unifier.resolved_node(candidate_range);
            return self.unifier.unify(self.sorts, range, range_node) && self.solve(index + 1);
        }
        false
    }

    /// Binds the join `target` to the lattice least-upper-bound of the branch
    /// `sources` and continues solving — the deterministic counterpart of two
    /// greedy `Sub`s to a shared variable (see [Join]). Every source must be
    /// ground and pairwise joinable; otherwise it defers to the per-source
    /// widening of [Self::solve_join_seq], which decides exactly as the
    /// pre-join two-`Sub` encoding did (so the rare underdetermined branches —
    /// e.g. two empty containers — are unchanged).
    fn solve_join(&mut self, join: &Join, index: usize) -> bool {
        let resolved: Option<Vec<ResolvedSortId>> = join
            .sources
            .iter()
            .map(|&source| self.unifier.resolve(self.sorts, source))
            .collect();
        let Some(resolved) = resolved else {
            return self.solve_join_seq(&join.sources, join.target, 0, index);
        };

        let mut lub = resolved[0];
        for &next in &resolved[1..] {
            match self.sorts.join(lub, next) {
                Some(joined) => lub = joined,
                // Not joinable by the simple lattice (e.g. unequal element
                // sorts): the per-source widening below decides the case the
                // same way the two `Sub`s used to.
                None => return self.solve_join_seq(&join.sources, join.target, 0, index),
            }
        }

        let lub_node = self.unifier.resolved_node(lub);
        if !self.unifier.unify(self.sorts, join.target, lub_node) {
            return false;
        }

        // One widening-distance measure component per source (0 for an exact
        // branch, the number of widening steps otherwise), matching the
        // `solve_sub` convention so the ranking is identical to the two-`Sub`
        // form.
        for &source in &resolved {
            self.measure.push(self.join_distance(source, lub));
        }
        let found = self.solve(index + 1);
        for _ in &resolved {
            self.measure.pop();
        }
        found
    }

    /// The number of lattice widening steps from `source` up to `target`
    /// (`source` a subsort of `target`), the measure a `Sub(source, target)`
    /// would contribute: number-sort generality difference, or one step for a
    /// finite-to-infinite container widening (`FSet` → `Set`, `FBag` → `Bag`).
    fn join_distance(&self, source: ResolvedSortId, target: ResolvedSortId) -> u8 {
        if source == target {
            return 0;
        }
        match (self.sorts.get(source), self.sorts.get(target)) {
            (ResolvedSort::Primitive(source), ResolvedSort::Primitive(target)) => {
                match (number_generality(*source), number_generality(*target)) {
                    (Some(source), Some(target)) if target >= source => (target - source) as u8,
                    _ => 1,
                }
            }
            // A single container-head step; the element sorts are equal (the
            // lattice join keeps them so).
            _ => 1,
        }
    }

    /// The fallback of [Self::solve_join] for underdetermined or non-joinable
    /// branches: widens each source to the shared `target` in turn, exactly as
    /// the two independent `Sub` constraints did before the join fast path
    /// (equality first, then widenings in ascending distance).
    fn solve_join_seq(&mut self, sources: &[InferSortId], target: InferSortId, i: usize, index: usize) -> bool {
        let Some(&source) = sources.get(i) else {
            return self.solve(index + 1);
        };

        // Equality first: it ranks strictly better than any widening.
        let snapshot = self.unifier.snapshot();
        let mut found = false;
        if self.unifier.unify(self.sorts, source, target) {
            self.measure.push(0);
            found = self.solve_join_seq(sources, target, i + 1, index);
            self.measure.pop();
        }
        self.unifier.rollback_to(snapshot);
        if found {
            return true;
        }

        // Then the strict widenings, nearest first (a concrete source upcast,
        // or a concrete target met from below).
        let pairs: Vec<(InferSortId, InferSortId)> =
            if let Some(supers) = self.unifier.strict_super_sorts(self.sorts, source) {
                supers.into_iter().map(|wider| (wider, target)).collect()
            } else if let Some(subsorts) = self.unifier.strict_sub_sorts(self.sorts, target) {
                subsorts.into_iter().map(|narrower| (source, narrower)).collect()
            } else {
                return false;
            };
        for (distance, (lhs, rhs)) in pairs.into_iter().enumerate() {
            let snapshot = self.unifier.snapshot();
            let mut found = false;
            if self.unifier.unify(self.sorts, lhs, rhs) {
                self.measure.push(1 + distance as u8);
                found = self.solve_join_seq(sources, target, i + 1, index);
                self.measure.pop();
            }
            self.unifier.rollback_to(snapshot);
            if found {
                return true;
            }
        }
        false
    }

    /// Branch-and-bound pruning: whether the measure accumulated so far is
    /// already strictly worse, component for component, than the incumbent's
    /// corresponding prefix. A `Disjunction`/`Comprehension` contributes no
    /// measure component of its own (every disjunct is tried, so a tie is
    /// still detected as ambiguity), so without this check every disjunct is
    /// explored to its leaf even once a strictly better solution is already
    /// known — on an equation with many independent overloaded operators
    /// (repeated arithmetic sub-expressions, say) that is exponential in the
    /// number of disjunctions. Pruning is exact: a prefix that is already
    /// strictly greater can never become equal or smaller, since earlier
    /// measure components dominate the lexicographic order, so this changes
    /// nothing about which typing wins or which equations are ambiguous.
    fn dominated(&self) -> bool {
        match &self.best {
            Some(best) => self.measure.as_slice() > &best.measure[..self.measure.len()],
            None => false,
        }
    }

    /// Commits to one disjunct and solves the remaining constraints; all
    /// disjuncts are explored so equal-measure leaves surface as ambiguity.
    fn solve_disjunction(&mut self, disjunction: &Disjunction, index: usize) -> bool {
        let mut found = false;
        for (target, sort) in &disjunction.disjuncts {
            let snapshot = self.unifier.snapshot();
            if self.unifier.unify(self.sorts, disjunction.sort, *sort) {
                trace!(
                    "solver: committing expression {:?} to disjunct {target:?}",
                    disjunction.expr
                );
                self.choices.push((disjunction.expr, *target));
                found |= self.solve(index + 1);
                self.choices.pop();
            }
            self.unifier.rollback_to(snapshot);
        }
        found
    }

    /// Commits to one reading of a set/bag comprehension and solves the rest:
    /// a `Bool` body makes a `Set`, a `Nat` or `Pos` body makes a `Bag`. Like a
    /// disjunction it contributes no measure component and explores every
    /// reading, so an equation where two readings both type (an overloaded body
    /// that can be either boolean or numeric) surfaces as ambiguity.
    fn solve_comprehension(&mut self, comprehension: &Comprehension, index: usize) -> bool {
        let readings = [
            (self.sorts.bool_sort(), ComplexSort::Set),
            (self.sorts.nat_sort(), ComplexSort::Bag),
            (self.sorts.pos_sort(), ComplexSort::Bag),
        ];

        let mut found = false;
        for (body, op) in readings {
            let container = self.sorts.generic(op, comprehension.element);
            let snapshot = self.unifier.snapshot();
            let body_node = self.unifier.resolved_node(body);
            let container_node = self.unifier.resolved_node(container);
            if self.unifier.unify(self.sorts, comprehension.body, body_node)
                && self.unifier.unify(self.sorts, comprehension.node, container_node)
            {
                found |= self.solve(index + 1);
            }
            self.unifier.rollback_to(snapshot);
        }
        found
    }

    fn solve_sub(&mut self, sub: &SubConstraint, index: usize) -> bool {
        // Equality first: it ranks strictly better than any widening, so when
        // it admits a solution the widening choices cannot improve on it and
        // are not explored.
        let snapshot = self.unifier.snapshot();
        let mut found = false;
        if self.unifier.unify(self.sorts, sub.lhs, sub.rhs) {
            self.measure.push(0);
            found = self.solve(index + 1);
            self.measure.pop();
        }
        self.unifier.rollback_to(snapshot);
        if found {
            return true;
        }

        // Otherwise enumerate the strict widenings: a concrete lhs may be
        // upcast, or a concrete rhs met from below. Two unbound variables
        // admit no enumeration and fail.
        let pairs: Vec<(InferSortId, InferSortId)> =
            if let Some(supers) = self.unifier.strict_super_sorts(self.sorts, sub.lhs) {
                supers.into_iter().map(|wider| (wider, sub.rhs)).collect()
            } else if let Some(subsorts) = self.unifier.strict_sub_sorts(self.sorts, sub.rhs) {
                subsorts.into_iter().map(|narrower| (sub.lhs, narrower)).collect()
            } else {
                return false;
            };

        // Widenings rank by distance (ranking them all equally would misreport
        // e.g. a `Pos` argument to `mod` as ambiguous between its `Nat` and
        // `Int` overloads; the minimal upcast is taken instead). The pairs
        // are ordered nearest first, so the first success is the best this
        // constraint can contribute and the rest need not be explored.
        for (distance, (lhs, rhs)) in pairs.into_iter().enumerate() {
            let snapshot = self.unifier.snapshot();
            let mut found = false;
            if self.unifier.unify(self.sorts, lhs, rhs) {
                self.measure.push(1 + distance as u8);
                found = self.solve(index + 1);
                self.measure.pop();
            }
            self.unifier.rollback_to(snapshot);
            if found {
                return true;
            }
        }
        false
    }

    fn solve_lit(&mut self, lit: &LitConstraint, index: usize) -> bool {
        match self.unifier.head(lit.sort) {
            InferSort::Resolved(resolved) => {
                let ResolvedSort::Primitive(sort) = *self.sorts.get(resolved) else {
                    return false;
                };
                let Some(generality) = number_generality(sort) else {
                    return false;
                };
                // `0` is not positive, so a natural literal cannot be `Pos`.
                if lit.kind == LitKind::Natural && sort == Sort::Pos {
                    return false;
                }
                self.measure.push(generality as u8);
                let found = self.solve(index + 1);
                self.measure.pop();
                found
            }
            InferSort::Var(_) => {
                let candidates: &[Sort] = match lit.kind {
                    LitKind::Positive => &[Sort::Pos, Sort::Nat, Sort::Int, Sort::Real],
                    LitKind::Natural => &[Sort::Nat, Sort::Int, Sort::Real],
                };
                // The candidates are ordered most specific first, so the first
                // success is the best this constraint can contribute and the
                // rest need not be explored. The component is the sort's
                // generality, matching the bound branch above, so a literal
                // bound in one branch and free in another still compares
                // consistently.
                for &sort in candidates {
                    let snapshot = self.unifier.snapshot();
                    let resolved = self.sorts.primitive(sort);
                    let node = self.unifier.resolved_node(resolved);
                    let unified = self.unifier.unify(self.sorts, lit.sort, node);
                    debug_assert!(unified, "an unbound variable unifies with any sort");
                    let generality = number_generality(sort).expect("the candidates are number sorts");
                    self.measure.push(generality as u8);
                    let found = self.solve(index + 1);
                    self.measure.pop();
                    self.unifier.rollback_to(snapshot);
                    if found {
                        return true;
                    }
                }
                false
            }
            InferSort::Generic { .. } | InferSort::Function { .. } => false,
        }
    }

    /// A full assignment: keep it when it beats the incumbent, flag a
    /// duplicate when it ties (ambiguity unless later beaten).
    fn leaf(&mut self) {
        debug_assert_eq!(
            self.measure.len(),
            self.constraints
                .iter()
                .map(|constraint| match constraint {
                    Constraint::Sub(_) | Constraint::Lit(_) => 1,
                    // A join contributes one widening component per branch.
                    Constraint::Join(join) => join.sources.len(),
                    Constraint::Disjunction(_) | Constraint::Comprehension(_) | Constraint::Numeric(_) => 0,
                })
                .sum::<usize>(),
            "every sub, literal and join-branch contributes exactly one measure component"
        );

        let ordering = match &self.best {
            None => Ordering::Less,
            Some(best) => self.measure.cmp(&best.measure),
        };
        match ordering {
            Ordering::Less => {
                trace!("solver: new best solution at measure {:?}", self.measure);
                let candidate = self.extract();
                self.best = Some(candidate);
            }
            Ordering::Equal => {
                trace!("solver: tie at measure {:?}, ambiguous unless beaten", self.measure);
                self.best.as_mut().expect("a tie requires an incumbent").duplicate = true;
            }
            Ordering::Greater => {}
        }
    }

    /// Reads the solution out of the current variable bindings, before
    /// backtracking destroys them.
    ///
    /// Any sort variable that is still free after solving (e.g. the element
    /// sort of `#[]` where only the container length is observed, never the
    /// element) defaults to `Bool`. This accepts such equations rather than
    /// raising a spurious `UnderdeterminedSort` error.
    fn extract(&mut self) -> Candidate {
        let bool_sort = self.sorts.bool_sort();
        let sorts: Vec<ResolvedSortId> = self
            .expr_sorts
            .iter()
            .map(|&node| self.unifier.resolve_or_default(self.sorts, node, bool_sort))
            .collect();

        let mut names = self.base_names.clone();
        for &(expr, target) in &self.choices {
            names.insert(expr, target);
        }
        Candidate {
            measure: self.measure.clone(),
            duplicate: false,
            typing: Some((sorts, names)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use merc_syntax::ComplexSort;
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::EquationTyping;
    use crate::ExprId;
    use crate::InferenceError;
    use crate::NameTarget;
    use crate::ResolvedSort;
    use crate::ResolvedSortId;
    use crate::WellTypedError;

    fn typed(text: &str) -> DataSpecification {
        DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap())
            .unwrap_or_else(|err| panic!("expected {text} to typecheck, got {err}"))
    }

    fn inference_error(text: &str) -> InferenceError {
        match DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()) {
            Err(WellTypedError::Inference(error)) => error,
            Err(other) => panic!("expected an inference error for {text}, got {other}"),
            Ok(_) => panic!("expected {text} to be rejected"),
        }
    }

    #[test]
    fn test_infers_basic_equation() {
        let spec = typed("map f: Nat -> Bool; var n: Nat; eqn f(n) = true;");

        // Ids: 0 = `f(n)`, 1 = `n` (arguments before the function), 2 = `f`,
        // 3 = `true`.
        let EquationTyping { sorts, names } = &*spec.equation_typings()[0][0];
        let interner = &spec.context().sorts;
        assert_eq!(sorts[0], interner.bool_sort());
        assert_eq!(sorts[1], interner.nat_sort());
        assert_eq!(sorts[3], interner.bool_sort());
        assert_eq!(names[&ExprId::new(1)], NameTarget::Variable);
    }

    #[test]
    fn test_lambda_over_anonymous_struct_is_inferred() {
        // An anonymous binder struct is hoisted, so a construct binding one is
        // typed like any other equation. (There is no longer a "skipped"
        // outcome: every equation that type checks is fully inferred.)
        let spec = typed("map f: (struct t) -> Bool; g: (struct t) -> Bool; eqn g = lambda x: struct t. f(x);");
        let EquationTyping { .. } = &*spec.equation_typings()[0][0];
    }

    #[test]
    fn test_numeric_literals_stay_positive() {
        let spec = typed("map p: Pos; eqn p = 1 + 2;");

        // Ids: 0 = `p`, 1 = `+(1, 2)`, 2 = `1`, 3 = `2`, 4 = `+`.
        let EquationTyping { sorts, .. } = &*spec.equation_typings()[0][0];
        let interner = &spec.context().sorts;
        assert_eq!(sorts[1], interner.pos_sort());
        assert_eq!(sorts[2], interner.pos_sort());
        assert_eq!(sorts[3], interner.pos_sort());
    }

    #[test]
    fn test_literal_argument_keeps_minimal_sort_under_upcast() {
        let spec = typed("map f: Int -> Bool; b: Bool; eqn b = f(1);");

        // Ids: 0 = `b`, 1 = `f(1)`, 2 = `1`, 3 = `f`. The literal keeps its
        // minimal sort; Phase-4 lowering inserts the upcast to `Int`.
        let EquationTyping { sorts, .. } = &*spec.equation_typings()[0][0];
        let interner = &spec.context().sorts;
        assert_eq!(sorts[1], interner.bool_sort());
        assert_eq!(sorts[2], interner.pos_sort());
    }

    #[test]
    fn test_equality_scheme_on_user_sort() {
        let spec = typed("sort D; cons d: D; map b: Bool; eqn b = d == d;");

        // Ids: 0 = `b`, 1 = `==(d, d)`, 2/3 = `d`, 4 = `==`.
        let EquationTyping { sorts, names } = &*spec.equation_typings()[0][0];
        assert_eq!(names[&ExprId::new(4)], NameTarget::Builtin);
        assert_eq!(sorts[1], spec.context().sorts.bool_sort());
        assert_eq!(sorts[2], spec.sort_of_constructor(merc_syntax::ConstructorId::new(0)));
    }

    #[test]
    fn test_if_scheme_infers_element_sort() {
        let spec = typed("map n: Nat; eqn n = if(true, 1, 2);");

        // Ids: 0 = `n`, 1 = the application, 2 = `true`, 3 = `1`, 4 = `2`,
        // 5 = `if`. The branches stay `Pos`; the join upcasts to `Nat`.
        let EquationTyping { sorts, names } = &*spec.equation_typings()[0][0];
        assert_eq!(names[&ExprId::new(5)], NameTarget::Builtin);
        assert_eq!(sorts[1], spec.context().sorts.pos_sort());
    }

    #[test]
    fn test_free_element_sort_defaults_to_bool() {
        // A free element sort (the element of an empty list whose sort is
        // never constrained by context) defaults to Bool rather than causing
        // UnderdeterminedSort.
        let spec = typed("map b: Bool; eqn b = [] == [];");
        // ExprIds: 0 = `b`, 1 = `==([], [])`, 2 = first `[]`, 3 = second
        // `[]`, 4 = `==`. Both empty lists take List(Bool).
        let (sorts, _) = typing(&spec);
        let interner = &spec.context().sorts;
        for &list_sort in &sorts[2..=3] {
            let ResolvedSort::Generic { op, subsort } = interner.get(list_sort) else {
                panic!("expected a container sort");
            };
            assert_eq!(*op, ComplexSort::List);
            assert_eq!(*subsort, interner.bool_sort());
        }
    }

    #[test]
    fn test_undeclared_name() {
        let text = "map b: Bool; eqn b = undeclared;";
        let error = inference_error(text);
        match &error {
            InferenceError::UndeclaredName { name, span } => {
                assert_eq!(name, "undeclared");
                assert_eq!(&text[span.start..span.end], "undeclared");
            }
            other => panic!("expected UndeclaredName, got {other}"),
        }
    }

    #[test]
    fn test_incompatible_sides_have_no_typing() {
        let text = "map f: Bool; eqn f = 1;";
        let error = inference_error(text);
        match &error {
            InferenceError::NoTyping { equation, span } => {
                // The whole equation (including its trailing `;`) is the
                // offending unit; nothing narrower pins down a sort to blame.
                assert_eq!(&text[span.start..span.end], "f = 1;");
                assert_eq!(equation, "f = 1");
            }
            other => panic!("expected NoTyping, got {other}"),
        }
    }

    #[test]
    fn test_non_boolean_condition() {
        let text = "map f: Nat -> Bool; var n: Nat; eqn n -> f(n) = true;";
        let error = inference_error(text);
        match &error {
            InferenceError::ConditionNotBool { condition, span } => {
                assert_eq!(condition, "n");
                // The span points at the condition `n`, not the variable
                // declaration earlier in the text.
                assert_eq!(&text[span.start..span.end], "n");
                assert_eq!(span.start, text.rfind("n ->").expect("condition is present"));
            }
            other => panic!("expected ConditionNotBool, got {other}"),
        }
    }

    #[test]
    fn test_sides_join_through_upcast() {
        let spec = typed("map f: Nat; eqn f = 1;");

        // Ids: 0 = `f`, 1 = `1`. The literal stays `Pos` and is upcast into
        // the join with the `Nat` left-hand side.
        let EquationTyping { sorts, .. } = &*spec.equation_typings()[0][0];
        let interner = &spec.context().sorts;
        assert_eq!(sorts[0], interner.nat_sort());
        assert_eq!(sorts[1], interner.pos_sort());
    }

    #[test]
    fn test_lambda_infers_function_sort() {
        let spec = typed("map f: Nat -> Bool; eqn f = lambda n: Nat. true;");

        // Ids: 0 = `f`, 1 = `lambda n: Nat. true`, 2 = `true`. The lambda's
        // own sort is the function from its bound variable's declared sort to
        // its body's sort; the bound variable `n` has no id of its own.
        let (sorts, _) = typing(&spec);
        let interner = &spec.context().sorts;
        assert_eq!(sorts[0], spec.sort_of_map(merc_syntax::MapId::new(0)));
        match interner.get(sorts[1]) {
            ResolvedSort::Function { domain, range } => {
                assert_eq!(domain.as_slice(), [interner.nat_sort()]);
                assert_eq!(*range, interner.bool_sort());
            }
            other => panic!("expected a function sort, got {other:?}"),
        }
        assert_eq!(sorts[2], interner.bool_sort());
    }

    #[test]
    fn test_quantifier_infers_bool_sort() {
        let spec = typed("map b: Bool; eqn b = forall n: Nat. n >= 0;");

        // A `forall`/`exists` is always `Bool`, regardless of the body.
        let (sorts, _) = typing(&spec);
        let interner = &spec.context().sorts;
        assert_eq!(sorts[0], interner.bool_sort());
        assert_eq!(sorts[1], interner.bool_sort());
    }

    #[test]
    fn test_quantifier_requires_boolean_body() {
        let text = "map b: Bool; eqn b = forall n: Nat. n;";
        let error = inference_error(text);
        match &error {
            InferenceError::QuantifierNotBool { body, span } => {
                assert_eq!(body, "n");
                // The span points at the body `n`, not the bound variable
                // declaration `n: Nat` just before it.
                assert_eq!(&text[span.start..span.end], "n");
                assert_eq!(span.start, text.len() - 2, "the body is the last token before ';'");
            }
            other => panic!("expected QuantifierNotBool, got {other}"),
        }
    }

    #[test]
    fn test_where_binds_variable_to_assignment_sort() {
        // `x` inside the body takes the sort inferred for its assignment `2`
        // (here upcast to `Nat`, matching `g`'s declared sort), rather than a
        // declared binder sort.
        let spec = typed("map g: Nat; eqn g = (x + 1) whr x = 2 end;");
        let EquationTyping { .. } = &*spec.equation_typings()[0][0];
    }

    #[test]
    fn test_where_assignments_do_not_see_each_other() {
        // Every assignment's right-hand side is typed against the outer
        // scope, not against sibling bindings, so `y`'s `x` resolves to the
        // declared `Nat` variable even though this `whr` also rebinds `x` to
        // a `Bool` (every assignment is typed against the original declared
        // variables, the context being extended once, for the body).
        let spec = typed("map f: Nat -> Bool; var x: Nat; eqn f(x) = true whr x = false, y = x end;");

        // Ids: 0 = `f(x)`, 1 = `x`, 2 = `f`, 3 = the `whr` expression,
        // 4 = `false` (the `x` assignment), 5 = `x` (the `y` assignment,
        // resolved before either name is shadowed), 6 = `true` (the body).
        let (sorts, _) = typing(&spec);
        let interner = &spec.context().sorts;
        assert_eq!(sorts[5], interner.nat_sort());
    }

    /// Extracts the inferred sorts and name targets of the first equation.
    fn typing(spec: &DataSpecification) -> (&[ResolvedSortId], &HashMap<ExprId, NameTarget>) {
        let EquationTyping { sorts, names } = &*spec.equation_typings()[0][0];
        (sorts, names)
    }

    #[test]
    fn test_set_literal_takes_finite_set_sort() {
        let spec = typed("map s: FSet(Pos); eqn s = {1, 2};");

        // Ids: 0 = `s`, 1 = `{1, 2}`, 2 = `1`, 3 = `2`. The literal's sort is
        // the declared `FSet(Pos)`.
        let (sorts, _) = typing(&spec);
        assert_eq!(sorts[1], spec.sort_of_map(merc_syntax::MapId::new(0)));
        assert_eq!(sorts[2], spec.context().sorts.pos_sort());
    }

    #[test]
    fn test_set_literal_widens_to_set_at_use() {
        let spec = typed("map s: Set(Nat); eqn s = {1, 2};");

        // Ids: 0 = `s`, 1 = `{1, 2}`, 2/3 = the literals. The enumeration
        // stays `FSet(Nat)` — Phase-4 lowering materializes the widening to
        // the `Set(Nat)` join — and the literals keep their minimal `Pos`.
        let (sorts, _) = typing(&spec);
        let interner = &spec.context().sorts;
        let ResolvedSort::Generic { op, subsort } = interner.get(sorts[1]) else {
            panic!("expected a container sort");
        };
        assert_eq!(*op, ComplexSort::FSet);
        assert_eq!(*subsort, interner.nat_sort());
        assert_eq!(sorts[2], interner.pos_sort());
    }

    #[test]
    fn test_set_elements_join_to_common_supersort() {
        let spec = typed("map s: FSet(Int); var n: Int; eqn s = {1, n};");

        // Ids: 0 = `s`, 1 = the set, 2 = `1`, 3 = `n`. The element sort is
        // the join `Int`; the literal itself stays `Pos`.
        let (sorts, _) = typing(&spec);
        assert_eq!(sorts[1], spec.sort_of_map(merc_syntax::MapId::new(0)));
        assert_eq!(sorts[2], spec.context().sorts.pos_sort());
        assert_eq!(sorts[3], spec.context().sorts.int_sort());
    }

    #[test]
    fn test_incompatible_set_elements_have_no_typing() {
        let error = inference_error("map s: FSet(Nat); eqn s = {1, true};");
        assert!(matches!(error, InferenceError::NoTyping { .. }), "{error}");
    }

    #[test]
    fn test_empty_set_takes_element_sort_from_context() {
        let spec = typed("map s: Set(Nat); eqn s = {};");

        // Ids: 0 = `s`, 1 = `{}`, typed `FSet(Nat)` under the `Set(Nat)` join.
        let (sorts, _) = typing(&spec);
        let interner = &spec.context().sorts;
        let ResolvedSort::Generic { op, subsort } = interner.get(sorts[1]) else {
            panic!("expected a container sort");
        };
        assert_eq!(*op, ComplexSort::FSet);
        assert_eq!(*subsort, interner.nat_sort());
    }

    #[test]
    fn test_free_empty_set_defaults_to_bool() {
        // Same as test_free_element_sort_defaults_to_bool: a free element sort
        // of an empty finite set defaults to Bool.
        let spec = typed("map b: Bool; eqn b = {} == {};");
        // ExprIds: 0 = `b`, 1 = `==([], [])`, 2 = first `{}`, 3 = second
        // `{}`, 4 = `==`. Both empty sets take FSet(Bool).
        let (sorts, _) = typing(&spec);
        let interner = &spec.context().sorts;
        for &set_sort in &sorts[2..=3] {
            let ResolvedSort::Generic { op, subsort } = interner.get(set_sort) else {
                panic!("expected a container sort");
            };
            assert_eq!(*op, ComplexSort::FSet);
            assert_eq!(*subsort, interner.bool_sort());
        }
    }

    #[test]
    fn test_bag_literal_counts_are_natural() {
        let spec = typed("map b: FBag(Nat); eqn b = {0: 2};");

        // Ids: 0 = `b`, 1 = the bag, 2 = `0`, 3 = the count `2`. The count
        // keeps its minimal `Pos` and is upcast into the `Nat` it must have.
        let (sorts, _) = typing(&spec);
        assert_eq!(sorts[1], spec.sort_of_map(merc_syntax::MapId::new(0)));
        assert_eq!(sorts[2], spec.context().sorts.nat_sort());
        assert_eq!(sorts[3], spec.context().sorts.pos_sort());
    }

    #[test]
    fn test_bag_count_must_be_a_natural_number() {
        let error = inference_error("map b: FBag(Nat); r: Real; eqn b = {0: r};");
        assert!(matches!(error, InferenceError::NoTyping { .. }), "{error}");
    }

    #[test]
    fn test_empty_bag_takes_element_sort_from_context() {
        let spec = typed("map b: Bag(Pos); eqn b = {:};");

        let (sorts, _) = typing(&spec);
        let interner = &spec.context().sorts;
        let ResolvedSort::Generic { op, subsort } = interner.get(sorts[1]) else {
            panic!("expected a container sort");
        };
        assert_eq!(*op, ComplexSort::FBag);
        assert_eq!(*subsort, interner.pos_sort());
    }

    #[test]
    fn test_set_comprehension_from_boolean_body() {
        let spec = typed("map s: Set(Nat); eqn s = { n: Nat | n < 3 };");

        // Ids: 0 = `s`, 1 = the comprehension, 2 = `<(n, 3)`, 3 = `n`,
        // 4 = `3`, 5 = `<`. The boolean body makes a `Set(Nat)`; the bound
        // variable resolves like an equation variable.
        let (sorts, names) = typing(&spec);
        assert_eq!(sorts[1], spec.sort_of_map(merc_syntax::MapId::new(0)));
        assert_eq!(sorts[2], spec.context().sorts.bool_sort());
        assert_eq!(sorts[3], spec.context().sorts.nat_sort());
        assert_eq!(names[&ExprId::new(3)], NameTarget::Variable);
    }

    #[test]
    fn test_bag_comprehension_from_numeric_body() {
        let spec = typed("map b: Bag(Nat); var m: Nat; eqn b = { n: Nat | m };");

        // Ids: 0 = `b`, 1 = the comprehension, 2 = `m`. The `Nat` body reads
        // as the multiplicity function of a `Bag(Nat)`.
        let (sorts, _) = typing(&spec);
        assert_eq!(sorts[1], spec.sort_of_map(merc_syntax::MapId::new(0)));
        assert_eq!(sorts[2], spec.context().sorts.nat_sort());
    }

    #[test]
    fn test_bag_comprehension_from_positive_body() {
        let spec = typed("map b: Bag(Pos); eqn b = { p: Pos | 2 };");

        // The `Pos` body also reads as a bag; the body keeps its minimal sort
        // and Phase-4 lowering inserts the `Pos` → `Nat` coercion, as mCRL2
        // does.
        let (sorts, _) = typing(&spec);
        assert_eq!(sorts[1], spec.sort_of_map(merc_syntax::MapId::new(0)));
        assert_eq!(sorts[2], spec.context().sorts.pos_sort());
    }

    #[test]
    fn test_comprehension_body_must_be_bool_or_number() {
        let error = inference_error("map r: Real; s: Set(Nat); eqn s = { n: Nat | r };");
        assert!(matches!(error, InferenceError::NoTyping { .. }), "{error}");
    }

    #[test]
    fn test_comprehension_readings_can_be_ambiguous() {
        // `f(n)` types as both `Bool` (a set) and `Nat` (a bag), and `==`
        // accepts either pair, so the equation is genuinely ambiguous.
        let error = inference_error(
            "map f: Nat -> Bool; f: Nat -> Nat; b: Bool; eqn b = { n: Nat | f(n) } == { n: Nat | f(n) };",
        );
        assert!(matches!(error, InferenceError::AmbiguousExpression { .. }), "{error}");
    }

    #[test]
    fn test_comprehension_variable_shadows_declarations() {
        // The bound `n: Nat` shadows the boolean map `n` inside the predicate
        // and stops shadowing it after the comprehension.
        let spec = typed("map n: Bool; s: Set(Nat); b: Bool; eqn b = ({ n: Nat | n < 3 } == s) && n;");

        let EquationTyping { .. } = &*spec.equation_typings()[0][0];
    }

    #[test]
    fn test_comprehension_over_alias_and_user_sort() {
        let spec = typed("sort A = Nat; map s: Set(A); eqn s = { a: A | a < 3 };");
        let (sorts, _) = typing(&spec);
        assert_eq!(sorts[1], spec.sort_of_map(merc_syntax::MapId::new(0)));

        typed("sort D = struct d1 | d2; map s: Set(D); eqn s = { x: D | x == d1 };");
    }

    #[test]
    fn test_product_binder_sort_is_rejected() {
        // A bare product is not a valid variable sort; a binder over one is
        // now rejected rather than left untyped (which previously let an
        // ill-typed body slip through unchecked).
        let text = "map s: Set(Nat); eqn s = { x: Nat # Nat | true };";
        let err = inference_error(text);
        match &err {
            InferenceError::InvalidBinderSort { sort, span, .. } => {
                // `Display` parenthesizes the product sort; the span still
                // points at the unparenthesized source text.
                assert_eq!(sort, "(Nat # Nat)");
                // The span covers the whole binder declaration `x: Nat # Nat`
                // (with the trailing whitespace up to the `|`).
                assert_eq!(&text[span.start..span.end], "x: Nat # Nat ");
            }
            other => panic!("expected InvalidBinderSort, got {other}"),
        }
    }

    #[test]
    fn test_polymorphic_membership_over_undeclared_container_sort() {
        // No container sort occurs in any declaration, so `in` exists only
        // through the polymorphic template signature (the mCRL2 corpus uses
        // this pattern heavily: `x in { a, b }` over an enumerated sort).
        let spec = typed("sort D = struct a | b | c; map f: D -> Bool; var x: D; eqn f(x) = x in { a, b };");

        // Ids: 0 = `f(x)`, 1 = `x`, 2 = `f`, 3 = `in(x, {a, b})`, 4 = `x`,
        // 5 = `{a, b}`, 6 = `a`, 7 = `b`, 8 = `in`.
        let (sorts, names) = typing(&spec);
        let interner = &spec.context().sorts;
        let ResolvedSort::Generic { op, subsort } = interner.get(sorts[5]) else {
            panic!("expected a container sort");
        };
        assert_eq!(*op, ComplexSort::FSet);
        assert_eq!(*subsort, spec.sort_of_constructor(merc_syntax::ConstructorId::new(0)));
        assert_eq!(names[&ExprId::new(8)], NameTarget::Builtin);
    }

    #[test]
    fn test_polymorphic_list_operations() {
        // `head` and `|>` come from the list template; no `List` sort is
        // declared anywhere.
        let spec = typed("map n: Nat; eqn n = head([1, 2]);");
        let (sorts, _) = typing(&spec);
        // Ids: 0 = `n`, 1 = `head([1, 2])`. The list elements stay `Pos` and
        // the result is upcast into the `Nat` join.
        assert_eq!(sorts[1], spec.context().sorts.pos_sort());
    }

    #[test]
    fn test_function_update_is_polymorphic() {
        let spec = typed("map f: Nat -> Bool; g: Nat -> Bool; eqn g = f[1 -> true];");

        // Ids: 0 = `g`, 1 = the update, 2 = `f`, 3 = `1`, 4 = `true`,
        // 5 = `@func_update`.
        let (sorts, names) = typing(&spec);
        assert_eq!(sorts[1], spec.sort_of_map(merc_syntax::MapId::new(0)));
        assert_eq!(names[&ExprId::new(5)], NameTarget::Builtin);
    }
}
