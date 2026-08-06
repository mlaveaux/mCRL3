/// Authors: Menno Bartels and Maurice Laveaux
///
/// Constructs the "symmetry detection graph" (SDG) of a PBES, and uses it
/// (via the GAP automorphism-group computation, see [`crate::gap`]) to derive
/// permutation symmetries of the PBES's parameters.
///
/// This implements the graph construction from "A Graph Construction for
/// Symmetry Detection in Parametrised Boolean Equation Systems" (Bartels,
/// Laveaux, Neele, Willemse). The full design rationale, the definitional
/// gaps found while reproducing the paper's Definitions 2-3, and the reasons
/// behind several deliberate deviations are written up in
/// `docs/graph-symmetry-detection-plan.md` at the repository root; the
/// highlights are repeated here as doc comments on the relevant types.
///
/// Unlike the (older) clique-based [`crate::symmetry`] algorithm, this module
/// works directly on the original `Pbes`: no conversion to standard recursive
/// form. This keeps the general recursive predicate-formula structure that
/// Definition 2's `sub`/`sub#` are actually defined over (SRF would flatten
/// it into disjunctive/conjunctive summands, which does not match the
/// paper's grammar). The one precondition inherited from the paper's
/// preliminaries ("every equation has exactly the same vector of data
/// parameters") is established by converting to SRF and calling
/// `unify_parameters` before building the graph: see [`build_sdg`].
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::ControlFlow;

use itertools::Itertools;
use log::debug;
use log::trace;
use petgraph::graph::NodeIndex;
use petgraph::graph::UnGraph;

use mcrl2::ATerm;
use mcrl2::ATermRef;
use mcrl2::DataAbstractionRef;
use mcrl2::DataApplicationRef;
use mcrl2::DataExpressionRef;
use mcrl2::DataFunctionSymbolRef;
use mcrl2::DataMachineNumberRef;
use mcrl2::DataVariable;
use mcrl2::DataVariableRef;
use mcrl2::Pbes;
use mcrl2::PbesDescend;
use mcrl2::PbesExistsRef;
use mcrl2::PbesForallRef;
use mcrl2::PbesImpRef;
use mcrl2::PbesNotRef;
use mcrl2::PbesPropositionalVariableInstantiationRef;
use mcrl2::SortExpression;
use mcrl2::flatten_associative;
use mcrl2::visit_pbes_expr_with;
use mcrl2::is_abstraction;
use mcrl2::is_application;
use mcrl2::is_exists_binder;
use mcrl2::is_forall_binder;
use mcrl2::is_function_symbol;
use mcrl2::is_lambda_binder;
use mcrl2::is_machine_number;
use mcrl2::is_pbes_and;
use mcrl2::is_pbes_exists;
use mcrl2::is_pbes_forall;
use mcrl2::is_pbes_imp;
use mcrl2::is_pbes_not;
use mcrl2::is_pbes_or;
use mcrl2::is_pbes_propositional_variable_instantiation;
use mcrl2::is_untyped_identifier;
use mcrl2::is_variable;
use mcrl2::is_where_clause;
use merc_utilities::MercError;

/// Binary function symbols treated as commutative, so their argument edges
/// are coloured uniformly ([`EdgeColour::Uncoloured`]) and argument order
/// does not constrain automorphisms.
///
/// mCRL2 exposes no associativity/commutativity metadata on data function
/// symbols (verified: there is no such annotation anywhere in its data
/// library, nor anything reachable through the `mcrl2-sys` FFI surface), so
/// this is a small, deliberately conservative, hand-maintained table.
/// Omitting a genuinely commutative symbol only loses symmetries (safe);
/// listing a non-commutative one would report symmetries that do not hold
/// (unsound). Extend carefully.
const COMMUTATIVE_FUNCTION_SYMBOLS: &[&str] = &["&&", "||", "==", "!=", "<=>", "+", "*", "max", "min"];

/// Subset of [`COMMUTATIVE_FUNCTION_SYMBOLS`] that are also associative.
/// Applications of these are flattened into a single n-ary vertex by
/// [`SdgBuilder::visit_children`], so `a && b && c` becomes one vertex with
/// three uncoloured child edges rather than nested binary nodes.
const FLAT_FUNCTION_SYMBOLS: &[&str] = &["&&", "||", "+", "*", "max", "min"];

/// Returns true iff `name` is a known commutative binary function symbol.
///
/// Only meaningful for arity-2 applications; the caller is responsible for
/// checking arity (an application of a "commutative" name with any other
/// arity is treated as non-commutative, since [`COMMUTATIVE_FUNCTION_SYMBOLS`]
/// only documents the binary-operator meaning of these names).
fn is_commutative(name: &str, arity: usize) -> bool {
    arity == 2 && COMMUTATIVE_FUNCTION_SYMBOLS.contains(&name)
}

/// Returns true iff `name` is a known associative commutative binary operator
/// whose chains should be flattened into a single n-ary SDG vertex.
fn is_flat_operator(name: &str, arity: usize) -> bool {
    arity == 2 && FLAT_FUNCTION_SYMBOLS.contains(&name)
}

/// Recursively collects the non-`is_op` leaves of a nested binary PBES connective
/// chain, flattening any depth. Analogous to [`flatten_associative`] but for
/// PBES-level connectives, which are not `is_application` nodes.
fn collect_pbes_flat<F>(term: ATermRef<'_>, is_op: F, out: &mut Vec<ATerm>)
where
    F: Fn(&ATermRef<'_>) -> bool + Copy,
{
    if is_op(&term) {
        collect_pbes_flat(term.arg(0), is_op, out);
        collect_pbes_flat(term.arg(1), is_op, out);
    } else {
        out.push(term.protect());
    }
}

/// A vertex of the symmetry detection graph.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum SdgVertex {
    /// The k'th (0-based) data parameter `d_k`. Allocated first, so
    /// `NodeIndex(k) == Parameter(k)`.
    Parameter(usize),

    /// A subformula or subterm, identified by its (maximally shared) term.
    /// Since mCRL2 terms are hash-consed, using the term itself as the
    /// deduplication key is exactly Definition 2's `sub(E)` being a *set*:
    /// two occurrences of the syntactically identical subterm (even across
    /// different equations) collapse to a single vertex, with [`Sdg`]'s
    /// per-vertex `C_eq` accumulating every equation that reaches it.
    Term(ATerm),

    /// The synthetic update position `X_{i,k}`: not a PBES term, and never
    /// deduplicated (one fresh vertex per `(equation, pvi-index,
    /// parameter-index)` triple, by construction).
    Update {
        equation: usize,
        pvi: usize,
        parameter: usize,
    },
}

/// `C(v)`, the "structural" colour of a vertex, excluding the orthogonal
/// `C_eq` component (see [`Sdg::equations`]).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum VertexColour {
    /// `C(x) = par` for a PBES parameter vertex.
    Parameter,

    /// A quantifier-bound variable, coloured by its sort but, like
    /// [`VertexColour::Quantifier`], deliberately not by name (matching the
    /// paper's note that the bound variable's name is not part of a
    /// quantifier's colour) -- otherwise an automorphism could map a bound
    /// variable onto an unrelated PBES parameter of the same sort.
    BoundVariable(SortExpression),

    /// `C(f(t1,...,tk)) = f`, identified by name. Also used for nullary
    /// function symbols/constants (`sub#(f()) = {}`, so these are leaves).
    Function(String),

    /// A machine number constant, coloured by its value so that an
    /// automorphism cannot equate two different constants.
    MachineNumber(u64),

    /// `C(phi1 (+) phi2) = (+)` for `(+) in {and, or, not, imp}` (the paper's
    /// grammar only has `{and, or}`; `not`/`imp` are additional mCRL2
    /// connectives given their own distinct colours the same way).
    Connective(Connective),

    /// `C(Qe:D.phi) = (Q,D)`. Generalized from a single sort to a vector of
    /// sorts, since mCRL2 quantifiers may bind more than one variable at
    /// once (`forall e1:D1, e2:D2 . phi`); this specializes to the paper's
    /// exact `(Q,D)` when exactly one variable is bound.
    Quantifier(Quantifier, Vec<SortExpression>),

    /// `C(X(t1,...,tn)) = pvi`.
    Pvi,

    /// `C(X_{i,k}) = update`.
    Update,
}

