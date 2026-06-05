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
use merc_explore::ExplorationStrategy;
use merc_vpg::PG;

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

    /// Explicitly choose the format of the input PBES file.
    #[arg(long, short('i'), value_enum)]
    format: Option<PbesFormat>,
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
        }
    }

    if cli.timings {
        timing.print();
    }

    Ok(ExitCode::SUCCESS)
}

fn handle_explore_explicit(args: ExploreExplicitArgs) -> Result<(), MercError> {
    let format = args.format.unwrap_or(PbesFormat::Pbes);
    let pbes = match format {
        PbesFormat::Pbes => Pbes::from_file(&args.filename)?,
        PbesFormat::Text => Pbes::from_text_file(&args.filename)?,
    };
    let game = parity_game_from_pbes(&pbes, ExplorationStrategy::Bfs)?;
    println!(
        "Parity game: {} vertices, {} edges",
        game.num_of_vertices(),
        game.num_of_edges()
    );
    Ok(())
}

fn handle_symmetry(args: SymmetryArgs) -> Result<(), MercError> {
    let format = args.format.unwrap_or(PbesFormat::Pbes);
    let pbes = match format {
        PbesFormat::Pbes => Pbes::from_file(&args.filename)?,
        PbesFormat::Text => Pbes::from_text_file(&args.filename)?,
    };
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
    let format = args.format.unwrap_or(PbesFormat::Pbes);
    let pbes = match format {
        PbesFormat::Pbes => Pbes::from_file(&args.filename)?,
        PbesFormat::Text => Pbes::from_text_file(&args.filename)?,
    };

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
