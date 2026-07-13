use std::cmp::Ordering;
use std::collections::HashMap;

use merc_data::BasicSort;
use merc_data::ContainerSortKind;
use merc_data::DataApplication;
use merc_data::DataExpression;
use merc_data::DataFunctionSymbol;
use merc_data::DataVariable;
use merc_data::SortArrow;
use merc_data::SortCons;
use merc_data::SortExpression as DataSortExpression;
use merc_syntax::ComplexSort;
use merc_syntax::DataExpr;
use merc_syntax::Sort;
use merc_syntax::UntypedDataSpecification;

use crate::EquationTyping;
use crate::ExprId;
use crate::NameTarget;
use crate::ResolvedSort;
use crate::ResolvedSortId;
use crate::TypeckContext;

/// The mCRL2 name of a basic sort, matching the literal `SortId` names the
/// binary aterm format uses (not `Sort`'s derived `Debug`/`Display`, which
/// happens to coincide but isn't a stated contract).
fn primitive_name(sort: Sort) -> &'static str {
    match sort {
        Sort::Bool => "Bool",
        Sort::Pos => "Pos",
        Sort::Nat => "Nat",
        Sort::Int => "Int",
        Sort::Real => "Real",
    }
}

/// The merc_data container kind for a [ComplexSort]; the two enums are kept
/// separate because `merc_data` sits below `merc_syntax` in the dependency
/// layering (docs/typecheck.md architecture) and cannot name it directly.
fn container_kind(op: ComplexSort) -> ContainerSortKind {
    match op {
        ComplexSort::List => ContainerSortKind::List,
        ComplexSort::Set => ContainerSortKind::Set,
        ComplexSort::FSet => ContainerSortKind::FSet,
        ComplexSort::FBag => ContainerSortKind::FBag,
        ComplexSort::Bag => ContainerSortKind::Bag,
    }
}

/// Widens `term` one step up the number lattice (`Pos <= Nat <= Int <= Real`),
/// returning the wrapped term and its new sort. mCRL2's type checker
/// (`UpCastNumericType`, `typecheck.cpp`) does *not* call a named `Pos2Nat`/…
/// conversion function — those are rewrite rules that reduce to exactly these
/// constructor applications (`nat.mcrl2`/`int.mcrl2`/`real.mcrl2`) — it builds
/// the constructor chain directly, composing steps for a non-adjacent pair
/// (e.g. `Pos -> Real` becomes `@cReal(@cInt(@cNat(x)), @c1)`, not a single
/// `Pos2Real` call).
fn widen_one_step(term: DataExpression, from: Sort) -> (DataExpression, Sort) {
    match from {
        Sort::Pos => {
            let cnat = function_symbol("@cNat", &[pos_sort()], nat_sort());
            (DataApplication::with_args(&cnat, &[term]).into(), Sort::Nat)
        }
        Sort::Nat => {
            let cint = function_symbol("@cInt", &[nat_sort()], int_sort());
            (DataApplication::with_args(&cint, &[term]).into(), Sort::Int)
        }
        Sort::Int => {
            let creal = function_symbol("@cReal", &[int_sort(), pos_sort()], real_sort());
            (
                DataApplication::with_args(&creal, &[term, pos_literal("1")]).into(),
                Sort::Real,
            )
        }
        Sort::Real | Sort::Bool => unreachable!("Real/Bool never widen further"),
    }
}

/// Widens `term` from `from` to `to` in the number lattice, composing
/// [widen_one_step] as many times as needed.
fn numeric_coerce(mut term: DataExpression, from: Sort, to: Sort) -> DataExpression {
    let mut current = from;
    while current != to {
        (term, current) = widen_one_step(term, current);
    }
    term
}

