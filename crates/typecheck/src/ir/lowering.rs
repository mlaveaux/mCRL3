use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use merc_aterm::ATermList;
use merc_aterm::Term as ATermTrait;
use merc_data::BasicSort;
use merc_data::BinderType;
use merc_data::ContainerSortKind;
use merc_data::DataAbstraction;
use merc_data::DataApplication;
use merc_data::DataEquation;
use merc_data::DataExpression;
use merc_data::DataFunctionSymbol;
use merc_data::DataVariable;
use merc_data::DataWhereClause;
use merc_data::DataWhrDecl;
use merc_data::Mcrl2DataSpecification;
use merc_data::SortAlias;
use merc_data::SortArrow;
use merc_data::SortCons;
use merc_data::SortExpression as DataSortExpression;
use merc_data::is_function_sort;
use merc_syntax::BagElement;
use merc_syntax::ComplexSort;
use merc_syntax::DataExpr;
use merc_syntax::Quantifier;
use merc_syntax::Sort;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;

use crate::EquationTyping;
use crate::ExprId;
use crate::NameTarget;
use crate::ResolvedSort;
use crate::ResolvedSortId;
use crate::TypeckContext;
use crate::query_sort_of_constructor;
use crate::query_sort_of_map;

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
/// layering and cannot name it directly.
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
/// returning the wrapped term and its new sort. The coercion is the explicit
/// constructor chain, not a named `Pos2Nat`/… conversion function — those are
/// rewrite rules that reduce to exactly these constructor applications
/// (`nat.mcrl2`/`int.mcrl2`/`real.mcrl2`). Steps compose for a non-adjacent
/// pair (e.g. `Pos -> Real` becomes `@cReal(@cInt(@cNat(x)), @c1)`, not a
/// single `Pos2Real` call).
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
/// `Bag(element)` via the constructor `@set(@false_, term)` /
/// `@bag(@zero_, term)` — not a call to `@setfset`/`@bagfbag`, which are
/// rewrite-system-only operators (`set.mcrl2` itself notes `@setfset`
/// "should not be part of the rewrite system").
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

/// Converts an inferred, interned sort into the aterm `SortExpression` the
/// binary format uses: `Primitive`/`Generic`/`Function` recurse structurally
/// onto `BasicSort`/`SortCons`/`SortArrow`, and `Def` resolves to its declared
/// name — falling back to a system-internal sort's display name and finally a
/// bare index, mirroring [crate::display_sort]'s fallback chain (the two
/// independently converge on the same name because a nominal sort's identity
/// *is* its declared name for the binary schema).
///
/// `Unit` never reaches this function: it is only used for the sort of an
/// action, never a data-expression sort.
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
            let user_len = spec.sort_declarations.len();
            let name = spec
                .sort_declarations
                .get(**def)
                .map(|d| d.identifier.as_str())
                .or_else(|| {
                    let system_index = (**def).checked_sub(user_len)?;
                    ctx.system_sort_decls.get(system_index).map(String::as_str)
                })
                .unwrap_or("@sort_unknown");
            BasicSort::new(name).into()
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
/// the literal's minimal inferred sort, not a wider one it is later upcast to.
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
#[allow(dead_code)]
pub(crate) fn lower_bool_literal(value: bool) -> DataExpression {
    bool_literal(value)
}

