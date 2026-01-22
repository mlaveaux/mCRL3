use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use merc_io::LargeFormatter;
use merc_ldd::len;
use merc_ldd::Storage;
use merc_lts::guess_lts_format_from_extension;
use merc_lts::write_bcg;
use merc_lts::AutStream;
use merc_lts::LtsBuilderMem;
use merc_lts::LtsFormat;
use merc_lts::guess_lts_format_from_extension;
use merc_lts::write_bcg;
use merc_symbolic::SymFormat;
use merc_symbolic::SymbolicLTS;
use merc_symbolic::SymbolicLtsBdd;
use merc_symbolic::convert_symbolic_lts;
use merc_symbolic::guess_format_from_extension;
use merc_symbolic::parse_compacted_dependency_graph;
use merc_symbolic::reachability;
use merc_symbolic::read_sylvan;
use merc_symbolic::read_symbolic_lts;
use merc_symbolic::reorder;
use merc_symbolic::sigref_symbolic;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;
use merc_utilities::Timing;
use which::which_in;

/// Default node capacity for the Oxidd decision diagram manager.
const DEFAULT_OXIDD_NODE_CAPACITY: usize = 2024;

/// A command line tool for symbolic labelled transition systems
#[derive(clap::Parser, Debug)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(flatten)]
    version: VersionFlag,

    #[command(flatten)]
    verbosity: VerbosityFlag,

    #[command(subcommand)]
    commands: Option<Commands>,

    #[arg(long, global = true, default_value_t = 1)]
    oxidd_workers: u32,

    #[arg(long, global = true, default_value_t = DEFAULT_OXIDD_NODE_CAPACITY)]
    oxidd_node_capacity: usize,

    #[arg(long, global = true)]
    oxidd_cache_capacity: Option<usize>,

    #[arg(long, global = true)]
    timings: bool,
}

/// Defines the subcommands for this tool.
#[derive(Debug, Subcommand)]
enum Commands {
    Info(InfoArgs),
    Explore(ExploreArgs),
    Reorder(ReorderArgs),
    Convert(ConvertArgs),
    Reduce(ReduceArgs),
}

/// Print information related to the given symbolic LTS
#[derive(clap::Args, Debug)]
#[command()]
struct InfoArgs {
    filename: PathBuf,

    #[arg(long, help = "Sets the input symbolic LTS format.")]
    format: Option<SymFormat>,
}

/// Explore the given symbolic LTS
#[derive(clap::Args, Debug)]
#[command()]
struct ExploreArgs {
    filename: PathBuf,

    #[arg(long, help = "Sets the input symbolic LTS format.")]
    format: Option<SymFormat>,
}

/// Compute a reordering for a dependency graph given by lpsreach or pbessolvesymbolic
#[derive(clap::Args, Debug)]
#[command()]
struct ReorderArgs {
    /// Path to the mCRL2 tools (lpsreach or pbessolvesymbolic).
    #[arg(long)]
    mcrl2_tool_path: Option<PathBuf>,

    #[arg(help = "The input linear process specification file in the mCRL2 .lps format.")]
    filename: PathBuf,
}

/// Convert a symbolic LTS to a concrete LTS
#[derive(clap::Args, Debug)]
#[command()]
struct ConvertArgs {
    #[arg(long, help = "Sets the input symbolic LTS format.")]
    format: Option<SymFormat>,

    #[arg(help = "The input symbolic LTS file path.")]
    filename: PathBuf,

    #[arg(long, help = "Sets the output LTS format.")]
    output_format: Option<LtsFormat>,

    #[arg(help = "The output LTS file path.")]
    output: PathBuf,
}

#[derive(clap::Args, Debug)]
#[command(about = "Applied reductions to a symbolic LTS")]
struct ReduceArgs {
    #[arg(help = "The input symbolic LTS file path.")]
    filename: PathBuf,

    #[arg(long, help = "Sets the input symbolic LTS format.")]
    format: Option<LtsFormat>,

    #[arg(long, help = "Visualize the reduction steps in oxidd-vis.")]
    visualize: bool,
}

fn main() -> Result<ExitCode, MercError> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .parse_default_env()
        .init();

    if cli.version.into() {
        eprintln!("{}", Version);
        return Ok(ExitCode::SUCCESS);
    }

    let mut timing = Timing::new();

    if let Some(command) = &cli.commands {
        match command {
            Commands::Info(args) => handle_info(args, &mut timing)?,
            Commands::Explore(args) => handle_explore(args, &mut timing)?,
            Commands::Reorder(args) => handle_reorder(args, &mut timing)?,
            Commands::Convert(args) => handle_convert(args, &mut timing)?,
            Commands::Reduce(args) => handle_reduce(&cli, args, &mut timing)?,
        }
    }

    if cli.timings {
        timing.print();
    }

    print_allocator_metrics();
    Ok(ExitCode::SUCCESS)
}

/// Reads the given symbolic LTS and prints information about it.
fn handle_info(args: &InfoArgs, timing: &mut Timing) -> Result<(), MercError> {
    let mut storage = Storage::new();

    let format = guess_format_from_extension(&args.filename, args.format).ok_or("Cannot determine input format")?;

    match format {
        SymFormat::Sylvan => {
            let mut time_read = timing.start("read_symbolic_lts");
            let lts = read_sylvan(&mut storage, &mut File::open(&args.filename)?)?;
            time_read.finish();

            println!("Symbolic LTS information:");
            println!(
                "  Number of states: {}",
                LargeFormatter(merc_ldd::len(&mut storage, lts.states()))
            );
            println!("  Number of summand groups: {}", lts.transition_groups().len());
        }
        SymFormat::Sym => {
            let mut time_read = timing.start("read_symbolic_lts");
            let lts = read_symbolic_lts(&mut storage, &mut File::open(&args.filename)?)?;
            time_read.finish();

            println!("Symbolic LTS information:");
            println!(
                "  Number of states: {}",
                LargeFormatter(merc_ldd::len(&mut storage, lts.states()))
            );
            println!("  Number of summand groups: {}", lts.transition_groups().len());
        }
    }

    Ok(())
}

