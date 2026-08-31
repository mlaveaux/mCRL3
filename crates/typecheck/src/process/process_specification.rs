//! [`ProcessSpecification`]: type checking for a whole `UntypedProcessSpecification` — the data
//! specification (delegated to [`DataSpecification`]) plus its actions, processes, and `init`.

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::ControlFlow;

use merc_syntax::ActDecl;
use merc_syntax::IdDecl;
use merc_syntax::ProcDecl;
use merc_syntax::ProcessExpr;
use merc_syntax::SortExpression;
use merc_syntax::SortExpressionKind;
use merc_syntax::Span;
use merc_syntax::Traverse;
use merc_syntax::UntypedProcessSpecification;

use crate::DataSpecification;
use crate::NumberEncoding;
use crate::ResolvedSortId;

use super::ProcessError;
use super::check;
use super::reparse;

/// A type-checked mCRL2 process specification: the data specification plus its `act`, `proc`,
/// `glob`, and `init` declarations, all resolved and checked against it.
///
/// See the crate README for what's scoped in and out of this — most notably, communication
/// sort-compatibility is not checked yet.
pub struct ProcessSpecification {
    /// The original specification, *minus* its data specification.
    spec: UntypedProcessSpecification,
    data: DataSpecification,
}

impl ProcessSpecification {
    /// Type checks `spec`, using the default number encoding. See [`Self::from_untyped_with`].
    pub fn from_untyped(spec: UntypedProcessSpecification) -> Result<Self, ProcessError> {
        Self::from_untyped_with(spec, NumberEncoding::default())
    }

    /// Type checks `spec`: its data specification first (exactly as
    /// [`DataSpecification::from_untyped_with`] does), then its action declarations' argument
    /// sorts, its global variables, and every `proc` body and `init` against them.
    pub fn from_untyped_with(mut spec: UntypedProcessSpecification, encoding: NumberEncoding) -> Result<Self, ProcessError> {
        // Semantic-aware reparsing first.
        reparse::reparse_process_specification(&mut spec);

        let data_spec = std::mem::take(&mut spec.data_specification);
        let mut data = DataSpecification::from_untyped_with(data_spec, encoding)?;

        let tables = DeclarationTables::build(&mut data, &spec)?;
        check::check_process_specification(&mut data, &tables, &spec)?;

        Ok(ProcessSpecification { spec, data })
    }

    /// The checked data specification.
    pub fn data_specification(&self) -> &DataSpecification {
        &self.data
    }

    /// Consumes `self`, returning the checked data specification.
    pub fn into_data_specification(self) -> DataSpecification {
        self.data
    }

    /// The `glob` declarations, in scope in every process body and in `init`.
    pub fn global_variables(&self) -> &[IdDecl] {
        &self.spec.global_variables
    }

    /// The `act` declarations.
    pub fn action_declarations(&self) -> &[ActDecl] {
        &self.spec.action_declarations
    }

    /// The `proc` declarations.
    pub fn process_declarations(&self) -> &[ProcDecl] {
        &self.spec.process_declarations
    }

    /// The `init` process, if the specification declares one. Absent is not itself an error: a
    /// library-only specification with no `init` is legitimate.
    pub fn init(&self) -> Option<&ProcessExpr> {
        self.spec.init.as_ref()
    }
}

/// The resolved `act`/`proc` declaration tables, built once by [`Self::build`] and used by
/// [`super::check`]'s scoped walk to resolve every `Action`/`Id` process-expression node it
/// reaches, and by [`super::process_specification`] to check each declaration's own body against
/// its own parameters.
pub(super) struct DeclarationTables {
    /// Resolved sort of each `glob` declaration, parallel to `spec.global_variables`.
    pub(super) global_sorts: Vec<ResolvedSortId>,
    /// Resolved `(name, sort)` parameters of each process declaration, parallel to
    /// `spec.process_declarations`.
    pub(super) process_params: Vec<Vec<(String, ResolvedSortId)>>,
    /// name -> indices into `spec.process_declarations`/`process_params` declaring it.
    pub(super) processes_by_name: HashMap<String, Vec<usize>>,
    /// Resolved argument-sort domain of each action declaration, parallel to
    /// `spec.action_declarations`.
    pub(super) action_domains: Vec<Vec<ResolvedSortId>>,
    /// name -> indices into `spec.action_declarations`/`action_domains` declaring it.
    pub(super) actions_by_name: HashMap<String, Vec<usize>>,
}

impl DeclarationTables {
    fn build(data: &mut DataSpecification, spec: &UntypedProcessSpecification) -> Result<Self, ProcessError> {
        let mut action_domains = Vec::with_capacity(spec.action_declarations.len());
        let mut actions_by_name: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, decl) in spec.action_declarations.iter().enumerate() {
            let domain = decl
                .args
                .iter()
                .map(|sort| resolve_declared_sort(data, sort))
                .collect::<Result<Vec<_>, _>>()?;
            actions_by_name.entry(decl.identifier.clone()).or_default().push(index);
            action_domains.push(domain);
        }

        let mut process_params = Vec::with_capacity(spec.process_declarations.len());
        let mut processes_by_name: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, decl) in spec.process_declarations.iter().enumerate() {
            let mut params = Vec::with_capacity(decl.params.len());
            let mut seen = HashSet::new();
            for param in &decl.params {
                if !seen.insert(param.identifier.as_str()) {
                    return Err(ProcessError::DuplicateProcessParameter {
                        process: decl.identifier.clone(),
                        name: param.identifier.clone(),
                        span: param.span.clone(),
                    });
                }
                let sort = resolve_declared_sort(data, &param.sort)?;
                params.push((param.identifier.clone(), sort));
            }
            processes_by_name.entry(decl.identifier.clone()).or_default().push(index);
            process_params.push(params);
        }

        // A name declared as both an action and a process would make `Action(name, args)`
        // (used for both an action instance and a positional process instantiation, see the crate
        // README) permanently ambiguous between the two tables. Iterated in source order, not via
        // `actions_by_name`'s (unordered) keys, so which conflicting name gets reported is
        // deterministic.
        for decl in &spec.action_declarations {
            if processes_by_name.contains_key(&decl.identifier) {
                return Err(ProcessError::ActionAndProcessConflict { name: decl.identifier.clone(), span: decl.span.clone() });
            }
        }

        let mut global_sorts = Vec::with_capacity(spec.global_variables.len());
        let mut seen_globals = HashSet::new();
        for decl in &spec.global_variables {
            if !seen_globals.insert(decl.identifier.as_str()) {
                return Err(ProcessError::DuplicateGlobalVariable { name: decl.identifier.clone(), span: decl.span.clone() });
            }
            global_sorts.push(resolve_declared_sort(data, &decl.sort)?);
        }

        Ok(DeclarationTables { global_sorts, process_params, processes_by_name, action_domains, actions_by_name })
    }
}

/// Resolves a sort expression occurring in an `act`/`proc`/`glob` declaration: rejects an
/// anonymous `struct` (never legal here), then defers to
/// [`DataSpecification::resolve_declared_sort`] for the rest.
pub(super) fn resolve_declared_sort(data: &mut DataSpecification, sort: &SortExpression) -> Result<ResolvedSortId, ProcessError> {
    if let Some(span) = find_anonymous_struct(sort) {
        return Err(ProcessError::AnonymousStructInDeclaration { span });
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
