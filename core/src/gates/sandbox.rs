use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_COMMAND_PATH: &str = "/usr/bin:/bin:/usr/local/bin:/opt/homebrew/bin";
const SANDBOX_HOME_DIR: &str = ".acceptability-home";
const COMMAND_PATH_ENV: &str = "AH_COMMAND_PATH";

pub fn apply_sandbox_policy(command: &mut Command) {
    let home_dir = sandbox_home(command.get_current_dir());

    command.env_clear();
    command.env("PATH", command_path());
    command.env("HOME", home_dir);
    command.env("CARGO_NET_OFFLINE", "true");
    command.env("CARGO_TERM_COLOR", "never");
    command.env("GIT_TERMINAL_PROMPT", "0");
}

fn command_path() -> String {
    std::env::var(COMMAND_PATH_ENV).unwrap_or_else(|_| DEFAULT_COMMAND_PATH.to_string())
}

fn sandbox_home(current_dir: Option<&Path>) -> PathBuf {
    current_dir
        .map(|path| path.join(SANDBOX_HOME_DIR))
        .unwrap_or_else(std::env::temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn applies_minimal_sandbox_environment() {
        let mut command = Command::new("cargo");
        command.current_dir("/tmp/workspace");
        command.env("HTTP_PROXY", "http://proxy.example");

        apply_sandbox_policy(&mut command);

        assert_eq!(
            command_env(&command, "PATH"),
            Some(OsStr::new(DEFAULT_COMMAND_PATH))
        );
        assert_eq!(
            command_env(&command, "HOME"),
            Some(Path::new("/tmp/workspace/.acceptability-home").as_os_str())
        );
        assert_eq!(
            command_env(&command, "CARGO_NET_OFFLINE"),
            Some(OsStr::new("true"))
        );
        assert_eq!(
            command_env(&command, "CARGO_TERM_COLOR"),
            Some(OsStr::new("never"))
        );
        assert_eq!(
            command_env(&command, "GIT_TERMINAL_PROMPT"),
            Some(OsStr::new("0"))
        );
        assert_eq!(command_env(&command, "HTTP_PROXY"), None);
    }

    #[test]
    fn uses_temp_home_without_current_dir() {
        let mut command = Command::new("cargo");

        apply_sandbox_policy(&mut command);

        let home = command
            .get_envs()
            .find(|(key, _)| *key == OsStr::new("HOME"))
            .and_then(|(_, value)| value);

        assert_eq!(home, Some(std::env::temp_dir().as_os_str()));
    }

    fn command_env<'a>(command: &'a Command, key: &str) -> Option<&'a OsStr> {
        command
            .get_envs()
            .find(|(env_key, _)| *env_key == OsStr::new(key))
            .and_then(|(_, env_value)| env_value)
    }
}
