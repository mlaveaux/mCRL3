use std::fs::File;
use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use clap::Subcommand;
use clap::ValueEnum;
use log::LevelFilter;
use mcrl3_rec_tests::load_rec_from_file;
use mcrl3_unsafety::print_allocator_metrics;
use mcrl3_utilities::MCRL3Error;
use mcrl3_version::Version;
use mcrl3rewrite::Rewriter;
use mcrl3rewrite::rewrite_rec;
use trs_format::TrsFormatter;

mod trs_format;

#[derive(clap::Parser, Debug)]
#[command(name = "Maurice Laveaux", about = "A command line rewriting tool")]
struct Cli {
    #[arg(long, default_value_t = false, help = "Print the version of this tool")]
    version: bool,

    #[arg(short, long, default_value_t = Verbosity::Quiet, help = "Sets the verbosity of the logger")]
    verbosity: Verbosity,

    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Rewrite(RewriteArgs),
    Convert(ConvertArgs),
}

#[derive(ValueEnum, Debug, Clone)]
enum Verbosity {
    Quiet,
    Verbose,
    Debug,
    Trace,
}

impl ToString for Verbosity {
    fn to_string(&self) -> String {
        match self {
            Verbosity::Quiet => "quiet".to_string(),
            Verbosity::Verbose => "verbose".to_string(),
            Verbosity::Debug => "debug".to_string(),
            Verbosity::Trace => "trace".to_string(),
        }
    }
}

impl Verbosity {
    fn log_level_filter(&self) -> LevelFilter {
        match self {
            Verbosity::Quiet => LevelFilter::Off,
            Verbosity::Verbose => LevelFilter::Info,
            Verbosity::Debug => LevelFilter::Debug,
            Verbosity::Trace => LevelFilter::Trace,
        }
    }
}


#[derive(clap::Args, Debug)]
#[command(about = "Rewrite mCRL2 data specifications and REC files")]
struct RewriteArgs {
    rewriter: Rewriter,

    #[arg(value_name = "SPEC")]
    specification: String,

    #[arg(help = "File containing the terms to be rewritten.")]
    terms: Option<String>,

    #[arg(long = "output", default_value_t = false, help = "Print the rewritten term(s)")]
    output: bool,
}

#[derive(clap::Args, Debug)]
#[command(about = "Convert input rewrite system to the TRS format")]
struct ConvertArgs {
    #[arg(value_name = "SPEC")]
    specification: String,

    output: String,
}

fn main() -> Result<ExitCode, MCRL3Error> {
    let cli = Cli::parse();
    
    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    if cli.version {
        eprintln!("{}", Version);
    }

    if let Some(command) = cli.commands {
        match command {
            Commands::Rewrite(args) => {
                if args.specification.ends_with(".rec") {
                    assert!(args.terms.is_none());
                    rewrite_rec(args.rewriter, &args.specification, args.output)?;
                }
            }
            Commands::Convert(args) => {
                if args.specification.ends_with(".rec") {
                    // Read the data specification
                    let (spec_text, _) = load_rec_from_file(args.specification.into())?;
                    let spec = spec_text.to_rewrite_spec();

                    let mut output = File::create(args.output)?;
                    write!(output, "{}", TrsFormatter::new(&spec))?;
                }
            }
        }
    }

    print_allocator_metrics();
    Ok(ExitCode::SUCCESS)
}
