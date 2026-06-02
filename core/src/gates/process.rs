use crate::error::process::ProcessError;
use crate::gates::result::ExecutionResult;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

pub fn execute_with_timeout(
    mut command: Command,
    gate_num: u8,
    success_message: &str,
    failure_message: &str,
    timeout_duration: Duration,
) -> Result<ExecutionResult, ProcessError> {
    let start_instant = Instant::now();

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ProcessError::SpawnFailed { source })?;

    let mut stdout_stream = child.stdout.take().unwrap();
    let mut stderr_stream = child.stderr.take().unwrap();

    let stdout_worker = thread::spawn(move || -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = Vec::new();
        stdout_stream.read_to_end(&mut buffer)?;
        Ok(buffer)
    });

    let stderr_worker = thread::spawn(move || -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = Vec::new();
        stderr_stream.read_to_end(&mut buffer)?;
        Ok(buffer)
    });

    let exit_status = match child.wait_timeout(timeout_duration) {
        Ok(Some(status)) => status,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ProcessError::Timeout {
                duration_ms: timeout_duration.as_millis() as u64,
            });
        }
        Err(source) => return Err(ProcessError::WaitFailed { source }),
    };

    let elapsed_ms = start_instant.elapsed().as_millis() as u64;
    let exit_code = exit_status.code().unwrap_or(-1);

    let stdout_buffer = stdout_worker
        .join()
        .map_err(|_| ProcessError::WaitFailed {
            source: std::io::Error::new(std::io::ErrorKind::Other, "stdout reader panicked thread state"),
        })?
        .map_err(|source| ProcessError::WaitFailed { source })?;

    let stderr_buffer = stderr_worker
        .join()
        .map_err(|_| ProcessError::WaitFailed {
            source: std::io::Error::new(std::io::ErrorKind::Other, "stderr reader panicked thread state"),
        })?
        .map_err(|source| ProcessError::WaitFailed { source })?;

    if exit_status.success() {
        return Ok(ExecutionResult::pass(
            gate_num,
            success_message,
            exit_code,
            elapsed_ms,
            stdout_buffer,
            stderr_buffer,
        ));
    }

    Ok(ExecutionResult::fail(
        gate_num,
        failure_message,
        exit_code,
        elapsed_ms,
        stdout_buffer,
        stderr_buffer,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    #[ignore]
    fn test_process_timeout() {
        let mut cmd = Command::new("sleep");
        cmd.arg("10");

        let result = execute_with_timeout(
            cmd,
            99,
            "pass",
            "fail",
            Duration::from_millis(100)
        );

        assert!(matches!(
            result,
            Err(ProcessError::Timeout { duration_ms: 100 })
        ));
    }
}
