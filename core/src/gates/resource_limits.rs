use crate::error::ProcessError;
use std::process::Command;
use std::time::Duration;

const ADDRESS_SPACE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const PROCESS_COUNT: u64 = 256;

#[cfg(test)]
static APPLY_RESOURCE_LIMITS_CALLS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(all(unix, target_os = "linux"))]
type RlimitResource = libc::__rlimit_resource_t;

#[cfg(all(unix, not(target_os = "linux")))]
type RlimitResource = libc::c_int;

pub fn apply_resource_limits(
    command: &mut Command,
    timeout_duration: Duration,
) -> Result<(), ProcessError> {
    #[cfg(test)]
    record_apply_resource_limits_call();
    apply_platform_resource_limits(command, timeout_duration)
}

#[cfg(test)]
fn record_apply_resource_limits_call() {
    APPLY_RESOURCE_LIMITS_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn reset_apply_resource_limits_calls() {
    APPLY_RESOURCE_LIMITS_CALLS.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn apply_resource_limits_calls() -> usize {
    APPLY_RESOURCE_LIMITS_CALLS.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(unix)]
fn apply_platform_resource_limits(
    command: &mut Command,
    timeout_duration: Duration,
) -> Result<(), ProcessError> {
    use std::os::unix::process::CommandExt;

    let cpu_seconds = cpu_limit_seconds(timeout_duration);
    unsafe {
        command.pre_exec(move || {
            set_limit_if_supported(libc::RLIMIT_CPU, cpu_seconds)?;
            set_limit_if_supported(libc::RLIMIT_AS, ADDRESS_SPACE_BYTES)?;
            set_limit_if_supported(libc::RLIMIT_NPROC, PROCESS_COUNT)
        });
    }
    Ok(())
}

#[cfg(not(unix))]
fn apply_platform_resource_limits(
    _command: &mut Command,
    _timeout_duration: Duration,
) -> Result<(), ProcessError> {
    Ok(())
}

#[cfg(unix)]
fn set_limit(resource: RlimitResource, value: u64) -> std::io::Result<()> {
    let limit = libc::rlimit {
        rlim_cur: value as libc::rlim_t,
        rlim_max: value as libc::rlim_t,
    };
    let result = unsafe { libc::setrlimit(resource, &limit) };
    if result == 0 {
        return Ok(());
    }
    Err(std::io::Error::last_os_error())
}

#[cfg(unix)]
fn set_limit_if_supported(resource: RlimitResource, value: u64) -> std::io::Result<()> {
    match set_limit(resource, value) {
        Ok(()) => Ok(()),
        Err(error) if is_unsupported_limit(&error) => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn is_unsupported_limit(error: &std::io::Error) -> bool {
    matches!(error.raw_os_error(), Some(libc::EINVAL))
}

#[cfg(unix)]
fn cpu_limit_seconds(timeout_duration: Duration) -> u64 {
    timeout_duration.as_secs().max(1)
}