/// Widens `term`, an `FSet(element)`/`FBag(element)`, to `Set(element)`/
/// `Bag(element)` via the constructor mCRL2's type checker actually inserts
/// (`sort_set::constructor`/`sort_bag::constructor`, `typecheck.cpp`):
/// `@set(@false_, term)` / `@bag(@zero_, term)` — not a call to
/// `@setfset`/`@bagfbag`, which are rewrite-system-only operators (`set.mcrl2`
/// itself notes `@setfset` "should not be part of the rewrite system").
fn container_coerce(term: DataExpression, op: ComplexSort, element: DataSortExpression) -> DataExpression {
    match op {
        ComplexSort::FSet => {
            let false_fn = function_symbol("@false_", std::slice::from_ref(&element), bool_sort());
            let set_sort = SortCons::new(ContainerSortKind::Set, element.clone());
            let fset_sort = SortCons::new(ContainerSortKind::FSet, element.clone());
            let predicate_sort: DataSortExpression = SortArrow::new(&[element], bool_sort()).into();
            let set_cons = function_symbol("@set", &[predicate_sort, fset_sort.into()], set_sort.into());
            DataApplication::with_args(&set_cons, &[false_fn.into(), term]).into()
        }
        ComplexSort::FBag => {
            let zero_fn = function_symbol("@zero_", std::slice::from_ref(&element), nat_sort());
            let bag_sort = SortCons::new(ContainerSortKind::Bag, element.clone());
            let fbag_sort = SortCons::new(ContainerSortKind::FBag, element.clone());
            let multiplicity_sort: DataSortExpression = SortArrow::new(&[element], nat_sort()).into();
            let bag_cons = function_symbol("@bag", &[multiplicity_sort, fbag_sort.into()], bag_sort.into());
            DataApplication::with_args(&bag_cons, &[zero_fn.into(), term]).into()
        }
        ComplexSort::List | ComplexSort::Set | ComplexSort::Bag => {
            unreachable!("only FSet and FBag widen to another container")
        }
    }
}

/// Converts an inferred, interned sort into the aterm `SortExpression` mCRL2's
/// binary format uses (§6a/§9a, docs/typecheck.md): `Primitive`/`Generic`/
/// `Function` recurse structurally onto `BasicSort`/`SortCons`/`SortArrow`,
/// and `Def` resolves to its declared name — falling back to a
/// system-internal sort's display name and finally a bare index, mirroring
/// [crate::display_sort]'s fallback chain (the two independently converge on
/// the same name because a nominal sort's identity *is* its declared name for
/// mCRL2's binary schema).
///
/// `Unit` never reaches this function: it is only used for the sort of an
/// action, never a data-expression sort.
// Consumed by the Phase-4 equation re-walk (docs/typecheck.md §9a); exercised by tests only until then.
#[allow(dead_code)]
pub(crate) fn lower_sort(
    ctx: &TypeckContext,
    spec: &UntypedDataSpecification,
    id: ResolvedSortId,
) -> DataSortExpression {
    match ctx.sorts.get(id) {
        ResolvedSort::Unit => {
            unreachable!("Unit is only used for the sort of an action, never a data-expression sort")
        }
        ResolvedSort::Primitive(sort) => BasicSort::new(primitive_name(*sort)).into(),
        ResolvedSort::Generic { op, subsort } => {
            SortCons::new(container_kind(*op), lower_sort(ctx, spec, *subsort)).into()
        }
        ResolvedSort::Function { domain, range } => {
            let domain: Vec<DataSortExpression> = domain.iter().map(|&sort| lower_sort(ctx, spec, sort)).collect();
            SortArrow::new(&domain, lower_sort(ctx, spec, *range)).into()
        }
        ResolvedSort::Def(def) => {
            let name = if let Some(decl) = spec.sort_declarations.get(**def) {
                decl.identifier.clone()
            } else if let Some(name) = ctx.system_sort_names.as_ref().and_then(|names| names.name(*def)) {
                name.to_string()
            } else {
                format!("@sort_{}", **def)
            };
            BasicSort::new(name.as_str()).into()
        }
    }
}

fn pos_sort() -> DataSortExpression {
    BasicSort::new("Pos").into()
}

fn nat_sort() -> DataSortExpression {
    BasicSort::new("Nat").into()
}

fn int_sort() -> DataSortExpression {
    BasicSort::new("Int").into()
}

