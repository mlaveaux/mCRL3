use std::fs::File;
use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use mcrl3_rec_tests::load_REC_from_file;
use mcrl3_utilities::MCRL3Error;
use mcrl3rewrite::Rewriter;
use mcrl3rewrite::rewrite_rec;
use trs_format::TrsFormatter;

mod trs_format;

#[cfg(feature = "mcrl3_measure-allocs")]
#[global_allocator]
static MEASURE_ALLOC: mcrl3_unsafety::AllocCounter = mcrl3_unsafety::AllocCounter;
#[cfg(feature = "mcrl3_measure-allocs")]
use log::info;

#[cfg(not(target_env = "msvc"))]
#[cfg(not(feature = "mcrl3_measure-allocs"))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(clap::Parser, Debug)]
#[command(name = "Maurice Laveaux", about = "A command line rewriting tool")]
pub(crate) enum Cli {
    Rewrite(RewriteArgs),
    Convert(ConvertArgs),
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
    env_logger::init();

    let cli = Cli::parse();

    match cli {
        Cli::Rewrite(args) => {
            if args.specification.ends_with(".rec") {
                assert!(args.terms.is_none());
                rewrite_rec(args.rewriter, &args.specification, args.output)?;
            }
        }
        Cli::Convert(args) => {
            if args.specification.ends_with(".rec") {
                // Read the data specification
                let (spec_text, _) = load_REC_from_file(args.specification.into())?;
                let spec = spec_text.to_rewrite_spec();

                let mut output = File::create(args.output)?;
                write!(output, "{}", TrsFormatter::new(&spec))?;
            }
        }
    }

    #[cfg(feature = "mcrl3_measure-allocs")]
    info!("Allocations: {}", MEASURE_ALLOC.number_of_allocations());

    Ok(ExitCode::SUCCESS)
}
