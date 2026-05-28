use std::fs::File;
use std::io::stdout;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use merc_ldd::Storage;
use merc_ldd::len;
use merc_lts::LTS;
use merc_lts::LtsBuilderFast;
use merc_lts::StateIndex;
use merc_lts::write_aut;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;
use merc_utilities::Timing;

use mcrl2::read_lps;
use mcrl2::set_reporting_level;
use mcrl2::verbosity_to_log_level;

use explore_symbolic::explore_lps_symbolic;
use explore_explicit::explore_lps_explicit;

mod explore_symbolic;
mod explore_explicit;

#[derive(clap::ValueEnum, Clone, Debug)]
enum LpsFormat {
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
    /// Explores the state space of an LPS explicitly
    ExploreExplicit(ExploreExplicitArgs),
}

#[derive(clap::Args, Debug)]
struct ExploreArgs {
    /// The input LPS file.
    filename: String,

    /// Explicitly choose the format of the input LPS file.
    #[arg(long, short('i'), value_enum)]
    format: Option<LpsFormat>,
}

#[derive(clap::Args, Debug)]
struct ExploreExplicitArgs {
    /// The input LPS file.
    filename: String,

    /// Explicitly choose the format of the input LPS file.
    #[arg(long, short('i'), value_enum)]
    format: Option<LpsFormat>,

    /// Specify the output LTS in AUT format. If not given, the LTS is not written.
    #[arg(long)]
    output: Option<PathBuf>,
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
            Commands::Explore(args) => handle_explore(args, &timing)?,
            Commands::ExploreExplicit(args) => handle_explore_explicit(args, &timing)?,
        }
    }

    if cli.timings {
        timing.print();
    }

    print_allocator_metrics();

    Ok(ExitCode::SUCCESS)
}

fn handle_explore(args: ExploreArgs, timing: &Timing) -> Result<(), MercError> {
    let format = args.format.unwrap_or(LpsFormat::Lps);
    let lps = match format {
        LpsFormat::Lps => read_lps(&args.filename)?,
    };

    let mut storage = Storage::new();

    let num_of_states = explore_lps_symbolic(&mut storage, &lps, timing)?;
    println!("Number of states: {}", len(&mut storage, &num_of_states));

    Ok(())
}

fn handle_explore_explicit(args: ExploreExplicitArgs, timing: &Timing) -> Result<(), MercError> {
    let format = args.format.unwrap_or(LpsFormat::Lps);
    let lps = match format {
        LpsFormat::Lps => read_lps(&args.filename)?,
    };

    let mut builder: LtsBuilderFast<String> = LtsBuilderFast::new(Vec::new(), Vec::new());
    explore_lps_explicit(&mut builder, &lps, timing)?;
    let lts = builder.finish(StateIndex::new(0), false);

    println!("Number of states: {}", lts.num_of_states());
    println!("Number of transitions: {}", lts.num_of_transitions());

    if let Some(output) = &args.output {
        write_aut(&mut File::create(output)?, &lts)?;
    } else {
        write_aut(&mut stdout(), &lts)?;
    }

    Ok(())
}