fn real_sort() -> DataSortExpression {
    BasicSort::new("Real").into()
}

fn bool_sort() -> DataSortExpression {
    BasicSort::new("Bool").into()
}

/// The binary digits of a non-negative decimal literal, least-significant
/// first, computed by repeated long division by two on the decimal digits
/// (so arbitrarily large literals need no fixed-width integer type). The
/// last element is always `true`: a positive number's leading bit is set by
/// definition, and `"0"` is never passed in (see [pos_literal]).
fn decimal_bits_lsb_first(decimal: &str) -> Vec<bool> {
    let mut digits: Vec<u8> = decimal.bytes().map(|b| b - b'0').collect();
    let mut bits = Vec::new();
    while !(digits.len() == 1 && digits[0] == 0) {
        let mut remainder = 0u8;
        for digit in &mut digits {
            let value = remainder * 10 + *digit;
            *digit = value / 2;
            remainder = value % 2;
        }
        bits.push(remainder == 1);
        while digits.len() > 1 && digits[0] == 0 {
            digits.remove(0);
        }
    }
    bits
}

fn bool_literal(value: bool) -> DataExpression {
    constant(if value { "true" } else { "false" }, bool_sort()).into()
}

/// Builds a nullary function symbol (constructor/constant) of `sort`.
fn constant(name: &str, sort: DataSortExpression) -> DataFunctionSymbol {
    DataFunctionSymbol::with_sort(name, sort.copy())
}

/// Builds a function symbol of `domain -> range`.
fn function_symbol(name: &str, domain: &[DataSortExpression], range: DataSortExpression) -> DataFunctionSymbol {
    let sort: DataSortExpression = SortArrow::new(domain, range).into();
    DataFunctionSymbol::with_sort(name, sort.copy())
}

/// Builds the `Pos` term for a positive decimal literal (`"0"` is not valid
/// input; `Pos` has no zero) as the binary `@c1`/`@cDub` chain
/// `crates/syntax/spec/pos.mcrl2` declares: `@cDub(b, p)` denotes `2*p + b`,
/// so the least-significant bit is the *outermost* `@cDub`, built up from the
/// leading (most-significant) bit's `@c1` inward.
fn pos_literal(decimal: &str) -> DataExpression {
    let bits = decimal_bits_lsb_first(decimal);
    debug_assert!(
        *bits.last().expect("a Pos literal has at least one bit"),
        "the leading bit of a Pos literal is always set"
    );

    let cdub = function_symbol("@cDub", &[bool_sort(), pos_sort()], pos_sort());
    let mut term: DataExpression = constant("@c1", pos_sort()).into();
    for &bit in bits[..bits.len() - 1].iter().rev() {
        term = DataApplication::with_args(&cdub, &[bool_literal(bit), term]).into();
    }
    term
}

/// Builds the `Nat` term for a decimal literal: `@c0` for `"0"`, otherwise
/// `@cNat` wrapping the `Pos` term.
fn nat_literal(decimal: &str) -> DataExpression {
    if decimal == "0" {
        constant("@c0", nat_sort()).into()
    } else {
        let cnat = function_symbol("@cNat", &[pos_sort()], nat_sort());
        DataApplication::with_args(&cnat, &[pos_literal(decimal)]).into()
    }
}

/// Builds the `Int` term for a decimal literal. A `Number` node is always a
/// non-negative decimal string (mCRL2 has no negative numeral syntax;
/// negation is the unary `-` operator applied afterwards), so this is always
/// `@cInt`, never `@cNeg`.
fn int_literal(decimal: &str) -> DataExpression {
    let cint = function_symbol("@cInt", &[nat_sort()], int_sort());
    DataApplication::with_args(&cint, &[nat_literal(decimal)]).into()
}

/// Builds the `Real` term for a decimal literal: `@cReal(n, 1)`, matching
/// `Int2Real`'s equation in `crates/syntax/spec/real.mcrl2`.
fn real_literal(decimal: &str) -> DataExpression {
    let creal = function_symbol("@cReal", &[int_sort(), pos_sort()], real_sort());
    DataApplication::with_args(&creal, &[int_literal(decimal), pos_literal("1")]).into()
}