impl std::fmt::Display for VertexColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VertexColour::Parameter          => f.write_str("par"),
            VertexColour::BoundVariable(s)   => write!(f, "bvar:{s}"),
            VertexColour::Function(name)     => f.write_str(name),
            VertexColour::MachineNumber(n)   => write!(f, "{n}"),
            VertexColour::Connective(c)      => write!(f, "{c}"),
            VertexColour::Quantifier(q, ss)  => write!(f, "{q}:{}", ss.iter().format(",")),
            VertexColour::Pvi                => f.write_str("pvi"),
            VertexColour::Update             => f.write_str("update"),
        }
    }
}

impl VertexColour {
    fn dot_fill_colour(&self) -> &'static str {
        match self {
            VertexColour::Parameter        => "#aec6cf",
            VertexColour::BoundVariable(_) => "#d5e8d4",
            VertexColour::Function(_)      => "#fff2cc",
            VertexColour::MachineNumber(_) => "#ffe6cc",
            VertexColour::Connective(_)    => "#f8cecc",
            VertexColour::Quantifier(..)   => "#e1d5e7",
            VertexColour::Pvi              => "#dae8fc",
            VertexColour::Update           => "#f5f5f5",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Connective {
    And,
    Or,
    Not,
    Imp,
}

impl std::fmt::Display for Connective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Connective::And => "&&",
            Connective::Or  => "||",
            Connective::Not => "!",
            Connective::Imp => "=>",
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Quantifier {
    Forall,
    Exists,
    /// Data-level lambda abstraction (`lambda x:D. body`).
    Lambda,
    /// Data-level set/bag comprehension or untyped set/bag comprehension binder.
    Comprehension,
}

impl std::fmt::Display for Quantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Quantifier::Forall        => "forall",
            Quantifier::Exists        => "exists",
            Quantifier::Lambda        => "lambda",
            Quantifier::Comprehension => "comp",
        })
    }
}

/// `C(e)`, the colour of an edge, passed to GAP as a native edge colour (no
/// port-vertex subdivision gadget is needed).
///
/// Both set-valued variants implement the same idea: when an edge's
/// "natural" single label would coincide with another edge already present
/// between the same pair of vertices, the labels are combined into one set
/// instead of creating a duplicate `(source, target, colour)` triple (GAP's
/// `AutomorphismGroup` with edge colours disallows two edges sharing source,
/// range *and* colour). In the common, non-colliding case this is just a
/// singleton set, so it costs nothing and changes nothing versus the paper's
/// literal per-position/per-role reading. See [`SdgBuilder::add_or_merge_edge`].
impl std::fmt::Display for EdgeColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeColour::Uncoloured => Ok(()),
            EdgeColour::Argument(positions) => write!(f, "{}", positions.iter().format(",")),
            EdgeColour::Update(roles)       => write!(f, "{}", roles.iter().format(",")),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum EdgeColour {
    /// The set of positions `{i | t_i = psi}` of a non-commutative
    /// function's argument `psi` -- a singleton `{i}` unless `psi` repeats
    /// across positions (e.g. `f(x,y,x)` gets one edge to `x` coloured
    /// `{1,3}`, not two parallel edges).
    Argument(BTreeSet<usize>),

    /// Any argument of a commutative function; any other `C = 0` edge (`(+)`
    /// operands, quantifier body, abstraction body, ...).
    Uncoloured,

    /// The set of [`UpdateRole`]s that coincide on the same target vertex of
    /// an update vertex `X_{i,k}` -- a singleton unless the PVI's k'th
    /// argument is literally the current value of parameter `d_k` (a
    /// "copy", which is extremely common in practice), in which case the
    /// `Data` and `Par` edges land on the same vertex and are combined.
    Update(BTreeSet<UpdateRole>),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum UpdateRole {
    Pvi,
    Data,
    Par,
}

impl std::fmt::Display for UpdateRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            UpdateRole::Pvi  => "pvi",
            UpdateRole::Data => "data",
            UpdateRole::Par  => "par",
        })
    }
}

/// The symmetry detection graph (SDG) of a PBES, as constructed by
/// [`build_sdg`].
pub(crate) struct Sdg {
    /// Undirected, matching the paper's "nondirected colored graph". Exactly
    /// one edge per `(phi, psi)` vertex pair: [`EdgeColour`]'s combined
    /// labels resolve the only two situations that could otherwise force a
    /// parallel edge, so `graph` is always simple (no parallel edges, no
    /// self-loops).
    graph: UnGraph<SdgVertex, EdgeColour>,

    /// `C(v)`, indexed by `NodeIndex::index()`.
    colours: Vec<VertexColour>,

    /// `C_eq(v)`: the set of (0-based) equation indices whose right-hand
    /// side reaches this vertex, indexed by `NodeIndex::index()`.
    equations: Vec<BTreeSet<usize>>,

    /// The unified parameter vector; `parameters[k]` is the vertex
    /// `NodeIndex(k)`.
    parameters: Vec<DataVariable>,

    /// Names of the bound predicate variables, in equation order, for
    /// diagnostics only.
    equation_names: Vec<String>,
}

impl Sdg {
    /// Returns the number of parameters (the size of the permutation domain
    /// that symmetries are ultimately expressed over).
    pub(crate) fn num_parameters(&self) -> usize {
        self.parameters.len()
    }

