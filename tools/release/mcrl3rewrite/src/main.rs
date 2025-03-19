use std::error::Error;
use std::process::ExitCode;

use clap::Parser;
#[cfg(feature = "measure-allocs")]
use log::info;

use mcrl3rewrite::Rewriter;
use mcrl3rewrite::rewrite_rec;

mod trs_format;

#[cfg(feature = "measure-allocs")]
#[global_allocator]
static MEASURE_ALLOC: unsafety::AllocCounter = unsafety::AllocCounter;

#[cfg(not(target_env = "msvc"))]
#[cfg(not(feature = "measure-allocs"))]
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

fn main() -> Result<ExitCode, Box<dyn Error>> {
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
            // Read the data specification
            // let data_spec_text = fs::read_to_string(args.specification)?;
            // let data_spec = DataSpecification::new(&data_spec_text)?;

            // let spec: RewriteSpecification = data_spec.into();

            // // Check if the lhs only contain constructor sorts.
            // for rule in &spec.rewrite_rules {
            //     for _t in rule.lhs.iter() {
            //         //let cons = data_spec.constructors(DataExpressionRef::from(t).sort());
            //     }
            // }

            // let mut output = File::create(args.output)?;
            // write!(output, "{}", TrsFormatter::new(&spec))?;
        }
    }

    #[cfg(feature = "measure-allocs")]
    info!("Allocations: {}", MEASURE_ALLOC.number_of_allocations());

    Ok(ExitCode::SUCCESS)
}
