This repository contains several tools and libraries that interface with the C++ library functionality of the [mCRL2 toolset](https://github.com/mCRL2org/mCRL2) demonstrating the ability to write tools in Rust that interface with mCRL2. See the [documentation](https://mlaveaux.github.io/mCRL2-rust/mcrl2/index.html) of the `mcrl2` crate for the Rust wrapper around the `mcrl2-sys` crate, which provides the C++ interface. This documentation can be locally produced using `cargo doc --open`. Compilation requires at least rustc version 1.70.0 and we use 2021 edition rust.

## Tools

The tools directory contains prototypes to show the viability of this approach. The `mcrl2rewrite` tool can for example be executed using `cargo run --release --bin mcrl2rewrite`. The libraries use the `RUST_LOG` environment variable to set the logging level, which can be set to `trace`, `info`, `debug`, `warn` and `error`. It can even be used to only show specific logging output, for example `RUST_LOG=mcrl2::aterm=trace`.

# Contributing

This is mostly a proof of concept demonstrating the capabilities of Rust and figuring out best practices, but contributions are welcome.
First the submodules must initialised to obtain the 3rd-party libraries. Furthermore, we need a C++ compiler to build the mCRL2 toolset. This can be Visual Studio on Windows, AppleClang on MacOS or either GCC or Clang on Linux. In the latter case it uses whatever compiler is provided by the CXX environment variable. After that the cargo workspace can be build. This will also build the necessary components of the mCRL2 toolset, which can take some time.

```bash
git submodule update --init --recursive
cargo build
```

By default this will build in `dev` or debug mode, and a release build can be obtained by passing `--release`. Note that it is necessary to run `git submodule update` after switching branches or pulling from the remote whenever any of the modules have been changed.

## Tests

Tests can be performed using `cargo test`, only tests of the Sabre crate can be executed with `cargo test -p sabre --lib` and `cargo test -- --no-capture` can be used to show the output of tests. Alternatively, an improved test runner called [nextest](https://nexte.st/) can be used with `cargo nextest run`. This can be installed using `cargo install cargo-nextest`. This test runner offers many improvements such as always showing output of failing tests, running more tests in parallel, and offer better error messages for segfaults. Some tests that are ignored by default require a larger stack size, which can be set using the environment variable `RUST_MIN_STACK`.

## LLVM Sanitizer

For Linux targets it is  possible to run the [LLVM address sanitizer](https://clang.llvm.org/docs/AddressSanitizer.html) to detect memory issues in unsafe and C++ code. This requires the nightly version of the rust compiler, which can acquired using `rustup toolchain install nightly` and the rust-src for the standard library, to be installed with `rustup component add rust-src --toolchain nightly`. To show the symbols for the resulting stacktrace it is also convenient to install `llvm-symbolizer`, for example using `sudo apt install llvm` on Ubuntu. Afterwards, the tests can be executed with the address sanitizer enabled using `cargo +nightly xtask address-sanitizer`. Similarly, we also provide a task for the thread sanitizer to detect data races, which can be executed by `cargo +nightly xtask thread-sanitizer`.
All `xtask` targets use `cargo nextest run`, so that must be installed prior. 

# Additional checks

To check for additional undefined behaviour at runtime we can also employ the `cargo careful` [project](https://github.com/RalfJung/cargo-careful). It compiles the standard library in nightly with many additional checks for undefined behaviour, because we cannot use [Miri](https://github.com/rust-lang/miri) due to the FFI calls. It can also be installed with `cargo install cargo-careful` and requires the nightly toolchain. Then it can be executed with `cargo +nightly careful nextest run --target=x86_64-unknown-linux-gnu` (or `test` when `nextest` has not been installed).

## Benchmarks

Micro-benchmarks can be executed using `cargo bench`. Additionally, we can also install `cargo-criterion` and run `cargo criterion` instead to keep track of more information such as changes over time. There is a benchmark task that can be executed with `cargo xtask benchmark` that runs multiple longer running benchmarks.

## Code Generation

There are a few procedural macros used to replace the code generation performed in the mCRL2 toolset. Working on procedural macros is typically difficult, but there are unit and integration tests to showcase common patterns. Alternatively, install `cargo install cargo-expand` and run the command `cargo expand` in for example `libraries/mcrl2-macros` to print the Rust code with the macros expanded.

## Code Coverage

Code coverage can be automatically generated for the full workspace using a cargo task. The code coverage itself is generated by LLVM with source code instrumentation, which requires the preview tools to be installed `rustup component add llvm-tools-preview`, and [grcov](https://github.com/mozilla/grcov), which can be acquired using `cargo install grcov`. To execute the code coverage task use `cargo xtask coverage nextest run`. The resulting HTML code coverage report can be found in `target/coverage` or online at the following [page](https://mlaveaux.github.io/mCRL2-rust/coverage/index.html). 

## Profiling

The `lpsreach` tool can be build using the `bench` compilation profile using `cargo build --profile bench` after which the resulting executable `target/release/lpsreach` can be profiled using any standard executable profiler, such as `Intel VTune` or `perf`. This compilation profile contains debugging information to show where time is being spent, but the code is optimised the same as in a release configuration.

Another useful technique for profiling is to generate a so-called `flamegraph`, which essentially takes the output of `perf` and produces a callgraph of time spent over time. These can be generated using the [flamegraph-rs](https://github.com/flamegraph-rs/flamegraph) tool, which can be acquired using `cargo install flamegraph`. Note that it relies on either `perf` or `dtrace` and as such is only supported on Linux and MacOS.

Finally, in performance critical situations it can be useful to view the generated assembly, which can be achieved with the `cargo asm --rust --simplify -p <package> [--lib] <path-to-function>` that can be obtained by `cargo install cargo-show-asm`.

## Formatting

All source code should be formatted using `rustfmt`, which can installed using `rustup component add rustfmt`. Individual source files can then be formatted using `rustfmt <filename>`.

# Related Work

This library is fully inspired by the work on [mCRL2](https://github.com/mCRL2org/mCRL2).