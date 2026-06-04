use crate::error::ValidationError;

pub const WORKSPACE_MODE_ENV: &str = "AH_WORKSPACE_MODE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceMode {
    Local,
    Git,
}

impl WorkspaceMode {
    pub fn from_env() -> Result<Self, ValidationError> {
        workspace_mode_from_value(std::env::var(WORKSPACE_MODE_ENV).ok())
    }

    pub fn as_str(self) -> &'static str {
        match self {
            WorkspaceMode::Local => "local",
            WorkspaceMode::Git => "git",
        }
    }
}

fn workspace_mode_from_value(value: Option<String>) -> Result<WorkspaceMode, ValidationError> {
    match value.as_deref().map(str::trim) {
        None | Some("") | Some("local") => Ok(WorkspaceMode::Local),
        Some("git") => Ok(WorkspaceMode::Git),
        Some(other) => Err(ValidationError::InvalidWorkspaceMode(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_local_workspace_mode() {
        assert_eq!(
            workspace_mode_from_value(None).unwrap(),
            WorkspaceMode::Local
        );
    }

    #[test]
    fn accepts_explicit_local_workspace_mode() {
        assert_eq!(
            workspace_mode_from_value(Some("local".to_string())).unwrap(),
            WorkspaceMode::Local
        );
    }

    #[test]
    fn accepts_git_workspace_mode() {
        assert_eq!(
            workspace_mode_from_value(Some("git".to_string())).unwrap(),
            WorkspaceMode::Git
        );
    }

    #[test]
    fn rejects_unknown_workspace_mode() {
        let error = workspace_mode_from_value(Some("remote".to_string())).unwrap_err();

        assert!(matches!(error, ValidationError::InvalidWorkspaceMode(_)));
    }
}