    /// Returns the number of vertices in the graph.
    pub(crate) fn num_vertices(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of edges in the graph.
    pub(crate) fn num_edges(&self) -> usize {
        self.graph.edge_count()
    }
}

/// Returns the shared parameter vector after unification. Panics if equations
/// disagree, which cannot happen because [`build_sdg`] calls
/// `SrfPbes::unify_parameters` before reaching this point.
fn unified_parameters(equations: &mcrl2::PbesEquations) -> Result<Vec<DataVariable>, MercError> {
    let Some(first) = equations.first() else {
        return Ok(Vec::new());
    };

    let parameters: Vec<DataVariable> = first.variable().parameters().iter().collect();
    for equation in equations.iter().skip(1) {
        let other: Vec<DataVariable> = equation.variable().parameters().iter().collect();
        if other != parameters {
            return Err(format!(
                "Equation for '{}' does not declare the same parameter vector as equation for '{}'; \
                 the symmetry detection graph currently requires every equation to share an \
                 identical (name, sort)-ordered parameter vector.\n  {}: [{}]\n  {}: [{}]",
                equation.variable().name(),
                first.variable().name(),
                first.variable().name(),
                parameters.iter().format(", "),
                equation.variable().name(),
                other.iter().format(", "),
            )
            .into());
        }
    }

    Ok(parameters)
}

/// Builds the symmetry detection graph of `pbes`.
///
/// Every equation must already share the same parameter vector; call
/// [`graph_symmetries`] (which calls [`Pbes::unify_parameters`] first) when
/// that precondition is not yet established.
pub(crate) fn build_sdg(pbes: &Pbes) -> Result<Sdg, MercError> {
    let equations = pbes.equations();
    let parameters = unified_parameters(&equations)?;;

    let mut builder = SdgBuilder::new();

    // Allocate the parameter vertices first, so `NodeIndex(k) == parameter
    // k`. This is what makes "restrict an automorphism to the parameter
    // vertices" trivial: GAP point `k+1` <-> parameter `k`.
    for parameter in &parameters {
        let term: ATerm = parameter.clone().into();
        let index = builder.add_vertex(SdgVertex::Term(term.clone()), VertexColour::Parameter);
        builder.term_map.insert(term, index);
    }
    debug_assert_eq!(builder.graph.node_count(), parameters.len());

    let n = parameters.len();
    debug!("SDG build: {} parameter(s): [{}]", n, parameters.iter().format(", "));

    for (e, equation) in equations.iter().enumerate() {
        let root = equation.formula();
        let root_ref: ATermRef<'_> = root.copy().into();
        builder.visit(root_ref, e);

        // Deduplicate PVIs by ATerm identity: `Y(n) && Y(n)` yields two occurrences
        // from the traversal but they share one vertex, so one set of update vertices suffices.
        debug!(
            "SDG build: equation {} '{}' — {} vertices so far",
            e,
            equation.variable().name(),
            builder.graph.node_count()
        );

        let mut seen_pvis: HashSet<ATerm> = HashSet::new();
        for pvi in pbes_expression_pvi(&equation.formula()) {
            let term: ATerm = pvi.clone().into();
            if seen_pvis.insert(term) {
                builder.add_update_vertices(e, &pvi, n)?;
            }
        }
    }

    // Postconditions.
    debug_assert_eq!(builder.colours.len(), builder.equations.len());
    debug_assert_eq!(builder.colours.len(), builder.graph.node_count());
    for k in 0..n {
        debug_assert_eq!(
            builder.colours[k],
            VertexColour::Parameter,
            "the first n vertices must be exactly the parameter vertices"
        );
    }
    for (index, colour) in builder.colours.iter().enumerate().skip(n) {
        debug_assert_ne!(
            *colour,
            VertexColour::Parameter,
            "no vertex beyond the first n may be coloured Parameter (found at index {index})"
        );
    }
    for node in builder.graph.node_indices() {
        debug_assert!(
            builder.graph.find_edge(node, node).is_none(),
            "the SDG must not contain self-loops"
        );
    }

    Ok(Sdg {
        graph: builder.graph,
        colours: builder.colours,
        equations: builder.equations,
        parameters,
        equation_names: equations.iter().map(|eq| eq.variable().name().to_string()).collect(),
    })
}

/// Returns all the PVIs occurring in the given PBES expression.
fn pbes_expression_pvi(expr: &mcrl2::PbesExpression) -> Vec<mcrl2::PbesPropositionalVariableInstantiation> {
    let mut result = Vec::new();
    let _: Option<()> = visit_pbes_expr_with(&expr.copy(), (), |term, ()| {
        if is_pbes_propositional_variable_instantiation(term) {
            result.push(PbesPropositionalVariableInstantiationRef::from(term.copy()).protect());
        }
        ControlFlow::Continue(PbesDescend::Descend(()))
    });
    result
}

/// Builds up an [`Sdg`] incrementally by walking a PBES's right-hand sides.
struct SdgBuilder {
    graph: UnGraph<SdgVertex, EdgeColour>,
    colours: Vec<VertexColour>,
    equations: Vec<BTreeSet<usize>>,

    /// Deduplicates [`SdgVertex::Term`] vertices by (maximally shared) term
    /// identity -- see [`SdgVertex::Term`].
    term_map: HashMap<ATerm, NodeIndex>,

    /// Bound (quantifier-/abstraction-scoped) variables currently in scope,
    /// innermost last, used to tell a bound-variable occurrence apart from a
    /// PBES parameter of the same name (see [`VertexColour::BoundVariable`]).
    scope: Vec<DataVariable>,
}

impl SdgBuilder {
    fn new() -> Self {
        SdgBuilder {
            graph: UnGraph::new_undirected(),
            colours: Vec::new(),
            equations: Vec::new(),
            term_map: HashMap::new(),
            scope: Vec::new(),
        }
    }

    /// Adds a fresh vertex, keeping `colours`/`equations` in lockstep with
    /// `graph`'s node indices.
    fn add_vertex(&mut self, vertex: SdgVertex, colour: VertexColour) -> NodeIndex {
        let index = self.graph.add_node(vertex);
        debug_assert_eq!(index.index(), self.colours.len());
        self.colours.push(colour);
        self.equations.push(BTreeSet::new());
        index
    }

    fn mark_equation(&mut self, node: NodeIndex, equation: usize) {
        self.equations[node.index()].insert(equation);
    }

    /// Adds an edge `(u, v, colour)`, merging into an already-existing `(u,
    /// v)` edge's label set instead of inserting a parallel edge if one is
    /// already present. See [`EdgeColour`].
    fn add_or_merge_edge(&mut self, u: NodeIndex, v: NodeIndex, colour: EdgeColour) {
        if let Some(edge) = self.graph.find_edge(u, v) {
            let existing = self
                .graph
                .edge_weight_mut(edge)
                .expect("find_edge returned a valid edge index");
            trace!("edge: merge v{}--v{} colour {:?} into {:?}", u.index(), v.index(), colour, existing);
            *existing = merge_edge_colour(existing.clone(), colour);
        } else {
            trace!("edge: add   v{}--v{} colour {:?}", u.index(), v.index(), colour);
            self.graph.add_edge(u, v, colour);
        }
    }

    /// Pushes `variables` onto the bound-variable scope, returning how many
    /// were pushed (for [`Self::pop_scope`]).
    fn push_scope(&mut self, variables: impl Iterator<Item = DataVariable>) -> usize {
        let mut count = 0;
        for variable in variables {
            self.scope.push(variable);
            count += 1;
        }
        count
    }

    fn pop_scope(&mut self, count: usize) {
        self.scope.truncate(self.scope.len() - count);
    }

    /// Visits `term` (either a PBES formula or a data expression -- both are
    /// represented by the same underlying `ATerm`, and Definition 2 treats a
    /// Boolean-sorted term `b` uniformly with the other terms `t`), interns
    /// a vertex for it (deduplicated by term identity, matching the
    /// set-based `sub(E)`), and recursively visits its children. If the
    /// vertex already existed its children are *not* re-visited: their
    /// edges are equation-independent and were already added, so only
    /// `C_eq` needs to be extended.
    ///
    /// This is a hand-rolled traversal rather than an implementation of
    /// [`mcrl2::DataExpressionContextVisitor`]/[`mcrl2::PbesExpressionContextVisitor`]
    /// because those traits cannot: thread a `NodeIndex` back to the caller (needed to
    /// add the parent→child edge); thread the argument position (needed for
    /// `C(f(..), t_i) = i`); exclude the head function symbol from being visited as a
    /// child (Definition 2 treats `f` as a colour, not a sub-vertex); or short-circuit
    /// on already-interned terms without re-walking a shared sub-DAG. The dispatch
    /// below mirrors the same `is_*` predicates the traits use, so the two are
    /// expected to stay in sync by inspection.
    fn visit(&mut self, term: ATermRef<'_>, equation: usize) -> NodeIndex {
        let key = term.protect();
        if let Some(&node) = self.term_map.get(&key) {
            trace!("visit: reused v{} for `{:?}` (eq {})", node.index(), term, equation);
            self.mark_equation(node, equation);
            return node;
        }

        let colour = self.colour_of(&key);
        let node = self.add_vertex(SdgVertex::Term(key.clone()), colour.clone());
        trace!("visit: new   v{} {:?} for `{:?}` (eq {})", node.index(), colour, term, equation);
        self.term_map.insert(key.clone(), node);
        self.mark_equation(node, equation);

        self.visit_children(&key, node, equation);
        node
    }

