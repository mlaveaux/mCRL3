use std::fs::File;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use clap::Subcommand;
use log::debug;
use log::info;

use mcrl2::Pbes;
use mcrl2::set_reporting_level;
use mcrl2::verbosity_to_log_level;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_tools::report_error;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::bsgs::Bsgs;
use crate::explore_common::explore_pbes_impl;
use crate::explore_common::explore_pbes_parallel_impl;
use crate::explore_pbes::PbesLps;
use crate::explore_pbes::explore_pbes;
use crate::explore_pbes::explore_pbes_parallel;
use crate::explore_srf::explore_srf_pbes;
use crate::explore_srf::explore_srf_pbes_parallel;
use crate::explore_symbolic_srf::explore_pbes_symbolic;
use crate::graph_symmetry::GapConfig;
use crate::graph_symmetry::graph_symmetries;
use crate::graph_symmetry::write_dot;
use crate::permutation::Permutation;
use crate::quotient_lps::QuotientLps;
use crate::symmetry::SymmetryAlgorithm;

use merc_explore::CacheLPS;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_vpg::PG;
use merc_vpg::Player;
use merc_vpg::Solver;
use merc_vpg::solve_priority_promotion;
use merc_vpg::solve_zielonka;
use merc_vpg::verify_solution;

mod bsgs;
mod clone_iterator;
mod explore_common;
mod explore_pbes;
mod explore_pbes_test;
mod explore_srf;
mod explore_srf_test;
mod explore_symbolic_srf;
mod explore_symbolic_srf_test;
mod graph_symmetry;
mod permutation;
mod quotient_lps;
mod symmetry;

/// Default number of nodes for the Oxidd LDD manager.
const DEFAULT_OXIDD_NODE_CAPACITY: usize = 1 << 24;

#[derive(clap::ValueEnum, Clone, Debug)]
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
    ExploreSymbolic(ExploreSymbolicArgs),
    /// Solve a PBES by exploring it into a parity game and solving the game.
    Solve(SolveArgs),
}

#[derive(clap::Args, Debug)]
struct SymmetryArgs {
    /// The input PBES file.
    filename: String,

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,

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
    /// The input PBES file.
    filename: String,

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,

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
    /// The input PBES file.
    filename: String,

    /// Strategy to explore the state space of the PBES. Only used for sequential
    /// exploration; ignored when `--threads > 1` (the parallel explorer always
    /// uses a level-synchronised BFS).
    #[arg(long, value_enum, default_value_t = ExplorationStrategy::Bfs)]
    strategy: ExplorationStrategy,

    /// Caching strategy to use during exploration.
    #[arg(long, value_enum, default_value_t = CachingStrategy::None)]
    caching: CachingStrategy,

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,

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
}

#[derive(clap::Args, Debug)]
struct ExploreSymbolicArgs {
    /// The input PBES file.
    filename: String,

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,
}

#[derive(clap::Args, Debug)]
struct SolveArgs {
    /// The input PBES file.
    filename: String,

    /// Strategy to explore the state space of the PBES. Only used for sequential
    /// exploration; ignored when `--threads > 1` (the parallel explorer always
    /// uses a level-synchronised BFS).
    #[arg(long, value_enum, default_value_t = ExplorationStrategy::Bfs)]
    strategy: ExplorationStrategy,

    /// Caching strategy to use during exploration.
    #[arg(long, value_enum, default_value_t = CachingStrategy::None)]
    caching: CachingStrategy,

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,

    /// Sets the algorithm used to solve the resulting parity game.
    #[arg(long, value_enum, default_value_t = Solver::Zielonka)]
    solver: Solver,

    /// Whether to verify the solution after computing it.
    #[arg(long, default_value_t = false)]
    verify_solution: bool,

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

    report_error(result)
}

fn handle_command(cli: &Cli, timing: &Timing) -> Result<(), MercError> {
    if let Some(command) = &cli.commands {
        match command {
            Commands::Symmetry(args) => handle_symmetry(args)?,
            Commands::GraphSymmetry(args) => handle_graph_symmetry(args)?,
            Commands::ExploreExplicit(args) => handle_explore_explicit(args)?,
            Commands::ExploreSymbolic(args) => handle_explore_symbolic(cli, args, timing)?,
            Commands::Solve(args) => handle_solve(args)?,
        }
    }

    Ok(())
}

/// Reads a PBES from the given file in the explicitly chosen format, or the
/// binary PBES format when no format is given.
fn read_pbes(filename: &str, format: Option<PbesFormat>) -> Result<Pbes, MercError> {
    match format.unwrap_or(PbesFormat::Pbes) {
        PbesFormat::Pbes => Ok(Pbes::from_file(filename)?),
        PbesFormat::Text => Ok(Pbes::from_text_file(filename)?),
    }
}

fn handle_explore_explicit(args: &ExploreExplicitArgs) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format.clone())?;
    let game = if !args.quotient.is_empty() {
        let bsgs = build_bsgs_from_user_generators(&pbes, &args.quotient, &args.gap_path)?;
        explore_with_symmetry(&pbes, args.strategy, args.caching, args.threads, args.pinned, bsgs)?
    } else if args.symmetry {
        let bsgs = build_bsgs_for_pbes(&pbes, &args.gap_path)?;
        explore_with_symmetry(&pbes, args.strategy, args.caching, args.threads, args.pinned, bsgs)?
    } else if args.threads > 1 && args.srf {
        explore_srf_pbes_parallel(&pbes, args.threads, args.caching, args.pinned)?
    } else if args.threads > 1 {
        explore_pbes_parallel(pbes, args.threads, args.caching, args.pinned)?
    } else if args.srf {
        explore_srf_pbes(&pbes, args.strategy, args.caching)?
    } else {
        explore_pbes(pbes, args.strategy, args.caching)?
    };
    println!(
        "Parity game: {} vertices, {} edges",
        game.num_of_vertices(),
        game.num_of_edges()
    );
    Ok(())
}