/// Builds the aterm literal for a `DataExpr::Number` node whose *own*
/// inferred sort is `sort` (`Pos`/`Nat`/`Int`/`Real`) — no coercion is
/// inserted here, so the caller must have already established that this is
/// the literal's minimal inferred sort, not a wider one it is later upcast
/// to (§9a step 2, docs/typecheck.md, is the coercion-insertion pass).
// Consumed by the Phase-4 equation re-walk; exercised by tests only until then.
#[allow(dead_code)]
pub(crate) fn lower_number_literal(decimal: &str, sort: Sort) -> DataExpression {
    match sort {
        Sort::Pos => pos_literal(decimal),
        Sort::Nat => nat_literal(decimal),
        Sort::Int => int_literal(decimal),
        Sort::Real => real_literal(decimal),
        Sort::Bool => unreachable!("a Number literal never infers to Bool"),
    }
}

/// Builds the aterm literal for a `DataExpr::Bool` node.
// Consumed by the Phase-4 equation re-walk; exercised by tests only until then.
#[allow(dead_code)]
pub(crate) fn lower_bool_literal(value: bool) -> DataExpression {
    bool_literal(value)
}

/// Names lowered as the polymorphic comparison/`if` schemes: their concrete
/// function sort is exactly the inferred sort of their own `Id` node (no
/// template reverse-engineering needed, unlike the container operations,
/// which are deferred — see [Lowering::lower_id]).
fn is_supported_scheme(name: &str) -> bool {
    matches!(name, "==" | "!=" | "<" | "<=" | ">" | ">=" | "if")
}

/// The result of lowering one equation (§9a step 1, docs/typecheck.md).
// Consumed by the eventual `DataSpecification` assembly (§9a step 5); exercised by tests only until then.
#[allow(dead_code)]
pub(crate) struct LoweredEquation {
    pub(crate) condition: Option<DataExpression>,
    pub(crate) lhs: DataExpression,
    pub(crate) rhs: DataExpression,
}

/// Re-walks one equation's condition/left/right-hand sides alongside its
/// `EquationTyping::Inferred` side tables, in the exact `ExprId` order
/// generation used (documented on `ExprId` in inference.rs: parents before
/// children, arguments before the applied function), building
/// `merc_data::DataExpression`s bottom-up.
///
/// Covers the "foundation + non-binder happy path" slice of Phase 4:
/// variables, user-declared-op applications, the polymorphic comparison/`if`
/// builtins, numeric/boolean literals, and the numeric/container coercions
/// widening an application argument or the equation's own LHS/RHS to a shared
/// sort (§9a step 2). Returns `None` — not an error — the moment the
/// equation needs anything outside that slice (a container literal/operation,
/// `@func_update`, or a binder), which is expected to exclude most
/// real-world equations for now; concrete-builtin/container recovery and
/// binder lowering are follow-up work (§9a steps 3–4).
// Consumed by the eventual `DataSpecification` assembly; exercised by tests only until then.
#[allow(dead_code)]
pub(crate) fn lower_equation(
    ctx: &TypeckContext,
    spec: &UntypedDataSpecification,
    typing: &EquationTyping,
    condition: Option<&DataExpr>,
    lhs: &DataExpr,
    rhs: &DataExpr,
) -> Option<LoweredEquation> {
    let EquationTyping::Inferred { sorts, names } = typing else {
        // Skipped (an unsupported binder sort): nothing to lower.
        return None;
    };

    let mut walker = Lowering {
        ctx,
        spec,
        sorts,
        names,
        next_id: 0,
    };
    let condition = match condition {
        Some(condition) => Some(walker.lower(condition)?),
        None => None,
    };

    // The equation itself joins `lhs` and `rhs` through a shared (possibly
    // wider) sort, exactly like an application's argument against its
    // parameter (see `Lowering::lower_application`): capture each side's own
    // id *before* lowering it, so the narrower side is coerced up to the
    // wider one rather than silently producing an ill-sorted equation.
    let lhs_id = ExprId::new(walker.next_id);
    let lhs = walker.lower(lhs)?;
    let rhs_id = ExprId::new(walker.next_id);
    let rhs = walker.lower(rhs)?;
    let lhs_sort = sorts[*lhs_id];
    let rhs_sort = sorts[*rhs_id];
    let (lhs, rhs) = match ctx.sorts.partial_cmp(lhs_sort, rhs_sort)? {
        Ordering::Equal => (lhs, rhs),
        Ordering::Less => (walker.coerce(lhs, lhs_sort, rhs_sort)?, rhs),
        Ordering::Greater => (lhs, walker.coerce(rhs, rhs_sort, lhs_sort)?),
    };

    Some(LoweredEquation { condition, lhs, rhs })
}

