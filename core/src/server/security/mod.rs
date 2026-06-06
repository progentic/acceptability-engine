mod auth;
mod limits;

use crate::contract::Contract;
use auth::{authenticate_api_key, parse_api_key_entry, ApiKey};
use axum::http::{HeaderMap, StatusCode};
use limits::{LimitConfig, LimitRejection, LimitState};
use std::sync::Arc;

pub use auth::SecurityIdentity;

const SECURITY_MODE_ENV: &str = "AH_SECURITY_MODE";
const API_KEYS_ENV: &str = "AH_API_KEYS";
const REQUEST_LIMIT_ENV: &str = "AH_RATE_LIMIT_PER_MINUTE";
const SUBMISSION_LIMIT_ENV: &str = "AH_RUN_QUOTA_PER_HOUR";
const DEFAULT_REQUEST_LIMIT: u32 = 120;
const DEFAULT_SUBMISSION_LIMIT: u32 = 20;

#[derive(Clone)]
pub struct TrustControls {
    config: Arc<SecurityConfig>,
    limits: Arc<LimitState>,
}

#[derive(Debug)]
pub struct SecurityRejection {
    pub status: StatusCode,
    pub reason: String,
    pub tenant_id: String,
    pub actor: String,
    pub role: String,
}

struct SecurityConfig {
    mode: SecurityMode,
    api_keys: Vec<ApiKey>,
}

enum SecurityMode {
    Disabled,
    ApiKey,
}

impl TrustControls {
    pub fn from_env() -> Result<Self, String> {
        let config = SecurityConfig::from_env()?;
        Ok(Self::new(config))
    }

    #[cfg(test)]
    pub fn disabled() -> Self {
        Self::new(SecurityConfig::disabled())
    }

    #[cfg(test)]
    pub fn api_key(api_key: &str) -> Self {
        let config = SecurityConfig {
            mode: SecurityMode::ApiKey,
            api_keys: vec![parse_api_key_entry(api_key).unwrap()],
        };
        Self::new(config)
    }

    pub async fn authorize_read(
        &self,
        headers: &HeaderMap,
    ) -> Result<SecurityIdentity, SecurityRejection> {
        let identity = self.authenticate(headers)?;
        require_read_role(&identity)?;
        self.enforce_request_limit(&identity).await?;
        Ok(identity)
    }

    pub async fn authorize_submit(
        &self,
        headers: &HeaderMap,
        contract: &Contract,
    ) -> Result<SecurityIdentity, SecurityRejection> {
        let identity = self.authenticate(headers)?;
        require_submit_role(&identity)?;
        require_repo_policy(&identity, contract)?;
        self.enforce_request_limit(&identity).await?;
        self.enforce_submission_limit(&identity).await?;
        Ok(identity)
    }

    pub async fn authorize_review(
        &self,
        headers: &HeaderMap,
    ) -> Result<SecurityIdentity, SecurityRejection> {
        let identity = self.authenticate(headers)?;
        require_review_role(&identity)?;
        self.enforce_request_limit(&identity).await?;
        Ok(identity)
    }

    fn new(config: SecurityConfig) -> Self {
        Self {
            config: Arc::new(config),
            limits: Arc::new(LimitState::new(LimitConfig::from_env())),
        }
    }

    fn authenticate(&self, headers: &HeaderMap) -> Result<SecurityIdentity, SecurityRejection> {
        match self.config.mode {
            SecurityMode::Disabled => Ok(SecurityIdentity::local_admin()),
            SecurityMode::ApiKey => authenticate_api_key(headers, &self.config.api_keys)
                .map_err(SecurityRejection::from_auth),
        }
    }

    async fn enforce_request_limit(
        &self,
        identity: &SecurityIdentity,
    ) -> Result<(), SecurityRejection> {
        self.limits
            .check_request(identity)
            .await
            .map_err(|error| SecurityRejection::from_limit(error, identity))
    }

    async fn enforce_submission_limit(
        &self,
        identity: &SecurityIdentity,
    ) -> Result<(), SecurityRejection> {
        self.limits
            .check_submission(identity)
            .await
            .map_err(|error| SecurityRejection::from_limit(error, identity))
    }
}

impl SecurityConfig {
    fn from_env() -> Result<Self, String> {
        match security_mode().as_str() {
            "disabled" => Ok(Self::disabled()),
            "api-key" => Self::api_key(api_key_entries()?),
            value => Err(format!("unsupported security mode '{value}'")),
        }
    }

    fn disabled() -> Self {
        Self {
            mode: SecurityMode::Disabled,
            api_keys: Vec::new(),
        }
    }

    fn api_key(entries: String) -> Result<Self, String> {
        let api_keys = parse_api_key_entries(&entries)?;
        if api_keys.is_empty() {
            return Err("AH_API_KEYS must contain at least one API key".to_string());
        }
        Ok(Self {
            mode: SecurityMode::ApiKey,
            api_keys,
        })
    }
}

