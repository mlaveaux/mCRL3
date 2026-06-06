use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use itertools::Itertools;
use merc_io::LargeFormatter;
use merc_lts::AutStream;
use merc_lts::LtsBuilderMem;
use merc_lts::LtsFormat;
use merc_lts::guess_lts_format_from_extension;
use merc_lts::write_bcg;
use merc_symbolic::SymFormat;
use merc_symbolic::SymbolicLPS;
use merc_symbolic::SymbolicLTS;
use merc_symbolic::SymbolicLtsBdd;
use merc_symbolic::convert_symbolic_lts;
use merc_symbolic::convert_symbolic_lts_bdd;
use merc_symbolic::guess_format_from_extension;
use merc_symbolic::parse_compacted_dependency_graph;
use merc_symbolic::quotient_symbolic;
use merc_symbolic::reachability;
use merc_symbolic::reachability_bdd;
use merc_symbolic::read_sylvan;
use merc_symbolic::read_symbolic_lts;
use merc_symbolic::refine_bisimulation;
use merc_symbolic::reorder;
use merc_symbolic::sigref_symbolic;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_tools::report_error;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;
use merc_utilities::Timing;
use oxidd::BooleanFunction;
use oxidd::util::SatCountCache;
use rustc_hash::FxBuildHasher;
use which::which_in;

/// Default node capacity for the Oxidd decision diagram manager.
const DEFAULT_OXIDD_NODE_CAPACITY: usize = 2048;

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

    /// Number of workers for the Oxidd decision diagram manager.
    #[arg(long, global = true, default_value_t = 1)]
    oxidd_workers: u32,

    /// Node capacity for the Oxidd decision diagram manager.
    #[arg(long, global = true, default_value_t = DEFAULT_OXIDD_NODE_CAPACITY)]
    oxidd_node_capacity: usize,

    /// Cache capacity for the Oxidd decision diagram manager, if `None` it is set to the node capacity.
    #[arg(long, global = true)]
    oxidd_cache_capacity: Option<usize>,

    #[arg(long, global = true)]
    timings: bool,
}

/// Defines the subcommands for this tool.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Print information related to the given symbolic LTS.
    Info(InfoArgs),
    /// Explore the given symbolic LTS.
    Explore(ExploreArgs),
    /// Computes a reordering for a dependency graph given by lpsreach or pbessolvesymbolic.
    Reorder(ReorderArgs),
    /// Convert a symbolic LTS to a concrete LTS.
    Convert(ConvertArgs),
    /// Apply reductions to a symbolic LTS.
    Reduce(ReduceArgs),
}

#[derive(clap::Args, Debug)]
struct InfoArgs {
    filename: PathBuf,

    /// Sets the input symbolic LTS format.
    #[arg(long)]
    format: Option<SymFormat>,
}

#[derive(clap::Args, Debug)]
struct ExploreArgs {
    filename: PathBuf,

    /// Sets the input symbolic LTS format.
    #[arg(long)]
    format: Option<SymFormat>,

    /// Use BDD based exploration by converting the symbolic LTS
    #[arg(long)]
    use_bdd: bool,

    /// Visualize intermediate BDDs using oxidd-vis.
    #[arg(long)]
    visualize: bool,
}

#[derive(clap::Args, Debug)]
struct ReorderArgs {
    /// Explicit path to the mCRL2 tools (lpsreach or pbessolvesymbolic)
    #[arg(long)]
    mcrl2_path: Option<PathBuf>,

    /// Explicit path to the kahypar tools
    #[arg(long)]
    kahypar_path: Option<PathBuf>,

    /// Explicit path to the kahypar.ini file to use.
    #[arg(long)]
    kahypar_ini_path: Option<PathBuf>,

    /// The input linear process specification file in the mCRL2 .lps format.
    filename: PathBuf,
}

#[derive(clap::Args, Debug)]
struct ConvertArgs {
    /// Sets the input symbolic LTS format.
    #[arg(long)]
    format: Option<SymFormat>,

    /// The input symbolic LTS file path.
    filename: PathBuf,

    /// Sets the output LTS format.
    #[arg(long)]
    output_format: Option<LtsFormat>,

    /// The output LTS file path.
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Clone, Copy, clap::ValueEnum, Debug)]
enum Equivalence {
    StrongBisim,
    StrongBisimSigref,
}

