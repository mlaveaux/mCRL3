use std::marker::PhantomData;
use std::ops::ControlFlow;

use crate::DataAbstractionRef;
use crate::DataApplicationRef;
use crate::DataExpression;
use crate::DataExpressionRef;
use crate::DataFunctionSymbolRef;
use crate::DataMachineNumberRef;
use crate::DataUntypedIdentifierRef;
use crate::DataVariable;
use crate::DataVariableRef;
use crate::DataWhereClauseRef;
use crate::PbesAndRef;
use crate::PbesExistsRef;
use crate::PbesExpression;
use crate::PbesExpressionRef;
use crate::PbesForallRef;
use crate::PbesImpRef;
use crate::PbesNotRef;
use crate::PbesOrRef;
use crate::PbesPropositionalVariableInstantiation;
use crate::PbesPropositionalVariableInstantiationRef;
use crate::is_abstraction;
use crate::is_application;
use crate::is_data_expression;
use crate::is_function_symbol;
use crate::is_machine_number;
use crate::is_pbes_and;
use crate::is_pbes_exists;
use crate::is_pbes_forall;
use crate::is_pbes_imp;
use crate::is_pbes_not;
use crate::is_pbes_or;
use crate::is_pbes_propositional_variable_instantiation;
use crate::is_untyped_identifier;
use crate::is_variable;
use crate::is_where_clause;

pub trait DataExpressionVisitor {
    fn visit_variable(&mut self, _var: &DataVariableRef<'_>) -> Option<DataExpression> {
        None
    }

    fn visit_application(&mut self, appl: &DataApplicationRef<'_>) -> Option<DataExpression> {
        let _head = self.visit(&appl.data_function_symbol().into());

        appl.data_arguments().for_each(|arg| {
            self.visit(&arg.into());
        });

        None
    }

    fn visit_abstraction(&mut self, abstraction: &DataAbstractionRef<'_>) -> Option<DataExpression> {
        let _body = self.visit(&abstraction.body());
        None
    }

    fn visit_function_symbol(&mut self, _function_symbol: &DataFunctionSymbolRef<'_>) -> Option<DataExpression> {
        None
    }

    fn visit_where_clause(&mut self, where_: &DataWhereClauseRef<'_>) -> Option<DataExpression> {
        for decl in where_.declarations().iter() {
            let rhs = DataExpression::new(decl.arg(1).protect());
            self.visit(&rhs.copy());
        }
        self.visit(&where_.body());
        None
    }

    fn visit_machine_number(&mut self, _number: &DataMachineNumberRef<'_>) -> Option<DataExpression> {
        None
    }

    fn visit_untyped_identifier(&mut self, _identifier: &DataUntypedIdentifierRef<'_>) -> Option<DataExpression> {
        None
    }

    fn visit(&mut self, expr: &DataExpressionRef<'_>) -> Option<DataExpression> {
        if is_variable(&expr.copy()) {
            self.visit_variable(&DataVariableRef::from(expr.copy()))
        } else if is_application(&expr.copy()) {
            self.visit_application(&DataApplicationRef::from(expr.copy()))
        } else if is_abstraction(&expr.copy()) {
            self.visit_abstraction(&DataAbstractionRef::from(expr.copy()))
        } else if is_function_symbol(&expr.copy()) {
            self.visit_function_symbol(&DataFunctionSymbolRef::from(expr.copy()))
        } else if is_where_clause(&expr.copy()) {
            self.visit_where_clause(&DataWhereClauseRef::from(expr.copy()))
        } else if is_machine_number(&expr.copy()) {
            self.visit_machine_number(&DataMachineNumberRef::from(expr.copy()))
        } else if is_untyped_identifier(&expr.copy()) {
            self.visit_untyped_identifier(&DataUntypedIdentifierRef::from(expr.copy()))
        } else {
            unreachable!("Unknown data expression type");
        }
    }
}

pub trait PbesExpressionVisitor {
    fn visit_propositional_variable_instantiation(
        &mut self,
        _inst: &PbesPropositionalVariableInstantiationRef<'_>,
    ) -> Option<PbesExpression> {
        None
    }

    fn visit_not(&mut self, not: &PbesNotRef<'_>) -> Option<PbesExpression> {
        self.visit(&not.body());
        None
    }

