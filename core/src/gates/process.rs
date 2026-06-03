use crate::error::process::ProcessError;
use crate::gates::result::ExecutionResult;
use crate::gates::sandbox::apply_sandbox_policy;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::thread::JoinHandle;
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

    apply_sandbox_policy(&mut command);
    configure_process_group(&mut command);

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
            terminate_process_tree(&mut child);
            let _ = collect_output(stdout_worker, "stdout")?;
            let _ = collect_output(stderr_worker, "stderr")?;
            return Err(ProcessError::Timeout {
                duration_ms: timeout_duration.as_millis() as u64,
            });
        }
        Err(source) => return Err(ProcessError::WaitFailed { source }),
    };

    let elapsed_ms = start_instant.elapsed().as_millis() as u64;
    let exit_code = exit_status.code().unwrap_or(-1);

    let stdout_buffer = collect_output(stdout_worker, "stdout")?;
    let stderr_buffer = collect_output(stderr_worker, "stderr")?;

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

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

fn terminate_process_tree(child: &mut Child) {
    terminate_process_group(child);
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(unix)]
fn terminate_process_group(child: &Child) {
    const SIGKILL: i32 = 9;
    unsafe extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }

    let process_group_id = -(child.id() as i32);
    unsafe {
        let _ = kill(process_group_id, SIGKILL);
    }
}

#[cfg(not(unix))]
fn terminate_process_group(_child: &Child) {}

fn collect_output(
    worker: JoinHandle<Result<Vec<u8>, std::io::Error>>,
    stream_name: &str,
) -> Result<Vec<u8>, ProcessError> {
    worker
        .join()
        .map_err(|_| ProcessError::WaitFailed {
            source: std::io::Error::other(format!("{stream_name} reader panicked thread state")),
        })?
        .map_err(|source| ProcessError::WaitFailed { source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    #[test]
    fn execute_with_timeout_uses_sandbox_environment() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg("printf '%s:%s:%s' \"${HTTP_PROXY-unset}\" \"$CARGO_NET_OFFLINE\" \"$GIT_TERMINAL_PROMPT\"");
        cmd.env("HTTP_PROXY", "http://proxy.example");

        let result = execute_with_timeout(cmd, 99, "pass", "fail", Duration::from_secs(1)).unwrap();

        assert_eq!(result.stdout, b"unset:true:0");
    }

    #[test]
    fn test_process_timeout() {
        let mut cmd = Command::new("sleep");
        cmd.arg("10");

        let result = execute_with_timeout(cmd, 99, "pass", "fail", Duration::from_millis(100));

        assert!(matches!(
            result,
            Err(ProcessError::Timeout { duration_ms: 100 })
        ));
    }

    #[test]
    #[cfg(unix)]
    fn timeout_kills_descendant_processes() {
        let marker_path = std::env::temp_dir().join("acceptability-engine-timeout-marker");
        let _ = fs::remove_file(&marker_path);

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(format!(
            "(sleep 1; printf done > {}) & wait",
            marker_path.to_string_lossy()
        ));

        let result = execute_with_timeout(cmd, 99, "pass", "fail", Duration::from_millis(100));
        thread::sleep(Duration::from_millis(1200));

        assert!(matches!(
            result,
            Err(ProcessError::Timeout { duration_ms: 100 })
        ));
        assert!(!marker_path.exists());
    }
}
