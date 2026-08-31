//! Public typing and name-resolution information for a checked data specification, keyed by
//! [`Span`] rather than [`crate::ExprId`]: `ExprId` is assigned over the *lowered* expression
//! tree and can contain nodes with no counterpart in the original `DataExpr` a caller parsed, so
//! it cannot be reconstructed externally.
//!
//! # Caveats
//!
//! - A node synthesized during lowering has no span of its own and inherits the span of the whole
//!   surface expression it was lowered from — so more than one [`TypedNode`] can share a span; see
//!   [`TypingInfo::at_offset`]'s tie-break rule.
//! - [`TypedNode::sort`] is *reconstructed* from the resolved sort the type checker interned, not
//!   read from a declaration. It always carries [`Span::default`]: a resolved sort has no reliable
//!   source location of its own — name resolution and alias normalization both discard or
//!   relocate a [`SortExpression`]'s original span. Do not read source positions out of it.

use std::collections::HashMap;

use merc_syntax::ConstructorId;
use merc_syntax::MapId;
use merc_syntax::Span;
use merc_syntax::SortExpression;
use merc_syntax::SortExpressionKind;
use merc_syntax::UntypedDataSpecification;

use crate::DataSpecification;
use crate::EquationTyping;
use crate::ExprId;
use crate::NameTarget;
use crate::ResolvedSort;
use crate::ResolvedSortId;
use crate::TypeCheckContext;

/// The typing of a document's data specification (or of one expression checked via
/// [`DataSpecification::typecheck_expression_with_typing`]): one [`TypedNode`] per checked
/// expression node, in generation order.
///
/// Built by [`DataSpecification::typing_info`], [`DataSpecification::equation_typing_info`], and
/// [`DataSpecification::typecheck_expression_with_typing`].
#[derive(Debug, Default, Clone)]
pub struct TypingInfo {
    nodes: Vec<TypedNode>,
}

/// The typing of a single expression node. See the [module docs](self) for the caveats on `span`
/// and `sort`.
#[derive(Debug, Clone)]
pub struct TypedNode {
    /// The node's location in the original source.
    pub span: Span,
    /// The node's inferred sort, reconstructed as a [`SortExpression`] so it can be printed via
    /// its existing [`std::fmt::Display`] impl.
    pub sort: SortExpression,
    /// What this node's identifier resolved to. `None` for every node that isn't an `Id`.
    pub name: Option<ResolvedName>,
}

/// What an identifier ([`TypedNode::name`]) resolved to.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ResolvedName {
    /// An equation variable of the enclosing `var` block.
    Variable { name: String },
    /// A user-declared constructor.
    Constructor {
        name: String,
        id: ConstructorId,
        /// The declaration's span, when it has a real one — absent for some struct-desugared
        /// constructors (see the crate README).
        declaration: Option<Span>,
    },
    /// A user-declared mapping.
    Mapping {
        name: String,
        id: MapId,
        /// See [`ResolvedName::Constructor::declaration`].
        declaration: Option<Span>,
    },
    /// A declared Appendix-B symbol with no user declaration to point at (`succ`, `@c0`, …).
    SystemDefined { name: String },
    /// A polymorphic built-in (`==`, `!=`, `<`, `<=`, `>`, `>=`, `if`, `in`, `#`, `|>`, …), whose
    /// concrete meaning follows from the inferred argument sorts rather than one declaration.
    Builtin { name: String },
}

impl TypingInfo {
    /// All nodes, in generation order.
    pub fn nodes(&self) -> &[TypedNode] {
        &self.nodes
    }

    /// Consumes `self`, returning the nodes.
    pub fn into_nodes(self) -> Vec<TypedNode> {
        self.nodes
    }

    /// The most specific node whose span contains `offset` — the usual hover/go-to-definition
    /// query.
    ///
    /// When several nodes tie for the smallest span (a synthesized node sharing its surface
    /// expression's span with that expression itself — `x + y`'s synthesized `Id("+")` and its
    /// `Application` both span the whole expression), the *last* one in generation order wins,
    /// which is the more specific of the two in every case this arises: the operator name rather
    /// than the application's result.
    pub fn at_offset(&self, offset: usize) -> Option<&TypedNode> {
        let mut best: Option<&TypedNode> = None;
        for node in &self.nodes {
            if node.span.start > offset || offset >= node.span.end {
                continue;
            }
            let width = node.span.end - node.span.start;
            let is_narrower_or_tied = match best {
                Some(current) => width <= current.span.end - current.span.start,
                None => true,
            };
            if is_narrower_or_tied {
                best = Some(node);
            }
        }
        best
    }

    pub(crate) fn merge(&mut self, mut other: TypingInfo) {
        self.nodes.append(&mut other.nodes);
    }
}

