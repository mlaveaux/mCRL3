use std::fs::File;
use std::io::Write;
use std::io::stdout;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use log::info;

use merc_io::LargeFormatter;
use merc_lts::GenericLts;
use merc_lts::LTS;
use merc_lts::LtsFormat;
use merc_lts::apply_lts;
use merc_lts::apply_lts_pair;
use merc_lts::guess_lts_format_from_extension;
use merc_lts::read_explicit_lts;
use merc_lts::write_aut;
use merc_lts::write_bcg;
use merc_reduction::Equivalence;
use merc_reduction::reduce_lts;
use merc_refinement::RefinementType;
use merc_refinement::generate_formula;
use merc_refinement::refines;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_tools::format_key_values_json;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;
use merc_utilities::Timing;

/// A command line tool for labelled transition systems.
#[derive(clap::Parser, Debug)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(flatten)]
    version: VersionFlag,

    #[command(flatten)]
    verbosity: VerbosityFlag,

    #[command(subcommand)]
    commands: Option<Commands>,

    #[arg(long, global = true)]
    timings: bool,
}

/// Defines the subcommands for this tool.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Prints information related to the given LTS.
    Info(InfoArgs),
    /// Reduces the given LTS modulo an equivalent relation.
    Reduce(ReduceArgs),
    /// Compares two LTS modulo an equivalent relation.
    Compare(CompareArgs),
    /// Checks whether the given implementation LTS refines the given specification LTS modulo various refinement relations.
    Refines(RefinesArgs),
    /// Converts an LTS from one format to another format.
    Convert(ConvertArgs),
}

#[derive(clap::Args, Debug)]
struct InfoArgs {
    /// Specify the input LTS.
    filename: String,

    /// Explicitly specify the LTS file format.
    #[arg(long)]
    format: Option<LtsFormat>,

    /// List of actions that should be considered tau actions.
    #[arg(short, long, value_delimiter = ',')]
    tau: Option<Vec<String>>,
}

#[derive(clap::Args, Debug)]
struct ReduceArgs {
    /// Selects the equivalence to reduce the LTS modulo.
    equivalence: Equivalence,

    /// Specify the input LTS.
    filename: PathBuf,

    /// Explicitly specify the LTS file format.
    #[arg(long)]
    filetype: Option<LtsFormat>,

    /// Specify the output LTS, if not given, output to stdout.
    #[arg(long)]
    output: Option<PathBuf>,

    /// List of actions that should be considered tau actions.
    #[arg(long, value_delimiter = ',')]
    tau: Option<Vec<String>>,

    /// Disables preprocessing of the LTS before reducing.
    #[arg(long)]
    no_preprocess: bool,
}

#[derive(clap::Args, Debug)]
struct CompareArgs {
    /// Selects the equivalence to compare the LTSs modulo.
    equivalence: Equivalence,

    /// Specify the input LTS.
    left_filename: PathBuf,

    /// Specify the input LTS.
    right_filename: PathBuf,

    /// Explicitly specify the LTS file format.
    #[arg(long)]
    format: Option<LtsFormat>,

    /// List of actions that should be considered tau actions.
    #[arg(long, value_delimiter = ',')]
    tau: Option<Vec<String>>,

    /// Disables preprocessing of the LTSs before checking equivalence.
    #[arg(long)]
    no_preprocess: bool,
}

#[derive(clap::Args, Debug)]
struct ConvertArgs {
    /// Explicitly specify the LTS input file format.
    #[arg(long)]
    format: Option<LtsFormat>,

    /// Specify the input LTS.
    filename: PathBuf,

    /// Explicitly specify the LTS output file format.
    #[arg(long)]
    output_format: Option<LtsFormat>,

    /// Specify the output LTS.
    output: Option<PathBuf>,

    /// List of actions that should be considered tau actions.
    #[arg(long, value_delimiter = ',')]
    tau: Option<Vec<String>>,
}

#[derive(clap::Args, Debug)]
struct RefinesArgs {
    /// Selects the preorder to check for refinement.
    refinement: RefinementType,

    /// Specify the implementation LTS.
    implementation_filename: PathBuf,

    /// Specify the specification LTS.
    specification_filename: PathBuf,

    /// If set, outputs a counter-example when refinement does not hold.
    #[arg(short = 'c', long)]
    counter_example: Option<PathBuf>,

    /// Explicitly specify the LTS file format.
    #[arg(long)]
    format: Option<LtsFormat>,

    /// List of actions that should be considered tau actions
    #[arg(long, value_delimiter = ',')]
    tau: Option<Vec<String>>,