    fn visit_and(&mut self, and: &PbesAndRef<'_>) -> Option<PbesExpression> {
        self.visit(&and.lhs());
        self.visit(&and.rhs());
        None
    }

    fn visit_or(&mut self, or: &PbesOrRef<'_>) -> Option<PbesExpression> {
        self.visit(&or.lhs());
        self.visit(&or.rhs());
        None
    }

    fn visit_imp(&mut self, imp: &PbesImpRef<'_>) -> Option<PbesExpression> {
        self.visit(&imp.lhs());
        self.visit(&imp.rhs());
        None
    }

    fn visit_forall(&mut self, forall: &PbesForallRef<'_>) -> Option<PbesExpression> {
        self.visit(&forall.body());
        None
    }

    fn visit_exists(&mut self, exists: &PbesExistsRef<'_>) -> Option<PbesExpression> {
        self.visit(&exists.body());
        None
    }

    /// Visits a data expression. By default, this does nothing, but it can be overridden to also visit data expressions when visiting PBES expressions.
    fn visit_data_expression(&mut self, _expr: &DataExpressionRef<'_>) -> Option<DataExpression> {
        None
    }

    fn visit(&mut self, expr: &PbesExpressionRef<'_>) -> Option<PbesExpression> {
        if is_pbes_propositional_variable_instantiation(&expr.copy()) {
            self.visit_propositional_variable_instantiation(&PbesPropositionalVariableInstantiationRef::from(
                expr.copy(),
            ))
        } else if is_pbes_not(&expr.copy()) {
            self.visit_not(&PbesNotRef::from(expr.copy()))
        } else if is_pbes_and(&expr.copy()) {
            self.visit_and(&PbesAndRef::from(expr.copy()))
        } else if is_pbes_or(&expr.copy()) {
            self.visit_or(&PbesOrRef::from(expr.copy()))
        } else if is_pbes_imp(&expr.copy()) {
            self.visit_imp(&PbesImpRef::from(expr.copy()))
        } else if is_pbes_forall(&expr.copy()) {
            self.visit_forall(&PbesForallRef::from(expr.copy()))
        } else if is_pbes_exists(&expr.copy()) {
            self.visit_exists(&PbesExistsRef::from(expr.copy()))
        } else if is_data_expression(&expr.copy()) {
            self.visit_data_expression(&expr.copy().into())
                .map(|inner| inner.into())
        } else {
            unreachable!("Unknown pbes expression type");
        }
    }
}

/// Returns all the PVIs occurring in the given PBES expression.
pub fn pbes_expression_pvi(expr: &PbesExpressionRef<'_>) -> Vec<PbesPropositionalVariableInstantiation> {
    let mut result = Vec::new();

    /// Local struct that is used to collect PVI occurrences.
    struct PviOccurrences<'a> {
        result: &'a mut Vec<PbesPropositionalVariableInstantiation>,
    }

    impl PbesExpressionVisitor for PviOccurrences<'_> {
        fn visit_propositional_variable_instantiation(
            &mut self,
            inst: &PbesPropositionalVariableInstantiationRef<'_>,
        ) -> Option<PbesExpression> {
            self.result.push(inst.protect());
            None
        }
    }

    let mut occurrences = PviOccurrences { result: &mut result };
    occurrences.visit(expr);
    result
}

/// Returns all the variables occurring in the given data expression.
///
/// This returns occurrences, so a variable that appears multiple times is listed
/// multiple times; deduplicate at the call site if a set is required.
pub fn variable_occurrences_data_expression(expr: &DataExpressionRef<'_>) -> Vec<DataVariable> {
    let mut result = Vec::new();

    /// Local struct that is used to collect variable occurrences.
    struct VariableOccurrences<'a> {
        result: &'a mut Vec<DataVariable>,
    }

    impl DataExpressionVisitor for VariableOccurrences<'_> {
        fn visit_variable(&mut self, var: &DataVariableRef<'_>) -> Option<DataExpression> {
            self.result.push(var.protect());
            None
        }
    }

    let mut occurrences = VariableOccurrences { result: &mut result };
    occurrences.visit(expr);
    result
}

