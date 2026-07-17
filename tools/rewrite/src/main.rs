use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use log::warn;

use merc_rec_tests::load_rec_from_file;
use merc_rewrite::Rewriter;
use merc_rewrite::rewrite_rec;
use merc_sabre::RewriteSpecification;
use merc_syntax::UntypedDataSpecification;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_tools::report_error;
use merc_typecheck::DataSpecification;
use merc_unsafety::print_allocator_metrics;
use merc_utilities::MercError;
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

    #[arg(long, global = true)]
    timings: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Rewrite a term using the rewrite rules specified in a REC file or mCRL2 specification
    Rewrite(RewriteArgs),

    /// Convert a REC specification to the TRS format, which is the format used
    /// by the term rewrite system termination checking tool called AProVE.
    Convert(ConvertArgs),
}

#[derive(Debug, clap::ValueEnum, Clone)]
enum Format {
    /// The REC format, which is the native format of this tool.
    Rec,
    /// The mCRL2 format, which is the format used by the mCRL2 toolset.
    Mcrl2,
}

#[derive(clap::Args, Debug)]
struct RewriteArgs {
    rewriter: Rewriter,

    /// The REC specification that contains the rewrite rules.
    #[arg(value_name = "SPEC")]
    specification: PathBuf,

    /// File containing the terms to be rewritten.
    terms: Option<String>,

    #[arg(long, value_enum)]
    format: Option<Format>,

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
    let result = handle_command(cli.commands, &timing);

    if cli.timings {
        timing.print();
    }

    print_allocator_metrics();
    report_error(result)
}

fn handle_command(commands: Option<Commands>, timing: &Timing) -> Result<(), MercError> {
    if let Some(command) = commands {
        match command {
            Commands::Rewrite(args) => {
                let format = if let Some(format) = args.format {
                    format
                } else if args.specification.extension() == Some(OsStr::new("rec")) {
                    Format::Rec
                } else if args.specification.extension() == Some(OsStr::new("mcrl2")) {
                    Format::Mcrl2
                } else {
                    return Err("Unsupported file extension for rewriting, expected .rec or .mcrl2".into());
                };

                match format {
                    Format::Rec => {
                        if args.terms.is_some() {
                            warn!(
                                "The --terms option is currently ignored when rewriting REC specifications, the terms are taken from the REC spec."
                            );
                        }

                        let (syntax_spec, syntax_terms) = load_rec_from_file(&args.specification)?;

                        let spec = syntax_spec.to_rewrite_spec();

                        rewrite_rec(args.rewriter, &spec, &syntax_terms, args.output, timing)?;
                    }
                    Format::Mcrl2 => {
                        if args.terms.is_some() {
                            warn!(
                                "The --terms option is not yet supported when rewriting mCRL2 specifications; only the rule count is reported."
                            );
                        }

                        let source = std::fs::read_to_string(&args.specification)?;
                        let untyped_spec = UntypedDataSpecification::parse(&source)?;

                        let mut data_spec = match DataSpecification::from_untyped(untyped_spec) {
                            Ok(data_spec) => data_spec,
                            Err(err) => return Err(err.render(&source).into()),
                        };

                        let mcrl2_spec = data_spec.lower_data_specification();
                        let spec = RewriteSpecification::from_data_specification(&mcrl2_spec);

                        if args.output {
                            warn!(
                                "The --output option is not yet supported when rewriting mCRL2 specifications; only the rule count is reported."
                            );
                        }
                        println!("Loaded {} rewrite rule(s)", spec.rewrite_rules().len());
                    }
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

    Ok(())
}