#[derive(clap::Args, Debug)]
struct ReduceArgs {
    /// The equivalence relation to reduce modulo.
    equivalence: Equivalence,

    /// The input symbolic LTS file path.
    filename: PathBuf,

    /// Sets the input symbolic LTS format.
    #[arg(long)]
    format: Option<SymFormat>,

    /// Sets the output LTS format.
    #[arg(long)]
    output_format: Option<LtsFormat>,

    /// The output LTS file path.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Visualize the reduction steps in oxidd-vis.
    #[arg(long)]
    visualize: bool,

    /// Split the signature per transition group.
    #[arg(long)]
    split_signature: bool,

    /// Extend each transition relation to the full state and next-state domain.
    #[arg(long)]
    extend_relation: bool,

    /// Merge transition relations into a single relation before signature computation.
    #[arg(long)]
    merge_transitions: bool,
}

/// Initializes the Oxidd BDD manager based on CLI arguments.
fn init_bdd_manager(cli: &Cli) -> oxidd::bdd::BDDManagerRef {
    oxidd::bdd::new_manager(
        cli.oxidd_node_capacity,
        cli.oxidd_cache_capacity.unwrap_or(cli.oxidd_node_capacity),
        cli.oxidd_workers,
    )
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .parse_default_env()
        .init();

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
            Commands::Info(args) => handle_info(args, timing)?,
            Commands::Explore(args) => handle_explore(cli, args, timing)?,
            Commands::Reorder(args) => handle_reorder(args, timing)?,
            Commands::Convert(args) => handle_convert(args, timing)?,
            Commands::Reduce(args) => handle_reduce(cli, args, timing)?,
        }
    }

    Ok(())
}

/// Reads the given symbolic LTS and prints information about it.
fn handle_info(cli: &Cli, args: &InfoArgs, timing: &Timing) -> Result<(), MercError> {
    let storage = init_ldd_manager(cli);

    let format =
        guess_format_from_extension(&args.filename, args.format).ok_or("Cannot determine input symbolic LTS format")?;

    if format != SymFormat::Sym {
        return Err("Currently only the .sym format is supported for info".into());
    }

    let lts = timing.measure("read_symbolic_lts", || {
        read_symbolic_lts(&storage, File::open(&args.filename)?)
    })?;

    println!("Number of states: {}", LargeFormatter(lts.states().len()));
    println!("Number of summand groups: {}", lts.transition_groups().len());

    Ok(())
}

/// Explores the given symbolic LTS.
fn handle_explore(cli: &Cli, args: &ExploreArgs, timing: &Timing) -> Result<(), MercError> {
    let storage = init_ldd_manager(cli);

    let format = guess_format_from_extension(&args.filename, args.format).ok_or("Cannot determine input format")?;

    let mut file = File::open(&args.filename)?;
    match format {
        SymFormat::Sylvan => {
            let mut lts = timing.measure("read_symbolic_lts", || read_sylvan(&storage, &mut file))?;
            // Sylvan format carries no action labels or parameter values, so BDD conversion is not supported.
            println!(
                "LTS has {} states",
                timing.measure("explore", || -> Result<_, MercError> {
                    Ok(reachability(&storage, &mut lts, timing)?.len())
                })?
            );
        }
        SymFormat::Sym => {
            let mut lts = timing.measure("read_symbolic_lts", || read_symbolic_lts(&storage, &mut file))?;
            explore_impl(&storage, cli, args, &mut lts, timing)?;
        }
    }

    Ok(())
}

fn explore_impl<L: SymbolicLTS>(
    storage: &oxidd::ldd::LDDManagerRef,
    cli: &Cli,
    args: &ExploreArgs,
    lts: &mut L,
    timing: &Timing,
) -> Result<(), MercError> {
    if args.use_bdd {
        let manager_ref = init_bdd_manager(cli);

        let lts_bdd = timing.measure("convert_bdd", || {
            SymbolicLtsBdd::from_symbolic_lts(storage, &manager_ref, lts)
        })?;

        println!(
            "LTS has {} states",
            timing.measure("explore_bdd", || -> Result<_, MercError> {
                let reachable_states_bdd = reachability_bdd(&manager_ref, &lts_bdd, args.visualize)?;

                let num_reachable_states_bdd = reachable_states_bdd.sat_count::<u64, FxBuildHasher>(
                    lts_bdd.state_variables().len() as u32,
                    &mut SatCountCache::default(),
                ) as usize;
                Ok(num_reachable_states_bdd)
            })?
        );
    } else {
        println!(
            "LTS has {} states",
            timing.measure("explore", || -> Result<_, MercError> {
                let states = reachability(storage, lts, timing)?;

                Ok(states.len())
            })?
        );
    }
    Ok(())
}