    /// Visits `child`, then adds edge `(parent, child, colour)`.
    fn visit_child(&mut self, parent: NodeIndex, child: ATermRef<'_>, colour: EdgeColour, equation: usize) {
        let child_node = self.visit(child, equation);
        self.add_or_merge_edge(parent, child_node, colour);
    }

    /// Determines `C(term)`. Does not recurse; see [`Self::visit_children`]
    /// for the edges to `term`'s children.
    fn colour_of(&self, term: &ATerm) -> VertexColour {
        let r = term.copy();

        if is_pbes_and(&r) {
            VertexColour::Connective(Connective::And)
        } else if is_pbes_or(&r) {
            VertexColour::Connective(Connective::Or)
        } else if is_pbes_not(&r) {
            VertexColour::Connective(Connective::Not)
        } else if is_pbes_imp(&r) {
            VertexColour::Connective(Connective::Imp)
        } else if is_pbes_forall(&r) {
            let forall = PbesForallRef::from(r);
            let sorts = forall.variables().iter().map(|v| v.sort().protect()).collect();
            VertexColour::Quantifier(Quantifier::Forall, sorts)
        } else if is_pbes_exists(&r) {
            let exists = PbesExistsRef::from(r);
            let sorts = exists.variables().iter().map(|v| v.sort().protect()).collect();
            VertexColour::Quantifier(Quantifier::Exists, sorts)
        } else if is_pbes_propositional_variable_instantiation(&r) {
            VertexColour::Pvi
        } else if is_variable(&r) {
            let variable = DataVariableRef::from(r);
            if self.scope.iter().any(|bound| bound.name() == variable.name()) {
                VertexColour::BoundVariable(variable.sort().protect())
            } else {
                VertexColour::Parameter
            }
        } else if is_application(&r) {
            let application = DataApplicationRef::from(r);
            VertexColour::Function(application.data_function_symbol().name().to_string())
        } else if is_function_symbol(&r) {
            let symbol = DataFunctionSymbolRef::from(r);
            VertexColour::Function(symbol.name().to_string())
        } else if is_machine_number(&r) {
            let number = DataMachineNumberRef::from(r);
            VertexColour::MachineNumber(number.value())
        } else if is_untyped_identifier(&r) {
            // Should not occur in a well-typed PBES; treat as an opaque
            // nullary "function" so it at least gets a stable colour.
            VertexColour::Function(format!("{r:?}"))
        } else if is_abstraction(&r) {
            // Data-level binder: lambda, forall, exists, set/bag comprehension.
            // Colour the same as the PBES-level quantifier for forall/exists so
            // that structurally identical sub-formulas remain deduplicated.
            let abstraction = DataAbstractionRef::from(r.copy());
            let sorts: Vec<SortExpression> =
                abstraction.variables().iter().map(|v| v.sort().protect()).collect();
            let bo: ATermRef<'_> = abstraction.binding_operator().into();
            let q = if is_forall_binder(&bo) {
                Quantifier::Forall
            } else if is_exists_binder(&bo) {
                Quantifier::Exists
            } else if is_lambda_binder(&bo) {
                Quantifier::Lambda
            } else {
                Quantifier::Comprehension
            };
            VertexColour::Quantifier(q, sorts)
        } else if is_where_clause(&r) {
            todo!("where clauses are not yet supported by the symmetry detection graph construction")
        } else {
            unreachable!("Unknown PBES/data expression kind for term {:?}", r)
        }
    }

    /// Adds edges from `node` (the vertex for `term`) to the vertices of its
    /// immediate children, recursing via [`Self::visit`]. Mirrors
    /// Definition 2's `sub#`, generalized per the module documentation.
    fn visit_children(&mut self, term: &ATerm, node: NodeIndex, equation: usize) {
        let r = term.copy();

        if is_pbes_and(&r) {
            let mut leaves = Vec::new();
            collect_pbes_flat(r, is_pbes_and, &mut leaves);
            for leaf in leaves {
                self.visit_child(node, leaf.copy().into(), EdgeColour::Uncoloured, equation);
            }
        } else if is_pbes_or(&r) {
            let mut leaves = Vec::new();
            collect_pbes_flat(r, is_pbes_or, &mut leaves);
            for leaf in leaves {
                self.visit_child(node, leaf.copy().into(), EdgeColour::Uncoloured, equation);
            }
        } else if is_pbes_imp(&r) {
            // Implication is NOT commutative: lhs => rhs ≠ rhs => lhs.
            // Color the edges by position to prevent spurious symmetries.
            let imp = PbesImpRef::from(r);
            self.visit_child(node, imp.lhs().into(), EdgeColour::Argument([1].into_iter().collect()), equation);
            self.visit_child(node, imp.rhs().into(), EdgeColour::Argument([2].into_iter().collect()), equation);
        } else if is_pbes_not(&r) {
            let not = PbesNotRef::from(r);
            self.visit_child(node, not.body().into(), EdgeColour::Uncoloured, equation);
        } else if is_pbes_forall(&r) {
            let forall = PbesForallRef::from(r);
            let pushed = self.push_scope(forall.variables().iter());
            self.visit_child(node, forall.body().into(), EdgeColour::Uncoloured, equation);
            self.pop_scope(pushed);
        } else if is_pbes_exists(&r) {
            let exists = PbesExistsRef::from(r);
            let pushed = self.push_scope(exists.variables().iter());
            self.visit_child(node, exists.body().into(), EdgeColour::Uncoloured, equation);
            self.pop_scope(pushed);
        } else if is_pbes_propositional_variable_instantiation(&r) {
            // A PVI is not itself descended into: Definition 3 reaches its
            // arguments only through update vertices (see
            // `SdgBuilder::add_update_vertices`), matching "phi is not
            // itself a PVI" in the edge rule.
        } else if is_variable(&r) || is_function_symbol(&r) || is_machine_number(&r) || is_untyped_identifier(&r) {
            // Leaves: sub#(x) = {} (also true for a nullary function symbol,
            // a machine number, and an untyped identifier).
        } else if is_application(&r) {
            let application = DataApplicationRef::from(r.copy());
            let head = application.data_function_symbol();
            let arity = application.data_arguments().len();

            if is_flat_operator(head.name(), arity) {
                // Flatten the entire chain into a single n-ary vertex so that
                // `a && b && c` (stored as `&&(&&(a,b),c)`) yields one vertex
                // with three uncoloured edges rather than nested binary nodes.
                let name = head.name();
                let expr: DataExpressionRef<'_> = r.into();
                let leaves = flatten_associative(&expr, |t| {
                    is_application(t) && DataApplicationRef::from(t.copy()).data_function_symbol().name() == name
                });
                for leaf in &leaves {
                    self.visit_child(node, leaf.copy().into(), EdgeColour::Uncoloured, equation);
                }
            } else {
                let commutative = is_commutative(head.name(), arity);

                // Group arguments by the vertex they resolve to, combining
                // positions for repeated arguments (see `EdgeColour::Argument`).
                let mut by_child: BTreeMap<NodeIndex, BTreeSet<usize>> = BTreeMap::new();
                for (position, argument) in application.data_arguments().enumerate() {
                    let child = self.visit(argument, equation);
                    by_child.entry(child).or_default().insert(position + 1);
                }

                for (child, positions) in by_child {
                    let colour = if commutative {
                        EdgeColour::Uncoloured
                    } else {
                        EdgeColour::Argument(positions)
                    };
                    self.add_or_merge_edge(node, child, colour);
                }
            }
        } else if is_abstraction(&r) {
            let abstraction = DataAbstractionRef::from(r);
            let pushed = self.push_scope(abstraction.variables().iter());
            self.visit_child(node, abstraction.body().into(), EdgeColour::Uncoloured, equation);
            self.pop_scope(pushed);
        } else if is_where_clause(&r) {
            todo!("where clauses are not yet supported by the symmetry detection graph construction")
        } else {
            unreachable!("Unknown PBES/data expression kind for term {:?}", r)
        }
    }