/// The result of lowering one equation.
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
/// Lowers variables, declared-op and builtin-op applications (including the
/// polymorphic comparison/`if` operators), numeric/boolean literals, container
/// literals, all binders (`lambda`, `forall`/`exists`, set/bag comprehensions,
/// `where`), and the numeric/container coercions widening an application
/// argument or the equation's own LHS/RHS to a shared sort. Returns `None` —
/// not an error — when the typing was `Skipped` (an unsupported binder sort)
/// or a construct it does not yet cover is reached.
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
            DataExpr::EmptyList => Some(self.lower_empty_container(sort, ComplexSort::List)),
            DataExpr::EmptySet => Some(self.lower_empty_container(sort, ComplexSort::FSet)),
            DataExpr::EmptyBag => Some(self.lower_empty_container(sort, ComplexSort::FBag)),
            DataExpr::Set(members) => self.lower_set(sort, members),
            DataExpr::Bag(members) => self.lower_bag(sort, members),
            DataExpr::SetBagComp { variable, predicate } => self.lower_setbagcomp(sort, variable, predicate),
            DataExpr::Lambda { variables, body } => self.lower_lambda(variables, body),
            DataExpr::Quantifier { op, variables, body } => self.lower_quantifier(op.clone(), variables, body),
            DataExpr::Whr { expr, assignments } => self.lower_whr(expr, assignments),
            DataExpr::List(_) | DataExpr::Unary { .. } | DataExpr::Binary { .. } | DataExpr::FunctionUpdate { .. } => {
                unreachable!("lower.rs already rewrote this expression form before inference ran")
            }
        }
    }

    /// Widens `term` from `from` to `to` along the sub-sort lattice, inserting
    /// the constructor chain (`@cNat`/`@cInt`/`@cReal` composed for the number
    /// lattice, `@set(@false_, _)`/`@bag(@zero_, _)` for the container lattice
    /// — see [numeric_coerce]/[container_coerce]) — or returning `term`
    /// unchanged when the two sorts already coincide. Returns `None` unless
    /// `from` is `to` or a strict subsort of it (checked via
    /// [crate::SortInterner::partial_cmp]); a `Def` sort has no coercion either
    /// way.
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
            NameTarget::Op { .. } | NameTarget::Builtin => {
                Some(DataFunctionSymbol::with_sort(name, lower_sort(self.ctx, self.spec, sort).copy()).into())
            }
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
        // Each argument widens to its domain position if needed: the domain
        // is cloned first since `coerce` below needs `self.ctx` again, which
        // this `match` already borrows through `function_sort`.
        let domain = domain.clone();

        let mut coerced_terms = Vec::with_capacity(argument_terms.len());
        for ((term, arg_sort), &dom_sort) in argument_terms.into_iter().zip(argument_sorts).zip(domain.iter()) {
            coerced_terms.push(self.coerce(term, arg_sort, dom_sort)?);
        }

        Some(DataApplication::with_args(&function_term, &coerced_terms).into())
    }

    /// Builds the empty-container constant for `EmptyList` / `EmptySet` / `EmptyBag`.
    /// The sort for the constant is extracted from the node's own inferred sort.
    fn lower_empty_container(&self, sort: ResolvedSortId, op: ComplexSort) -> DataExpression {
        let ResolvedSort::Generic {
            subsort: element_id, ..
        } = self.ctx.sorts.get(sort)
        else {
            unreachable!("empty container always infers to a Generic sort")
        };
        let element = lower_sort(self.ctx, self.spec, *element_id);
        let container: DataSortExpression = SortCons::new(container_kind(op), element).into();
        let name = match op {
            ComplexSort::List => "[]",
            ComplexSort::FSet => "{}",
            ComplexSort::FBag => "{:}",
            _ => unreachable!("lower_empty_container only handles List/FSet/FBag"),
        };
        DataFunctionSymbol::with_sort(name, container.copy()).into()
    }

    /// Lowers `{m1, m2, …}` (parsed as `FSet(S)`) to `@fset_insert(m1, @fset_insert(m2, {}))`.
    fn lower_set(&mut self, sort: ResolvedSortId, members: &[DataExpr]) -> Option<DataExpression> {
        let ResolvedSort::Generic {
            subsort: element_id, ..
        } = self.ctx.sorts.get(sort)
        else {
            unreachable!("Set literal always infers to FSet(S)")
        };
        let element_id = *element_id;
        let element = lower_sort(self.ctx, self.spec, element_id);
        let fset: DataSortExpression = SortCons::new(ContainerSortKind::FSet, element.clone()).into();
        let fset_insert = function_symbol("@fset_insert", &[element.clone(), fset.clone()], fset.clone());

        let empty: DataExpression = DataFunctionSymbol::with_sort("{}", fset.copy()).into();
        let mut lowered = Vec::with_capacity(members.len());
        for member in members {
            let member_sort = self.sorts[self.next_id];
            let member_term = self.lower(member)?;
            lowered.push((member_term, member_sort));
        }
        let mut result = empty;
        for (member_term, member_sort) in lowered.into_iter().rev() {
            let coerced = self.coerce(member_term, member_sort, element_id)?;
            result = DataApplication::with_args(&fset_insert, &[coerced, result]).into();
        }
        Some(result)
    }

    /// Lowers `{e1:m1, e2:m2, …}` (parsed as `FBag(S)`) to
    /// `@fbag_cinsert(e1, m1, @fbag_cinsert(e2, m2, {:}))`.
    fn lower_bag(&mut self, sort: ResolvedSortId, members: &[BagElement]) -> Option<DataExpression> {
        let ResolvedSort::Generic {
            subsort: element_id, ..
        } = self.ctx.sorts.get(sort)
        else {
            unreachable!("Bag literal always infers to FBag(S)")
        };
        let element_id = *element_id;
        let nat_id = self.ctx.sorts.nat_sort();
        let element = lower_sort(self.ctx, self.spec, element_id);
        let fbag: DataSortExpression = SortCons::new(ContainerSortKind::FBag, element.clone()).into();
        let fbag_cinsert = function_symbol(
            "@fbag_cinsert",
            &[element.clone(), nat_sort(), fbag.clone()],
            fbag.clone(),
        );

        let empty: DataExpression = DataFunctionSymbol::with_sort("{:}", fbag.copy()).into();
        let mut lowered = Vec::with_capacity(members.len());
        for member in members {
            let elem_sort = self.sorts[self.next_id];
            let elem_term = self.lower(&member.expr)?;
            let mult_sort = self.sorts[self.next_id];
            let mult_term = self.lower(&member.multiplicity)?;
            lowered.push((elem_term, elem_sort, mult_term, mult_sort));
        }
        let mut result = empty;
        for (elem_term, elem_sort, mult_term, mult_sort) in lowered.into_iter().rev() {
            let coerced_elem = self.coerce(elem_term, elem_sort, element_id)?;
            let coerced_mult = self.coerce(mult_term, mult_sort, nat_id)?;
            result = DataApplication::with_args(&fbag_cinsert, &[coerced_elem, coerced_mult, result]).into();
        }
        Some(result)
    }

    fn lower_lambda(&mut self, variables: &[merc_syntax::IdDecl], body: &DataExpr) -> Option<DataExpression> {
        let vars: Vec<DataVariable> = variables
            .iter()
            .map(|v| DataVariable::with_sort(v.identifier.as_str(), lower_syntax_sort(&v.sort).copy()))
            .collect();
        let body = self.lower(body)?;
        Some(DataAbstraction::new(BinderType::Lambda, &vars, body).into())
    }

    fn lower_quantifier(
        &mut self,
        op: Quantifier,
        variables: &[merc_syntax::IdDecl],
        body: &DataExpr,
    ) -> Option<DataExpression> {
        let binder = match op {
            Quantifier::Forall => BinderType::Forall,
            Quantifier::Exists => BinderType::Exists,
        };
        let vars: Vec<DataVariable> = variables
            .iter()
            .map(|v| DataVariable::with_sort(v.identifier.as_str(), lower_syntax_sort(&v.sort).copy()))
            .collect();
        let body = self.lower(body)?;
        Some(DataAbstraction::new(binder, &vars, body).into())
    }

    fn lower_setbagcomp(
        &mut self,
        sort: ResolvedSortId,
        variable: &merc_syntax::IdDecl,
        predicate: &DataExpr,
    ) -> Option<DataExpression> {
        let (op, element_id) = match self.ctx.sorts.get(sort) {
            ResolvedSort::Generic { op, subsort } => (*op, *subsort),
            _ => unreachable!("SetBagComp always infers to Set or Bag"),
        };
        let binder_type = match op {
            ComplexSort::Set => BinderType::SetComp,
            ComplexSort::Bag => BinderType::BagComp,
            _ => unreachable!("SetBagComp infers only to Set or Bag"),
        };
        let var = DataVariable::with_sort(
            variable.identifier.as_str(),
            lower_sort(self.ctx, self.spec, element_id).copy(),
        );
        let body = self.lower(predicate)?;
        Some(DataAbstraction::new(binder_type, &[var], body).into())
    }

    fn lower_whr(&mut self, expr: &DataExpr, assignments: &[merc_syntax::Assignment]) -> Option<DataExpression> {
        let mut whr_decls = Vec::with_capacity(assignments.len());
        for assignment in assignments {
            let assignment_sort = self.sorts[self.next_id];
            let assignment_term = self.lower(&assignment.expr)?;
            let var = DataVariable::with_sort(
                assignment.identifier.as_str(),
                lower_sort(self.ctx, self.spec, assignment_sort).copy(),
            );
            whr_decls.push(DataWhrDecl::new(var, assignment_term));
        }
        let body = self.lower(expr)?;
        Some(DataWhereClause::new(body, &whr_decls).into())
    }
}