struct Lowering<'a> {
    ctx: &'a TypeckContext,
    spec: &'a UntypedDataSpecification,
    sorts: &'a [ResolvedSortId],
    names: &'a HashMap<ExprId, NameTarget>,
    /// The `ExprId` the next node visited will be assigned, mirroring
    /// `ConstraintGenerator::visit`'s `id = ExprId::new(self.expr_sorts.len())`.
    next_id: usize,
}

impl Lowering<'_> {
    /// Lowers `expr`, consuming exactly the `ExprId`s generation would have
    /// assigned to its subtree, or `None` the moment an unsupported
    /// construct is reached (see [lower_equation]).
    fn lower(&mut self, expr: &DataExpr) -> Option<DataExpression> {
        let id = ExprId::new(self.next_id);
        self.next_id += 1;
        let sort = self.sorts[*id];

        match expr {
            DataExpr::Id(name) => self.lower_id(id, name, sort),
            DataExpr::Number(value) => self.lower_number(sort, value),
            DataExpr::Bool(value) => Some(lower_bool_literal(*value)),
            DataExpr::Application { function, arguments } => self.lower_application(sort, function, arguments),
            // Deferred: container literals/operations and binders (§9a steps 3 and 4).
            DataExpr::EmptyList
            | DataExpr::EmptySet
            | DataExpr::EmptyBag
            | DataExpr::Set(_)
            | DataExpr::Bag(_)
            | DataExpr::SetBagComp { .. }
            | DataExpr::Lambda { .. }
            | DataExpr::Quantifier { .. }
            | DataExpr::Whr { .. } => None,
            DataExpr::List(_) | DataExpr::Unary { .. } | DataExpr::Binary { .. } | DataExpr::FunctionUpdate { .. } => {
                unreachable!("lower.rs already rewrote this expression form before inference ran")
            }
        }
    }

    /// Widens `term` from `from` to `to` along the sub-sort lattice (§9a step
    /// 2), inserting the constructor chain mCRL2's type checker actually
    /// builds (`@cNat`/`@cInt`/`@cReal` composed for the number lattice,
    /// `@set(@false_, _)`/`@bag(@zero_, _)` for the container lattice — see
    /// [numeric_coerce]/[container_coerce]) — or returning `term` unchanged
    /// when the two sorts already coincide. Returns `None` unless `from` is
    /// `to` or a strict subsort of it (checked via
    /// [crate::SortInterner::partial_cmp]); a `Def` sort has no mCRL2
    /// coercion either way.
    fn coerce(&self, term: DataExpression, from: ResolvedSortId, to: ResolvedSortId) -> Option<DataExpression> {
        if from == to {
            return Some(term);
        }
        if self.ctx.sorts.partial_cmp(from, to) != Some(Ordering::Less) {
            return None;
        }

        match (self.ctx.sorts.get(from), self.ctx.sorts.get(to)) {
            (ResolvedSort::Primitive(from_sort), ResolvedSort::Primitive(to_sort)) => {
                Some(numeric_coerce(term, *from_sort, *to_sort))
            }
            (ResolvedSort::Generic { op, subsort }, ResolvedSort::Generic { .. }) => {
                let element = lower_sort(self.ctx, self.spec, *subsort);
                Some(container_coerce(term, *op, element))
            }
            _ => None,
        }
    }

    fn lower_id(&self, id: ExprId, name: &str, sort: ResolvedSortId) -> Option<DataExpression> {
        match self.names.get(&id)? {
            NameTarget::Variable => {
                Some(DataVariable::with_sort(name, lower_sort(self.ctx, self.spec, sort).copy()).into())
            }
            NameTarget::Op { .. } => {
                Some(DataFunctionSymbol::with_sort(name, lower_sort(self.ctx, self.spec, sort).copy()).into())
            }
            NameTarget::Builtin if is_supported_scheme(name) => {
                Some(DataFunctionSymbol::with_sort(name, lower_sort(self.ctx, self.spec, sort).copy()).into())
            }
            // A container operation or `@func_update`: recovering the
            // concrete operator from the template needs the reverse mapping
            // §9a step 3 describes, not yet implemented.
            NameTarget::Builtin => None,
        }
    }

    fn lower_number(&self, sort: ResolvedSortId, value: &str) -> Option<DataExpression> {
        let ResolvedSort::Primitive(sort) = self.ctx.sorts.get(sort) else {
            unreachable!("a Number literal always infers to a primitive numeric sort")
        };
        Some(lower_number_literal(value, *sort))
    }

    fn lower_application(
        &mut self,
        sort: ResolvedSortId,
        function: &DataExpr,
        arguments: &[DataExpr],
    ) -> Option<DataExpression> {
        // Arguments before the applied function, matching generation order.
        let mut argument_terms = Vec::with_capacity(arguments.len());
        let mut argument_sorts = Vec::with_capacity(arguments.len());
        for argument in arguments {
            argument_sorts.push(self.sorts[self.next_id]);
            argument_terms.push(self.lower(argument)?);
        }
        let function_sort = self.sorts[self.next_id];
        let function_term = self.lower(function)?;

        let ResolvedSort::Function { domain, range } = self.ctx.sorts.get(function_sort) else {
            unreachable!("an applied expression always infers to a function sort")
        };
        debug_assert_eq!(*range, sort, "the application's own sort is the function's range");
        if domain.len() != argument_sorts.len() {
            return None;
        }
        // Each argument widens to its domain position if needed (§9a step 2):
        // the domain is cloned first since `coerce` below needs `self.ctx`
        // again, which this `match` already borrows through `function_sort`.
        let domain = domain.clone();

        let mut coerced_terms = Vec::with_capacity(argument_terms.len());
        for ((term, arg_sort), &dom_sort) in argument_terms.into_iter().zip(argument_sorts).zip(domain.iter()) {
            coerced_terms.push(self.coerce(term, arg_sort, dom_sort)?);
        }

        Some(DataApplication::with_args(&function_term, &coerced_terms).into())
    }
}