/// Handles symbolic exploration of a PBES, reporting the number of reachable
/// BES equations (states).
fn handle_explore_symbolic(cli: &Cli, args: &ExploreSymbolicArgs, timing: &Timing) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format.clone())?;
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
fn build_bsgs_from_user_generators(pbes: &Pbes, strs: &[String], gap_path: &str) -> Result<Arc<Bsgs>, MercError> {
    let config = GapConfig {
        executable: gap_path.to_string(),
        dump_script: None,
    };
    let generators = parse_generators(strs)?;
    let lps = PbesLps::new(pbes.clone())?;
    let n = lps.num_params();

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

    let bsgs = Arc::new(Bsgs::from_generators(&generators, n, &config)?);
    info!(
        "User-supplied generators: |G| = {} ({} generator(s))",
        bsgs.order(),
        generators.len()
    );
    Ok(bsgs)
}

/// Compute graph symmetries for `pbes` and build a BSGS from them.
fn build_bsgs_for_pbes(pbes: &Pbes, gap_path: &str) -> Result<Arc<Bsgs>, MercError> {
    let config = GapConfig {
        executable: gap_path.to_string(),
        dump_script: None,
    };
    let sym_result = graph_symmetries(pbes, &config)?;
    let lps = PbesLps::new(pbes.clone())?;
    let n = lps.num_params();
    let bsgs = Arc::new(Bsgs::from_generators(&sym_result.generators, n, &config)?);
    info!("|G| = {} ({} generator(s))", bsgs.order(), sym_result.generators.len());
    Ok(bsgs)
}

/// Explore `pbes` into a parity game, canonicalizing every next-state via `bsgs`.
fn explore_with_symmetry(
    pbes: &Pbes,
    strategy: ExplorationStrategy,
    caching: CachingStrategy,
    threads: usize,
    pinned: bool,
    bsgs: Arc<Bsgs>,
) -> Result<merc_vpg::ParityGame, MercError> {
    let lps = PbesLps::new(pbes.clone())?;
    let timing = Timing::new();

    match caching {
        CachingStrategy::None => {
            let qlps = QuotientLps::new(&lps, bsgs, 1);
            if threads > 1 {
                explore_pbes_parallel_impl(&qlps, threads, pinned)
            } else {
                explore_pbes_impl(&qlps, strategy, &timing)
            }
        }
        _ => {
            // The cache sits *inside* the quotient (see [`QuotientLps`]) so the
            // keys stay the narrow read-position projections of the raw states
            // instead of covering every parameter touched by canonicalization.
            let cached = CacheLPS::new(&lps, caching);
            let qlps = QuotientLps::new(&cached, bsgs, 1);
            let game = if threads > 1 {
                explore_pbes_parallel_impl(&qlps, threads, pinned)
            } else {
                explore_pbes_impl(&qlps, strategy, &timing)
            }?;
            debug!("{}", cached.metrics());
            Ok(game)
        }
    }
}

/// Handles the solve command, which explores a PBES into a parity game and
/// solves the game, printing the solution of the initial vertex.
fn handle_solve(args: &SolveArgs) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format.clone())?;
    let game = if !args.quotient.is_empty() {
        let bsgs = build_bsgs_from_user_generators(&pbes, &args.quotient, &args.gap_path)?;
        explore_with_symmetry(&pbes, args.strategy, args.caching, args.threads, args.pinned, bsgs)?
    } else if args.symmetry {
        let bsgs = build_bsgs_for_pbes(&pbes, &args.gap_path)?;
        explore_with_symmetry(&pbes, args.strategy, args.caching, args.threads, args.pinned, bsgs)?
    } else if args.threads > 1 && args.srf {
        explore_srf_pbes_parallel(&pbes, args.threads, args.caching, args.pinned)?
    } else if args.threads > 1 {
        explore_pbes_parallel(pbes, args.threads, args.caching, args.pinned)?
    } else if args.srf {
        explore_srf_pbes(&pbes, args.strategy, args.caching)?
    } else {
        explore_pbes(pbes, args.strategy, args.caching)?
    };
    info!(
        "Parity game: {} vertices, {} edges",
        game.num_of_vertices(),
        game.num_of_edges()
    );

    let (solution, strategy) = match args.solver {
        Solver::Zielonka => solve_zielonka(&game, args.verify_solution),
        Solver::PriorityPromotion => solve_priority_promotion(&game, args.verify_solution),
    };

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

fn handle_graph_symmetry(args: &GraphSymmetryArgs) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format.clone())?;

    let config = GapConfig {
        executable: args.gap_path.clone(),
        dump_script: args.dump_gap_script.as_deref().map(Path::new).map(|p| p.to_path_buf()),
    };

    let result = graph_symmetries(&pbes, &config)?;

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

fn handle_symmetry(args: &SymmetryArgs) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format.clone())?;
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
