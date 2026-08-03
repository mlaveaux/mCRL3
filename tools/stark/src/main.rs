use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use log::info;

use log::trace;
use merc_stark::DefKind;
use merc_stark::Diagnostics;
use merc_stark::StarkSpecification;
use merc_stark::UntypedStarkSpecification;
use merc_stark::eval::Analysis;
use merc_stark::eval::AnalysisOptions;
use merc_stark::eval::RecordingObserver;
use merc_stark::eval::Simulation;
use merc_stark::eval::TruthValue;
use merc_stark::ir::IrProgram;
use merc_stark::ir::SlotId;
use merc_stark::lower;
use merc_stark::value::Value;
use merc_tools::VerbosityFlag;
use merc_tools::Version;
use merc_tools::VersionFlag;
use merc_tools::report_error;
use merc_utilities::MercError;
use merc_utilities::Timing;

/// A command line tool for STARK specifications.
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
    /// Parses, resolves and type checks the given STARK specification, reporting every problem found.
    Check(CheckArgs),
    /// Runs a single trajectory of the specification for a fixed number of steps under a fixed seed.
    Simulate(SimulateArgs),
    /// Verifies the specification's `formula` declarations by robustness analysis, reporting each verdict.
    Verify(VerifyArgs),
}

#[derive(clap::Args, Debug)]
struct CheckArgs {
    /// The STARK specification to check.
    #[arg(value_name = "SPEC")]
    specification: PathBuf,

    /// Print every declaration in the specification with its inferred type.
    #[arg(long)]
    print_symbols: bool,

    /// Lower the checked specification to its IR program and print a summary of it.
    #[arg(long)]
    lower: bool,
}

#[derive(clap::Args, Debug)]
struct SimulateArgs {
    /// The STARK specification to simulate.
    #[arg(value_name = "SPEC")]
    specification: PathBuf,

    /// The number of macro-steps to run.
    #[arg(long, default_value_t = 50)]
    steps: u64,

    /// The seed the run is reproducible from.
    #[arg(long, default_value_t = 0)]
    seed: u64,

    /// Print the state after every step, rather than only the final state.
    #[arg(long)]
    trajectory: bool,

    /// Decimal places to round real-valued state to when printing.
    #[arg(long, default_value_t = 2)]
    precision: usize,
}

#[derive(clap::Args, Debug)]
struct VerifyArgs {
    /// The STARK specification to verify.
    #[arg(value_name = "SPEC")]
    specification: PathBuf,

    /// The seed the analysis is reproducible from.
    #[arg(long, default_value_t = 0)]
    seed: u64,

    /// The time step at which each formula is evaluated.
    #[arg(long, default_value_t = 0)]
    step: usize,

    /// Samples per step in the reference evolution sequence — larger is more accurate and linearly more expensive.
    #[arg(long, default_value_t = AnalysisOptions::default().sample_size)]
    samples: usize,

    /// Perturbed samples drawn per reference sample.
    #[arg(long, default_value_t = AnalysisOptions::default().scale)]
    scale: usize,

    /// Bootstrap replicas behind the three-valued confidence interval.
    #[arg(long = "bootstrap-replicas", default_value_t = AnalysisOptions::default().bootstrap_replicas)]
    bootstrap_replicas: usize,

    /// The standard-normal quantile the confidence interval spans.
    #[arg(long, default_value_t = AnalysisOptions::default().quantile)]
    quantile: f64,

    /// Use the two-valued boolean semantics instead of the three-valued one.
    #[arg(long)]
    boolean: bool,
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

    report_error(result)
}

fn handle_command(commands: Option<Commands>, timing: &Timing) -> Result<(), MercError> {
    if let Some(command) = commands {
        match command {
            Commands::Check(args) => check(args, timing)?,
            Commands::Simulate(args) => simulate(args, timing)?,
            Commands::Verify(args) => verify(args, timing)?,
        }
    }

    Ok(())
}

/// Checks a specification, optionally printing its symbols and/or a summary of
/// its lowered IR program.
fn check(args: CheckArgs, timing: &Timing) -> Result<(), MercError> {
    let source = read_source(&args.specification)?;
    let spec = check_specification(&source, &args.specification, timing)?;

    if args.print_symbols {
        print_symbols(&spec);
    }

    if args.lower {
        let program = lower_specification(&spec, &source, &args.specification, timing)?;
        print_ir_summary(&program);
    }

    info!("{} is a valid STARK specification", args.specification.display());
    Ok(())
}

