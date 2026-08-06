# Overview

This crate provides a command-line tool for working with Parameterised Boolean
Equation Systems (PBESs) from the mCRL2 toolset. It supports explicit and
symbolic exploration into parity games, symmetry detection, and solving.

The `graph-symmetry` subcommand constructs the Symmetry Detection Graph (SDG)
of the PBES and calls an external [GAP](https://www.gap-system.org/) process to
compute the automorphism group. Both GAP itself and its **Digraphs** package
must be installed.

Use `--dot <file.dot>` to write the SDG as a Graphviz DOT file for
visualization. If the `dot` binary (part of [Graphviz](https://graphviz.org/))
is on `$PATH`, a PDF is generated automatically alongside it. To convert
manually:

```sh
dot -Tpdf file.dot -o file.pdf
dot -Tsvg file.dot -o file.svg
```

### DOT vertex legend

| Shape | Fill colour | Label | Meaning |
|---|---|---|---|
| Rectangle | Blue | parameter name | PBES parameter |
| Tiny diamond (unlabelled) | Grey | — | Update vertex `X_{i,k}` |
| Hexagon | Light blue | `X` | Propositional variable instantiation of `X` |
| Parallelogram | Purple | `forall x:D,…` | Quantifier |
| Ellipse | Yellow | function name | Data function application |
| Ellipse | Orange | number | Machine-number constant |
| Ellipse | Red | `&&` / `\|\|` / `!` / `=>` | Boolean connective |
| Ellipse | Green | `x` | Bound (quantifier-scoped) variable |

Edges between formula nodes are solid; edges are unlabelled for commutative and
flat operators (e.g. `&&`, `||`) and carry a 1-based position label for
non-commutative applications. Update edges are dashed and drawn with zero spring
weight so they do not distort the formula-tree layout.

## Installing GAP

Download and install GAP 4 from <https://www.gap-system.org/>. The `gap`
binary must be on `$PATH`, or its location passed via `--gap-path`.

Verified against **GAP 4.12.1**.

## Installing the Digraphs package

The Digraphs package is not bundled with all GAP distributions. Install it from
<https://digraphs.github.io/Digraphs/> or, if your GAP installation includes
the package manager, run inside a GAP session:

```gap
InstallPackage("digraphs");
```

To verify that the package loads correctly:

```gap
LoadPackage("digraphs");
```

If this returns `fail`, `graph-symmetry` will report an error pointing to the
Digraphs website.

## Safety

This crate contains no `unsafe` code.

## Minimum Supported Rust Version

The minimum supported Rust version is **1.91.0**.

## License

All MERC crates are licensed under the `BSL-1.0` license. See the
[LICENSE](https://raw.githubusercontent.com/MERCorg/merc/refs/heads/main/LICENSE)
file in the repository root for more information.