/// Returns all the free variables of the given data expression.
///
/// This returns occurrences, so a free variable that appears multiple times is
/// listed multiple times; deduplicate at the call site if a set is required.
pub fn free_variables_data_expression(expr: &DataExpressionRef<'_>) -> Vec<DataVariable> {
    let mut result = Vec::new();

    /// Local struct that is used to collect variable occurrences.
    struct FreeVariableOccurrences<'a> {
        result: &'a mut Vec<DataVariable>,

        /// Keeps track of variables that are bound in the context, since these are not free.
        context: Vec<DataVariable>,
    }

    impl DataExpressionVisitor for FreeVariableOccurrences<'_> {
        fn visit_abstraction(&mut self, abstraction: &DataAbstractionRef<'_>) -> Option<DataExpression> {
            let prev_len = self.context.len();
            self.context.extend(abstraction.variables().iter());
            self.visit(&abstraction.body());
            self.context.truncate(prev_len);
            None
        }

        fn visit_where_clause(&mut self, where_: &DataWhereClauseRef<'_>) -> Option<DataExpression> {
            // Where clauses are non-recursive: RHS expressions see the outer scope.
            for decl in where_.declarations().iter() {
                let rhs = DataExpression::new(decl.arg(1).protect());
                self.visit(&rhs.copy());
            }
            // The body sees the outer scope extended with the declared variables.
            let prev_len = self.context.len();
            for decl in where_.declarations().iter() {
                self.context.push(DataVariable::new(decl.arg(0).protect()));
            }
            self.visit(&where_.body());
            self.context.truncate(prev_len);
            None
        }

        fn visit_variable(&mut self, var: &DataVariableRef<'_>) -> Option<DataExpression> {
            if !self.context.contains(&var.protect()) {
                self.result.push(var.protect());
            }
            None
        }
    }

    let mut occurrences = FreeVariableOccurrences {
        result: &mut result,
        context: Vec::new(),
    };
    occurrences.visit(expr);
    result
}

