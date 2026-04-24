use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use mcrl2::read_lps;
use mcrl2::read_lps_text;
use mcrl2::set_reporting_level;
use mcrl2::verbosity_to_log_level;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_utilities::MercError;
use merc_utilities::Timing;

use explore::explore_lps;

mod explore;

#[derive(clap::ValueEnum, Clone, Debug)]
enum LpsFormat {
    Text,
    Lps,
}

/// A command line tool for linear process specifications (LPSs)
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
    /// Explores the state space of an LPS symbolically
    Explore(ExploreArgs),
}

#[derive(clap::Args, Debug)]
struct ExploreArgs {
    /// The input LPS file.
    filename: String,

    /// Explicitly choose the format of the input LPS file.
    #[arg(long, short('i'), value_enum)]
    format: Option<LpsFormat>,
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
            Commands::Explore(args) => handle_explore(args)?,
        }
    }

    if cli.timings {
        timing.print();
    }

    Ok(ExitCode::SUCCESS)
}

fn handle_explore(args: ExploreArgs) -> Result<(), MercError> {
    let format = args.format.unwrap_or(LpsFormat::Lps);
    let lps = match format {
        LpsFormat::Lps => read_lps(&args.filename)?,
        LpsFormat::Text => read_lps_text(&args.filename)?,
    };

    explore_lps(&lps)?;

    Ok(())
}