use crate::contract::Contract;
use axum::http::{HeaderMap, StatusCode};
use sha2::{Digest, Sha256};

const AUTHORIZATION_HEADER: &str = "authorization";
const API_KEY_HEADER: &str = "x-api-key";
const BEARER_PREFIX: &str = "Bearer ";
const WILDCARD_REPO: &str = "*";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Viewer,
    Submitter,
    Reviewer,
    Admin,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Submitter => "submitter",
            Self::Reviewer => "reviewer",
            Self::Admin => "admin",
        }
    }

    fn can_read(&self) -> bool {
        matches!(
            self,
            Self::Viewer | Self::Submitter | Self::Reviewer | Self::Admin
        )
    }

    fn can_submit(&self) -> bool {
        matches!(self, Self::Submitter | Self::Admin)
    }

    fn can_review(&self) -> bool {
        matches!(self, Self::Reviewer | Self::Admin)
    }
}

#[derive(Clone, Debug)]
pub struct SecurityIdentity {
    pub tenant_id: String,
    pub actor: String,
    pub role: Role,
    pub(crate) repo_prefixes: Vec<String>,
}

impl SecurityIdentity {
    pub fn local_admin() -> Self {
        Self {
            tenant_id: "local".to_string(),
            actor: "local-dev".to_string(),
            role: Role::Admin,
            repo_prefixes: vec![WILDCARD_REPO.to_string()],
        }
    }

    pub fn can_read(&self) -> bool {
        self.role.can_read()
    }

    pub fn can_submit(&self) -> bool {
        self.role.can_submit()
    }

    pub fn can_review(&self) -> bool {
        self.role.can_review()
    }

    pub fn allows_contract(&self, contract: &Contract) -> bool {
        self.repo_prefixes
            .iter()
            .any(|prefix| repo_prefix_matches(prefix, &contract.repo_url))
    }
}

#[derive(Clone, Debug)]
pub struct ApiKey {
    token: String,
    identity: SecurityIdentity,
}

impl ApiKey {
    pub fn new(token: String, role: Role, tenant_id: String, repo_prefixes: Vec<String>) -> Self {
        let actor = token_fingerprint(&token);
        Self {
            token,
            identity: SecurityIdentity {
                tenant_id,
                actor,
                role,
                repo_prefixes,
            },
        }
    }

    fn matches_token(&self, token: &str) -> bool {
        secure_eq(self.token.as_bytes(), token.as_bytes())
    }
}

#[derive(Debug)]
pub struct AuthRejection {
    pub status: StatusCode,
    pub reason: String,
}

pub fn authenticate_api_key(
    headers: &HeaderMap,
    api_keys: &[ApiKey],
) -> Result<SecurityIdentity, AuthRejection> {
    let token = request_token(headers)?;
    find_identity(token, api_keys)
}

pub fn parse_api_key_entry(entry: &str) -> Result<ApiKey, String> {
    let fields = split_entry_fields(entry)?;
    let role = parse_role(fields[1])?;
    let prefixes = parse_repo_prefixes(fields[3])?;
    Ok(ApiKey::new(
        fields[0].to_string(),
        role,
        fields[2].to_string(),
        prefixes,
    ))
}

fn request_token(headers: &HeaderMap) -> Result<&str, AuthRejection> {
    if let Some(token) = bearer_token(headers) {
        return Ok(token);
    }
    api_key_header(headers)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION_HEADER)?.to_str().ok()?;
    value.strip_prefix(BEARER_PREFIX)
}

fn api_key_header(headers: &HeaderMap) -> Result<&str, AuthRejection> {
    headers
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| rejection(StatusCode::UNAUTHORIZED, "missing API key"))
}

fn find_identity(token: &str, api_keys: &[ApiKey]) -> Result<SecurityIdentity, AuthRejection> {
    api_keys
        .iter()
        .find(|api_key| api_key.matches_token(token))
        .map(|api_key| api_key.identity.clone())
        .ok_or_else(|| rejection(StatusCode::UNAUTHORIZED, "invalid API key"))
}

fn split_entry_fields(entry: &str) -> Result<Vec<&str>, String> {
    let fields = entry.split('|').collect::<Vec<_>>();
    if fields.len() != 4 {
        return Err("API key entry must be token|role|tenant|repo_prefixes".to_string());
    }
    if fields.iter().any(|field| field.trim().is_empty()) {
        return Err("API key entry fields must not be empty".to_string());
    }
    Ok(fields)
}

fn parse_role(value: &str) -> Result<Role, String> {
    match value {
        "viewer" => Ok(Role::Viewer),
        "submitter" => Ok(Role::Submitter),
        "reviewer" => Ok(Role::Reviewer),
        "admin" => Ok(Role::Admin),
        _ => Err(format!("unsupported API key role '{value}'")),
    }
}

fn parse_repo_prefixes(value: &str) -> Result<Vec<String>, String> {
    let prefixes = value.split(',').map(str::trim).collect::<Vec<_>>();
    if prefixes.iter().any(|prefix| prefix.is_empty()) {
        return Err("repo policy prefixes must not be empty".to_string());
    }
    Ok(prefixes.into_iter().map(str::to_string).collect())
}

fn repo_prefix_matches(prefix: &str, repo_url: &str) -> bool {
    prefix == WILDCARD_REPO || repo_url.starts_with(prefix)
}

fn rejection(status: StatusCode, reason: &str) -> AuthRejection {
    AuthRejection {
        status,
        reason: reason.to_string(),
    }
}

fn token_fingerprint(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!("{:x}", digest)[..12].to_string()
}

fn secure_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0_u8, |acc, (left, right)| acc | (left ^ right))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_api_key_entry() {
        let key =
            parse_api_key_entry("token|submitter|tenant-a|https://github.com/progentic/").unwrap();

        assert!(key.identity.can_submit());
        assert_eq!(key.identity.tenant_id, "tenant-a");
    }

    #[test]
    fn rejects_unknown_role() {
        let result = parse_api_key_entry("token|owner|tenant-a|*");

        assert!(result.is_err());
    }
}