    /// Adds the update vertices `X_{i,k}` (for `k` in `1..=n`) for a single
    /// PVI `pvi` occurring in the right-hand side of `equation`, along with
    /// their edges to the PVI vertex, the PVI's argument vertices, and the
    /// global parameter vertices.
    fn add_update_vertices(
        &mut self,
        equation: usize,
        pvi: &mcrl2::PbesPropositionalVariableInstantiation,
        n: usize,
    ) -> Result<(), MercError> {
        let pvi_term: ATerm = pvi.clone().into();
        let pvi_ref: ATermRef<'_> = pvi_term.copy();
        // The PVI vertex must already exist: it was interned while walking
        // the right-hand side (PVIs are leaves of that walk, but they are
        // still visited and given a vertex -- see `visit_children`).
        let pvi_node = self.visit(pvi_ref, equation);

        let arguments: Vec<ATerm> = pvi
            .arguments()
            .iter()
            .map(|argument| argument.protect().into())
            .collect();
        if arguments.len() != n {
            return Err(format!(
                "Predicate variable instance '{}' has {} argument(s), but the unified parameter \
                 vector has {} parameter(s); every predicate variable instance must supply exactly \
                 one argument per parameter.",
                pvi.name(),
                arguments.len(),
                n
            )
            .into());
        }

        // The update vertex's index within this equation's right-hand side
        // (the paper's `i` in `X_{i,k}`): the PVI-index counter used here
        // only needs to be unique per `(equation, pvi term)`, since update
        // vertices are never deduplicated regardless of its value -- so the
        // PVI's own vertex index doubles as a stable, sufficiently unique
        // `i`.
        let i = pvi_node.index();
        debug!("update vertices: eq {} pvi '{}' (v{}) — {} parameter(s)", equation, pvi.name(), i, n);

        for k in 0..n {
            let update_vertex = SdgVertex::Update {
                equation,
                pvi: i,
                parameter: k,
            };
            let update_node = self.add_vertex(update_vertex, VertexColour::Update);
            trace!("update vertex v{} X_({},{},{}) (eq {})", update_node.index(), equation, i, k, equation);
            self.mark_equation(update_node, equation);

            let data_node = self.visit(arguments[k].copy(), equation);
            let par_node = NodeIndex::new(k);

            // Group the (at most three) targets by vertex identity, and
            // colour each resulting edge with the combined set of roles
            // that land on it (see `EdgeColour::Update`).
            let mut by_target: BTreeMap<NodeIndex, BTreeSet<UpdateRole>> = BTreeMap::new();
            by_target.entry(pvi_node).or_default().insert(UpdateRole::Pvi);
            by_target.entry(data_node).or_default().insert(UpdateRole::Data);
            by_target.entry(par_node).or_default().insert(UpdateRole::Par);

            for (target, roles) in by_target {
                self.add_or_merge_edge(update_node, target, EdgeColour::Update(roles));
            }
        }

        Ok(())
    }
}

/// Merges two edge colours that were found to coincide on the same `(u, v)`
/// vertex pair. See [`EdgeColour`] and [`SdgBuilder::add_or_merge_edge`].
fn merge_edge_colour(a: EdgeColour, b: EdgeColour) -> EdgeColour {
    match (a, b) {
        (EdgeColour::Argument(mut xs), EdgeColour::Argument(ys)) => {
            xs.extend(ys);
            EdgeColour::Argument(xs)
        }
        (EdgeColour::Update(mut xs), EdgeColour::Update(ys)) => {
            xs.extend(ys);
            EdgeColour::Update(xs)
        }
        (EdgeColour::Uncoloured, EdgeColour::Uncoloured) => EdgeColour::Uncoloured,
        (a, b) => unreachable!(
            "Cannot merge incompatible edge colours {:?} and {:?} on the same vertex pair",
            a, b
        ),
    }
}

// ── GAP-facing conversion ─────────────────────────────────────────────────────

/// The SDG in the form GAP's Digraphs package expects: a symmetric digraph
/// (every undirected edge as two opposite directed arcs) with 1-based points.
pub(crate) struct GapGraph {
    /// `out_neighbours[u]` lists the 1-based out-neighbours of vertex `u+1`.
    out_neighbours: Vec<Vec<usize>>,
    /// Positionally aligned with `out_neighbours`; `edge_colours[u][j]` is the
    /// dense colour index of the j-th arc out of vertex `u+1`.
    edge_colours: Vec<Vec<usize>>,
    /// `vertex_colours[u]` is the dense colour index of vertex `u+1`.
    vertex_colours: Vec<usize>,
    /// How many of the leading vertices (NodeIndex 0..n-1) are parameter vertices.
    pub(crate) num_parameters: usize,
}

impl Sdg {
    pub(crate) fn to_gap_graph(&self) -> GapGraph {
        let n = self.graph.node_count();
        let mut vc_map: HashMap<String, usize> = HashMap::new();
        let mut next_vc = 1usize;
        let mut vertex_colours = vec![0usize; n];

        for node in self.graph.node_indices() {
            let key = format!("{:?}|{:?}", self.colours[node.index()], self.equations[node.index()]);
            let dense = *vc_map.entry(key).or_insert_with(|| {
                let c = next_vc;
                next_vc += 1;
                c
            });
            vertex_colours[node.index()] = dense;
        }

        let mut ec_map: HashMap<String, usize> = HashMap::new();
        let mut next_ec = 1usize;
        let mut out_neighbours = vec![Vec::new(); n];
        let mut edge_colours = vec![Vec::new(); n];

        for edge in self.graph.edge_indices() {
            let (u, v) = self.graph.edge_endpoints(edge).unwrap();
            let colour = self.graph.edge_weight(edge).unwrap();
            let ec_key = format!("{:?}", colour);
            let dense_ec = *ec_map.entry(ec_key).or_insert_with(|| {
                let c = next_ec;
                next_ec += 1;
                c
            });
            out_neighbours[u.index()].push(v.index() + 1);
            edge_colours[u.index()].push(dense_ec);
            out_neighbours[v.index()].push(u.index() + 1);
            edge_colours[v.index()].push(dense_ec);
        }

        debug_assert!(
            out_neighbours
                .iter()
                .zip(&edge_colours)
                .all(|(nb, ec)| nb.len() == ec.len()),
            "out_neighbours and edge_colours must be positionally aligned"
        );
        debug_assert!(
            out_neighbours
                .iter()
                .enumerate()
                .all(|(u, nb)| { nb.iter().zip(&edge_colours[u]).collect::<HashSet<_>>().len() == nb.len() }),
            "no two arcs from the same source may share both target and colour"
        );

        GapGraph {
            out_neighbours,
            edge_colours,
            vertex_colours,
            num_parameters: self.parameters.len(),
        }
    }
}

// ── DOT export ───────────────────────────────────────────────────────────────

/// Writes the SDG as a Graphviz DOT file, including vertex/edge colours.
pub(crate) fn write_dot(sdg: &Sdg, w: &mut impl std::io::Write) -> Result<(), MercError> {
    writeln!(w, "graph sdg {{")?;
    // fdp (force-directed) produces far more readable layouts than the default
    // hierarchical `dot` engine for general undirected graphs like the SDG.
    writeln!(w, "  graph [fontname=\"DejaVu Sans\", layout=fdp, overlap=scale, splines=curved];")?;
    writeln!(w, "  node  [fontname=\"DejaVu Sans\", style=filled, shape=ellipse];")?;
    writeln!(w, "  edge  [fontname=\"DejaVu Sans\"];")?;

    for node in sdg.graph.node_indices() {
        let i = node.index();
        let vc = &sdg.colours[i];

        let label = match vc {
            VertexColour::Parameter => format!("par:{}", sdg.parameters[i].name()),
            VertexColour::Update => {
                if let SdgVertex::Update { parameter, .. } = &sdg.graph[node] {
                    format!("upd:{}", sdg.parameters[*parameter].name())
                } else {
                    unreachable!()
                }
            }
            _ => vc.to_string(),
        };

        // Distinct shapes make vertex roles identifiable without reading the label.
        let (shape, penwidth) = match vc {
            VertexColour::Parameter      => ("box",           "2"),
            VertexColour::Update         => ("diamond",       "1"),
            VertexColour::Pvi            => ("hexagon",       "1"),
            VertexColour::Quantifier(..) => ("parallelogram", "1"),
            _                            => ("ellipse",       "1"),
        };

        let fill = vc.dot_fill_colour();

        let eqs = sdg.equations[i]
            .iter()
            .map(|&e| sdg.equation_names[e].as_str())
            .collect::<Vec<_>>()
            .join(",");
        let tooltip = if eqs.is_empty() {
            label.clone()
        } else {
            format!("{label} [{eqs}]")
        };

        writeln!(
            w,
            "  n{i} [label=\"{label}\", shape={shape}, fillcolor=\"{fill}\", penwidth={penwidth}, tooltip=\"{tooltip}\"];"
        )?;
    }

    for edge in sdg.graph.edge_indices() {
        let (u, v) = sdg.graph.edge_endpoints(edge).unwrap();
        let colour = sdg.graph.edge_weight(edge).unwrap();
        let elabel = colour.to_string();

        let mut attrs = Vec::<String>::new();
        if !elabel.is_empty() {
            attrs.push(format!("label=\"{elabel}\""));
        }
        if matches!(colour, EdgeColour::Update(_)) {
            attrs.push("style=dashed".to_string());
        }

        if attrs.is_empty() {
            writeln!(w, "  n{} -- n{};", u.index(), v.index())?;
        } else {
            writeln!(w, "  n{} -- n{} [{}];", u.index(), v.index(), attrs.join(", "))?;
        }
    }

    writeln!(w, "}}")?;
    Ok(())
}

// ── graph6 export ─────────────────────────────────────────────────────────────

/// Renders the structural skeleton of the SDG in graph6 format.
///
/// graph6 encodes only the adjacency structure of a simple undirected graph;
/// vertex and edge colours are not representable. Use this as a debug or
/// interop artifact (nauty's `showg`, GAP's `DigraphFromGraph6String`), not
/// as the channel through which colours reach GAP (that goes through the script).
pub(crate) fn graph6_string(sdg: &Sdg) -> Result<String, MercError> {
    if sdg.graph.node_count() > 258_047 {
        return Err("graph6 does not support graphs over 258047 vertices".into());
    }
    use petgraph::graph6::ToGraph6;
    Ok(sdg.graph.graph6_string())
}

// ── GAP script generation, invocation, and output parsing ────────────────────

/// Generates a self-contained GAP script that computes `Aut(G)` via the
/// Digraphs package, restricts generators to the parameter vertices, and
/// prints the result between `SDG-BEGIN`/`SDG-END` sentinels.
///
/// Every statement ends in `;;` because a script fed on stdin is read as an
/// interactive session where a single `;` echoes the value to stdout.
/// The sentinels also defend against GAP's exit-0-on-syntax-error behaviour.
fn gap_script(graph: &GapGraph) -> String {
    let n = graph.num_parameters;
    let num_vertices = graph.out_neighbours.len();

    let neighbours_str = (0..num_vertices)
        .map(|u| {
            if graph.out_neighbours[u].is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", graph.out_neighbours[u].iter().join(","))
            }
        })
        .join(",");