/// Runs one trajectory of the specification and prints its state.
fn simulate(args: SimulateArgs, timing: &Timing) -> Result<(), MercError> {
    let source = read_source(&args.specification)?;
    let spec = check_specification(&source, &args.specification, timing)?;
    let program = lower_specification(&spec, &source, &args.specification, timing)?;

    let mut simulation = Simulation::new(&program, args.seed)
        .map_err(|err| MercError::from(format!("cannot initialise simulation: {err}")))?;
    let mut observer = RecordingObserver::default();

    if let Err(err) = timing.measure("simulation", || simulation.run(args.steps, &mut observer)) {
        return Err(MercError::from(format!(
            "simulation failed at step {}: {err}",
            simulation.step_count() + 1
        )));
    }

    print_trajectory(&program, &observer.trajectory, args.trajectory, args.precision);
    Ok(())
}

/// Verifies every `formula` declaration by robustness analysis, printing each
/// formula's verdict.
fn verify(args: VerifyArgs, timing: &Timing) -> Result<(), MercError> {
    let source = read_source(&args.specification)?;
    let spec = check_specification(&source, &args.specification, timing)?;
    let program = lower_specification(&spec, &source, &args.specification, timing)?;

    if program.formula_decls().is_empty() {
        info!("{} declares no formulas to verify", args.specification.display());
        return Ok(());
    }

    let options = AnalysisOptions {
        sample_size: args.samples,
        scale: args.scale,
        bootstrap_replicas: args.bootstrap_replicas,
        quantile: args.quantile,
    };
    let mut analysis = Analysis::new(&program, args.seed, options)
        .map_err(|err| MercError::from(format!("cannot initialise analysis: {err}")))?;

    let mut sequence = timing
        .measure("sampling", || analysis.sample())
        .map_err(|err| MercError::from(format!("cannot sample the system: {err}")))?;

    // One sequence is sampled once and reused across every formula — see the
    // module doc comment on `eval::Analysis`.
    for decl in program.formula_decls() {
        let verdict = timing.measure("verification", || {
            if args.boolean {
                analysis
                    .check_boolean(&mut sequence, args.step, decl.root)
                    .map(|value| if value { "true" } else { "false" }.to_string())
            } else {
                analysis
                    .check(&mut sequence, args.step, decl.root)
                    .map(|verdict| describe_truth(verdict).to_string())
            }
        });

        match verdict {
            Ok(verdict) => println!("{}: {verdict}", decl.name),
            Err(err) => return Err(MercError::from(format!("verifying `{}` failed: {err}", decl.name))),
        }
    }

    Ok(())
}

/// Reads `path` into memory, turning an I/O error into a [MercError].
fn read_source(path: &Path) -> Result<String, MercError> {
    read_to_string(path).map_err(|err| MercError::from(format!("cannot read {}: {err}", path.display())))
}

/// Parses and checks `source` into a [StarkSpecification].
///
/// Diagnostics are rendered against the source text here rather than being
/// propagated as a plain error, since a bare `Diagnostics` has no way to show
/// the offending lines — the whole point of the spans it carries.
fn check_specification(source: &str, path: &Path, timing: &Timing) -> Result<StarkSpecification, MercError> {
    let untyped = timing.measure("parsing", || UntypedStarkSpecification::parse(source))?;
    trace!("AST: {:#?}", untyped);

    timing
        .measure("resolving and type checking", || untyped.check())
        .map_err(|diagnostics| render_diagnostics(&diagnostics, source, path))
}

/// Lowers a checked specification to its [IrProgram], rendering any lowering
/// diagnostic against the source the same way [check_specification] does.
fn lower_specification(
    spec: &StarkSpecification,
    source: &str,
    path: &Path,
    timing: &Timing,
) -> Result<IrProgram, MercError> {
    timing
        .measure("lowering", || lower(spec))
        .map_err(|diagnostics| render_diagnostics(&diagnostics, source, path))
}

/// Renders a [Diagnostics] against `source` into the error a command returns.
fn render_diagnostics(diagnostics: &Diagnostics, source: &str, path: &Path) -> MercError {
    let count = diagnostics.items().len();
    let plural = if count == 1 { "error" } else { "errors" };

    MercError::from(format!(
        "{count} {plural} in {}\n\n{}",
        path.display(),
        diagnostics.render(source)
    ))
}

