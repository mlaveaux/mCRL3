use std::fs::File;
use std::io::BufWriter;
use std::io::stdout;
use std::process::ExitCode;

use clap::Parser;
use clap::ValueEnum;

use mcrl3_lts::read_aut;
use mcrl3_lts::write_aut;
use mcrl3_reduction::IndexedPartition;
use mcrl3_reduction::branching_bisim_sigref;
use mcrl3_reduction::branching_bisim_sigref_naive;
use mcrl3_reduction::quotient_lts;
use mcrl3_reduction::strong_bisim_sigref;
use mcrl3_reduction::strong_bisim_sigref_naive;

use mcrl3_unsafety::print_allocator_metrics;
use mcrl3_utilities::MCRL3Error;
use mcrl3_utilities::Timing;

#[derive(Clone, Debug, ValueEnum)]
enum Equivalence {
    StrongBisim,
    StrongBisimNaive,
    BranchingBisim,
    BranchingBisimNaive,
}

#[derive(clap::Parser, Debug)]
#[command(name = "Maurice Laveaux", about = "A command line rewriting tool")]
struct Cli {
    equivalence: Equivalence,

    filename: String,

    output: Option<String>,

    #[arg(short, long)]
    tau: Option<Vec<String>>,

    #[arg(long)]
    time: bool,
}

fn main() -> Result<ExitCode, MCRL3Error> {
    env_logger::init();

    let cli = Cli::parse();

    let file = File::open(cli.filename)?;

    let mut timing = Timing::new();
    let lts = read_aut(&file, cli.tau.unwrap_or_default())?;

    let (preprocessed_lts, partition) = match cli.equivalence {
        Equivalence::StrongBisim => strong_bisim_sigref(lts, &mut timing),
        Equivalence::StrongBisimNaive => strong_bisim_sigref_naive(lts, &mut timing),
        Equivalence::BranchingBisim => branching_bisim_sigref(lts, &mut timing),
        Equivalence::BranchingBisimNaive => branching_bisim_sigref_naive(lts, &mut timing),
        _ => unreachable!(),
    };

    let mut quotient_time = timing.start("quotient");
    let quotient_lts = quotient_lts(
        &preprocessed_lts,
        &partition,
        matches!(cli.equivalence, Equivalence::BranchingBisim)
            || matches!(cli.equivalence, Equivalence::BranchingBisimNaive),
    );
    if let Some(file) = cli.output {
        let mut writer = BufWriter::new(File::create(file)?);
        write_aut(&mut writer, &quotient_lts)?;
    } else {
        write_aut(&mut stdout(), &quotient_lts)?;
    }
    quotient_time.finish();

    if cli.time {
        timing.print();
    }

    print_allocator_metrics();
    Ok(ExitCode::SUCCESS)
}
