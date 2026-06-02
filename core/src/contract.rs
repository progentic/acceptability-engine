use crate::error::validation::ValidationError;
use serde::{Deserialize, Serialize};

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
        if self.base_sha.is_empty() {
            return Err(ValidationError::MissingBaseSha);
        }
        if self.base_sha.len() != 40 {
            return Err(ValidationError::InvalidBaseShaLength {
                len: self.base_sha.len(),
                value: self.base_sha.clone(),
            });
        }
        if !self.base_sha.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidBaseShaChars(self.base_sha.clone()));
        }
        if self.scopes.is_empty() {
            return Err(ValidationError::EmptyScopes);
        }
        for (index, scope) in self.scopes.iter().enumerate() {
            if scope.trim().is_empty() {
                return Err(ValidationError::EmptyScope { index });
            }
        }
        Ok(())
    }
}
