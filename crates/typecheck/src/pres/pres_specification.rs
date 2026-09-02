//! [`PresSpecification`]: type checking for a whole `UntypedPres` — the data specification
//! (delegated to [`DataSpecification`]) plus its global variables, propositional-variable
//! equations, and `init`.
//!
//! **Scope.** This checks *sorts and names*: parameter/argument sort-checking (a PRES equation's
//! embedded data expressions are real-valued, unlike a PBES's boolean-valued ones — see
//! [`super::check`]), propositional-variable name/arity resolution, `inf`/`sup`/`sum`-binder
//! scoping. It does **not** check PRES well-formedness properties like monotonicity of
//! propositional variables — that is a separate semantic analysis, not type checking, out of
//! scope here (mirrors [`crate::PbesSpecification`]'s own scoping note).

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::ControlFlow;

use merc_syntax::IdDecl;
use merc_syntax::PresEquation;
use merc_syntax::PropVarInst;
use merc_syntax::SortExpression;
use merc_syntax::SortExpressionKind;
use merc_syntax::Span;
use merc_syntax::Traverse;
use merc_syntax::UntypedPres;

use crate::DataSpecification;
use crate::NumberEncoding;
use crate::ResolvedSortId;
use crate::TypingInfo;

use super::PresError;
use super::check;

/// A type-checked mCRL2 PRES: the data specification plus its `glob`, propositional-variable
/// equations, and `init`, all resolved and checked against it. See the module doc comment for
/// what's in and out of scope.
pub struct PresSpecification {
    /// The original specification, *minus* its data specification.
    spec: UntypedPres,
    data: DataSpecification,
    /// Every checked expression's `TypingInfo`, merged during the construction walk (see
    /// [`Self::typing_info`]).
    typing: TypingInfo,
}

impl PresSpecification {
    /// Type checks `spec`, using the default number encoding. See [`Self::from_untyped_with`].
    pub fn from_untyped(spec: UntypedPres) -> Result<Self, PresError> {
        Self::from_untyped_with(spec, NumberEncoding::default())
    }

    /// Type checks `spec`: its data specification first (exactly as
    /// [`DataSpecification::from_untyped_with`] does), then its global variables, its
    /// propositional-variable equations' parameters, and every equation's formula and `init`
    /// against them.
    pub fn from_untyped_with(mut spec: UntypedPres, encoding: NumberEncoding) -> Result<Self, PresError> {
        // A pure syntactic pass, before anything else needs `spec` — see
        // `resolution::variable_resolution`.
        crate::resolve_pres_variables(&mut spec);

        let data_spec = std::mem::take(&mut spec.data_specification);
        let mut data = DataSpecification::from_untyped_with(data_spec, encoding)?;

        let tables = DeclarationTables::build(&mut data, &spec)?;
        let typing = check::check_pres_specification(&mut data, &tables, &spec)?;

        Ok(PresSpecification { spec, data, typing })
    }

    /// The checked data specification.
    pub fn data_specification(&self) -> &DataSpecification {
        &self.data
    }

    /// Consumes `self`, returning the checked data specification.
    pub fn into_data_specification(self) -> DataSpecification {
        self.data
    }

    /// The `glob` declarations, in scope in every equation's formula and in `init`.
    pub fn global_variables(&self) -> &[IdDecl] {
        &self.spec.global_variables
    }

    /// The propositional-variable equations.
    pub fn equations(&self) -> &[PresEquation] {
        &self.spec.equations
    }

    /// The `init` propositional-variable instantiation.
    pub fn init(&self) -> &PropVarInst {
        &self.spec.init
    }

    /// Every checked expression's typing across the *whole* specification — every `eqn` (via
    /// [`DataSpecification::typing_info`]) plus every PRES expression (a `val(...)` expression, a
    /// `PropVarInst` argument, a constant multiplier, …), span-keyed so hover/go-to-definition can
    /// look up a sub-expression by source position anywhere in the document (see
    /// [`TypingInfo::at_offset`]) without caring which half of the grammar it came from.
    ///
    /// The PRES-expression half needs no separate memoization: it was already computed once,
    /// during the construction walk `from_untyped_with` runs anyway, and is just cloned out of the
    /// stored value here. The `eqn` half *is* memoized, one level down — see
    /// [`DataSpecification::typing_info`] — so a repeated call is cheap either way. Mirrors
    /// [`crate::PbesSpecification::typing_info`].
    pub fn typing_info(&mut self) -> TypingInfo {
        let mut info = self.data.typing_info();
        info.merge(self.typing.clone());
        info
    }
}

