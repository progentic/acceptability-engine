use crate::error::ProcessError;
use crate::gates::resource_limits::apply_resource_limits;
use std::ffi::{OsStr, OsString};
use std::process::{Command, ExitStatus};
use std::time::Duration;

pub const RUNNER_FLAG: &str = "--sandbox-runner";
const RUNNER_SEPARATOR: &str = "--";
const RUNNER_TIMEOUT_ENV: &str = "AH_SANDBOX_TIMEOUT_MS";

pub fn runner_command(command: &Command) -> Result<Command, ProcessError> {
    #[cfg(test)]
    {
        return direct_command(command);
    }

    #[cfg(not(test))]
    {
        sandbox_runner_command(command)
    }
}

#[cfg(not(test))]
fn sandbox_runner_command(command: &Command) -> Result<Command, ProcessError> {
    let mut runner = Command::new(runner_program()?);
    runner.arg(RUNNER_FLAG).arg(RUNNER_SEPARATOR);
    runner.arg(command.get_program());
    runner.args(command.get_args());
    if let Some(current_dir) = command.get_current_dir() {
        runner.current_dir(current_dir);
    }
    Ok(runner)
}

pub fn set_runner_timeout(command: &mut Command, timeout_duration: Duration) {
    command.env(RUNNER_TIMEOUT_ENV, timeout_duration.as_millis().to_string());
}

#[cfg(test)]
fn direct_command(command: &Command) -> Result<Command, ProcessError> {
    let mut direct = Command::new(command.get_program());
    direct.args(command.get_args());
    if let Some(current_dir) = command.get_current_dir() {
        direct.current_dir(current_dir);
    }
    Ok(direct)
}

pub fn run_from_args(args: impl IntoIterator<Item = OsString>) -> i32 {
    match run_sandboxed_command(args) {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            eprintln!("{error}");
            125
        }
    }
}

fn run_sandboxed_command(
    args: impl IntoIterator<Item = OsString>,
) -> Result<ExitStatus, ProcessError> {
    let mut args = args.into_iter();
    discard_separator(&mut args)?;
    let program = runner_program_arg(&mut args)?;
    let mut command = Command::new(program);
    command.args(args);
    apply_resource_limits(&mut command, runner_timeout())?;
    command
        .status()
        .map_err(|source| ProcessError::SpawnFailed { source })
}

fn discard_separator(args: &mut impl Iterator<Item = OsString>) -> Result<(), ProcessError> {
    match args.next().as_deref() {
        Some(value) if value == OsStr::new(RUNNER_SEPARATOR) => Ok(()),
        _ => Err(ProcessError::RunnerLaunchFailed {
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "sandbox runner missing argument separator",
            ),
        }),
    }
}

fn runner_program_arg(args: &mut impl Iterator<Item = OsString>) -> Result<OsString, ProcessError> {
    args.next().ok_or_else(|| ProcessError::RunnerLaunchFailed {
        source: std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "sandbox runner missing command program",
        ),
    })
}

#[cfg(not(test))]
fn runner_program() -> Result<std::path::PathBuf, ProcessError> {
    std::env::current_exe().map_err(|source| ProcessError::RunnerLaunchFailed { source })
}

fn runner_timeout() -> Duration {
    std::env::var(RUNNER_TIMEOUT_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_secs(1))
}
