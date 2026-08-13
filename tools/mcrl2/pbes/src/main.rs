use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use clap::Subcommand;
use log::debug;
use log::info;
use log::warn;

use mcrl2::Pbes;
use mcrl2::set_reporting_level;
use mcrl2::verbosity_to_log_level;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_tools::report_error;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;
use merc_utilities::Timing;

use merc_pbes::Bsgs;
use merc_pbes::GapConfig;
use merc_pbes::ParameterLayoutLPS;
use merc_pbes::PbesLps;
use merc_pbes::PbesSrfLps;
use merc_pbes::PbesVertex;
use merc_pbes::Permutation;
use merc_pbes::QuotientLps;
use merc_pbes::SymmetryAlgorithm;
use merc_pbes::check_parameter_basis;
use merc_pbes::explore_pbes;
use merc_pbes::explore_pbes_impl;
use merc_pbes::explore_pbes_parallel;
use merc_pbes::explore_pbes_parallel_impl;
use merc_pbes::explore_pbes_symbolic;
use merc_pbes::explore_srf_pbes;
use merc_pbes::explore_srf_pbes_parallel;
use merc_pbes::graph_symmetries;
use merc_pbes::symmetry_parameter_basis;
use merc_pbes::write_dot;

use merc_explore::CacheLPS;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_explore::Summand;
use merc_vpg::PG;
use merc_vpg::Player;
use merc_vpg::Solver;
use merc_vpg::solve_priority_promotion;
use merc_vpg::solve_zielonka;
use merc_vpg::verify_solution;
use merc_vpg::write_pg;

/// Default number of nodes for the Oxidd LDD manager.
const DEFAULT_OXIDD_NODE_CAPACITY: usize = 1 << 24;

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum PbesFormat {
    Text,
    Pbes,
}

/// A command line tool for parameterised boolean equation systems (PBESs)
#[derive(clap::Parser, Debug)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(flatten)]
    version: VersionFlag,

    #[command(flatten)]
    verbosity: VerbosityFlag,

    #[arg(long, global = true)]
    timings: bool,

    /// Skip the preprocessing that mCRL2 applies to a PBES before instantiating
    /// it (instantiate global variables, simplify, one point rule, order
    /// quantified variables). `pbessolve` always applies it, so leaving it on is
    /// what makes the two tools comparable.
    #[arg(long, global = true, default_value_t = false)]
    no_preprocess: bool,

    /// The number of worker threads for the Oxidd LDD manager.
    #[arg(long, global = true, default_value_t = 1)]
    oxidd_workers: u32,

    /// The number of nodes for the Oxidd LDD manager.
    #[arg(long, global = true, default_value_t = DEFAULT_OXIDD_NODE_CAPACITY)]
    oxidd_node_capacity: usize,

    /// The apply cache capacity for the Oxidd LDD manager, defaults to the node capacity.
    #[arg(long, global = true)]
    oxidd_cache_capacity: Option<usize>,

    #[command(subcommand)]
    commands: Option<Commands>,
}

/// Initializes the Oxidd LDD manager based on CLI arguments.
fn init_ldd_manager(cli: &Cli) -> oxidd::ldd::LDDManagerRef {
    oxidd::ldd::new_manager(
        cli.oxidd_node_capacity,
        cli.oxidd_cache_capacity.unwrap_or(cli.oxidd_node_capacity),
        cli.oxidd_workers,
    )
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Analyze symmetries of a PBES
    Symmetry(SymmetryArgs),
    /// Compute symmetries of a PBES via the Symmetry Detection Graph and GAP.
    GraphSymmetry(GraphSymmetryArgs),
    /// Explore a PBES explicitly into a parity game.
    ExploreExplicit(ExploreExplicitArgs),
    /// Explore a PBES symbolically using LDD-based reachability.
    ExploreSymbolic(InputArgs),
    /// Solve a PBES by exploring it into a parity game and solving the game.
    Solve(SolveArgs),
}

/// The PBES to read, shared by every subcommand.
#[derive(clap::Args, Debug)]
struct InputArgs {
    /// The input PBES file.
    filename: String,

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,
}

/// How to turn a PBES into a parity game, shared by every subcommand that
/// explores one. `solve` is `explore-explicit` followed by solving the resulting
/// game, so both accept exactly these flags.
#[derive(clap::Args, Debug)]
struct ExploreArgs {
    /// Strategy to explore the state space of the PBES. Only used for sequential
    /// exploration; ignored when `--threads > 1` (the parallel explorer always
    /// uses a level-synchronised BFS).
    #[arg(long, value_enum, default_value_t = ExplorationStrategy::Bfs)]
    strategy: ExplorationStrategy,