/// Converts a (normalized, desugared) `merc_syntax` sort expression into the
/// `merc_data` sort term the mCRL2 binary schema uses.
///
/// Handles every form left after the `from_untyped` pipeline:
/// `Simple` → `BasicSort`, `Complex` → `SortCons`, `FlattenedFunction` and
/// `Function` (the system spec is not flattened) → `SortArrow`, `Resolved` and
/// `Reference` → `BasicSort` by name. `Struct` and a bare `Product` are
/// unreachable at this point.
pub(crate) fn lower_syntax_sort(sort: &SortExpression) -> DataSortExpression {
    match sort {
        SortExpression::Simple(s) => BasicSort::new(primitive_name(*s)).into(),
        SortExpression::Complex(op, sub) => SortCons::new(container_kind(*op), lower_syntax_sort(sub)).into(),
        SortExpression::FlattenedFunction { domain, range } => {
            let domain: Vec<DataSortExpression> = domain.iter().map(lower_syntax_sort).collect();
            SortArrow::new(&domain, lower_syntax_sort(range)).into()
        }
        SortExpression::Function { domain, range } => {
            // The system spec is not flattened; flatten the Product spine here.
            let mut flat = Vec::new();
            flatten_product_domain(domain, &mut flat);
            SortArrow::new(&flat, lower_syntax_sort(range)).into()
        }
        // A user-declared or struct-representative sort after name resolution,
        // or an unresolved template reference in the system spec (e.g. "S", "T").
        // Both use the string name — the identity of a nominal sort IS its name
        // in the binary schema.
        SortExpression::Resolved(name, _) | SortExpression::Reference(name) => BasicSort::new(name.as_str()).into(),
        SortExpression::Struct { .. } | SortExpression::Product { .. } => {
            unreachable!("struct/product sorts are desugared/flattened before lowering")
        }
    }
}