/// Explores the given symbolic LTS.
fn handle_explore(args: &ExploreArgs, _timing: &mut Timing) -> Result<(), MercError> {
    let mut storage = Storage::new();

    let format = guess_format_from_extension(&args.filename, args.format).ok_or("Cannot determine input format")?;

    let mut file = File::open(&args.filename)?;
    let timing = Timing::new();

    match format {
        SymFormat::Sylvan => {
            let mut time_read = timing.start("read_symbolic_lts");
            let lts = read_sylvan(&mut storage, &mut file)?;
            time_read.finish();

            timing.measure("explore", || -> Result<(), MercError> {
                println!("LTS has {} states", reachability(&mut storage, &lts)?);
                Ok(())
            })?;
        }
        SymFormat::Sym => {
            let lts = timing.measure("read_symbolic_lts", || read_symbolic_lts(&mut storage, &mut file))?;

            println!("LTS has {} states", len(&mut storage, lts.states()));
        }
    }

    Ok(())
}

/// Computes a variable reordering for the output of lpsreach.
fn handle_reorder(args: &ReorderArgs, _timing: &mut Timing) -> Result<(), MercError> {
    if args.filename.extension() == Some(OsStr::new("lps")) {
        // Find lpsreach
        let lpsreach_path = if let Some(path) = &args.mcrl2_tool_path {
            which_in("lpsreach", Some(path), std::env::current_dir()?)?
        } else {
            which::which("lpsreach").map_err(|_e| "Cannot find lpsreach in PATH")?
        };

        // Run lpsreach with the --info flag to get dependency information
        let proc = duct::cmd!(lpsreach_path, "--info", &args.filename)
            .stdout_capture()
            .run()
            .map_err(|e| e.to_string())?;

        let graph = parse_compacted_dependency_graph(str::from_utf8(&proc.stdout)?);

        let order = reorder(&graph)?;
        println!("Computed variable order: {:?}", order);
    } else if args.filename.extension() == Some(OsStr::new("pbes")) {
        // Find pbessolvesymbolic
        let pbessolvesymbolic = if let Some(path) = &args.mcrl2_tool_path {
            which_in("pbessolvesymbolic", Some(path), std::env::current_dir()?)?
        } else {
            which::which("pbessolvesymbolic").map_err(|_e| "Cannot find pbessolvesymbolic in PATH")?
        };

        // Run pbessolvesymbolic with the --info flag to get dependency information
        let proc = duct::cmd!(pbessolvesymbolic, "--info", &args.filename)
            .stdout_capture()
            .run()
            .map_err(|e| e.to_string())?;

        let graph = parse_compacted_dependency_graph(str::from_utf8(&proc.stdout)?);
        let mut order = reorder(&graph)?;

        // Ensure that the first variable is 0 by removing it and keeping the rest
        order.retain(|&x| x != 0);
        println!("Computed variable order: 0, {:?}", order);
    } else {
        return Err("Input file must be either a .lps or .pbes file".into());
    }

    Ok(())
}

/// Converts a symbolic LTS to an explicit LTS.
fn handle_convert(args: &ConvertArgs, _timing: &mut Timing) -> Result<(), MercError> {
    let mut storage = Storage::new();

    let format =
        guess_format_from_extension(&args.filename, args.format).ok_or("Cannot determine input symbolic LTS format")?;
    if format != SymFormat::Sym {
        return Err("Currently only the .sym format is supported for conversion".into());
    }

    let mut file = File::open(&args.filename)?;
    let lts = read_symbolic_lts(&mut storage, &mut file)?;

    let output_format = guess_lts_format_from_extension(&args.output, args.output_format)
        .ok_or("Cannot determine output LTS format")?;

    match output_format {
        LtsFormat::Lts => {
            unimplemented!("Writing LTS format is not yet implemented");
        }
        LtsFormat::Aut => {
            let mut output = File::create(&args.output)?;
            let mut stream = AutStream::new(&mut output);
            convert_symbolic_lts(&mut storage, &mut stream, &lts)?;
        }
        LtsFormat::Bcg => {
            let explicit_lts =
                convert_symbolic_lts(&mut storage, &mut LtsBuilderMem::new(Vec::new(), Vec::new()), &lts)?;
            let explicit_lts =
                convert_symbolic_lts(&mut storage, &mut LtsBuilderMem::new(Vec::new(), Vec::new()), &lts)?;
            write_bcg(&explicit_lts, &args.output)?;
        }
    }

    Ok(())
}

/// Applies reductions to a symbolic LTS.
fn handle_reduce(cli: &Cli, args: &ReduceArgs, timing: &mut Timing) -> Result<(), MercError> {
    let mut storage = Storage::new();

    let manager_ref = oxidd::bdd::new_manager(
        cli.oxidd_node_capacity,
        cli.oxidd_cache_capacity.unwrap_or(cli.oxidd_node_capacity),
        cli.oxidd_workers,
    );

    let mut file = File::open(&args.filename)?;
    let lts = read_symbolic_lts(&mut storage, &mut file)?;

    let mut convert_time = timing.start("convert_bdd");
    let lts_bdd = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &lts)?;
    convert_time.finish();

    let mut reduction_time = timing.start("reduction");
    sigref_symbolic(&manager_ref, &lts_bdd, args.visualize)?;
    reduction_time.finish();

    Ok(())
}
