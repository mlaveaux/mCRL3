# Overview

The `merc_typecheck` crate type checks mCRL2 data specifications, following the
definitions in *Modeling and Analysis of Communicating Systems* (Groote &
Mousavi, MIT Press 2014). It turns the loosely-structured syntax tree produced
by the `merc_syntax` parser into a fully typed specification: it resolves every
name, decides the sort of every expression, chooses between overloaded
operators, and inserts the implicit coercions the surface language leaves out
(such as reading a natural number where a real number is expected).

Type checking runs as a pipeline of phases — sort resolution, desugaring,
signature building, constraint-based sort inference, then lowering to the
aterm representation used by the rest of merc (`merc_data`, `merc_sabre`,
`merc_explore`). For the full design — the query-based architecture, the sort
lattice, and the ranked backtracking search that drives inference — see the
[Type Checking](https://MERCorg.github.io/merc/developer/typechecking/) page
on the documentation site.

## Usage

The entry point is `DataSpecification::from_untyped`, which takes an
`UntypedDataSpecification` (produced by `merc_syntax`) and returns a type
checked specification, or a `WellTypedError` describing the first problem
found. Calling `lower_data_specification` on the result produces the
`merc_data::Mcrl2DataSpecification` (aterm, fully typed) consumed by rewriting.

```rust
use merc_syntax::UntypedDataSpecification;
use merc_typecheck::DataSpecification;

let untyped = UntypedDataSpecification::parse(
    "sort D = struct c(pr: Nat, other: Bool)?is_c | d;
     map f: D -> Nat;
     var d: D;
     eqn f(d) = pr(d);",
)
.unwrap();

let spec = DataSpecification::from_untyped(untyped).unwrap();
let lowered = spec.lower_data_specification();
```

`DataSpecification::typing_info` (and its per-equation counterpart,
`equation_typing_info`) returns a `TypingInfo`: the sort and name resolution of
every checked expression node, keyed by source `Span` rather than any internal
id, so a consumer such as an editor integration can look up hover text or a
go-to-definition target by byte offset via `TypingInfo::at_offset` without
needing to know anything about how inference numbers its nodes internally.
`typecheck_expression_with_typing` returns the same information for a single
standalone expression. Note that a resolved *sort* embedded in a `TypedNode`
carries no reliable source span of its own — only spans on the original
specification's declarations and expressions are meaningful.

### Whole-process type checking

`ProcessSpecification::from_untyped` extends the above to a full
`UntypedProcessSpecification`: it type checks the data specification first
(exactly as `DataSpecification::from_untyped_with` does), then every `act`
argument sort, every `glob`/`proc` parameter sort, and every `proc` body and
`init` — action and process-instantiation arguments against their declared
sorts (with overload resolution when a name is declared more than once),
`sum`/`dist`-bound variables in scope for the subtree they bind, conditions
against `Bool`, and time bounds/`dist` weights against `Real`. Errors are
`ProcessError`, a superset of `WellTypedError`/`InferenceError`.

**Known limitation**: mCRL2's concrete syntax overloads several tokens between
the process algebra and the data language — most notably `.` (process
sequential composition vs. the data "at"/indexing operator) and `+` (process
choice vs. data addition) — and disambiguating between the two readings needs
semantic information a context-free grammar doesn't have, so `merc_syntax`'s
grammar always parses the greedier data-expression reading first. This means
`act(args) . cond -> P <> Q` parses `act(args) . cond` as a single data
expression rather than an action step followed by the real condition.
`ProcessSpecification` recovers the common case of this — a single leading
action or process-instantiation step before a genuine condition — using the
same declaration tables it already built (see `crate::process::check`'s
`check_condition`), but not every shape the ambiguity produces, most commonly
a `+`-separated chain of guarded alternatives whose guard bodies contain
actions. `crates/typecheck/tests/example_tests.rs` documents which specifications
in the example corpus are affected. Fully resolving this needs either a
semantics-directed reparse or a grammar change in `merc_syntax`; both are out
of scope for this crate.

`DataSpecification::from_untyped_with` additionally takes a `NumberEncoding`,
selecting how number literals are lowered to their Appendix-B constructor
chains: the recursive-binary encoding (the default), or a 64-bit machine-word
encoding.

## Safety

This crate contains no unsafe code.

## Minimum Supported Rust Version

We do not maintain an official minimum supported rust version (MSRV), and it may be upgraded at any time when necessary.

## License

All MERC crates are licensed under the `BSL-1.0` license. See the [LICENSE](https://raw.githubusercontent.com/MERCorg/merc/refs/heads/main/LICENSE) file in the repository root for more information.