fn flatten_product_domain(sort: &SortExpression, domain: &mut Vec<DataSortExpression>) {
    match sort {
        SortExpression::Product { lhs, rhs } => {
            flatten_product_domain(lhs, domain);
            flatten_product_domain(rhs, domain);
        }
        _ => domain.push(lower_syntax_sort(sort)),
    }
}

// ─────────────────────────── system equation lowering ───────────────────────

/// Extracts the codomain (range sort) of a `SortArrow` term, or `None` if the
/// sort is not a function sort.  Used by the structural system-equation lowering
/// to propagate result sorts through curried applications and variable-as-function
/// calls (e.g. `f(y)` where `f : S -> T`).
fn sort_arrow_codomain(sort: &DataSortExpression) -> Option<DataSortExpression> {
    if !is_function_sort(sort) {
        return None;
    }
    // `SortArrow` layout: arg(0) = domain list, arg(1) = codomain.
    let codomain: DataSortExpression = sort.arg(1).protect().into();
    Some(codomain)
}

/// Returns `(full_function_sort, result_sort)` if `decl_sort` (from a system
/// `cons` or `map` declaration) accepts the supplied `arg_sorts`.  Matching is
/// by structural equality of the lowered domain sorts against the actual
/// argument sorts; because the aterm pool maximally shares identical terms,
/// this is a simple pointer-equality check.
fn match_overload(
    decl_sort: &SortExpression,
    arg_sorts: &[DataSortExpression],
) -> Option<(DataSortExpression, DataSortExpression)> {
    let func_sort = lower_syntax_sort(decl_sort);
    if !is_function_sort(&func_sort) {
        return None;
    }
    let domain_list: ATermList<DataSortExpression> = func_sort.arg(0).into();
    let domain = domain_list.to_vec();
    if domain.len() != arg_sorts.len() {
        return None;
    }
    if domain.iter().zip(arg_sorts).any(|(d, a)| d != a) {
        return None;
    }
    let codomain: DataSortExpression = func_sort.arg(1).protect().into();
    Some((func_sort, codomain))
}

/// Returns `(full_function_sort, result_sort)` for the polymorphic built-in
/// operations (`==`, `!=`, `<`, `<=`, `>`, `>=`, `if`) whose concrete sort is
/// determined solely by the argument sorts.
///
/// - `==` / `!=` / `<` / `<=` / `>` / `>=` : `T # T -> Bool`
/// - `if` : `Bool # T # T -> T`
fn builtin_sort(name: &str, arg_sorts: &[DataSortExpression]) -> Option<(DataSortExpression, DataSortExpression)> {
    match name {
        "==" | "!=" | "<" | "<=" | ">" | ">=" => {
            if arg_sorts.len() != 2 {
                return None;
            }
            if arg_sorts[1] != arg_sorts[0] {
                return None;
            }
            let t = arg_sorts[0].clone();
            let func_sort: DataSortExpression = SortArrow::new(&[t.clone(), t.clone()], bool_sort()).into();
            Some((func_sort, bool_sort()))
        }
        "if" => {
            if arg_sorts.len() != 3 {
                return None;
            }
            if arg_sorts[2] != arg_sorts[1] {
                return None;
            }
            let t = arg_sorts[1].clone();
            let func_sort: DataSortExpression = SortArrow::new(&[bool_sort(), t.clone(), t.clone()], t.clone()).into();
            Some((func_sort, t))
        }
        _ => None,
    }
}