/// Recursively collects the arguments of a nested application chain where `is_op` matches,
/// flattening any depth of nesting. Intermediate matching applications are not included.
pub fn flatten_associative<F>(expr: &DataExpressionRef<'_>, mut is_op: F) -> Vec<DataExpression>
where
    F: FnMut(&DataExpressionRef<'_>) -> bool,
{
    let mut result = Vec::new();
    collect_flat(expr, &mut is_op, &mut result);
    result
}

fn collect_flat<F>(expr: &DataExpressionRef<'_>, is_op: &mut F, out: &mut Vec<DataExpression>)
where
    F: FnMut(&DataExpressionRef<'_>) -> bool,
{
    if is_op(expr) {
        for arg in DataApplicationRef::from(expr.copy()).data_arguments() {
            let owned = DataExpression::new(arg.protect());
            collect_flat(&owned.copy(), is_op, out);
        }
    } else {
        out.push(expr.copy().protect());
    }
}

/// Controls how [`DataExpressionContextVisitor::try_visit`] proceeds below the current node.
///
/// Mirrors `SortDescend` from the syntax crate; see [`PbesDescend`] for the PBES variant.
pub enum DataDescend<C> {
    /// Visit children with this context.
    Descend(C),
    /// Do not visit children.
    Prune,
}

/// Context-threading traversal over data expressions. Override [`visit_node`] to inspect each
/// node; the provided [`try_visit`] drives recursion into children automatically.
///
/// Use [`ClosureVisitor`] or call [`try_visit_data_expr_with`] to drive traversal from a closure.
///
/// [`visit_node`]: DataExpressionContextVisitor::visit_node
/// [`try_visit`]: DataExpressionContextVisitor::try_visit
pub trait DataExpressionContextVisitor {
    type Context: Copy;
    type Break;
    type Error;

    fn visit_variable(
        &mut self,
        _var: &DataVariableRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, DataDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(DataDescend::Descend(ctx)))
    }

    fn visit_application(
        &mut self,
        _appl: &DataApplicationRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, DataDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(DataDescend::Descend(ctx)))
    }

    fn visit_abstraction(
        &mut self,
        _abstraction: &DataAbstractionRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, DataDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(DataDescend::Descend(ctx)))
    }

    fn visit_function_symbol(
        &mut self,
        _function_symbol: &DataFunctionSymbolRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, DataDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(DataDescend::Descend(ctx)))
    }

    fn visit_where_clause(
        &mut self,
        _where_: &DataWhereClauseRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, DataDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(DataDescend::Descend(ctx)))
    }

    fn visit_machine_number(
        &mut self,
        _number: &DataMachineNumberRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, DataDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(DataDescend::Descend(ctx)))
    }

    fn visit_untyped_identifier(
        &mut self,
        _identifier: &DataUntypedIdentifierRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, DataDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(DataDescend::Descend(ctx)))
    }

    fn try_visit(
        &mut self,
        expr: &DataExpressionRef<'_>,
        ctx: Self::Context,
    ) -> Result<Option<Self::Break>, Self::Error> {
        if is_variable(&expr.copy()) {
            let var = DataVariableRef::from(expr.copy());
            match self.visit_variable(&var, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(_) => {} // leaf
            }
        } else if is_application(&expr.copy()) {
            let appl = DataApplicationRef::from(expr.copy());
            let ctx = match self.visit_application(&appl, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(DataDescend::Prune) => return Ok(None),
                ControlFlow::Continue(DataDescend::Descend(ctx)) => ctx,
            };
            for arg in appl.data_arguments() {
                let arg_ref: DataExpressionRef<'_> = arg.into();
                if let Some(result) = self.try_visit(&arg_ref, ctx)? {
                    return Ok(Some(result));
                }
            }
        } else if is_abstraction(&expr.copy()) {
            let abstraction = DataAbstractionRef::from(expr.copy());
            let ctx = match self.visit_abstraction(&abstraction, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(DataDescend::Prune) => return Ok(None),
                ControlFlow::Continue(DataDescend::Descend(ctx)) => ctx,
            };
            if let Some(result) = self.try_visit(&abstraction.body(), ctx)? {
                return Ok(Some(result));
            }
        } else if is_function_symbol(&expr.copy()) {
            let fs = DataFunctionSymbolRef::from(expr.copy());
            match self.visit_function_symbol(&fs, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(_) => {} // leaf
            }
        } else if is_where_clause(&expr.copy()) {
            let where_ = DataWhereClauseRef::from(expr.copy());
            let ctx = match self.visit_where_clause(&where_, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(DataDescend::Prune) => return Ok(None),
                ControlFlow::Continue(DataDescend::Descend(ctx)) => ctx,
            };
            for decl in where_.declarations().iter() {
                let rhs = DataExpression::new(decl.arg(1).protect());
                if let Some(result) = self.try_visit(&rhs.copy(), ctx)? {
                    return Ok(Some(result));
                }
            }
            if let Some(result) = self.try_visit(&where_.body(), ctx)? {
                return Ok(Some(result));
            }
        } else if is_machine_number(&expr.copy()) {
            let number = DataMachineNumberRef::from(expr.copy());
            match self.visit_machine_number(&number, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(_) => {} // leaf
            }
        } else if is_untyped_identifier(&expr.copy()) {
            let identifier = DataUntypedIdentifierRef::from(expr.copy());
            match self.visit_untyped_identifier(&identifier, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(_) => {} // leaf
            }
        } else {
            unreachable!("Unknown data expression type");
        }

        Ok(None)
    }
}

/// Adapts a `FnMut` closure into a [`DataExpressionContextVisitor`].
pub struct ClosureVisitor<C, T, E, F> {
    f: F,
    _marker: PhantomData<fn(C) -> (T, E)>,
}

impl<C, T, E, F> ClosureVisitor<C, T, E, F> {
    pub fn new(f: F) -> Self {
        ClosureVisitor { f, _marker: PhantomData }
    }
}