    let vc_str = graph.vertex_colours.iter().join(",");

    let ec_str = (0..num_vertices)
        .map(|u| {
            if graph.edge_colours[u].is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", graph.edge_colours[u].iter().join(","))
            }
        })
        .join(",");

    format!(
        r#"SetPrintFormattingStatus("*stdout*", false);;
if LoadPackage("digraphs") = fail then
  Error("the GAP package 'Digraphs' could not be loaded; install it from https://digraphs.github.io/Digraphs/");
fi;;
D := Digraph([{neighbours_str}]);;
vcolours := [{vc_str}];;
ecolours := [{ec_str}];;
A := AutomorphismGroup(D, vcolours, ecolours);;
R := List(GeneratorsOfGroup(A), g -> RestrictedPerm(g, [1..{n}]));;
P := GroupByGenerators(R, ());;
Print("SDG-BEGIN\n");;
Print("order ", Size(A), "\n");;
Print("restricted ", Size(P), "\n");;
for g in GeneratorsOfGroup(P) do
  for i in [1..{n}] do Print(i^g, " "); od;
  Print("\n");
od;;
Print("SDG-END\n");;
QUIT;;
"#
    )
}

/// Configuration for invoking the external GAP process.
pub(crate) struct GapConfig {
    /// Path or name of the GAP executable (default: `"gap"` on `$PATH`).
    pub(crate) executable: String,
    /// If set, the generated GAP script is also written to this file.
    pub(crate) dump_script: Option<std::path::PathBuf>,
}

impl Default for GapConfig {
    fn default() -> Self {
        GapConfig {
            executable: "gap".to_string(),
            dump_script: None,
        }
    }
}

/// Invokes GAP with the generated script on stdin and returns the captured stdout.
///
/// Flags used (verified on GAP 4.12.1):
/// - `-q` suppresses the banner and prompt
/// - `-A` disables autoloading of suggested packages
/// - `-r` ignores the user's gap.ini
/// - `--quitonbreak` makes runtime errors exit with non-zero status
///   (note: do NOT add `-T`/`--nobreakloop`, which defeats `--quitonbreak`)
fn run_gap(script: &str, config: &GapConfig) -> Result<String, MercError> {
    use std::io::ErrorKind;

    if let Some(path) = &config.dump_script {
        std::fs::write(path, script)
            .map_err(|e| MercError::from(format!("failed to write GAP script to '{}': {}", path.display(), e)))?;
    }

    let output = duct::cmd(&*config.executable, ["-q", "-A", "-r", "--quitonbreak"])
        .stdin_bytes(script.to_owned())
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                MercError::from(format!(
                    "GAP executable '{}' not found; install GAP from https://www.gap-system.org/ \
                     or pass --gap-path to specify its location",
                    config.executable
                ))
            } else {
                MercError::from(e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut msg = format!("GAP exited with {}", output.status);
        if stderr.contains("digraphs") || stderr.contains("Digraphs") {
            msg.push_str(
                "; the Digraphs package may not be installed — \
                 see https://digraphs.github.io/Digraphs/",
            );
        } else if !stderr.trim().is_empty() {
            msg.push_str(&format!(": {}", stderr.trim()));
        }
        return Err(msg.into());
    }

    String::from_utf8(output.stdout).map_err(|e| MercError::from(e.to_string()))
}