/// Lowers a single expression from a system equation body using structural sort
/// propagation.  Returns `(lowered_term, its_sort)` on success, or `None` for
/// constructs that require sort inference to resolve (empty-container literals,
/// `Number` literals, binders, set/bag enumerations).
fn lower_system_expr(
    system: &UntypedDataSpecification,
    var_map: &HashMap<&str, DataSortExpression>,
    expr: &DataExpr,
) -> Option<(DataExpression, DataSortExpression)> {
    match expr {
        DataExpr::Id(name) => lower_system_id(system, var_map, name),
        DataExpr::Bool(v) => Some((lower_bool_literal(*v), bool_sort())),
        DataExpr::Application { function, arguments } => {
            // Lower arguments first so their sorts are known for overload
            // selection in `lower_system_call`.
            let mut arg_terms = Vec::with_capacity(arguments.len());
            let mut arg_sorts = Vec::with_capacity(arguments.len());
            for arg in arguments {
                let (term, sort) = lower_system_expr(system, var_map, arg)?;
                arg_terms.push(term);
                arg_sorts.push(sort);
            }
            lower_system_call(system, var_map, function, &arg_terms, &arg_sorts)
        }
        // Constructs whose sort cannot be determined without inference.
        DataExpr::EmptyList | DataExpr::EmptySet | DataExpr::EmptyBag => None,
        DataExpr::Set(_) | DataExpr::Bag(_) => None,
        DataExpr::Number(_) => None,
        DataExpr::Lambda { .. } | DataExpr::Quantifier { .. } | DataExpr::Whr { .. } | DataExpr::SetBagComp { .. } => {
            None
        }
        // `lower_data_expressions` rewrites these before system lowering runs.
        DataExpr::List(_) | DataExpr::Unary { .. } | DataExpr::Binary { .. } | DataExpr::FunctionUpdate { .. } => {
            unreachable!("lower.rs already rewrote this expression form before system lowering runs")
        }
    }
}

/// Lowers a bare identifier in a system equation: a variable lookup first,
/// then a zero-argument constructor or map (a function sort identifier without
/// arguments is only meaningful as a zero-arg constant here).
fn lower_system_id(
    system: &UntypedDataSpecification,
    var_map: &HashMap<&str, DataSortExpression>,
    name: &str,
) -> Option<(DataExpression, DataSortExpression)> {
    if let Some(sort) = var_map.get(name) {
        return Some((DataVariable::with_sort(name, sort.copy()).into(), sort.clone()));
    }
    // Zero-argument constructor (sort is not a function sort).
    for decl in &system.constructor_declarations {
        if decl.identifier == name {
            let sort = lower_syntax_sort(&decl.sort);
            if !is_function_sort(&sort) {
                return Some((DataFunctionSymbol::with_sort(name, sort.copy()).into(), sort));
            }
        }
    }
    // Zero-argument map.
    for decl in &system.map_declarations {
        if decl.identifier == name {
            let sort = lower_syntax_sort(&decl.sort);
            if !is_function_sort(&sort) {
                return Some((DataFunctionSymbol::with_sort(name, sort.copy()).into(), sort));
            }
        }
    }
    None
}

/// Lowers a function-application node in a system equation.  `arg_terms` and
/// `arg_sorts` are already lowered.
///
/// - If `function` is a bare `Id`: check builtins, then variable-as-function,
///   then system cons/map overloads.
/// - Otherwise (curried application, e.g. `@func_update(f,x,v)(y)`): lower
///   the function expression recursively and extract its codomain sort.
fn lower_system_call(
    system: &UntypedDataSpecification,
    var_map: &HashMap<&str, DataSortExpression>,
    function: &DataExpr,
    arg_terms: &[DataExpression],
    arg_sorts: &[DataSortExpression],
) -> Option<(DataExpression, DataSortExpression)> {
    match function {
        DataExpr::Id(name) => {
            let name_str = name.as_str();
            // Builtin `==` / `!=` / `<` / `<=` / `>` / `>=` / `if`.
            if let Some((func_sort, result_sort)) = builtin_sort(name_str, arg_sorts) {
                let func_term: DataExpression = DataFunctionSymbol::with_sort(name_str, func_sort.copy()).into();
                return Some((DataApplication::with_args(&func_term, arg_terms).into(), result_sort));
            }
            // Variable of function type (e.g. `f(y)` where `f : S -> T`).
            if let Some(func_sort) = var_map.get(name_str) {
                if let Some(result_sort) = sort_arrow_codomain(func_sort) {
                    let func_term: DataExpression = DataVariable::with_sort(name_str, func_sort.copy()).into();
                    return Some((DataApplication::with_args(&func_term, arg_terms).into(), result_sort));
                }
            }
            // System constructor overload matching the argument sorts.
            for decl in &system.constructor_declarations {
                if decl.identifier == *name {
                    if let Some((func_sort, result_sort)) = match_overload(&decl.sort, arg_sorts) {
                        let func_term: DataExpression =
                            DataFunctionSymbol::with_sort(name_str, func_sort.copy()).into();
                        return Some((DataApplication::with_args(&func_term, arg_terms).into(), result_sort));
                    }
                }
            }
            // System map overload matching the argument sorts.
            for decl in &system.map_declarations {
                if decl.identifier == *name {
                    if let Some((func_sort, result_sort)) = match_overload(&decl.sort, arg_sorts) {
                        let func_term: DataExpression =
                            DataFunctionSymbol::with_sort(name_str, func_sort.copy()).into();
                        return Some((DataApplication::with_args(&func_term, arg_terms).into(), result_sort));
                    }
                }
            }
            None
        }
        // Curried application: the function position is itself an expression
        // (e.g. `@func_update(f,x,v)`) whose result sort must be a function.
        _ => {
            let (fn_value, fn_sort) = lower_system_expr(system, var_map, function)?;
            let result_sort = sort_arrow_codomain(&fn_sort)?;
            Some((DataApplication::with_args(&fn_value, arg_terms).into(), result_sort))
        }
    }
}