    /// Disables preprocessing of the LTSs before checking refinement.
    #[arg(long)]
    no_preprocess: bool,
}

fn main() -> Result<ExitCode, MercError> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .format_key_values(|formatter, source| format_key_values_json(formatter, source))
        .parse_default_env()
        .init();

    if cli.version.into() {
        eprintln!("{}", Version);
        return Ok(ExitCode::SUCCESS);
    }

    let mut timing = Timing::new();

    if let Some(command) = &cli.commands {
        match command {
            Commands::Info(args) => {
                handle_info(args, &mut timing)?;
            }
            Commands::Reduce(args) => {
                handle_reduce(args, &mut timing)?;
            }
            Commands::Compare(args) => {
                handle_compare(args, &mut timing)?;
            }
            Commands::Refines(args) => {
                handle_refinement(args, &mut timing)?;
            }
            Commands::Convert(args) => {
                handle_convert(args, &mut timing)?;
            }
        }
    }

    if cli.timings {
        timing.print();
    }

    print_allocator_metrics();
    Ok(ExitCode::SUCCESS)
}

/// Display information about the given LTS.
fn handle_info(args: &InfoArgs, timing: &mut Timing) -> Result<(), MercError> {
    let path = Path::new(&args.filename);

    let format = guess_lts_format_from_extension(path, args.format).ok_or("Unknown LTS file format.")?;
    let lts = read_explicit_lts(path, format, args.tau.clone().unwrap_or_default(), timing)?;
    println!(
        "LTS has {} states and {} transitions.",
        LargeFormatter(lts.num_of_states()),
        LargeFormatter(lts.num_of_transitions())
    );

    apply_lts!(lts, (), |lts, _| {
        println!("Labels:");
        for label in lts.labels() {
            println!("  {}", label);
        }

        let num_of_silent_transitions = lts.iter_states().fold(0, |acc, s| {
            acc + lts
                .outgoing_transitions(s)
                .filter(|t| lts.is_hidden_label(t.label))
                .count()
        });

        // Count the number of silent transitions.
        println!("Silent transitions: {}", num_of_silent_transitions);

        // Structured output.
        println!("{{\"num_of_silent_transitions\": {}}}", num_of_silent_transitions);
    });

    Ok(())
}

/// Reduce the given LTS into another LTS modulo any of the supported equivalences.
fn handle_reduce(args: &ReduceArgs, timing: &mut Timing) -> Result<(), MercError> {
    let path = Path::new(&args.filename);
    let format = guess_lts_format_from_extension(path, args.filetype).ok_or("Unknown LTS file format.")?;

    let lts = read_explicit_lts(path, format, args.tau.clone().unwrap_or_default(), timing)?;
    info!(
        "LTS has {} states and {} transitions.",
        LargeFormatter(lts.num_of_states()),
        LargeFormatter(lts.num_of_transitions())
    );

    apply_lts!(lts, timing, |lts, timing| -> Result<(), MercError> {
        let reduced_lts = reduce_lts(lts, args.equivalence, !args.no_preprocess, timing);

        info!(
            "Reduced LTS has {} states and {} transitions.",
            LargeFormatter(reduced_lts.num_of_states()),
            LargeFormatter(reduced_lts.num_of_transitions())
        );

        if let Some(file) = &args.output {
            let mut writer = File::create(file)?;
            write_aut(&mut writer, &reduced_lts)?;
        } else {
            write_aut(&mut stdout(), &reduced_lts)?;
        }

        Ok(())
    })?;

    Ok(())
}

/// Handles the refinement checking between two LTSs.
fn handle_refinement(args: &RefinesArgs, timing: &mut Timing) -> Result<(), MercError> {
    let impl_path = Path::new(&args.implementation_filename);
    let spec_path = Path::new(&args.specification_filename);
    let format = guess_lts_format_from_extension(impl_path, args.format).ok_or("Unknown LTS file format.")?;

    let impl_lts = read_explicit_lts(impl_path, format, args.tau.clone().unwrap_or_default(), timing)?;
    let spec_lts = read_explicit_lts(spec_path, format, args.tau.clone().unwrap_or_default(), timing)?;

    info!(
        "Implementation LTS has {} states and {} transitions.",
        LargeFormatter(impl_lts.num_of_states()),
        LargeFormatter(impl_lts.num_of_transitions())
    );
    info!(
        "Specification LTS has {} states and {} transitions.",
        LargeFormatter(spec_lts.num_of_states()),
        LargeFormatter(spec_lts.num_of_transitions())
    );

    apply_lts_pair!(impl_lts, spec_lts, timing, |left,
                                                 right,
                                                 timing|
     -> Result<(), MercError> {
        let (result, counter_example) = refines(
            left,
            right,
            args.refinement,
            !args.no_preprocess,
            args.counter_example.is_some(),
            timing,
        );

        if result {
            println!("true");
        } else {
            if let Some(counter_example) = counter_example {
                if let Some(path) = &args.counter_example {
                    // Generate a counterexample formula and output it to the given path.
                    let mut writer = File::create(path)?;
                    writeln!(&mut writer, "{}", generate_formula(&counter_example))?;
                } else {
                    panic!("Counter example path not provided.");
                }
            }

            println!("false");
        }

        Ok(())
    })?;

    Ok(())
}

