use std::io;
use std::process::Command;
use std::process::ExitStatus;

use log::trace;

/// Runs the given command and logs the command line before executing it, returning the exit status.
pub fn traced_command(command: &mut Command) -> io::Result<ExitStatus> {
    trace!(
        "{} {}",
        command.get_program().to_string_lossy(),
        command
            .get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
    command.status()
}