impl<C, T, E, F> DataExpressionContextVisitor for ClosureVisitor<C, T, E, F>
where
    C: Copy,
    F: FnMut(&DataExpressionRef<'_>, C) -> Result<ControlFlow<T, DataDescend<C>>, E>,
{
    type Context = C;
    type Break = T;
    type Error = E;

    fn visit_variable(&mut self, var: &DataVariableRef<'_>, ctx: C) -> Result<ControlFlow<T, DataDescend<C>>, E> {
        (self.f)(&DataExpressionRef::from(var.copy()), ctx)
    }

    fn visit_application(&mut self, appl: &DataApplicationRef<'_>, ctx: C) -> Result<ControlFlow<T, DataDescend<C>>, E> {
        (self.f)(&DataExpressionRef::from(appl.copy()), ctx)
    }

    fn visit_abstraction(&mut self, abstraction: &DataAbstractionRef<'_>, ctx: C) -> Result<ControlFlow<T, DataDescend<C>>, E> {
        (self.f)(&DataExpressionRef::from(abstraction.copy()), ctx)
    }

    fn visit_function_symbol(&mut self, fs: &DataFunctionSymbolRef<'_>, ctx: C) -> Result<ControlFlow<T, DataDescend<C>>, E> {
        (self.f)(&DataExpressionRef::from(fs.copy()), ctx)
    }

    fn visit_where_clause(&mut self, where_: &DataWhereClauseRef<'_>, ctx: C) -> Result<ControlFlow<T, DataDescend<C>>, E> {
        (self.f)(&DataExpressionRef::from(where_.copy()), ctx)
    }

    fn visit_machine_number(&mut self, number: &DataMachineNumberRef<'_>, ctx: C) -> Result<ControlFlow<T, DataDescend<C>>, E> {
        (self.f)(&DataExpressionRef::from(number.copy()), ctx)
    }

    fn visit_untyped_identifier(&mut self, identifier: &DataUntypedIdentifierRef<'_>, ctx: C) -> Result<ControlFlow<T, DataDescend<C>>, E> {
        (self.f)(&DataExpressionRef::from(identifier.copy()), ctx)
    }
}

/// Visits subexpressions of a data expression top-down, routing through [`DataExpressionContextVisitor`].
pub fn try_visit_data_expr_with<C, T, E, F>(
    expr: &DataExpressionRef<'_>,
    ctx: C,
    visitor: F,
) -> Result<Option<T>, E>
where
    C: Copy,
    F: FnMut(&DataExpressionRef<'_>, C) -> Result<ControlFlow<T, DataDescend<C>>, E>,
{
    ClosureVisitor::<C, T, E, F>::new(visitor).try_visit(expr, ctx)
}

/// Non-fallible convenience wrapper for [`try_visit_data_expr_with`].
pub fn visit_data_expr_with<T, C, F>(expr: &DataExpressionRef<'_>, ctx: C, mut visitor: F) -> Option<T>
where
    C: Copy,
    F: FnMut(&DataExpressionRef<'_>, C) -> ControlFlow<T, DataDescend<C>>,
{
    use std::convert::Infallible;
    try_visit_data_expr_with(expr, ctx, |e, c| -> Result<_, Infallible> { Ok(visitor(e, c)) })
        .expect("infallible")
}

/// Controls how [`PbesExpressionContextVisitor::try_visit`] proceeds below the current node.
pub enum PbesDescend<C> {
    Descend(C),
    Prune,
}

/// Adapts a `FnMut` closure into a [`PbesExpressionContextVisitor`].
pub struct PbesClosureVisitor<C, T, E, F> {
    f: F,
    _marker: PhantomData<fn(C) -> (T, E)>,
}

impl<C, T, E, F> PbesClosureVisitor<C, T, E, F> {
    pub fn new(f: F) -> Self {
        PbesClosureVisitor { f, _marker: PhantomData }
    }
}

/// Context-threading traversal over PBES expressions, consistent with [`DataExpressionContextVisitor`].
///
/// Data-expression leaves (e.g. condition terms) are passed to [`visit_node`] but their
/// subterms are **not** recursed into automatically; use [`try_visit_data_expr_with`]
/// from inside [`visit_node`] to continue into data subterms if desired.
///
/// [`visit_node`]: PbesExpressionContextVisitor::visit_node
pub trait PbesExpressionContextVisitor {
    type Context: Copy;
    type Break;
    type Error;

    fn visit_propositional_variable_instantiation(
        &mut self,
        _inst: &PbesPropositionalVariableInstantiationRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, PbesDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(PbesDescend::Descend(ctx)))
    }

    fn visit_not(
        &mut self,
        _not: &PbesNotRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, PbesDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(PbesDescend::Descend(ctx)))
    }

    fn visit_and(
        &mut self,
        _and: &PbesAndRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, PbesDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(PbesDescend::Descend(ctx)))
    }

    fn visit_or(
        &mut self,
        _or: &PbesOrRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, PbesDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(PbesDescend::Descend(ctx)))
    }

    fn visit_imp(
        &mut self,
        _imp: &PbesImpRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, PbesDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(PbesDescend::Descend(ctx)))
    }

    fn visit_forall(
        &mut self,
        _forall: &PbesForallRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, PbesDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(PbesDescend::Descend(ctx)))
    }

    fn visit_exists(
        &mut self,
        _exists: &PbesExistsRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, PbesDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(PbesDescend::Descend(ctx)))
    }

    /// Visits a data expression leaf. By default does not recurse into data subterms;
    /// use [`try_visit_data_expr_with`] inside this method to do so.
    fn visit_data_expression(
        &mut self,
        _expr: &DataExpressionRef<'_>,
        ctx: Self::Context,
    ) -> Result<ControlFlow<Self::Break, PbesDescend<Self::Context>>, Self::Error> {
        Ok(ControlFlow::Continue(PbesDescend::Descend(ctx)))
    }

    fn try_visit(
        &mut self,
        expr: &PbesExpressionRef<'_>,
        ctx: Self::Context,
    ) -> Result<Option<Self::Break>, Self::Error> {
        if is_pbes_propositional_variable_instantiation(&expr.copy()) {
            let inst = PbesPropositionalVariableInstantiationRef::from(expr.copy());
            match self.visit_propositional_variable_instantiation(&inst, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(_) => {} // leaf
            }
        } else if is_data_expression(&expr.copy()) {
            let data_expr: DataExpressionRef<'_> = expr.copy().into();
            match self.visit_data_expression(&data_expr, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(_) => {} // leaf
            }
        } else if is_pbes_and(&expr.copy()) {
            let and = PbesAndRef::from(expr.copy());
            let ctx = match self.visit_and(&and, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(PbesDescend::Prune) => return Ok(None),
                ControlFlow::Continue(PbesDescend::Descend(ctx)) => ctx,
            };
            if let Some(result) = self.try_visit(&and.lhs(), ctx)? {
                return Ok(Some(result));
            }
            if let Some(result) = self.try_visit(&and.rhs(), ctx)? {
                return Ok(Some(result));
            }
        } else if is_pbes_or(&expr.copy()) {
            let or = PbesOrRef::from(expr.copy());
            let ctx = match self.visit_or(&or, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(PbesDescend::Prune) => return Ok(None),
                ControlFlow::Continue(PbesDescend::Descend(ctx)) => ctx,
            };
            if let Some(result) = self.try_visit(&or.lhs(), ctx)? {
                return Ok(Some(result));
            }
            if let Some(result) = self.try_visit(&or.rhs(), ctx)? {
                return Ok(Some(result));
            }
        } else if is_pbes_imp(&expr.copy()) {
            let imp = PbesImpRef::from(expr.copy());
            let ctx = match self.visit_imp(&imp, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(PbesDescend::Prune) => return Ok(None),
                ControlFlow::Continue(PbesDescend::Descend(ctx)) => ctx,
            };
            if let Some(result) = self.try_visit(&imp.lhs(), ctx)? {
                return Ok(Some(result));
            }
            if let Some(result) = self.try_visit(&imp.rhs(), ctx)? {
                return Ok(Some(result));
            }
        } else if is_pbes_not(&expr.copy()) {
            let not = PbesNotRef::from(expr.copy());
            let ctx = match self.visit_not(&not, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(PbesDescend::Prune) => return Ok(None),
                ControlFlow::Continue(PbesDescend::Descend(ctx)) => ctx,
            };
            if let Some(result) = self.try_visit(&not.body(), ctx)? {
                return Ok(Some(result));
            }
        } else if is_pbes_forall(&expr.copy()) {
            let forall = PbesForallRef::from(expr.copy());
            let ctx = match self.visit_forall(&forall, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(PbesDescend::Prune) => return Ok(None),
                ControlFlow::Continue(PbesDescend::Descend(ctx)) => ctx,
            };
            if let Some(result) = self.try_visit(&forall.body(), ctx)? {
                return Ok(Some(result));
            }
        } else if is_pbes_exists(&expr.copy()) {
            let exists = PbesExistsRef::from(expr.copy());
            let ctx = match self.visit_exists(&exists, ctx)? {
                ControlFlow::Break(result) => return Ok(Some(result)),
                ControlFlow::Continue(PbesDescend::Prune) => return Ok(None),
                ControlFlow::Continue(PbesDescend::Descend(ctx)) => ctx,
            };
            if let Some(result) = self.try_visit(&exists.body(), ctx)? {
                return Ok(Some(result));
            }
        } else {
            unreachable!("Unknown PBES expression type");
        }

        Ok(None)
    }
}