/// The resolved global-variable and per-equation-parameter tables, built once by [`Self::build`]
/// and used by [`super::check`]'s scoped walk to resolve every `PropVarInst` it reaches.
pub(super) struct DeclarationTables {
    /// Resolved sort of each `glob` declaration, parallel to `spec.global_variables`.
    pub(super) global_sorts: Vec<ResolvedSortId>,
    /// Resolved `(name, sort)` parameters of each equation, parallel to `spec.equations`.
    pub(super) equation_params: Vec<Vec<(String, ResolvedSortId)>>,
    /// `spec.equations[i].variable.span`, parallel to `equation_params` — mirrors
    /// `crate::pbes::pbes_specification::DeclarationTables::equation_decl_spans`.
    pub(super) equation_decl_spans: Vec<Span>,
    /// Propositional-variable name -> index into `spec.equations`/`equation_params`. A single
    /// slot per name, not a `Vec<usize>`: like PBES equations, PRES equations are declared once
    /// each — no overloading.
    pub(super) equations_by_name: HashMap<String, usize>,
}

impl DeclarationTables {
    fn build(data: &mut DataSpecification, spec: &UntypedPres) -> Result<Self, PresError> {
        let mut global_sorts = Vec::with_capacity(spec.global_variables.len());
        let mut seen_globals = HashSet::new();
        for decl in &spec.global_variables {
            if !seen_globals.insert(decl.identifier.as_str()) {
                return Err(PresError::DuplicateGlobalVariable {
                    name: decl.identifier.clone(),
                    span: decl.span.clone(),
                });
            }
            global_sorts.push(resolve_declared_sort(data, &decl.sort)?);
        }

        let mut equation_params = Vec::with_capacity(spec.equations.len());
        let mut equation_decl_spans = Vec::with_capacity(spec.equations.len());
        let mut equations_by_name: HashMap<String, usize> = HashMap::new();
        for (index, eqn) in spec.equations.iter().enumerate() {
            let mut params = Vec::with_capacity(eqn.variable.parameters.len());
            let mut seen = HashSet::new();
            for param in &eqn.variable.parameters {
                if !seen.insert(param.identifier.as_str()) {
                    return Err(PresError::DuplicateEquationParameter {
                        equation: eqn.variable.identifier.clone(),
                        name: param.identifier.clone(),
                        span: param.span.clone(),
                    });
                }
                let sort = resolve_declared_sort(data, &param.sort)?;
                params.push((param.identifier.clone(), sort));
            }

            if equations_by_name
                .insert(eqn.variable.identifier.clone(), index)
                .is_some()
            {
                return Err(PresError::DuplicatePropositionalVariable {
                    name: eqn.variable.identifier.clone(),
                    span: eqn.variable.span.clone(),
                });
            }
            equation_params.push(params);
            equation_decl_spans.push(eqn.variable.span.clone());
        }

        Ok(DeclarationTables {
            global_sorts,
            equation_params,
            equation_decl_spans,
            equations_by_name,
        })
    }
}

/// Resolves a sort expression occurring in a `glob`/PRES-equation-parameter declaration: rejects
/// an anonymous `struct` (never legal here), then defers to
/// [`DataSpecification::resolve_declared_sort`] for the rest. Mirrors
/// `crate::pbes::pbes_specification::resolve_declared_sort`.
pub(super) fn resolve_declared_sort(
    data: &mut DataSpecification,
    sort: &SortExpression,
) -> Result<ResolvedSortId, PresError> {
    if let Some(span) = find_anonymous_struct(sort) {
        return Err(PresError::AnonymousStructInDeclaration { span });
    }
    Ok(data.resolve_declared_sort(sort)?)
}

/// The span of the first anonymous `struct` anywhere within `sort`, if any.
fn find_anonymous_struct(sort: &SortExpression) -> Option<Span> {
    sort.visit(|expr| match &expr.node {
        SortExpressionKind::Struct { .. } => ControlFlow::Break(expr.span.clone()),
        _ => ControlFlow::Continue(()),
    })
}
