use clap::Parser;
use clap::ValueEnum;
use mcrl3_syntax::UntypedProcessSpecification;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::{self};
use std::process::ExitCode;

#[derive(ValueEnum, Debug, Clone)]
enum Method {
    Regular,
    Regular2,
    Stack,
}

#[derive(Parser, Debug)]
#[command(name = "mcrl22lps")]
#[command(author = "Jan Friso Groote")]
#[command(about = "Translate an mCRL2 specification to an LPS")]
#[command(
    long_about = "Linearises the mCRL2 specification in INFILE and writes the resulting LPS to OUTFILE. If OUTFILE is not present, stdout is used. If INFILE is not present, stdin is used."
)]
struct Args {
    input: Option<String>,
    output: Option<String>,

    /// Input file (stdin if not present)
    no_cluster: bool,

    /// Do not apply alphabet reductions
    #[arg(short = 'z', long)]
    no_alpha: bool,

    /// Use enumerated types for state variables
    #[arg(short = 'w', long)]
    newstate: bool,

    /// Use binary case functions when clustering
    #[arg(short = 'b', long)]
    binary: bool,

    /// Use process-based state variable names
    #[arg(short = 'a', long)]
    statenames: bool,

    /// Do not rewrite data terms while linearising
    #[arg(short = 'o', long)]
    no_rewrite: bool,

    /// Do not use global variables for don't care values
    #[arg(short = 'f', long)]
    no_globvars: bool,

    /// Avoid applying sum elimination in parallel composition
    #[arg(short = 'm', long)]
    no_sumelm: bool,

    /// Avoid removing spurious delta summands
    #[arg(short = 'g', long)]
    no_deltaelm: bool,

    /// Add true->delta summands (default)
    #[arg(short = 'D', long)]
    delta: bool,

    /// Preserve timed information
    #[arg(short = 'T', long)]
    timed: bool,

    /// Do not apply constant elimination
    #[arg(long)]
    no_constelm: bool,

    /// Check syntax and static semantics only
    #[arg(short = 'e', long)]
    check_only: bool,

    /// Balance input expressions before linearising
    #[arg(long)]
    balance_summands: bool,

    lin_method: Method,
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let args = Args::parse();

    // Read input specification
    let _spec = if let Some(path) = args.input {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        UntypedProcessSpecification::parse(&content)?
    } else {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        UntypedProcessSpecification::parse(&content)?
    };

    // If check-only, exit after parsing
    if args.check_only {
        println!("Input contains a well-formed mCRL2 specification");
        return Ok(ExitCode::SUCCESS);
    }

    // Perform linearisation
    // let lps = linearise(spec, options)?;

    // // Write output
    // if let Some(path) = args.output {
    //     lps.save(&path)?;
    // } else {
    //     lps.write_to(io::stdout())?;
    // }

    Ok(ExitCode::SUCCESS)
}