/// Computes a variable reordering for the output of lpsreach.
fn handle_reorder(args: &ReorderArgs, _timing: &Timing) -> Result<(), MercError> {
    // Find kahypar
    let kahypar_path = if let Some(path) = &args.kahypar_path {
        which_in("KaHyPar", Some(path), std::env::current_dir()?).map_err(|_e| "Cannot find KaHyPar")?
    } else {
        which::which("KaHyPar").map_err(|_e| "Cannot find KaHyPar in PATH")?
    };

    let kahypar_ini_path = if let Some(path) = &args.kahypar_ini_path {
        if !path.is_file() {
            return Err(format!(
                "The specified kahypar.ini path '{}' does not exist or is not a file.",
                path.display()
            )
            .into());
        }
        path.clone()
    } else {
        // Get path relative to the current executable, and obtain a path to the `kahypar.ini` configuration file.
        let mut default_kahypar_ini_path = std::env::current_exe()?;
        default_kahypar_ini_path.pop(); // remove the executable filename
        default_kahypar_ini_path.push("kahypar.ini");

        if !default_kahypar_ini_path.is_file() {
            return Err(format!(
                "Could not find '{}'. The 'kahypar.ini' file must be present next to the executable, or passed via --kahypar-ini-path.",
                default_kahypar_ini_path.display()
            )
            .into());
        }
        default_kahypar_ini_path
    };

    if args.filename.extension() == Some(OsStr::new("lps")) {
        // Find lpsreach
        let lpsreach_path = if let Some(path) = &args.mcrl2_path {
            which_in("lpsreach", Some(path), std::env::current_dir()?).map_err(|_e| "Cannot find lpsreach")?
        } else {
            which::which("lpsreach").map_err(|_e| "Cannot find lpsreach in PATH")?
        };

        // Run lpsreach with the --info flag to get dependency information
        let proc = duct::cmd!(lpsreach_path, "--info", &args.filename)
            .stdout_capture()
            .run()
            .map_err(|e| e.to_string())?;

        let graph = parse_compacted_dependency_graph(str::from_utf8(&proc.stdout)?);

        let order = reorder(&kahypar_path, &kahypar_ini_path, &graph)?;
        println!("Computed variable order: {}", order.iter().format(" "));
    } else if args.filename.extension() == Some(OsStr::new("pbes")) {
        // Find pbessolvesymbolic
        let pbessolvesymbolic = if let Some(path) = &args.mcrl2_path {
            which_in("pbessolvesymbolic", Some(path), std::env::current_dir()?)
                .map_err(|_e| "Cannot find pbessolvesymbolic")?
        } else {
            which::which("pbessolvesymbolic").map_err(|_e| "Cannot find pbessolvesymbolic in PATH")?
        };

        // Run pbessolvesymbolic with the --info flag to get dependency information
        let proc = duct::cmd!(pbessolvesymbolic, "--info", &args.filename)
            .stdout_capture()
            .run()
            .map_err(|e| e.to_string())?;

        let graph = parse_compacted_dependency_graph(str::from_utf8(&proc.stdout)?);
        let order = reorder(&kahypar_path, &kahypar_ini_path, &graph)?;

        // Ensure that the first variable is 0 by removing it from order and printing it explicitly
        println!(
            "Computed variable order: 0 {}",
            order.iter().filter(|&x| *x != 0).format(" ")
        );
    } else {
        return Err("Input file must be either a .lps or .pbes file".into());
    }

    Ok(())
}