/// Compares two LTSs for equivalence modulo any of the available equivalences.
fn handle_compare(args: &CompareArgs, timing: &mut Timing) -> Result<(), MercError> {
    let format = guess_lts_format_from_extension(&args.left_filename, args.format).ok_or("Unknown LTS file format.")?;

    info!("Assuming format {:?} for both LTSs.", format);
    let left_lts = read_explicit_lts(
        &args.left_filename,
        format,
        args.tau.clone().unwrap_or_default(),
        timing,
    )?;
    let right_lts = read_explicit_lts(
        &args.right_filename,
        format,
        args.tau.clone().unwrap_or_default(),
        timing,
    )?;

    info!(
        "Left LTS has {} states and {} transitions.",
        LargeFormatter(left_lts.num_of_states()),
        LargeFormatter(left_lts.num_of_transitions())
    );
    info!(
        "Right LTS has {} states and {} transitions.",
        LargeFormatter(right_lts.num_of_states()),
        LargeFormatter(right_lts.num_of_transitions())
    );

    let equivalent = apply_lts_pair!(left_lts, right_lts, timing, |left, right, timing| {
        merc_reduction::compare_lts(args.equivalence, left, right, !args.no_preprocess, timing)
    });

    if equivalent {
        println!("true");
    } else {
        println!("false");
    }

    Ok(())
}

/// Converts an LTS from one format to another, does not do any reduction, see [handle_reduce] for that.
fn handle_convert(args: &ConvertArgs, timing: &mut Timing) -> Result<(), MercError> {
    let format = guess_lts_format_from_extension(&args.filename, args.format).ok_or("Unknown LTS file format.")?;
    let input_lts = read_explicit_lts(&args.filename, format, args.tau.clone().unwrap_or_default(), timing)?;

    let output_format = if let Some(output) = &args.output {
        guess_lts_format_from_extension(output, args.output_format).ok_or("Unknown LTS file format.")?
    } else if let Some(format) = args.output_format {
        format
    } else {
        return Err("Either output path or output file format must be specified.".into());
    };

    match input_lts {
        GenericLts::Aut(lts) => match output_format {
            LtsFormat::Bcg => {
                if let Some(path) = &args.output {
                    write_bcg(&lts, path)?;
                } else {
                    return Err("Output path must be specified when writing BCG files.".into());
                }
            }
            LtsFormat::Aut => {
                return Err("Conversion from AUT to AUT is not useful.".into());
            }
            _ => {
                return Err(format!("Conversion to {output_format:?} format is not yet implemented.").into());
            }
        },
        GenericLts::Lts(lts) => match output_format {
            LtsFormat::Aut => {
                if let Some(path) = &args.output {
                    write_aut(&mut File::create(path)?, &lts.relabel(|label| label.to_string()))?;
                } else {
                    write_aut(&mut stdout(), &lts.relabel(|label| label.to_string()))?;
                }
            }
            LtsFormat::Bcg => {
                if let Some(path) = &args.output {
                    write_bcg(&lts.relabel(|label| label.to_string()), path)?;
                } else {
                    return Err("Output path must be specified when writing BCG files.".into());
                }
            }
            LtsFormat::Lts => {
                return Err("Conversion from LTS to LTS is not useful.".into());
            }
        },
        GenericLts::Bcg(lts) => match output_format {
            LtsFormat::Aut => {
                if let Some(path) = &args.output {
                    write_aut(&mut File::create(path)?, &lts)?;
                } else {
                    write_aut(&mut stdout(), &lts)?;
                }
            }
            _ => {
                return Err(format!("Conversion to {output_format:?}LTS format is not yet implemented.").into());
            }
        },
    }

    Ok(())
}