    /// Caching strategy to use during exploration.
    #[arg(long, value_enum, default_value_t = CachingStrategy::None)]
    caching: CachingStrategy,

    /// Number of worker threads used for exploration.
    #[arg(long, default_value_t = 1)]
    threads: usize,

    /// Pin each worker thread round-robin to the available CPU cores.
    #[arg(long, default_value_t = false)]
    pinned: bool,

    /// Apply symmetry reduction: compute graph automorphisms, build a BSGS, and
    /// canonicalize every next-state to its orbit representative before adding
    /// it to the state space.
    #[arg(long, default_value_t = false)]
    symmetry: bool,

    /// Supply generators directly in mapping '[0->1,...]' or cycle '(0 1)'
    /// notation to build the BSGS without running GAP symmetry detection.
    /// Repeat the flag for multiple generators: `--quotient '(0 1)' --quotient '(2 3)'`.
    #[arg(long, value_name = "PERM")]
    quotient: Vec<String>,

    /// Path or name of the GAP executable used to compute the BSGS (only
    /// relevant when `--symmetry` or `--quotient` is set).
    #[arg(long, default_value = "gap")]
    gap_path: String,

    /// Convert to SRF before exploring (legacy; default is the direct structure-graph algorithm).
    #[arg(long, default_value_t = false)]
    srf: bool,

    /// Write the resulting parity game to this file in the PGSolver `.pg` format.
    #[arg(long, short('o'), value_name = "FILE")]
    output: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct SymmetryArgs {
    #[command(flatten)]
    input: InputArgs,

    /// Pass a single permutation in mapping notation '[0->1,1->0,...]' or cycles notation '(0 1)' to check whether it is a symmetry.
    #[arg(long)]
    permutation: Option<String>,

    /// Search for all symmetries instead of only the first one.    
    #[arg(long, default_value_t = false)]
    all_symmetries: bool,

    /// Partition data parameters into their sorts before considering their permutation groups.
    #[arg(long, default_value_t = false)]
    partition_data_sorts: bool,

    /// Partition data parameters based on their updates.
    #[arg(long, default_value_t = false)]
    partition_data_updates: bool,

    /// Print the symmetry in the mapping notation instead of the cycle notation.
    #[arg(long, default_value_t = false)]
    mapping_notation: bool,

    /// Print the SRF representation of the PBES.
    #[arg(long, default_value_t = false)]
    print_srf: bool,
}

#[derive(clap::Args, Debug)]
struct GraphSymmetryArgs {
    #[command(flatten)]
    input: InputArgs,

    /// Path or name of the GAP executable.
    #[arg(long, default_value = "gap")]
    gap_path: String,

    /// Write the generated GAP script to this file (for debugging).
    #[arg(long)]
    dump_gap_script: Option<String>,

    /// Write the SDG as a Graphviz DOT file to this path.
    #[arg(long)]
    dot: Option<String>,

    /// Print symmetries in mapping notation instead of cycle notation.
    #[arg(long, default_value_t = false)]
    mapping_notation: bool,
}

#[derive(clap::Args, Debug)]
struct ExploreExplicitArgs {
    #[command(flatten)]
    input: InputArgs,

    #[command(flatten)]
    explore: ExploreArgs,
}

#[derive(clap::Args, Debug)]
struct SolveArgs {
    #[command(flatten)]
    input: InputArgs,

    #[command(flatten)]
    explore: ExploreArgs,

    /// Sets the algorithm used to solve the resulting parity game.
    #[arg(long, value_enum, default_value_t = Solver::Zielonka)]
    solver: Solver,

    /// Whether to verify the solution after computing it.
    #[arg(long, default_value_t = false)]
    verify_solution: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .parse_default_env()
        .init();

    // Enable logging on the mCRL2 side
    set_reporting_level(verbosity_to_log_level(cli.verbosity.verbosity()));

    if cli.version.into() {
        eprintln!("{}", Version);
        return ExitCode::SUCCESS;
    }

    let timing = Timing::new();
    let result = handle_command(&cli, &timing);

    if cli.timings {
        timing.print();
    }