/// Converts a symbolic LTS to an explicit LTS.
fn handle_convert(cli: &Cli, args: &ConvertArgs, _timing: &Timing) -> Result<(), MercError> {
    let storage = init_ldd_manager(cli);

    let format =
        guess_format_from_extension(&args.filename, args.format).ok_or("Cannot determine input symbolic LTS format")?;
    if format != SymFormat::Sym {
        return Err("Currently only the .sym format is supported for conversion".into());
    }

    let mut file = File::open(&args.filename)?;
    let lts = read_symbolic_lts(&storage, &mut file)?;

    if let Some(output) = &args.output {
        let output_format =
            guess_lts_format_from_extension(output, args.output_format).ok_or("Cannot determine output LTS format")?;

        match output_format {
            LtsFormat::Lts => {
                unimplemented!("Writing LTS format is not yet implemented");
            }
            LtsFormat::Aut => {
                let mut output = File::create(output)?;
                let mut stream = AutStream::new(&mut output)?;
                convert_symbolic_lts(&storage, &mut stream, &lts)?;
            }
            LtsFormat::AutMcrl2 => {
                let mut output = File::create(output)?;
                let mut stream = AutStream::new_mcrl2(&mut output)?;
                convert_symbolic_lts(&storage, &mut stream, &lts)?;
            }
            LtsFormat::Bcg => {
                let explicit_lts =
                    convert_symbolic_lts(&storage, &mut LtsBuilderMem::new(Vec::new(), Vec::new()), &lts)?;
                write_bcg(&explicit_lts, output)?;
            }
        }
    }

    Ok(())
}

/// Applies reductions to a symbolic LTS.
fn handle_reduce(cli: &Cli, args: &ReduceArgs, timing: &Timing) -> Result<(), MercError> {
    let format =
        guess_format_from_extension(&args.filename, args.format).ok_or("Cannot determine input symbolic LTS format")?;
    if format != SymFormat::Sym {
        return Err("Currently only the .sym format is supported for reduction".into());
    }

    let storage = init_ldd_manager(cli);
    let manager_ref = init_bdd_manager(cli);

    let mut file = File::open(&args.filename)?;
    let lts = read_symbolic_lts(&storage, &mut file)?;

    let lts_bdd = timing.measure("convert_bdd", || {
        SymbolicLtsBdd::from_symbolic_lts(&storage, &manager_ref, &lts)
    })?;

    let quotient_lts = timing.measure("reduction", || -> Result<Option<SymbolicLtsBdd>, MercError> {
        match args.equivalence {
            Equivalence::StrongBisimSigref => {
                let (partition, block_vars, _num_of_blocks) = sigref_symbolic(
                    &manager_ref,
                    &lts_bdd,
                    timing,
                    args.split_signature,
                    args.extend_relation,
                    args.merge_transitions,
                    args.visualize,
                )?;

                let quotient = timing.measure("quotient", || {
                    quotient_symbolic(&manager_ref, &lts_bdd, &partition, &block_vars)
                })?;
                Ok(Some(quotient))
            }
            Equivalence::StrongBisim => {
                let _ = refine_bisimulation(&manager_ref, &lts_bdd)?;
                Ok(None)
            }
        }
    })?;

    if let Some(output) = &args.output {
        let quotient_lts =
            quotient_lts.ok_or("Writing the quotient is not yet supported for the selected equivalence")?;

        let output_format =
            guess_lts_format_from_extension(output, args.output_format).ok_or("Cannot determine output LTS format")?;

        match output_format {
            LtsFormat::Lts => {
                unimplemented!("Writing LTS format is not yet implemented");
            }
            LtsFormat::Aut => {
                let mut output = File::create(output)?;
                let mut stream = AutStream::new(&mut output)?;
                convert_symbolic_lts_bdd(&manager_ref, &mut stream, &quotient_lts)?;
            }
            LtsFormat::AutMcrl2 => {
                let mut output = File::create(output)?;
                let mut stream = AutStream::new_mcrl2(&mut output)?;
                convert_symbolic_lts_bdd(&manager_ref, &mut stream, &quotient_lts)?;
            }
            LtsFormat::Bcg => {
                let explicit_lts = convert_symbolic_lts_bdd(
                    &manager_ref,
                    &mut LtsBuilderMem::new(Vec::new(), Vec::new()),
                    &quotient_lts,
                )?;
                write_bcg(&explicit_lts, output)?;
            }
        }
    }

    Ok(())
}