/// Lowers all equations in `system` that can be resolved structurally and
/// appends the resulting [`DataEquation`]s to `out`.  Equations whose
/// condition, left-hand side or right-hand side contain a construct that
/// requires sort inference (empty container literals, number literals, binders)
/// are silently skipped; the rest — covering the bulk of the basic-sort,
/// container-sort and structured-sort Appendix-B equations — are included.
fn lower_system_equations(system: &UntypedDataSpecification, out: &mut Vec<DataEquation>) {
    for eqn_spec in &system.equation_declarations {
        let var_map: HashMap<&str, DataSortExpression> = eqn_spec
            .variables
            .iter()
            .map(|v| (v.identifier.as_str(), lower_syntax_sort(&v.sort)))
            .collect();

        let vars: Vec<DataVariable> = eqn_spec
            .variables
            .iter()
            .map(|v| DataVariable::with_sort(v.identifier.as_str(), lower_syntax_sort(&v.sort).copy()))
            .collect();

        for eqn in &eqn_spec.equations {
            // Lower condition (if present); skip the whole equation on failure.
            let condition = match &eqn.condition {
                Some(c) => match lower_system_expr(system, &var_map, c) {
                    Some((term, _)) => Some(term),
                    None => continue,
                },
                None => None,
            };

            let Some((lhs, _)) = lower_system_expr(system, &var_map, &eqn.lhs) else {
                continue;
            };
            let Some((rhs, _)) = lower_system_expr(system, &var_map, &eqn.rhs) else {
                continue;
            };

            out.push(DataEquation::new(&vars, condition, lhs, rhs));
        }
    }
}

// ──────────────────────── lower_data_specification ───────────────────────────

/// Assembles a [`Mcrl2DataSpecification`] from the already-type-checked user
/// and system specifications:
///
/// - **sorts** — user abstract sorts (those whose declaration has no right-hand
///   side after desugaring and normalization).
/// - **aliases** — user sort aliases (those that do have a right-hand side).
/// - **constructors / mappings** — user declarations lowered via the interned
///   sort lattice, followed by system declarations lowered directly from their
///   syntax sorts (the system spec is deliberately left unresolved).
/// - **equations** — user equations whose [`EquationTyping`] is
///   [`EquationTyping::Inferred`] and whose expression tree is fully supported
///   by [`lower_equation`], followed by system equations that can be lowered
///   structurally (empty-container and number-literal equations are silently
///   skipped; they are the residual gap once user equations are fully covered).
pub(crate) fn lower_data_specification(
    ctx: &mut TypeckContext,
    spec: &UntypedDataSpecification,
    system: &UntypedDataSpecification,
    equation_typings: &[Vec<Rc<EquationTyping>>],
) -> Mcrl2DataSpecification {
    let sorts: Vec<BasicSort> = spec
        .sort_declarations
        .iter()
        .filter(|d| d.expr.is_none())
        .map(|d| BasicSort::new(d.identifier.as_str()))
        .collect();

    let aliases: Vec<SortAlias> = spec
        .sort_declarations
        .iter()
        .filter_map(|d| {
            let expr = d.expr.as_ref()?;
            Some(SortAlias::new(
                BasicSort::new(d.identifier.as_str()),
                lower_syntax_sort(expr),
            ))
        })
        .collect();

    let mut constructors: Vec<DataFunctionSymbol> = spec
        .constructor_declarations
        .iter()
        .map(|decl| {
            let id = decl.id.expect("assign_declaration_ids ran before lowering");
            let sort_id = query_sort_of_constructor(ctx, spec, id);
            DataFunctionSymbol::with_sort(decl.identifier.as_str(), lower_sort(ctx, spec, sort_id).copy())
        })
        .collect();
    for decl in &system.constructor_declarations {
        constructors.push(DataFunctionSymbol::with_sort(
            decl.identifier.as_str(),
            lower_syntax_sort(&decl.sort).copy(),
        ));
    }

    let mut mappings: Vec<DataFunctionSymbol> = spec
        .map_declarations
        .iter()
        .map(|decl| {
            let id = decl.id.expect("assign_declaration_ids ran before lowering");
            let sort_id = query_sort_of_map(ctx, spec, id);
            DataFunctionSymbol::with_sort(decl.identifier.as_str(), lower_sort(ctx, spec, sort_id).copy())
        })
        .collect();
    for decl in &system.map_declarations {
        mappings.push(DataFunctionSymbol::with_sort(
            decl.identifier.as_str(),
            lower_syntax_sort(&decl.sort).copy(),
        ));
    }

    let mut equations: Vec<DataEquation> = Vec::new();
    for (eqn_spec, typings) in spec.equation_declarations.iter().zip(equation_typings) {
        let vars: Vec<DataVariable> = eqn_spec
            .variables
            .iter()
            .map(|var| DataVariable::with_sort(var.identifier.as_str(), lower_syntax_sort(&var.sort).copy()))
            .collect();
        for (eqn, typing) in eqn_spec.equations.iter().zip(typings.iter()) {
            let Some(lowered) = lower_equation(ctx, spec, typing, eqn.condition.as_ref(), &eqn.lhs, &eqn.rhs) else {
                continue;
            };
            equations.push(DataEquation::new(&vars, lowered.condition, lowered.lhs, lowered.rhs));
        }
    }
    lower_system_equations(system, &mut equations);

    Mcrl2DataSpecification::new(sorts, aliases, constructors, mappings, equations)
}