/// Builds the [`TypingInfo`] for `typing`, the already-computed Phase-3 result of one equation or
/// standalone expression. `typing.spans`/`typing.identifier_names` must be filled — true for
/// every `EquationTyping` this crate ever hands to a public caller, since both are only ever
/// omitted for `EquationRole::System`, which never reaches here.
pub(crate) fn build(spec: &DataSpecification, typing: &EquationTyping) -> TypingInfo {
    debug_assert_eq!(
        typing.spans.len(),
        typing.sorts.len(),
        "typing_info::build requires an EquationTyping built for EquationRole::User"
    );

    let index = DeclarationIndex::build(spec);
    let ctx = spec.context();
    let user_spec = spec.data_specification();
    let system_spec = spec.system_defined_specification();

    let nodes = typing
        .spans
        .iter()
        .zip(&typing.sorts)
        .enumerate()
        .map(|(i, (span, &sort))| {
            let id = ExprId::new(i);
            let name = typing.names.get(&id).map(|&target| {
                let name = typing
                    .identifier_names
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| unreachable!("every named node has a recorded identifier"));
                resolved_name(&index, target, name)
            });
            TypedNode {
                span: span.clone(),
                sort: sort_expression(ctx, user_spec, system_spec, sort),
                name,
            }
        })
        .collect();

    TypingInfo { nodes }
}

fn resolved_name(index: &DeclarationIndex<'_>, target: NameTarget, name: String) -> ResolvedName {
    match target {
        NameTarget::Variable => ResolvedName::Variable { name },
        NameTarget::Builtin => ResolvedName::Builtin { name },
        NameTarget::Op { sort } => {
            let constructor = index.constructors.get(&(name.as_str(), sort)).cloned();
            let mapping = index.mappings.get(&(name.as_str(), sort)).cloned();

            if let Some((id, declaration)) = constructor {
                ResolvedName::Constructor { name, id, declaration }
            } else if let Some((id, declaration)) = mapping {
                ResolvedName::Mapping { name, id, declaration }
            } else {
                // Declared only on the system-defined specification: no ConstructorId/MapId of
                // the *user* spec names it, and the system spec's own ids aren't meaningful to
                // an outside caller.
                ResolvedName::SystemDefined { name }
            }
        }
    }
}

/// A `(name, resolved sort) -> declaration` reverse lookup, built once per [`build`] call from
/// the user specification's own declaration lists. `(name, ResolvedSortId)` uniquely identifies a
/// symbol (`compute_signature` rejects a `cons`/`map` conflict exactly when both the name and the
/// resolved sort coincide); the residual case of two *literally duplicated* declarations (legal:
/// `map f: Nat; f: Nat;`) resolves to the first, via `HashMap::entry(..).or_insert(..)` below.
struct DeclarationIndex<'a> {
    constructors: HashMap<(&'a str, ResolvedSortId), (ConstructorId, Option<Span>)>,
    mappings: HashMap<(&'a str, ResolvedSortId), (MapId, Option<Span>)>,
}

impl<'a> DeclarationIndex<'a> {
    fn build(spec: &'a DataSpecification) -> Self {
        let mut constructors = HashMap::new();
        for decl in &spec.data_specification().constructor_declarations {
            let Some(id) = decl.id else {
                // Every declaration is assigned an id during `from_untyped`; a `DataSpecification`
                // that exists at all has already been through it.
                continue;
            };
            let sort = spec.sort_of_constructor(id);
            let declaration = declared_span(&decl.span);
            constructors.entry((decl.identifier.as_str(), sort)).or_insert((id, declaration));
        }

        let mut mappings = HashMap::new();
        for decl in &spec.data_specification().map_declarations {
            let Some(id) = decl.id else {
                continue;
            };
            let sort = spec.sort_of_map(id);
            let declaration = declared_span(&decl.span);
            mappings.entry((decl.identifier.as_str(), sort)).or_insert((id, declaration));
        }

        DeclarationIndex { constructors, mappings }
    }
}

/// `Span::default()` marks a declaration synthesized without a real source location (some
/// struct-desugared constructors, projections and recognisers) rather than a real position at the
/// start of the file; normalize it to `None` so a consumer doesn't render a misleading
/// declaration site.
fn declared_span(span: &Span) -> Option<Span> {
    (*span != Span::default()).then(|| span.clone())
}

/// Rebuilds `id` as a [`SortExpression`], so it can be displayed via its existing
/// [`std::fmt::Display`] impl and so a `Def` sort carries a `DefId` a consumer can use for sort
/// go-to-definition. Mirrors [`crate::lower_sort`]'s structural recursion (same crate, targeting
/// the binary aterm format instead of the AST's own sort type).
///
/// Every produced node gets [`Span::default`] — see the [module docs](self).
fn sort_expression(
    ctx: &TypeCheckContext,
    spec: &UntypedDataSpecification,
    system: &UntypedDataSpecification,
    id: ResolvedSortId,
) -> SortExpression {
    match ctx.sorts.get(id) {
        ResolvedSort::Unit => unreachable!(
            "Unit is only used for the sort of an action, never a data-expression sort, and \
             typing_info only ever renders the sort of a data expression"
        ),
        ResolvedSort::Primitive(sort) => SortExpressionKind::Simple(*sort).into(),
        ResolvedSort::Generic { op, subsort } => {
            SortExpressionKind::Complex(*op, Box::new(sort_expression(ctx, spec, system, *subsort))).into()
        }
        ResolvedSort::Function { domain, range } => SortExpressionKind::FlattenedFunction {
            domain: domain.iter().map(|&sort| sort_expression(ctx, spec, system, sort)).collect(),
            range: Box::new(sort_expression(ctx, spec, system, *range)),
        }
        .into(),
        ResolvedSort::Def(def) => {
            let name = ctx
                .sort_name(spec, system, *def)
                .map(str::to_string)
                .unwrap_or_else(|| format!("@sort_{}", def.value()));
            SortExpressionKind::Resolved(name, *def).into()
        }
    }
}