/// Prints every top-level declaration with the type checker's verdict on it.
fn print_symbols(spec: &StarkSpecification) {
    for (index, def) in spec.symbols().defs.iter().enumerate() {
        let id = merc_stark::DefId::new(index);

        // Functions carry a signature rather than a single type, and kinds like
        // components have neither, so what is worth printing differs per kind.
        if let Some(signature) = spec.types().signature_of(id) {
            let arguments = signature
                .arguments
                .iter()
                .map(|argument| argument.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            println!("{}: ({arguments}) -> {}", def.name, signature.return_type);
        } else if let Some(ty) = spec.types().type_of(id) {
            println!("{}: {ty}", def.name);
        } else {
            println!("{}: {}", def.name, describe(&def.kind));
        }
    }
}

/// Prints the size of a lowered IR program — how many slots, and how many of
/// each kind of declaration the evaluator will drive.
fn print_ir_summary(program: &IrProgram) {
    println!("variables:     {}", program.variables().len());
    println!("globals:       {}", program.globals().len());
    println!("functions:     {}", program.functions().len());
    println!("penalties:     {}", program.penalties().len());
    println!("components:    {}", program.components().len());
    println!("perturbations: {}", program.perturbation_decls().len());
    println!("distances:     {}", program.distance_decls().len());
    println!("formulas:      {}", program.formula_decls().len());
    println!("total slots:   {}", program.n_slots());
}

/// Prints a simulation's state as an aligned table: one column per variable,
/// and either every step's row (`full`) or only the final one. Reals are
/// rounded to `precision` decimals, since a full `f64` rendering makes the
/// columns far wider than a table meant for eyeballing trends needs.
fn print_trajectory(program: &IrProgram, trajectory: &[Vec<Value>], full: bool, precision: usize) {
    let names: Vec<&str> = (0..program.n_variables())
        .map(|index| program.slot(SlotId::new(index)).name.as_str())
        .collect();

    // Column widths accommodate the header name and every value printed under
    // it, so the columns line up regardless of how wide the values grow.
    let rows: &[Vec<Value>] = if full || trajectory.is_empty() {
        trajectory
    } else {
        &trajectory[trajectory.len() - 1..]
    };

    // Values are rendered to a `String` before being padded: `Value`'s
    // `Display` writes straight through with `write!`, so it ignores the
    // formatter's width flag and `{value:>width$}` would not pad at all.
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|value| render(value, precision)).collect())
        .collect();

    let mut widths: Vec<usize> = names.iter().map(|name| name.len()).collect();
    for row in &cells {
        for (column, value) in row.iter().enumerate() {
            widths[column] = widths[column].max(value.len());
        }
    }

    // The step column is only meaningful when more than one state is shown;
    // for the final state alone it would be a column of one.
    let step_width = if full {
        rows.len().to_string().len().max("step".len())
    } else {
        0
    };
    let step_column = |label: &str| -> String {
        if full {
            format!("{label:>step_width$}  ")
        } else {
            String::new()
        }
    };

    let header = names
        .iter()
        .enumerate()
        .map(|(column, name)| format!("{name:>width$}", width = widths[column]))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{}{header}", step_column("step"));

    for (step, row) in cells.iter().enumerate() {
        let line = row
            .iter()
            .enumerate()
            .map(|(column, value)| format!("{value:>width$}", width = widths[column]))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{}{line}", step_column(&(step + 1).to_string()));
    }
}

/// Renders a state value for the trajectory table, rounding a real to
/// `precision` decimals. Only reals are reformatted — an integer state
/// variable is exact, and padding it with decimals would suggest otherwise.
fn render(value: &Value, precision: usize) -> String {
    match value {
        Value::Real(real) => format!("{real:.precision$}"),
        other => other.to_string(),
    }
}

/// A human readable name for the kinds that have no type of their own.
fn describe(kind: &DefKind) -> &'static str {
    match kind {
        DefKind::Constant => "constant",
        DefKind::Parameter => "parameter",
        DefKind::Variable { .. } => "variable",
        DefKind::Function { .. } => "function",
        DefKind::Penalty => "penalty",
        DefKind::Component => "component",
        DefKind::TypeElement { .. } => "type element",
        DefKind::Type => "type",
        DefKind::Perturbation => "perturbation",
        DefKind::Distance => "distance",
        DefKind::Formula => "formula",
    }
}

/// A human readable rendering of a three-valued verdict.
fn describe_truth(verdict: TruthValue) -> &'static str {
    match verdict {
        TruthValue::True => "true",
        TruthValue::False => "false",
        TruthValue::Unknown => "unknown",
    }
}
