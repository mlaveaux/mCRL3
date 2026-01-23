use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use merc_rec_tests::load_rec_from_file;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;

use merc_rewrite::Rewriter;
use merc_rewrite::rewrite_rec;

mod trs_format;

use merc_utilities::Timing;
pub use trs_format::*;

/// A command line rewriting tool
#[derive(clap::Parser, Debug)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(flatten)]
    version: VersionFlag,

    #[command(flatten)]
    verbosity: VerbosityFlag,

    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Rewrite(RewriteArgs),
    Convert(ConvertArgs),
}

/// Rewrite mCRL2 data specifications and REC files
#[derive(clap::Args, Debug)]
#[command()]
struct RewriteArgs {
    rewriter: Rewriter,

    /// The REC specification that contains the rewrite rules.
    #[arg(value_name = "SPEC")]
    specification: PathBuf,

    /// File containing the terms to be rewritten.
    terms: Option<String>,

    /// Print the rewritten term(s)
    #[arg(long)]
    output: bool,
}

/// Convert input rewrite system to the TRS format"
#[derive(clap::Args, Debug)]
#[command()]
struct ConvertArgs {
    /// The REC specification that contains the rewrite rules.
    #[arg(value_name = "SPEC")]
    specification: PathBuf,

    output: String,
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

    let timing = Timing::new();

    if let Some(command) = cli.commands {
        match command {
            Commands::Rewrite(args) => {
                if args.specification.extension() == Some(OsStr::new("rec")) {
                    assert!(args.terms.is_none());
                    rewrite_rec(args.rewriter, &args.specification, args.output, &timing)?;
                }
            }
            Commands::Convert(args) => {
                if args.specification.extension() == Some(OsStr::new("rec")) {
                    // Read the data specification
                    let (spec_text, _) = load_rec_from_file(&args.specification)?;
                    let spec = spec_text.to_rewrite_spec();

                    let mut output = File::create(args.output)?;
                    write!(output, "{}", TrsFormatter::new(&spec))?;
                }
            }
        }
    }

    timing.print();

    print_allocator_metrics();
    Ok(ExitCode::SUCCESS)
}
