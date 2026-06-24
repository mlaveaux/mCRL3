use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_lts::AutFormat;
use merc_lts::AutStream;
use merc_lts::MutexLtsBuilder;
use merc_symbolic::ExplorationStrategy as SymbolicExplorationStrategy;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_tools::report_error;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;
use merc_utilities::Timing;

use mcrl2::read_lps;
use mcrl2::read_lps_text;
use mcrl2::set_reporting_level;
use mcrl2::verbosity_to_log_level;

use explore_explicit::Mcrl2MultiActionLabel;
use explore_explicit::explore_lps_explicit;
use explore_explicit::explore_lps_explicit_parallel;
use explore_symbolic::explore_lps_symbolic;

mod cfg_lps;
mod cfg_lps_test;
mod control_flow;
mod explore_explicit;
mod explore_symbolic;
mod explore_test;

/// Default number of nodes for the Oxidd LDD manager.
const DEFAULT_OXIDD_NODE_CAPACITY: usize = 1 << 24;

#[derive(clap::ValueEnum, Clone, Debug)]
enum LpsFormat {
    Lps,
    Text,
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

    /// The strategy used to apply the transition groups during reachability.
    #[arg(long, short('s'), value_enum, default_value_t = SymbolicExplorationStrategy::default())]
    strategy: SymbolicExplorationStrategy,

    /// Detect and report deadlock states (reachable states with no outgoing transition).
    #[arg(long)]
    deadlocks: bool,
}

#[derive(clap::Args, Debug)]
struct ExploreExplicitArgs {
    /// The input LPS file.
    filename: String,

    /// Explicitly choose the format of the input LPS file.
    #[arg(long, short('i'), value_enum)]
    format: Option<LpsFormat>,

    #[arg(long, short('o'), value_enum, default_value_t = AutFormat::Aut)]
    out_format: AutFormat,

    /// Specify the output LTS in AUT format. If not given, the LTS is not written.
    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long, short('c'), value_enum, default_value_t = CachingStrategy::None)]
    caching: CachingStrategy,

    /// Order in which discovered states are explored.
    #[arg(long, short('s'), value_enum, default_value_t = ExplorationStrategy::Dfs)]
    strategy: ExplorationStrategy,

    /// Use a control flow graph analysis to prune summands whose control flow
    /// guard cannot hold in the current state. The explored transition system is
    /// unchanged.
    #[arg(long)]
    control_flow: bool,

    /// Number of worker threads used for exploration.
    #[arg(long, default_value_t = 1)]
    threads: usize,

    /// Pin each worker thread round-robin to the available CPU cores.
    #[arg(long, default_value_t = false)]
    pinned: bool,
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
    if let Some(command) = &cli.commands {
        match command {
            Commands::Explore(args) => handle_explore(cli, args, timing)?,
            Commands::ExploreExplicit(args) => handle_explore_explicit(args, timing)?,
        }
    }

    Ok(())
}

/// Handles symbolic exploration of an LPS.
fn handle_explore(cli: &Cli, args: &ExploreArgs, timing: &Timing) -> Result<(), MercError> {
    let format = args.format.clone().unwrap_or(LpsFormat::Lps);
    let lps = match format {
        LpsFormat::Lps => read_lps(&args.filename)?,
        LpsFormat::Text => read_lps_text(&args.filename)?,
    };

    let storage = init_ldd_manager(cli);

    let result = explore_lps_symbolic(&storage, &lps, args.strategy, args.deadlocks, timing)?;
    println!("Number of states: {}", result.states.len());
    if let Some(deadlocks) = &result.deadlocks {
        println!("Number of deadlocks: {}", deadlocks.len());
    }

    Ok(())
}

/// Handles the explicit exploration of an LPS.
fn handle_explore_explicit(args: &ExploreExplicitArgs, timing: &Timing) -> Result<(), MercError> {
    let format = args.format.clone().unwrap_or(LpsFormat::Lps);
    let lps = match format {
        LpsFormat::Lps => read_lps(&args.filename)?,
        LpsFormat::Text => read_lps_text(&args.filename)?,
    };

    if args.threads > 1 {
        // Parallel exploration adds transitions concurrently, so the AUT writer
        // is wrapped in a `MutexLtsBuilder` that serialises the writes; the
        // expensive enumeration still happens outside the lock.
        if let Some(output) = &args.output {
            let mut file = BufWriter::new(File::create(output)?);
            let mut builder = MutexLtsBuilder::new(AutStream::with_format(&mut file, args.out_format)?);
            explore_lps_explicit_parallel(
                &mut builder,
                &lps,
                args.caching,
                args.threads,
                args.control_flow,
                args.pinned,
                timing,
            )?;
        } else {
            // No output requested, discard the explored transitions.
            explore_lps_explicit_parallel(
                &mut (),
                &lps,
                args.caching,
                args.threads,
                args.control_flow,
                args.pinned,
                timing,
            )?;
        }
    } else if let Some(output) = &args.output {
        let mut file = BufWriter::new(File::create(output)?);
        let mut builder: AutStream<_, Mcrl2MultiActionLabel> = AutStream::with_format(&mut file, args.out_format)?;
        explore_lps_explicit(
            &mut builder,
            &lps,
            args.caching,
            args.strategy,
            args.control_flow,
            timing,
        )?;
    } else {
        // No output requested, discard the explored transitions.
        let mut builder: () = ();
        explore_lps_explicit(
            &mut builder,
            &lps,
            args.caching,
            args.strategy,
            args.control_flow,
            timing,
        )?;
    }

    Ok(())
}
