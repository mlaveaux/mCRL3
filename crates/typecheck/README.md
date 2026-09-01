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
specification's declarations and expressions are meaningful. Both methods take
`&mut self` and memoize their result on `DataSpecification` (once per equation,
once for the whole document): a `DataSpecification` is immutable once built, so
a second call reuses the cached result instead of re-deriving it.

### Process and PBES specifications

`ProcessSpecification::from_untyped` extends `DataSpecification` to also check
a whole `UntypedProcessSpecification`: `act`/`glob`/`proc` declarations and
every `proc` body and `init`, resolving action/process-instantiation
overloads and checking conditions against `Bool`. Before checking runs,
`ProcessSpecification` reparses the specification to undo a handful of
grammar-level ambiguities mCRL2's concrete syntax has between the process
algebra and the data language — `.`, `+`, and `||` all lex the same in both,
and a single-action `hide`/`block`/`allow` application can appear in the same
ambiguous position. See `crates/typecheck/src/process/reparse.rs`'s module
doc comment for the exact shapes, or the [Process
Specification](https://MERCorg.github.io/merc/developer/typechecking/process-specification/)
page for a worked example with parse trees and the known limitations.
`ProcessSpecification::typing_info` exposes the same span-keyed `TypingInfo`
as `DataSpecification`, merged over every checked process-body expression.

`PbesSpecification::from_untyped` similarly extends `DataSpecification` to
check a whole PBES: `glob` declarations, each propositional-variable
equation's parameters, and every equation's formula and `init` — resolving
each `PropVarInst` by name and arity, checking `val(...)` expressions against
`Bool`, and scoping quantifier binders. This covers *sorts and names* only;
PBES well-formedness properties such as monotonicity or alternation depth are
a separate semantic analysis and are out of scope here.

## Safety

This crate contains no unsafe code.

## Minimum Supported Rust Version

We do not maintain an official minimum supported rust version (MSRV), and it may be upgraded at any time when necessary.

## License

All MERC crates are licensed under the `BSL-1.0` license. See the [LICENSE](https://raw.githubusercontent.com/MERCorg/merc/refs/heads/main/LICENSE) file in the repository root for more information.