#[cfg(test)]
mod tests {
    use merc_data::is_container_sort;
    use merc_data::is_function_sort;
    use merc_syntax::Sort;
    use merc_syntax::UntypedDataSpecification;

    use super::LoweredEquation;
    use super::lower_bool_literal;
    use super::lower_equation;
    use super::lower_number_literal;
    use super::lower_sort;
    use crate::DataSpecification;

    fn typed(text: &str) -> DataSpecification {
        DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()).unwrap()
    }

    /// Lowers the single equation of `text`'s only `eqn` block (the shape
    /// every test spec here uses).
    fn lower(text: &str) -> Option<LoweredEquation> {
        let spec = typed(text);
        let eqn_spec = &spec.data_specification().equation_declarations[0];
        let eqn = &eqn_spec.equations[0];
        let typing = &spec.equation_typings()[0][0];
        lower_equation(
            spec.context(),
            spec.data_specification(),
            typing,
            eqn.condition.as_ref(),
            &eqn.lhs,
            &eqn.rhs,
        )
    }

    #[test]
    fn test_lower_primitive_sort() {
        let spec = typed("map f: Nat;");
        let sort = lower_sort(
            spec.context(),
            spec.data_specification(),
            spec.declaration_sorts().mappings[0],
        );
        assert_eq!(sort.to_string(), "Nat");
    }

    #[test]
    fn test_lower_generic_sort() {
        let spec = typed("map f: List(Nat);");
        let sort = lower_sort(
            spec.context(),
            spec.data_specification(),
            spec.declaration_sorts().mappings[0],
        );
        assert!(is_container_sort(&sort));
    }

    #[test]
    fn test_lower_function_sort() {
        let spec = typed("map f: Nat -> Bool;");
        let sort = lower_sort(
            spec.context(),
            spec.data_specification(),
            spec.declaration_sorts().mappings[0],
        );
        assert!(is_function_sort(&sort));
    }

    #[test]
    fn test_lower_def_sort() {
        let spec = typed("sort D; map f: D;");
        let sort = lower_sort(
            spec.context(),
            spec.data_specification(),
            spec.declaration_sorts().mappings[0],
        );
        assert_eq!(sort.to_string(), "D");
    }

    #[test]
    fn test_pos_literals() {
        assert_eq!(lower_number_literal("1", Sort::Pos).to_string(), "@c1");
        assert_eq!(lower_number_literal("2", Sort::Pos).to_string(), "@cDub(false, @c1)");
        assert_eq!(lower_number_literal("3", Sort::Pos).to_string(), "@cDub(true, @c1)");
        assert_eq!(
            lower_number_literal("5", Sort::Pos).to_string(),
            "@cDub(true, @cDub(false, @c1))"
        );
        // 255 = 0b11111111 (all-ones): a `Pos` literal built from a decimal
        // string too large for a machine word exercises the
        // arbitrary-precision long-division encoding, not just a lookup.
        let text = lower_number_literal("255", Sort::Pos).to_string();
        assert_eq!(text.matches("@cDub(true, ").count(), 7, "{text}");
        assert!(text.contains("@c1)"), "{text}");
        assert_eq!(text.matches(')').count(), 7, "{text}");
    }

    #[test]
    fn test_nat_literals() {
        assert_eq!(lower_number_literal("0", Sort::Nat).to_string(), "@c0");
        assert_eq!(
            lower_number_literal("2", Sort::Nat).to_string(),
            "@cNat(@cDub(false, @c1))"
        );
    }

    #[test]
    fn test_int_literal() {
        assert_eq!(lower_number_literal("0", Sort::Int).to_string(), "@cInt(@c0)");
    }

    #[test]
    fn test_real_literal() {
        assert_eq!(
            lower_number_literal("0", Sort::Real).to_string(),
            "@cReal(@cInt(@c0), @c1)"
        );
        assert_eq!(
            lower_number_literal("1", Sort::Real).to_string(),
            "@cReal(@cInt(@cNat(@c1)), @c1)"
        );
    }

    #[test]
    fn test_bool_literals() {
        assert_eq!(lower_bool_literal(true).to_string(), "true");
        assert_eq!(lower_bool_literal(false).to_string(), "false");
    }

    #[test]
    fn test_literal_sort_is_embedded() {
        // The `@cDub` `OpId` embeds its own (function) sort, `Bool # Pos -> Pos`.
        let cdub = lower_number_literal("2", Sort::Pos);
        assert!(is_function_sort(&cdub.data_function_symbol().sort()));
    }

    // === lower_equation: the non-binder happy path ===

    #[test]
    fn test_user_op_application_no_coercion() {
        let equation = lower("map f: Bool -> Bool; var x: Bool; eqn f(x) = x;").expect("no coercion, no binder");
        assert_eq!(equation.lhs.to_string(), "f(x)");
        assert_eq!(equation.rhs.to_string(), "x");
    }

    #[test]
    fn test_comparison_scheme_on_declared_sort() {
        let equation = lower("sort D; cons d: D; map b: Bool; eqn b = (d == d);").expect("== is a supported scheme");
        assert_eq!(equation.lhs.to_string(), "b");
        assert_eq!(equation.rhs.to_string(), "==(d, d)");
    }

    #[test]
    fn test_if_scheme_on_declared_sort() {
        let equation = lower("sort D; cons d: D; map f: D; eqn f = if(true, d, d);").expect("if is a supported scheme");
        assert_eq!(equation.rhs.to_string(), "if(true, d, d)");
    }

    #[test]
    fn test_literal_at_its_natural_sort() {
        // `1`'s minimal inferred sort is `Pos`, exactly `p`'s declared sort:
        // no coercion needed.
        let equation = lower("map p: Pos; eqn p = 1;").expect("no coercion needed");
        assert_eq!(equation.rhs.to_string(), "@c1");
    }

    #[test]
    fn test_zero_literal_at_nat_sort() {
        let equation = lower("map n: Nat; eqn n = 0;").expect("0 is already Nat");
        assert_eq!(equation.rhs.to_string(), "@c0");
    }

    // === lower_equation: coercion insertion (§9a step 2) ===

    #[test]
    fn test_equation_level_coercion_widens_rhs() {
        // `1`'s minimal sort is `Pos`, but `n` is declared `Nat`: the
        // equation itself needs a `Pos -> Nat` coercion, inserted on the
        // narrower (right-hand) side. mCRL2's type checker builds the
        // constructor application directly (`@cNat`), not a call to a
        // `Pos2Nat` conversion function (that name is only a rewrite rule
        // that reduces to this same term, `nat.mcrl2`).
        let equation = lower("map n: Nat; eqn n = 1;").expect("Pos widens to Nat");
        assert_eq!(equation.rhs.to_string(), "@cNat(@c1)");
    }

    #[test]
    fn test_equation_level_coercion_widens_lhs() {
        // Symmetric to the above, with the narrower side on the left.
        let equation = lower("map n: Nat; eqn 1 = n;").expect("Pos widens to Nat");
        assert_eq!(equation.lhs.to_string(), "@cNat(@c1)");
        assert_eq!(equation.rhs.to_string(), "n");
    }

    #[test]
    fn test_direct_coercion_composes_intermediate_sorts() {
        // A `Pos -> Real` coercion composes every intermediate constructor
        // mCRL2's `UpCastNumericType` would (`@cReal(@cInt(@cNat(x)), @c1)`),
        // it does not call a single `Pos2Real` function.
        let equation = lower("map r: Real; eqn r = 1;").expect("Pos widens to Real");
        assert_eq!(equation.rhs.to_string(), "@cReal(@cInt(@cNat(@c1)), @c1)");
    }

    #[test]
    fn test_argument_coercion_widens_to_domain() {
        // `f`'s parameter is `Nat`, but `1` naturally infers to `Pos`: an
        // argument coercion.
        let equation = lower("map f: Nat -> Bool; eqn f(1) = true;").expect("Pos widens to Nat");
        assert_eq!(equation.lhs.to_string(), "f(@cNat(@c1))");
    }

    #[test]
    fn test_fset_argument_widens_to_set() {
        // mCRL2's type checker inserts the `@set` constructor directly
        // (`sort_set::constructor`, `typecheck.cpp`), not a call to
        // `@setfset` (a rewrite-system-only operator, per `set.mcrl2`'s own
        // comment that it "should not be part of the rewrite system").
        let equation =
            lower("map e: FSet(Nat); map s: Set(Nat) -> Bool; eqn s(e) = true;").expect("FSet widens to Set");
        assert_eq!(equation.lhs.to_string(), "s(@set(@false_, e))");
    }

    #[test]
    fn test_fbag_argument_widens_to_bag() {
        let equation =
            lower("map e: FBag(Nat); map s: Bag(Nat) -> Bool; eqn s(e) = true;").expect("FBag widens to Bag");
        assert_eq!(equation.lhs.to_string(), "s(@bag(@zero_, e))");
    }

    #[test]
    fn test_container_literal_bails() {
        assert!(lower("map s: List(Nat); eqn s = [];").is_none());
    }

    #[test]
    fn test_binder_bails() {
        assert!(lower("map f: Bool -> Bool; eqn f = lambda x: Bool. x;").is_none());
    }
}
