use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use log::info;

use log::trace;
use merc_stark::DefKind;
use merc_stark::StarkSpecification;
use merc_stark::UntypedStarkSpecification;
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
}

#[derive(clap::Args, Debug)]
struct CheckArgs {
    /// The STARK specification to check.
    #[arg(value_name = "SPEC")]
    specification: PathBuf,

    /// Print every declaration in the specification with its inferred type.
    #[arg(long)]
    print_symbols: bool,
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
            Commands::Check(args) => {
                let spec = load_specification(&args.specification, timing)?;

                if args.print_symbols {
                    print_symbols(&spec);
                }

                info!("{} is a valid STARK specification", args.specification.display());
            }
        }
    }

    Ok(())
}

/// Reads `path` into an [UntypedStarkSpecification] and checks it into a
/// [StarkSpecification].
///
/// Diagnostics are rendered against the source text here rather than being
/// propagated as a plain error, since a bare `Diagnostics` has no way to show
/// the offending lines — the whole point of the spans it carries.
fn load_specification(path: &Path, timing: &Timing) -> Result<StarkSpecification, MercError> {
    let source =
        read_to_string(path).map_err(|err| MercError::from(format!("cannot read {}: {err}", path.display())))?;

    let untyped = timing.measure("parsing", || UntypedStarkSpecification::parse(&source))?;
    trace!("AST: {:#?}", untyped);

    timing
        .measure("resolving and type checking", || untyped.check())
        .map_err(|diagnostics| {
            let count = diagnostics.items().len();
            let plural = if count == 1 { "error" } else { "errors" };

            MercError::from(format!(
                "{count} {plural} in {}\n\n{}",
                path.display(),
                diagnostics.render(&source)
            ))
        })
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