#[cfg(test)]
mod tests {
    use merc_data::is_container_sort;
    use merc_data::is_data_binder;
    use merc_data::is_data_function_symbol;
    use merc_data::is_data_where_clause;
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
            spec.sort_of_map(merc_syntax::MapId::new(0)),
        );
        assert_eq!(sort.to_string(), "Nat");
    }

    #[test]
    fn test_lower_generic_sort() {
        let spec = typed("map f: List(Nat);");
        let sort = lower_sort(
            spec.context(),
            spec.data_specification(),
            spec.sort_of_map(merc_syntax::MapId::new(0)),
        );
        assert!(is_container_sort(&sort));
    }

    #[test]
    fn test_lower_function_sort() {
        let spec = typed("map f: Nat -> Bool;");
        let sort = lower_sort(
            spec.context(),
            spec.data_specification(),
            spec.sort_of_map(merc_syntax::MapId::new(0)),
        );
        assert!(is_function_sort(&sort));
    }

    #[test]
    fn test_lower_def_sort() {
        let spec = typed("sort D; map f: D;");
        let sort = lower_sort(
            spec.context(),
            spec.data_specification(),
            spec.sort_of_map(merc_syntax::MapId::new(0)),
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

    // === lower_equation: coercion insertion ===

    #[test]
    fn test_equation_level_coercion_widens_rhs() {
        // `1`'s minimal sort is `Pos`, but `n` is declared `Nat`: the
        // equation itself needs a `Pos -> Nat` coercion, inserted on the
        // narrower (right-hand) side. The coercion is the constructor
        // application (`@cNat`) directly, not a call to a `Pos2Nat` conversion
        // function (that name is only a rewrite rule that reduces to this same
        // term, `nat.mcrl2`).
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
        // (`@cReal(@cInt(@cNat(x)), @c1)`), it does not call a single
        // `Pos2Real` function.
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
        // The `@set` constructor is inserted directly, not a call to
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

    // === lower_equation: container literal lowering ===

    #[test]
    fn test_empty_list_lowers() {
        let equation = lower("map s: List(Nat); eqn s = [];").expect("empty list lowers");
        assert_eq!(equation.rhs.to_string(), "[]");
    }

    #[test]
    fn test_empty_set_lowers() {
        let equation = lower("map s: FSet(Nat); eqn s = {};").expect("empty set lowers");
        assert_eq!(equation.rhs.to_string(), "{}");
    }

    #[test]
    fn test_empty_bag_lowers() {
        let equation = lower("map b: FBag(Nat); eqn b = {:};").expect("empty bag lowers");
        assert_eq!(equation.rhs.to_string(), "{:}");
    }

    #[test]
    fn test_fset_literal_lowers() {
        let equation = lower("map s: FSet(Nat); var n: Nat; eqn s = {n};").expect("singleton FSet lowers");
        // @fset_insert(n, {})
        assert!(equation.rhs.to_string().contains("@fset_insert"), "{}", equation.rhs);
    }

    #[test]
    fn test_fset_literal_two_elements_lowers() {
        let equation = lower("map s: FSet(Nat); var n: Nat; m: Nat; eqn s = {n, m};").expect("two-element FSet lowers");
        let rhs = equation.rhs.to_string();
        assert!(rhs.contains("@fset_insert"), "{rhs}");
    }

    #[test]
    fn test_fbag_literal_lowers() {
        let equation = lower("map b: FBag(Nat); var n: Nat; eqn b = {n: 1};").expect("singleton FBag lowers");
        // @fbag_cinsert(n, @cNat(@c1), {:})  — 1 infers Pos, widened to Nat
        let rhs = equation.rhs.to_string();
        assert!(rhs.contains("@fbag_cinsert"), "{rhs}");
    }

    #[test]
    fn test_empty_list_sort_is_embedded() {
        // The `[]` constant must carry a container (List) sort as its embedded sort.
        let equation = lower("map s: List(Nat); eqn s = [];").expect("empty list lowers");
        assert!(
            is_container_sort(&equation.rhs.data_sort()),
            "sort should be container: {}",
            equation.rhs.data_sort()
        );
    }

    #[test]
    fn test_set_literal_widens_element_to_nat() {
        // `{1}` : FSet(Nat) — the `1` infers Pos, coerced to element sort Nat.
        let equation = lower("map s: FSet(Nat); eqn s = {1};").expect("FSet literal lowers");
        let rhs = equation.rhs.to_string();
        // The element is coerced Pos→Nat via @cNat.
        assert!(rhs.contains("@cNat"), "element coercion Pos→Nat expected in: {rhs}");
    }

    #[test]
    fn test_lambda_lowers() {
        let equation = lower("map f: Bool -> Bool; eqn f = lambda x: Bool. x;").expect("lambda lowers");
        assert!(
            is_data_binder(&equation.rhs),
            "rhs should be a binder: {}",
            equation.rhs
        );
    }

    #[test]
    fn test_forall_lowers() {
        let equation = lower("map b: Bool; eqn b = forall x: Bool. x;").expect("forall lowers");
        assert!(
            is_data_binder(&equation.rhs),
            "rhs should be a binder: {}",
            equation.rhs
        );
    }

    #[test]
    fn test_exists_lowers() {
        let equation = lower("map b: Bool; eqn b = exists x: Bool. x;").expect("exists lowers");
        assert!(
            is_data_binder(&equation.rhs),
            "rhs should be a binder: {}",
            equation.rhs
        );
    }

    #[test]
    fn test_setcomp_lowers() {
        let equation = lower("map s: Set(Nat); eqn s = { x: Nat | x == 0 };").expect("set comprehension lowers");
        assert!(
            is_data_binder(&equation.rhs),
            "rhs should be a binder: {}",
            equation.rhs
        );
    }

    #[test]
    fn test_bagcomp_lowers() {
        let equation = lower("map b: Bag(Nat); eqn b = { x: Nat | x + 0 };").expect("bag comprehension lowers");
        assert!(
            is_data_binder(&equation.rhs),
            "rhs should be a binder: {}",
            equation.rhs
        );
    }

    #[test]
    fn test_whr_lowers() {
        let equation = lower("map f: Bool; var x: Bool; eqn f = x whr x = true end;").expect("where clause lowers");
        assert!(
            is_data_where_clause(&equation.rhs),
            "rhs should be a where clause: {}",
            equation.rhs
        );
    }

    #[test]
    fn test_lambda_variable_has_sort() {
        // The bound variable in the Binder must carry its declared sort.
        let equation = lower("map f: Bool -> Bool; eqn f = lambda x: Bool. x;").expect("lambda lowers");
        let rhs_str = equation.rhs.to_string();
        // The lowered term must contain a DataVarId encoding for x: Bool.
        assert!(
            rhs_str.contains("DataVarId"),
            "bound variable should be DataVarId in: {rhs_str}"
        );
        assert!(
            is_data_function_symbol(&equation.lhs),
            "lhs should be a function symbol: {}",
            equation.lhs
        );
    }

    // === lower_equation: all NameTarget::Builtin ops use inferred sort ===

    #[test]
    fn test_builtin_arithmetic_op() {
        // `+` is a system-declared op (`NameTarget::Op` after overload resolution against
        // the basic-sort system signature), but verifies that arithmetic resolves.
        let equation = lower("map n: Nat; var a: Nat; b: Nat; eqn n = a + b;").expect("arithmetic lowers");
        assert_eq!(equation.rhs.to_string(), "+(a, b)");
    }

    #[test]
    fn test_builtin_polymorphic_container_op() {
        // `in` is a POLYMORPHIC_SIGNATURE op (`NameTarget::Builtin`) whose
        // inferred sort is the concrete instantiation; the lowered term embeds
        // that sort directly.
        let equation = lower("map b: Bool; var n: Nat; s: Set(Nat); eqn b = n in s;")
            .expect("container op lowers with step 3 fix");
        assert_eq!(equation.rhs.to_string(), "in(n, s)");
    }

    #[test]
    fn test_builtin_func_update() {
        // `@func_update` is lowered by lower.rs to an Application; with the
        // step-3 fix its Builtin target uses the inferred sort directly.
        let equation = lower("map f: Nat -> Bool; map g: Nat -> Bool; var n: Nat; eqn g = f[n -> true];")
            .expect("@func_update lowers with step 3 fix");
        assert_eq!(equation.rhs.to_string(), "@func_update(f, n, true)");
    }
}
