
# Overview

This crate provides functionality for working with (variability) parity games.
This includes reading and writing for parity games in the
[PGSolver](https://github.com/tcsprojects/pgsolver) `.pg` format. For
variability parity games this format is extended with feature configurations
encoded as BDDs on the edges, with a corresponding `.vpg` format. These games
can be solved using Zielonka's recursive algorithm, displayed in
[Graphviz](https://graphviz.org/) `DOT` format and generated from modal
mu-calculus formulas.

A central `PG` or parity game trait is used to allow writing generic algorithms
for (variability) parity games. Various helpers are introduced for working with
`strong` types for priorities, explicitly representing the even and odd players
etc. This crate uses [OxiDD](https://oxidd.net/) for the binary decision
diagrams, which are used to represent feature configurations in variability
parity games, and for the state and edge sets of `SymbolicParityGame`, a
purely symbolic (LDD-based) game representation and solver.

## Usage

Reading a `.pg` from disk and subsequently solving it can simply be done as
follows.

```rust
use merc_vpg::read_pg;
use merc_vpg::solve_zielonka;

let parity_game = read_pg(b"parity 3;
0 0 0 1;
1 0 0 2;
2 1 0 2;
" as &[u8]).unwrap();

// Solve the game, produces a full solution for all vertices.
let (_solution, _strategy) = solve_zielonka(&parity_game, false);
```

## Changelog

### 3.0

Added a symbolic (LDD-based) parity game representation and solver, alongside
the existing explicit one: `SymbolicParityGame`, built from any
`merc_symbolic::TransitionGroup`, and `solve_symbolic_zielonka`, a port of
mCRL2's `symbolic_pbessolve_algorithm` used by `merc-pbes`'s `solve-symbolic`
command. Includes:

- Winning-strategy computation (`compute_strategy` on the game), and
  `check_strategy`, a native LDD-level certificate checker (mCRL2's
  `--check-strategy`) that restricts the game to a player's own strategy via
  `SymbolicParityGame::apply_strategy` and re-solves, without decoding to an
  explicit game — the same way the rest of symbolic solving scales.
  `verify_symbolic_solution` offers a second, decode-to-explicit
  cross-check for testing, reusing the explicit `verify_solution` checker.
- Partial-solving accelerators ported from `symbolic_pbessolve.h`
  (`partial_solve`, `detect_solitair_cycles`, `detect_forced_cycles`,
  `detect_fatal_attractors`, each with a `_within_safe_vertices` counterpart),
  which take an "incomplete vertices" set the way mCRL2's do — sound today
  (exercised with an empty incomplete set, since merc has no
  partial-exploration front end yet to produce a real one), and ready for one
  once it exists. Ported from:

  > Maurice Laveaux, Wieger Wesselink, Tim A.C. Willemse, *On-The-Fly Solving
  > for Symbolic Parity Games*, TACAS 2022, LNCS 13244, pp. 137-155.
  > https://doi.org/10.1007/978-3-030-99527-0_8
- `convert_symbolic_parity_game`/`encode_parity_game` as a symbolic/explicit
  round-trip oracle, cross-checking every solver above against the explicit
  `solve_zielonka` on random games.

See `docs/symbolic-parity-game-plan.md` in the repository root for the design
rationale, in particular the priority-direction inversion needed because merc
uses max-parity while mCRL2's symbolic solver uses min-parity.

### 2.0

Added strategy computations to the regular Zielonka solver, which can also be
checked for correctness using the `verify_solution` function.

Extended the translation of modal equation systems to deal with regular formulas
and various edge cases.

Added translation for modal mu-calculus formulas and labelled transition systems
to regular parity games, was only implemented for feature transition systems and
variability parity games.

Added the `clap` feature to conditionally enable the `clap` dependency to derive
some convenience traits.

Optimised the implementation to avoid unnecessary `with_manager_shared` calls
when operating on BDDs. This is important since `oxidd` is otherwise not
efficient when doing many individual operations.

## Authors

The implementation of this crate was developed by Sjef van Loo and Maurice
Laveaux. The theoretical foundations were laid by Maurice Ter Beek, Erik de Vink
and Tim A.C. Willemse, in the following publication:

  > Maurice Ter Beek, Maurice Laveaux, Sjef van Loo, Erik de Vink and Tim A.C. Willemse. "Family-Based Model Checking Using Variability Parity Games". XXX.

## Safety

This crate contains no unsafe code.

## Minimum Supported Rust Version

We do not maintain an official minimum supported rust version (MSRV), and it may be upgraded at any time when necessary.

## License

All MERC crates are licensed under the `BSL-1.0` license. See the [LICENSE](https://raw.githubusercontent.com/MERCorg/merc/refs/heads/main/LICENSE) file in the repository root for more information.