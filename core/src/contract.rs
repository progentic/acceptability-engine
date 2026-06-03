use crate::error::validation::ValidationError;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path};

const GIT_SUFFIX: &str = ".git";
const HTTPS_GIT_PREFIX: &str = "https://";
const SSH_GIT_PREFIX: &str = "ssh://";
const SCP_GIT_USER_PREFIX: &str = "git@";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: String,
    pub repo_url: String,
    pub base_sha: String,
    pub scopes: Vec<String>,
    pub requires_human_review: bool,
}

impl Contract {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_contract_id(&self.id)?;
        validate_repo_url(&self.repo_url)?;
        validate_base_sha(&self.base_sha)?;
        validate_scopes(&self.scopes)?;
        Ok(())
    }
}

fn validate_contract_id(id: &str) -> Result<(), ValidationError> {
    if id.trim().is_empty() {
        return Err(ValidationError::MissingContractId);
    }
    if !is_safe_contract_id(id) {
        return Err(ValidationError::InvalidContractId(id.to_string()));
    }
    Ok(())
}

fn validate_repo_url(repo_url: &str) -> Result<(), ValidationError> {
    if repo_url.trim().is_empty() {
        return Err(ValidationError::MissingRepoUrl);
    }
    if !is_supported_git_url(repo_url) {
        return Err(ValidationError::InvalidRepoUrl(repo_url.to_string()));
    }
    Ok(())
}

fn validate_base_sha(base_sha: &str) -> Result<(), ValidationError> {
    if base_sha.is_empty() {
        return Err(ValidationError::MissingBaseSha);
    }
    if base_sha.len() != 40 {
        return Err(ValidationError::InvalidBaseShaLength {
            len: base_sha.len(),
            value: base_sha.to_string(),
        });
    }
    if !base_sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidBaseShaChars(base_sha.to_string()));
    }
    Ok(())
}

fn validate_scopes(scopes: &[String]) -> Result<(), ValidationError> {
    if scopes.is_empty() {
        return Err(ValidationError::EmptyScopes);
    }
    for (index, scope) in scopes.iter().enumerate() {
        validate_scope(index, scope)?;
    }
    Ok(())
}

fn validate_scope(index: usize, scope: &str) -> Result<(), ValidationError> {
    if scope.trim().is_empty() {
        return Err(ValidationError::EmptyScope { index });
    }
    if !is_normalized_relative_path(scope) {
        return Err(ValidationError::InvalidScopePath {
            index,
            value: scope.to_string(),
        });
    }
    Ok(())
}

fn is_safe_contract_id(id: &str) -> bool {
    id.chars().all(is_contract_id_character) && id != "." && id != ".."
}

fn is_contract_id_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
}

fn is_supported_git_url(repo_url: &str) -> bool {
    has_no_url_control_chars(repo_url)
        && (is_https_git_url(repo_url)
            || is_ssh_git_url(repo_url)
            || is_scp_style_git_url(repo_url))
}

fn has_no_url_control_chars(repo_url: &str) -> bool {
    !repo_url.chars().any(char::is_whitespace)
}

fn is_https_git_url(repo_url: &str) -> bool {
    repo_url.starts_with(HTTPS_GIT_PREFIX)
        && has_git_path(repo_url.trim_start_matches(HTTPS_GIT_PREFIX))
}

fn is_ssh_git_url(repo_url: &str) -> bool {
    repo_url.starts_with(SSH_GIT_PREFIX)
        && has_git_path(repo_url.trim_start_matches(SSH_GIT_PREFIX))
}

fn is_scp_style_git_url(repo_url: &str) -> bool {
    repo_url.starts_with(SCP_GIT_USER_PREFIX)
        && repo_url.contains(':')
        && repo_url.ends_with(GIT_SUFFIX)
}

fn has_git_path(remote: &str) -> bool {
    let Some((host, path)) = remote.split_once('/') else {
        return false;
    };
    !host.is_empty() && path.contains('/') && path.ends_with(GIT_SUFFIX)
}

fn is_normalized_relative_path(path: &str) -> bool {
    !contains_windows_separator(path)
        && !contains_empty_path_segment(path)
        && has_only_normal_relative_components(path)
}

fn contains_windows_separator(path: &str) -> bool {
    path.contains('\\')
}

fn contains_empty_path_segment(path: &str) -> bool {
    path.split('/').any(str::is_empty) && !path.ends_with('/')
}

fn has_only_normal_relative_components(path: &str) -> bool {
    let mut has_component = false;
    for component in Path::new(path).components() {
        match component {
            Component::Normal(_) => has_component = true,
            _ => return false,
        }
    }
    has_component
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_contract() -> Contract {
        Contract {
            id: "run-001".to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["core/src".to_string()],
            requires_human_review: false,
        }
    }

    #[test]
    fn validates_safe_contract() {
        assert!(valid_contract().validate().is_ok());
    }

    #[test]
    fn rejects_path_like_contract_ids() {
        let mut contract = valid_contract();
        contract.id = "../escape".to_string();

        assert!(matches!(
            contract.validate(),
            Err(ValidationError::InvalidContractId(_))
        ));
    }

    #[test]
    fn rejects_unsupported_repo_urls() {
        let mut contract = valid_contract();
        contract.repo_url = "file:///tmp/repo.git".to_string();

        assert!(matches!(
            contract.validate(),
            Err(ValidationError::InvalidRepoUrl(_))
        ));
    }

    #[test]
    fn accepts_ssh_repo_urls() {
        let mut contract = valid_contract();
        contract.repo_url = "git@github.com:progentic/acceptability-engine.git".to_string();

        assert!(contract.validate().is_ok());
    }

    #[test]
    fn rejects_absolute_scope_paths() {
        let mut contract = valid_contract();
        contract.scopes = vec!["/core/src".to_string()];

        assert!(matches!(
            contract.validate(),
            Err(ValidationError::InvalidScopePath { .. })
        ));
    }

    #[test]
    fn rejects_parent_scope_paths() {
        let mut contract = valid_contract();
        contract.scopes = vec!["core/../secrets".to_string()];

        assert!(matches!(
            contract.validate(),
            Err(ValidationError::InvalidScopePath { .. })
        ));
    }

    #[test]
    fn rejects_repeated_scope_separators() {
        let mut contract = valid_contract();
        contract.scopes = vec!["core//src".to_string()];

        assert!(matches!(
            contract.validate(),
            Err(ValidationError::InvalidScopePath { .. })
        ));
    }

    #[test]
    fn accepts_trailing_scope_separator() {
        let mut contract = valid_contract();
        contract.scopes = vec!["core/src/".to_string()];

        assert!(contract.validate().is_ok());
    }
}
