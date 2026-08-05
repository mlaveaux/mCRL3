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