/// Parses the output produced by the GAP script between the `SDG-BEGIN`/`SDG-END`
/// sentinels back into `(automorphism_group_order, symmetry_group_order, generators)`.
///
/// GAP prints permutations as 1-indexed image vectors; this function converts
/// them to 0-indexed and builds [`Permutation`] values via [`Permutation::from_mapping`].
fn parse_gap_output(
    stdout: &str,
    num_parameters: usize,
) -> Result<(u128, u128, Vec<crate::permutation::Permutation>), MercError> {
    // Extract lines strictly between the sentinels.
    let begin_pos = stdout.find("SDG-BEGIN").ok_or_else(|| {
        MercError::from("GAP output is missing 'SDG-BEGIN' sentinel — check for syntax errors in the generated script")
    })?;
    let after_begin = &stdout[begin_pos + "SDG-BEGIN".len()..];
    let end_pos = after_begin
        .find("SDG-END")
        .ok_or_else(|| MercError::from("GAP output is missing 'SDG-END' sentinel"))?;
    let inner = after_begin[..end_pos].trim();

    let mut lines = inner.lines();

    let aut_order: u128 = lines
        .next()
        .and_then(|l| l.strip_prefix("order "))
        .and_then(|s| s.trim().parse().ok())
        .ok_or_else(|| MercError::from("expected 'order <n>' as first line inside sentinels"))?;

    let sym_order: u128 = lines
        .next()
        .and_then(|l| l.strip_prefix("restricted "))
        .and_then(|s| s.trim().parse().ok())
        .ok_or_else(|| MercError::from("expected 'restricted <n>' as second line inside sentinels"))?;

    let mut generators = Vec::new();
    for (line_no, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let images: Vec<usize> = line
            .split_whitespace()
            .map(|s| {
                s.parse::<usize>()
                    .map(|v| v - 1) // 1-indexed → 0-indexed
                    .map_err(|_| MercError::from(format!("invalid image '{}' on generator line {}", s, line_no)))
            })
            .collect::<Result<_, _>>()?;

        if images.len() != num_parameters {
            return Err(format!(
                "generator line {} has {} images but expected {} (num_parameters)",
                line_no,
                images.len(),
                num_parameters
            )
            .into());
        }

        let mapping: Vec<(usize, usize)> = images.into_iter().enumerate().filter(|(from, to)| from != to).collect();

        if !mapping.is_empty() {
            generators.push(crate::permutation::Permutation::from_mapping(mapping));
        }
    }

    Ok((aut_order, sym_order, generators))
}

/// Result returned by [`graph_symmetries`].
pub(crate) struct GraphSymmetryResult {
    pub(crate) sdg: Sdg,
    pub(crate) automorphism_group_order: u128,
    pub(crate) symmetry_group_order: u128,
    pub(crate) generators: Vec<crate::permutation::Permutation>,
}

/// Builds the SDG of `pbes`, invokes GAP to compute its automorphism group,
/// and returns the generators restricted to the parameter vertices.
pub(crate) fn graph_symmetries(pbes: &Pbes, config: &GapConfig) -> Result<GraphSymmetryResult, MercError> {
    use log::info;
    // Unify parameters here so build_sdg works on the original PBES structure.
    let mut pbes = Pbes::from_text(&pbes.to_string())?;
    pbes.unify_parameters(false, false)?;
    let sdg = build_sdg(&pbes)?;
    info!(
        "SDG: {} vertices, {} edges, {} parameters",
        sdg.num_vertices(),
        sdg.num_edges(),
        sdg.num_parameters()
    );
    debug!("SDG equation names: [{}]", sdg.equation_names.iter().format(", "));

    let gap_graph = sdg.to_gap_graph();
    let script = gap_script(&gap_graph);
    let stdout = run_gap(&script, config)?;
    let (aut_order, sym_order, generators) = parse_gap_output(&stdout, gap_graph.num_parameters)?;

    info!(
        "|Aut(G)| = {}, |Sym(pbes)| = {}, {} generator(s)",
        aut_order,
        sym_order,
        generators.len()
    );

    Ok(GraphSymmetryResult {
        sdg,
        automorphism_group_order: aut_order,
        symmetry_group_order: sym_order,
        generators,
    })
}

#[cfg(test)]
mod tests {
    use mcrl2::Pbes;
    use merc_utilities::test_logger;
    use petgraph::graph::NodeIndex;
    use petgraph::visit::EdgeRef;

    use super::EdgeColour;
    use super::UpdateRole;
    use super::VertexColour;
    use super::build_sdg;

    /// The known symmetry `(0 2)(1 3)` for `examples/pbes/c.text.pbes`,
    /// matching `symmetry::tests::test_symmetry_examples_c` -- the graph
    /// construction should at least be shaped consistently with that result
    /// (this test does not run GAP, so it only checks the graph, not the
    /// derived automorphism group).
    #[test]
    fn test_c_pbes_parameter_vertices() {
        test_logger();
        let pbes = Pbes::from_text(include_str!("../../../../examples/pbes/c.text.pbes")).unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        assert_eq!(sdg.num_parameters(), 4, "c.text.pbes has 4 parameters");
        for k in 0..4 {
            assert_eq!(
                sdg.colours[k],
                VertexColour::Parameter,
                "vertex {k} should be the k'th parameter"
            );
        }
        // No other vertex may be coloured Parameter.
        assert!(sdg.colours.iter().skip(4).all(|c| *c != VertexColour::Parameter));

        assert!(sdg.num_vertices() > 4, "there should be vertices beyond the parameters");
        assert!(sdg.num_edges() > 0);
    }

    #[test]
    fn test_a_and_b_pbes_build_without_error() {
        test_logger();
        for source in [
            include_str!("../../../../examples/pbes/a.text.pbes"),
            include_str!("../../../../examples/pbes/b.text.pbes"),
        ] {
            let pbes = Pbes::from_text(source).unwrap();
            build_sdg(&pbes).unwrap();
        }
    }

    /// Identical subterms across two different equations collapse to a
    /// single vertex (Definition 2's `sub(E)` is a set), with `C_eq`
    /// accumulating both equations.
    #[test]
    fn test_identical_subterms_share_one_vertex_across_equations() {
        test_logger();
        let pbes = Pbes::from_text(
            "pbes mu X(n: Nat) = val(n == 0) && Y(n);
                  mu Y(n: Nat) = val(n == 0) && X(n);
             init X(0);",
        )
        .unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        // `n == 0` occurs (syntactically identically) in both equations, so
        // it must be exactly one vertex, reachable from both equation 0 (X)
        // and equation 1 (Y).
        let condition_node = sdg
            .graph
            .node_indices()
            .find(|&index| matches!(&sdg.colours[index.index()], VertexColour::Function(name) if name == "=="))
            .expect("there should be a vertex for the '==' application");
        assert_eq!(
            sdg.equations[condition_node.index()],
            [0, 1].into_iter().collect(),
            "the shared condition must be reachable from both equations"
        );
    }

    /// Update vertices are never deduplicated, even when two `(equation,
    /// pvi, parameter)` triples would otherwise look structurally identical.
    #[test]
    fn test_update_vertices_are_never_deduplicated() {
        test_logger();
        let pbes = Pbes::from_text(
            "pbes mu X(n: Nat) = Y(n) && Y(n);
                  mu Y(n: Nat) = val(n == 0);
             init X(0);",
        )
        .unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        // Two distinct PVI occurrences `Y(n)` in X's right-hand side, each
        // contributing its own update vertex X_{i,0}, even though both
        // PVIs are syntactically identical (and therefore share one PVI
        // vertex -- update vertices are keyed by the *PVI's own* node
        // index, not by structural equality, so a shared PVI vertex still
        // only yields one set of update vertices; the real regression this
        // guards is that update vertices are fresh per call site).
        let update_count = sdg
            .graph
            .node_indices()
            .filter(|&index| sdg.colours[index.index()] == VertexColour::Update)
            .count();
        assert_eq!(update_count, 1, "one PVI occurrence (post-dedup) times one parameter");
    }

