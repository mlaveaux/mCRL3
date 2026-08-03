# Overview

STARK is a specification language for *robustness analysis* of stochastic,
discrete-time systems: a specification describes a system as a set of state
variables driven by component controllers and an environment, and then asks how
much the system's behaviour changes when that environment is perturbed.

This crate is a Rust port of the original Java STARK tool. It contains the
whole front end — parser, name resolution, type checker and a lowering pass to
an evaluation IR — together with an evaluator that both simulates a
specification and verifies its robustness properties.

## Usage

A specification travels through a fixed pipeline, one type per stage, so a
stage can never be skipped by accident:

```text
&str -> UntypedStarkSpecification -> StarkSpecification -> IrProgram -> [ evaluate ]
        parse                        from_untyped          from_spec
```

[`UntypedStarkSpecification::parse`] yields a faithful syntax tree whose
references are unresolved and whose expressions have no types yet.
[`StarkSpecification::from_untyped`] runs name resolution followed by type
checking, and either reports *every* problem at once through a `Diagnostics` or
produces a [`StarkSpecification`]. Only that constructor can produce the type,
so anything holding one knows resolution and type checking already succeeded
and never has to re-derive or re-validate it. [`IrProgram::from_spec`] then
flattens it into an [`IrProgram`], the arena the evaluator walks.

```rust
use merc_stark::UntypedStarkSpecification;
use merc_stark::StarkSpecification;
use merc_stark::IrProgram;
use merc_stark::eval::RecordingObserver;
use merc_stark::eval::Simulation;

let source = r#"
    variables {
        real x range [0, 100] = 50;
        real y range [0, 100] = 50;
    }

    environment {
        x' = x + U[-1,0,1];
        y' = y + U[-1,0,1];
    }
"#;

let untyped = UntypedStarkSpecification::parse(source).expect("should parse");
let specification = StarkSpecification::from_untyped(untyped)
    .unwrap_or_else(|diagnostics| panic!("{}", diagnostics.render(source)));
let program = IrProgram::from_spec(&specification)
    .unwrap_or_else(|diagnostics| panic!("{}", diagnostics.render(source)));

// Run one trajectory of twenty macro-steps, recording every state.
let mut simulation = Simulation::new(&program, 42).expect("should initialise");
let mut observer = RecordingObserver::default();
simulation.run(20, &mut observer).expect("should run");

assert_eq!(observer.trajectory.len(), 20);
```

There are two entry points into the evaluator, one per thing you can ask of a
specification:

- [`eval::Simulation`] — *run* it. One trajectory, stepped on demand, with
  states pushed to an [`eval::Observer`].
- [`eval::Analysis`] — *verify* it. Checks the specification's `formula` and
  `distance` declarations by comparing an ensemble of trajectories against a
  perturbed copy of itself, yielding a [`eval::TruthValue`] (or a raw
  distance).

Both are seeded explicitly, so a whole run or analysis is reproducible from its
seed. The random stream is deliberately **not** bit-compatible with the
original Java tool's; only the distributions match.

Every entry point is fallible: evaluation returns `Result<_, EvalError>` rather
than propagating an absorbing error *value* the way the original does — see the
[`value`] module for why.

## Crate layout

The front end is a sequence of passes, each in its own module. Those modules
are private and their contents are re-exported flat from the crate root, but
each carries the design rationale for its pass in its module documentation —
build the documentation with `--document-private-items` to read it.

| Module           | Pass                                                                      |
| ---------------- | ------------------------------------------------------------------------- |
| `parse`          | `pest` grammar entry point (`stark_grammar.pest`).                        |
| `consume`        | Turns the `pest` parse tree into the AST.                                 |
| `precedence`     | Pratt parsers for the expression and robustness sub-languages.            |
| `ast`            | The syntax tree the two above produce.                                    |
| `resolve`        | Name resolution: assigns every declaration a stable id.                   |
| `typecheck`      | Type inference over the resolved tree.                                    |
| `types`          | The STARK type lattice.                                                   |
| `diagnostics`    | What resolution and type checking can complain about.                     |
| `specification`  | The `check` entry point and the checked-specification type.               |
| `lower`          | Lowers a checked specification to the evaluation IR.                      |

Three modules are public rather than flattened into the crate root:

- [`ir`] — the evaluation IR. Kept separate because [`ir::BinaryOp`]
  deliberately collides in name (not in meaning) with `ast::BinaryOp`;
  flattening both would be an ambiguous glob re-export.
- [`value`] — runtime values and evaluation errors, for the same reason.
- [`eval`] — the evaluator, whose own submodules are private.

## Related work

This crate is a port of the Java [STARK
tool](https://github.com/the-stark-tool/STARK), which is also where the
specifications in `examples/stark` come from. Where this port deviates from the
reference semantics, the module documentation of the pass in question says so
and why.

## Minimum Supported Rust Version

We do not maintain an official minimum supported rust version (MSRV), and it
may be upgraded at any time when necessary.

## License

All MERC crates are licensed under the `BSL-1.0` license. See the
[LICENSE](https://raw.githubusercontent.com/MERCorg/merc/refs/heads/main/LICENSE)
file in the repository root for more information.