impl<C, T, E, F> PbesExpressionContextVisitor for PbesClosureVisitor<C, T, E, F>
where
    C: Copy,
    F: FnMut(&PbesExpressionRef<'_>, C) -> Result<ControlFlow<T, PbesDescend<C>>, E>,
{
    type Context = C;
    type Break = T;
    type Error = E;

    fn visit_propositional_variable_instantiation(&mut self, inst: &PbesPropositionalVariableInstantiationRef<'_>, ctx: C) -> Result<ControlFlow<T, PbesDescend<C>>, E> {
        (self.f)(&PbesExpressionRef::from(inst.copy()), ctx)
    }

    fn visit_not(&mut self, not: &PbesNotRef<'_>, ctx: C) -> Result<ControlFlow<T, PbesDescend<C>>, E> {
        (self.f)(&PbesExpressionRef::from(not.copy()), ctx)
    }

    fn visit_and(&mut self, and: &PbesAndRef<'_>, ctx: C) -> Result<ControlFlow<T, PbesDescend<C>>, E> {
        (self.f)(&PbesExpressionRef::from(and.copy()), ctx)
    }

    fn visit_or(&mut self, or: &PbesOrRef<'_>, ctx: C) -> Result<ControlFlow<T, PbesDescend<C>>, E> {
        (self.f)(&PbesExpressionRef::from(or.copy()), ctx)
    }

    fn visit_imp(&mut self, imp: &PbesImpRef<'_>, ctx: C) -> Result<ControlFlow<T, PbesDescend<C>>, E> {
        (self.f)(&PbesExpressionRef::from(imp.copy()), ctx)
    }

    fn visit_forall(&mut self, forall: &PbesForallRef<'_>, ctx: C) -> Result<ControlFlow<T, PbesDescend<C>>, E> {
        (self.f)(&PbesExpressionRef::from(forall.copy()), ctx)
    }

    fn visit_exists(&mut self, exists: &PbesExistsRef<'_>, ctx: C) -> Result<ControlFlow<T, PbesDescend<C>>, E> {
        (self.f)(&PbesExpressionRef::from(exists.copy()), ctx)
    }

    fn visit_data_expression(&mut self, expr: &DataExpressionRef<'_>, ctx: C) -> Result<ControlFlow<T, PbesDescend<C>>, E> {
        (self.f)(&PbesExpressionRef::from(expr.copy()), ctx)
    }
}

/// Visits subexpressions of a PBES expression top-down, routing through [`PbesExpressionContextVisitor`].
pub fn try_visit_pbes_expr_with<C, T, E, F>(
    expr: &PbesExpressionRef<'_>,
    ctx: C,
    visitor: F,
) -> Result<Option<T>, E>
where
    C: Copy,
    F: FnMut(&PbesExpressionRef<'_>, C) -> Result<ControlFlow<T, PbesDescend<C>>, E>,
{
    PbesClosureVisitor::<C, T, E, F>::new(visitor).try_visit(expr, ctx)
}

/// Non-fallible convenience wrapper for [`try_visit_pbes_expr_with`].
pub fn visit_pbes_expr_with<T, C, F>(expr: &PbesExpressionRef<'_>, ctx: C, mut visitor: F) -> Option<T>
where
    C: Copy,
    F: FnMut(&PbesExpressionRef<'_>, C) -> ControlFlow<T, PbesDescend<C>>,
{
    use std::convert::Infallible;
    try_visit_pbes_expr_with(expr, ctx, |e, c| -> Result<_, Infallible> { Ok(visitor(e, c)) })
        .expect("infallible")
}