    /// The head function symbol of an application is a *colour*, not a
    /// vertex: `n + n` should produce exactly the vertices for `+`, `n` --
    /// two vertices, not three.
    #[test]
    fn test_head_function_symbol_is_not_a_vertex() {
        test_logger();
        let pbes = Pbes::from_text("pbes mu X(n: Nat) = val(n + n == n); init X(0);").unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        let function_names: Vec<&String> = sdg
            .colours
            .iter()
            .filter_map(|colour| match colour {
                VertexColour::Function(name) => Some(name),
                _ => None,
            })
            .collect();

        // "+" and "==" should both appear as vertex colours (the function
        // symbol is a colour), but never as a bare unlabelled child vertex.
        assert!(function_names.iter().any(|name| name.as_str() == "+"));
        assert!(function_names.iter().any(|name| name.as_str() == "=="));
    }

    /// A non-commutative function applied twice to the *same* argument
    /// (`n - n`) produces exactly one edge to `n`, coloured with the
    /// combined position set `{1,2}` -- not two parallel edges.
    #[test]
    fn test_noncommutative_repeated_argument_gets_combined_label() {
        test_logger();
        let pbes = Pbes::from_text("pbes mu X(n: Int) = val(n - n == 0); init X(0);").unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        let minus_node = sdg
            .graph
            .node_indices()
            .find(|&index| matches!(&sdg.colours[index.index()], VertexColour::Function(name) if name == "-"))
            .expect("there should be a vertex for the '-' application");

        // Use edges_connecting to count only edges between minus_node and the parameter `n`
        // (NodeIndex(0)), not all incident edges (which would include the parent `==` edge).
        let n_node = NodeIndex::new(0);
        let edges: Vec<_> = sdg.graph.edges_connecting(minus_node, n_node).collect();
        assert_eq!(edges.len(), 1, "n - n should have exactly one edge to 'n'");
        assert_eq!(*edges[0].weight(), EdgeColour::Argument([1, 2].into_iter().collect()));
    }

    /// A commutative function applied twice to the same argument (`n ==
    /// n`) still produces exactly one (uncoloured) edge: the combined-label
    /// mechanism only matters for non-commutative operators.
    #[test]
    fn test_commutative_repeated_argument_stays_uncoloured() {
        test_logger();
        let pbes = Pbes::from_text("pbes mu X(n: Nat) = val(n == n); init X(0);").unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        let eq_node = sdg
            .graph
            .node_indices()
            .find(|&index| matches!(&sdg.colours[index.index()], VertexColour::Function(name) if name == "=="))
            .expect("there should be a vertex for the '==' application");

        let edges: Vec<_> = sdg.graph.edges(eq_node).collect();
        assert_eq!(edges.len(), 1, "n == n should reach 'n' via exactly one edge");
        assert_eq!(*edges[0].weight(), EdgeColour::Uncoloured);
    }

    /// A PVI that copies a parameter unchanged (`X(n)` from within `X`'s own
    /// right-hand side) makes the `data(X,i,k)` and `d_k` update-edge
    /// targets coincide: the update vertex must have exactly one combined
    /// `{Data, Par}`-coloured edge to `n`, plus a separate `{Pvi}`-coloured
    /// edge to the PVI vertex -- not three edges, and not a silently
    /// dropped role.
    #[test]
    fn test_update_edge_combines_roles_on_parameter_copy() {
        test_logger();
        let pbes = Pbes::from_text("pbes mu X(n: Nat) = X(n); init X(0);").unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        let update_node = sdg
            .graph
            .node_indices()
            .find(|&index| sdg.colours[index.index()] == VertexColour::Update)
            .expect("there should be exactly one update vertex");

        let mut edges: Vec<_> = sdg.graph.edges(update_node).map(|e| e.weight().clone()).collect();
        edges.sort_by_key(|colour| format!("{colour:?}"));

        assert_eq!(
            edges.len(),
            2,
            "expected one {{Pvi}} edge and one combined {{Data,Par}} edge"
        );
        assert!(edges.contains(&EdgeColour::Update([UpdateRole::Pvi].into_iter().collect())));
        assert!(edges.contains(&EdgeColour::Update(
            [UpdateRole::Data, UpdateRole::Par].into_iter().collect()
        )));

        // And the combined edge's target must be the parameter vertex `n`,
        // i.e. NodeIndex(0).
        let combined_target = sdg
            .graph
            .edges(update_node)
            .find(|e| *e.weight() == EdgeColour::Update([UpdateRole::Data, UpdateRole::Par].into_iter().collect()))
            .map(|e| e.target())
            .unwrap();
        assert_eq!(combined_target, NodeIndex::new(0));
    }

    /// Two equations declaring different parameter vectors are rejected
    /// with a clear error rather than silently producing a malformed graph.
    #[test]
    fn test_rejects_non_uniform_parameter_vectors() {
        test_logger();
        // Use the same arity but different parameter names so mCRL2 accepts the PBES
        // while `unified_parameters` still rejects it (n:Nat ≠ m:Nat as ATerms).
        let pbes = Pbes::from_text(
            "pbes mu X(n: Nat) = Y(n);
                  mu Y(m: Nat) = val(true);
             init X(0);",
        )
        .unwrap();

        let result = build_sdg(&pbes);
        assert!(result.is_err(), "differing parameter vectors must be rejected");
    }

    /// A bound (quantifier-scoped) variable must not be coloured the same
    /// as a PBES parameter, even when it shares a name with one -- otherwise
    /// an automorphism could conflate the two.
    #[test]
    fn test_bound_variable_is_not_coloured_as_parameter() {
        test_logger();
        // Use a different name for the bound variable so it is a distinct ATerm from the
        // parameter `n`. Under mCRL2 hash-consing, `n:Nat` and `n:Nat` are the same ATerm
        // and would therefore collapse to one vertex regardless of scope.
        let pbes = Pbes::from_text("pbes mu X(n: Nat) = exists m: Nat . val(n == m); init X(0);").unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        let parameter_count = sdg.colours.iter().filter(|c| **c == VertexColour::Parameter).count();
        assert_eq!(parameter_count, 1);

        let has_bound_variable = sdg.colours.iter().any(|c| matches!(c, VertexColour::BoundVariable(_)));
        assert!(
            has_bound_variable,
            "the quantifier-bound 'm' should be coloured BoundVariable"
        );
    }

    /// A data-level binder expression (e.g. `val(exists m:Nat. n==m)`) must not
    /// cause an `unreachable!` panic: it must be assigned a `Quantifier` colour and
    /// its bound variable must be coloured `BoundVariable`, not `Parameter`.
    #[test]
    fn test_data_level_binder_does_not_panic() {
        test_logger();
        // `val(exists m: Nat . n == m)` wraps a data-level binder inside val(...)
        // so the ATerm reaching colour_of is `Binder(Exists, [m:Nat], ==(n,m))`.
        let pbes = Pbes::from_text(
            "pbes mu X(n: Nat) = val(exists m: Nat . n == m); init X(0);",
        )
        .unwrap();
        let sdg = build_sdg(&pbes).unwrap();

        use super::Quantifier;
        let has_quantifier = sdg.colours.iter().any(|c| {
            matches!(c, VertexColour::Quantifier(Quantifier::Exists, _))
        });
        assert!(has_quantifier, "data-level exists binder must produce a Quantifier(Exists) vertex");

        let has_bound_variable = sdg.colours.iter().any(|c| matches!(c, VertexColour::BoundVariable(_)));
        assert!(has_bound_variable, "the data-level bound 'm' must be coloured BoundVariable");
    }
}
