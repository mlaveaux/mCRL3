use std::process::ExitCode;

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
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::explore_srf::parity_game_from_pbes;
use crate::permutation::Permutation;
use crate::symmetry::SymmetryAlgorithm;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_vpg::PG;
use merc_vpg::Player;
use merc_vpg::Solver;
use merc_vpg::solve_priority_promotion;
use merc_vpg::solve_zielonka;
use merc_vpg::verify_solution;

mod clone_iterator;
mod explore_srf;
mod explore_srf_test;
mod export;
mod permutation;
mod symmetry;

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

    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Analyze symmetries of a PBES
    Symmetry(SymmetryArgs),
    /// Exports the control flow graphs of a PBES in JSON format.
    Export(ExportArgs),
    /// Explore a PBES explicitly into a parity game.
    ExploreExplicit(ExploreExplicitArgs),
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
struct ExportArgs {
    /// The input PBES file.
    filename: String,

    /// The JSON output file. If not provided, the output will be written to stdout.
    #[arg(long)]
    output: Option<String>,

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,
}

#[derive(clap::Args, Debug)]
struct ExploreExplicitArgs {
    /// The input PBES file.
    filename: String,

    /// Strategy to explore the state space of the PBES.
    #[arg(long, value_enum, default_value_t = ExplorationStrategy::Bfs)]
    strategy: ExplorationStrategy,

    /// Caching strategy to use during exploration.
    #[arg(long, value_enum, default_value_t = CachingStrategy::None)]
    caching: CachingStrategy,

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,
}

#[derive(clap::Args, Debug)]
struct SolveArgs {
    /// The input PBES file.
    filename: String,

    /// Strategy to explore the state space of the PBES.
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
}

fn main() -> Result<ExitCode, MercError> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .parse_default_env()
        .init();

    // Enable logging on the mCRL2 side
    set_reporting_level(verbosity_to_log_level(cli.verbosity.verbosity()));

    if cli.version.into() {
        eprintln!("{}", Version);
        return Ok(ExitCode::SUCCESS);
    }

    let timing = Timing::new();

    if let Some(command) = cli.commands {
        match command {
            Commands::Symmetry(args) => handle_symmetry(args)?,
            Commands::Export(args) => handle_export(args)?,
            Commands::ExploreExplicit(args) => handle_explore_explicit(args)?,
            Commands::Solve(args) => handle_solve(args)?,
        }
    }

    if cli.timings {
        timing.print();
    }

    Ok(ExitCode::SUCCESS)
}

/// Reads a PBES from the given file in the explicitly chosen format, or the
/// binary PBES format when no format is given.
fn read_pbes(filename: &str, format: Option<PbesFormat>) -> Result<Pbes, MercError> {
    match format.unwrap_or(PbesFormat::Pbes) {
        PbesFormat::Pbes => Ok(Pbes::from_file(filename)?),
        PbesFormat::Text => Ok(Pbes::from_text_file(filename)?),
    }
}

fn handle_explore_explicit(args: ExploreExplicitArgs) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format)?;
    let game = parity_game_from_pbes(&pbes, args.strategy, args.caching)?;
    println!(
        "Parity game: {} vertices, {} edges",
        game.num_of_vertices(),
        game.num_of_edges()
    );
    Ok(())
}

/// Handles the solve command, which explores a PBES into a parity game and
/// solves the game, printing the solution of the initial vertex.
fn handle_solve(args: SolveArgs) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format)?;
    let game = parity_game_from_pbes(&pbes, args.strategy, args.caching)?;
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

fn handle_symmetry(args: SymmetryArgs) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format)?;
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

/// Handles the export command, which exports the control flow graphs of a PBES in JSON format.
fn handle_export(args: ExportArgs) -> Result<(), MercError> {
    let pbes = read_pbes(&args.filename, args.format)?;

    if let Some(output_filename) = args.output {
        let mut file = std::fs::File::create(output_filename)?;
        export::export(&mut file, &pbes)?;
    } else {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        export::export(&mut handle, &pbes)?;
    }

    Ok(())
}