    print_allocator_metrics();
    report_error(result)
}

fn handle_command(cli: &Cli, timing: &Timing) -> Result<(), MercError> {
    let preprocess = !cli.no_preprocess;

    if let Some(command) = &cli.commands {
        match command {
            Commands::Symmetry(args) => handle_symmetry(args, timing, preprocess)?,
            Commands::GraphSymmetry(args) => handle_graph_symmetry(args, timing, preprocess)?,
            Commands::ExploreExplicit(args) => handle_explore_explicit(args, timing, preprocess)?,
            Commands::ExploreSymbolic(args) => handle_explore_symbolic(cli, args, timing, preprocess)?,
            Commands::Solve(args) => handle_solve(args, timing, preprocess)?,
        }
    }

    Ok(())
}

impl InputArgs {
    /// Reads the PBES in the explicitly chosen format, or the binary PBES format
    /// when no format is given.
    ///
    /// Unless `preprocess` is false, the PBES is put through the same preprocessing
    /// that mCRL2 applies before instantiating one. Doing it here rather than inside
    /// a single explorer keeps every consumer of this PBES — the explorers, the
    /// symmetry detection and the parameter basis the generators index into —
    /// looking at the same equations.
    fn read(&self, timing: &Timing, preprocess: bool) -> Result<Pbes, MercError> {
        let mut pbes = timing.measure("load", || match self.format.unwrap_or(PbesFormat::Pbes) {
            PbesFormat::Pbes => Pbes::from_file(&self.filename),
            PbesFormat::Text => Pbes::from_text_file(&self.filename),
        })?;

        if preprocess {
            pbes.preprocess(timing)?;
        } else {
            info!("Skipping PBES preprocessing (--no-preprocess)");
        }

        Ok(pbes)
    }
}

impl ExploreArgs {
    /// Explores `pbes` into a parity game, applying symmetry reduction when
    /// generators are supplied or detected, and writes the game to `--output`.
    fn explore(&self, pbes: Pbes, timing: &Timing) -> Result<merc_vpg::ParityGame, MercError> {
        // Explicit generators take precedence over detection: giving both means
        // the user already knows the group and only detection would be redundant.
        let bsgs = if !self.quotient.is_empty() {
            Some(build_bsgs_from_user_generators(
                &pbes,
                &self.quotient,
                &self.gap_path,
                timing,
            )?)
        } else if self.symmetry {
            Some(build_bsgs_for_pbes(&pbes, &self.gap_path, timing)?)
        } else {
            None
        };

        let game = if let Some(bsgs) = bsgs {
            explore_with_symmetry(&pbes, self, bsgs, timing)?
        } else if self.threads > 1 && self.srf {
            explore_srf_pbes_parallel(&pbes, self.threads, self.caching, self.pinned, timing)?
        } else if self.threads > 1 {
            explore_pbes_parallel(pbes, self.threads, self.caching, self.pinned, timing)?
        } else if self.srf {
            explore_srf_pbes(&pbes, self.strategy, self.caching, timing)?
        } else {
            explore_pbes(pbes, self.strategy, self.caching, timing)?
        };

        info!(
            "Parity game: {} vertices, {} edges",
            game.num_of_vertices(),
            game.num_of_edges()
        );

        if let Some(output) = &self.output {
            let mut output_file = File::create(output)?;
            write_pg(&mut output_file, &game)?;
            info!("Parity game written to '{}'", output.display());
        }

        Ok(game)
    }
}

fn handle_explore_explicit(args: &ExploreExplicitArgs, timing: &Timing, preprocess: bool) -> Result<(), MercError> {
    let game = args.explore.explore(args.input.read(timing, preprocess)?, timing)?;

    println!(
        "Parity game: {} vertices, {} edges",
        game.num_of_vertices(),
        game.num_of_edges()
    );

    Ok(())
}

/// Handles symbolic exploration of a PBES, reporting the number of reachable
/// BES equations (states).
fn handle_explore_symbolic(cli: &Cli, args: &InputArgs, timing: &Timing, preprocess: bool) -> Result<(), MercError> {
    let pbes = args.read(timing, preprocess)?;
    let storage = init_ldd_manager(cli);
    let states = explore_pbes_symbolic(&storage, &pbes, timing)?;
    println!("Number of states: {}", states.len());
    Ok(())
}

/// Parse permutation strings in mapping `[0->1,...]` or cycle `(0 1)` notation.
fn parse_generators(strs: &[String]) -> Result<Vec<Permutation>, MercError> {
    strs.iter()
        .map(|s| {
            let s = s.trim();
            if s.starts_with('[') {
                Permutation::from_mapping_notation(s)
            } else {
                Permutation::from_cycle_notation(s)
            }
        })
        .collect()
}

/// Build a BSGS from user-supplied generator strings without running graph-symmetry detection.
fn build_bsgs_from_user_generators(
    pbes: &Pbes,
    strs: &[String],
    gap_path: &str,
    timing: &Timing,
) -> Result<Arc<Bsgs>, MercError> {
    let config = GapConfig {
        executable: gap_path.to_string(),
        dump_script: None,
    };
    let generators = parse_generators(strs)?;
    let n = symmetry_parameter_basis(pbes)?.len();

    // Reject out-of-range points here: converting to a dense permutation would
    // silently truncate them to `0..n`, producing a mapping that is no longer a
    // permutation and panics when inverted.
    for (generator, text) in generators.iter().zip(strs) {
        if let Some(max_point) = generator.max_point()
            && max_point >= n
        {
            return Err(MercError::from(format!(
                "generator '{}' mentions parameter {max_point}, but the PBES has {n} parameter(s) (0..{})",
                text.trim(),
                n.saturating_sub(1)
            )));
        }
    }

    let bsgs = Arc::new(timing.measure("symmetry: BSGS", || Bsgs::from_generators(&generators, n, &config))?);
    info!(
        "User-supplied generators: |G| = {} ({} generator(s))",
        bsgs.order(),
        generators.len()
    );
    Ok(bsgs)
}

/// Compute graph symmetries for `pbes` and build a BSGS from them.
fn build_bsgs_for_pbes(pbes: &Pbes, gap_path: &str, timing: &Timing) -> Result<Arc<Bsgs>, MercError> {
    let config = GapConfig {
        executable: gap_path.to_string(),
        dump_script: None,
    };
    let sym_result = timing.measure("symmetry: detection", || graph_symmetries(pbes, &config))?;
    let n = symmetry_parameter_basis(pbes)?.len();
    let bsgs = Arc::new(timing.measure("symmetry: BSGS", || {
        Bsgs::from_generators(&sym_result.generators, n, &config)
    })?);
    info!("|G| = {} ({} generator(s))", bsgs.order(), sym_result.generators.len());

    // The two orders are computed by entirely separate routes — GAP's
    // `Size(Stabilizer(...))` on the detection graph versus the product of the
    // transversal sizes of the stabilizer chain built from the rendered
    // generators — so a disagreement means a generator was rendered, parsed or
    // truncated wrongly somewhere in between. The quotient stays sound either
    // way (canonicalization only ever uses the chain), so warn rather than fail.
    if bsgs.order() != sym_result.symmetry_group_order {
        warn!(
            "symmetry group order mismatch: graph symmetry detection reports |Sym(pbes)| = {}, \
             but the BSGS built from its generators has order {}; the quotient will reduce by \
             the smaller group",
            sym_result.symmetry_group_order,
            bsgs.order()
        );
    }
    Ok(bsgs)
}

/// Explore `pbes` into a parity game, canonicalizing every next-state via `bsgs`.
fn explore_with_symmetry(
    pbes: &Pbes,
    args: &ExploreArgs,
    bsgs: Arc<Bsgs>,
    timing: &Timing,
) -> Result<merc_vpg::ParityGame, MercError> {
    // Both explorers unify with the same flags as `symmetry_parameter_basis`, so
    // they are expected to agree with it; SRF normalisation is the one that could
    // still add or reorder parameters on the way, since it introduces equations
    // of its own. Checking both keeps the guarantee where it can be seen.
    let basis = symmetry_parameter_basis(pbes)?;

    if args.srf {
        let lps = PbesSrfLps::new(pbes)?;
        check_parameter_basis(&basis, &lps.parameters(), "SRF")?;
        quotient_explore(&lps, args, bsgs, timing)
    } else {
        let lps = PbesLps::new(pbes.clone())?;
        check_parameter_basis(&basis, &lps.parameters(), "structure-graph")?;
        quotient_explore(&lps, args, bsgs, timing)
    }
}

/// Explore `lps` into a parity game, canonicalizing every next-state via `bsgs`.
fn quotient_explore<P>(
    lps: &P,
    args: &ExploreArgs,
    bsgs: Arc<Bsgs>,
    timing: &Timing,
) -> Result<merc_vpg::ParityGame, MercError>
where
    P: ParameterLayoutLPS<Value = usize, Label = (), StateInfo = PbesVertex> + Sync,
    <P::Summand as Summand>::Context: Send,
{
    match args.caching {
        CachingStrategy::None => {
            let qlps = QuotientLps::new(lps, bsgs, 1);
            if args.threads > 1 {
                explore_pbes_parallel_impl(&qlps, args.threads, args.pinned, timing)
            } else {
                explore_pbes_impl(&qlps, args.strategy, timing)
            }
        }
        caching => {
            // The cache sits *inside* the quotient (see [`QuotientLps`]) so the
            // keys stay the narrow read-position projections of the raw states
            // instead of covering every parameter touched by canonicalization.
            let cached = CacheLPS::new(lps, caching);
            let qlps = QuotientLps::new(&cached, bsgs, 1);
            let game = if args.threads > 1 {
                explore_pbes_parallel_impl(&qlps, args.threads, args.pinned, timing)
            } else {
                explore_pbes_impl(&qlps, args.strategy, timing)
            }?;
            debug!("{}", cached.metrics());
            Ok(game)
        }
    }
}

/// Handles the solve command, which explores a PBES into a parity game and
/// solves the game, printing the solution of the initial vertex.
fn handle_solve(args: &SolveArgs, timing: &Timing, preprocess: bool) -> Result<(), MercError> {
    let game = args.explore.explore(args.input.read(timing, preprocess)?, timing)?;

    let (solution, strategy) = timing.measure("solve", || match args.solver {
        Solver::Zielonka => solve_zielonka(&game, args.verify_solution),
        Solver::PriorityPromotion => solve_priority_promotion(&game, args.verify_solution),
    });

    if let Some(strategy) = strategy
        && args.verify_solution
    {
        verify_solution(&game, &solution, &strategy);
    }

    let winner = if solution[Player::Even.to_index()][game.initial_vertex().value()] {
        Player::Even
    } else {
        Player::Odd
    };
    println!("{}", winner.solution());

    Ok(())
}

fn handle_graph_symmetry(args: &GraphSymmetryArgs, timing: &Timing, preprocess: bool) -> Result<(), MercError> {
    let pbes = args.input.read(timing, preprocess)?;

    let config = GapConfig {
        executable: args.gap_path.clone(),
        dump_script: args.dump_gap_script.as_deref().map(Path::new).map(|p| p.to_path_buf()),
    };

    let result = timing.measure("symmetry: detection", || graph_symmetries(&pbes, &config))?;

    if let Some(dot_path) = &args.dot {
        let mut f = File::create(dot_path)?;
        write_dot(&result.sdg, &mut f)?;
        log::info!("DOT file written to '{dot_path}'");
        if let Ok(dot_bin) = which::which("dot") {
            log::info!("Generating PDF using dot...");
            duct::cmd!(dot_bin, "-Tpdf", dot_path, "-O").run()?;
        }
    }

    for generator in &result.generators {
        if args.mapping_notation {
            println!("{:?}", generator);
        } else {
            println!("{}", generator);
        }
    }

    if result.generators.is_empty() {
        println!("No non-trivial symmetries found.");
    }

    Ok(())
}

fn handle_symmetry(args: &SymmetryArgs, timing: &Timing, preprocess: bool) -> Result<(), MercError> {
    let pbes = args.input.read(timing, preprocess)?;
    let algorithm = SymmetryAlgorithm::new(&pbes, args.print_srf)?;
    if let Some(permutation) = &args.permutation {
        let pi = if permutation.trim_start().starts_with("[") {
            Permutation::from_mapping_notation(permutation)?
        } else {
            Permutation::from_cycle_notation(permutation)?
        };

        if let Err(x) = algorithm.is_valid_permutation(&pi) {
            return Err(format!("The given permutation is not valid: {x}").into());
        }

        info!("Checking permutation: {}", pi);
        if algorithm.check_symmetry(&pi) {
            println!("true");
        } else {
            println!("false");
        }
    } else {
        for candidate in algorithm.candidates(args.partition_data_sorts, args.partition_data_updates) {
            debug!("Found candidate: {}", candidate);

            if candidate.is_identity() {
                // Skip the identity permutation
                continue;
            }

            if algorithm.check_symmetry(&candidate) {
                if args.mapping_notation {
                    info!("Found symmetry: {:?}", candidate);
                } else {
                    info!("Found symmetry: {}", candidate);
                }

                if !args.all_symmetries {
                    // Only search for the first symmetry
                    info!("Stopping search after first non-trivial symmetry.");
                    break;
                }
            }
        }
    }

    Ok(())
}