impl LimitConfig {
    fn from_env() -> Self {
        Self {
            requests_per_minute: env_u32(REQUEST_LIMIT_ENV, DEFAULT_REQUEST_LIMIT),
            submissions_per_hour: env_u32(SUBMISSION_LIMIT_ENV, DEFAULT_SUBMISSION_LIMIT),
        }
    }
}

impl SecurityRejection {
    fn from_auth(error: auth::AuthRejection) -> Self {
        Self {
            status: error.status,
            reason: error.reason,
            tenant_id: "unknown".to_string(),
            actor: "anonymous".to_string(),
            role: "none".to_string(),
        }
    }

    fn from_identity(status: StatusCode, reason: &str, identity: SecurityIdentity) -> Self {
        Self {
            status,
            reason: reason.to_string(),
            tenant_id: identity.tenant_id,
            actor: identity.actor,
            role: identity.role.as_str().to_string(),
        }
    }

    fn from_limit(error: LimitRejection, identity: &SecurityIdentity) -> Self {
        Self {
            status: error.status,
            reason: error.reason,
            tenant_id: identity.tenant_id.clone(),
            actor: identity.actor.clone(),
            role: identity.role.as_str().to_string(),
        }
    }
}

fn require_read_role(identity: &SecurityIdentity) -> Result<(), SecurityRejection> {
    if identity.can_read() {
        return Ok(());
    }
    Err(SecurityRejection::from_identity(
        StatusCode::FORBIDDEN,
        "role cannot read runs",
        identity.clone(),
    ))
}

fn require_submit_role(identity: &SecurityIdentity) -> Result<(), SecurityRejection> {
    if identity.can_submit() {
        return Ok(());
    }
    Err(SecurityRejection::from_identity(
        StatusCode::FORBIDDEN,
        "role cannot submit runs",
        identity.clone(),
    ))
}

fn require_review_role(identity: &SecurityIdentity) -> Result<(), SecurityRejection> {
    if identity.can_review() {
        return Ok(());
    }
    Err(SecurityRejection::from_identity(
        StatusCode::FORBIDDEN,
        "role cannot review runs",
        identity.clone(),
    ))
}

fn require_repo_policy(
    identity: &SecurityIdentity,
    contract: &Contract,
) -> Result<(), SecurityRejection> {
    if identity.allows_contract(contract) {
        return Ok(());
    }
    Err(SecurityRejection::from_identity(
        StatusCode::FORBIDDEN,
        "repo policy denied contract repository",
        identity.clone(),
    ))
}

fn parse_api_key_entries(entries: &str) -> Result<Vec<ApiKey>, String> {
    entries
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .map(parse_api_key_entry)
        .collect()
}

fn security_mode() -> String {
    std::env::var(SECURITY_MODE_ENV).unwrap_or_else(|_| "disabled".to_string())
}

fn api_key_entries() -> Result<String, String> {
    std::env::var(API_KEYS_ENV).map_err(|_| "AH_API_KEYS is required in api-key mode".to_string())
}

fn env_u32(name: &str, fallback: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract(repo_url: &str) -> Contract {
        Contract {
            id: "run-001".to_string(),
            repo_url: repo_url.to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_sha: "b9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_ref: None,
            scopes: vec!["core/src".to_string()],
            requires_human_review: false,
            admission_policy: crate::policy::AdmissionPolicy::default(),
        }
    }

    fn auth_headers(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", token.parse().unwrap());
        headers
    }

    #[tokio::test]
    async fn authorizes_submitter_for_allowed_repo() {
        let controls =
            TrustControls::api_key("secret|submitter|tenant-a|https://github.com/progentic/");

        let identity = controls
            .authorize_submit(
                &auth_headers("secret"),
                &contract("https://github.com/progentic/acceptability-engine.git"),
            )
            .await
            .unwrap();

        assert_eq!(identity.tenant_id, "tenant-a");
    }

    #[tokio::test]
    async fn rejects_viewer_submission() {
        let controls = TrustControls::api_key("secret|viewer|tenant-a|*");

        let error = controls
            .authorize_submit(
                &auth_headers("secret"),
                &contract("https://example.com/repo.git"),
            )
            .await
            .unwrap_err();

        assert_eq!(error.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_repo_outside_policy() {
        let controls =
            TrustControls::api_key("secret|submitter|tenant-a|https://github.com/progentic/");

        let error = controls
            .authorize_submit(
                &auth_headers("secret"),
                &contract("https://github.com/other/repo.git"),
            )
            .await
            .unwrap_err();

        assert_eq!(error.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn authorizes_reviewer_decisions() {
        let controls = TrustControls::api_key("secret|reviewer|tenant-a|*");

        let identity = controls
            .authorize_review(&auth_headers("secret"))
            .await
            .unwrap();

        assert_eq!(identity.tenant_id, "tenant-a");
    }

    #[tokio::test]
    async fn rejects_submitter_review_decision() {
        let controls = TrustControls::api_key("secret|submitter|tenant-a|*");

        let error = controls
            .authorize_review(&auth_headers("secret"))
            .await
            .unwrap_err();

        assert_eq!(error.status, StatusCode::FORBIDDEN);
    }
}
