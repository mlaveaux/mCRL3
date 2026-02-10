use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use log::warn;

use merc_rec_tests::load_rec_from_file;
use merc_sabre::data_expr_to_term;
use merc_sabre::to_rewrite_spec;
use merc_syntax::DataExpr;
use merc_syntax::UntypedDataSpecification;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;
use merc_rewrite::Rewriter;
use merc_rewrite::rewrite_rec;
use merc_utilities::Timing;

mod trs_format;

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
    /// Rewrite a term using the rewrite rules specified in a REC file or mCRL2 specification
    Rewrite(RewriteArgs),
    
    /// Convert a REC specification to the TRS format
    Convert(ConvertArgs),
}

#[derive(clap::Args, Debug)]
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

#[derive(clap::Args, Debug)]
struct ConvertArgs {
    /// The REC specification that contains the rewrite rules.
    #[arg(value_name = "SPEC")]
    specification: PathBuf,

    /// The output file to write the TRS to.
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
                    if args.terms.is_some() {
                        warn!("The --terms option is currently ignored when rewriting REC specifications, the terms are taken from the REC spec.");
                    }

                    let (syntax_spec, syntax_terms) = load_rec_from_file(&args.specification)?;

                    let spec = syntax_spec.to_rewrite_spec();

                    rewrite_rec(args.rewriter, &spec, &syntax_terms, args.output, &timing)?;
                } else if args.specification.extension() == Some(OsStr::new("mcrl2")) {
                    let spec_text = std::fs::read_to_string(&args.specification)?;
                    let spec = UntypedDataSpecification::parse(&spec_text)?;

                    if let Some(term) = args.terms {
                        let data_expr = DataExpr::parse(&term)?;
                        let term = data_expr_to_term(&data_expr);

                        let rewrite_spec = to_rewrite_spec(&spec)?;
                        rewrite_rec(args.rewriter, &rewrite_spec, &[term.into()], args.output, &timing)?;
                    } else {
                        return Err("No terms provided for rewriting mCRL2 specification".into());
                    }
                } else {
                    return Err("Unsupported file extension for rewriting, expected .rec or .mcrl2".into());
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
