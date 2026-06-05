use crate::error::ValidationError;

pub const SANDBOX_PROFILE_ENV: &str = "AH_SANDBOX_PROFILE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxProfile {
    Development,
    KubernetesRestricted,
}

impl SandboxProfile {
    pub fn from_env() -> Result<Self, ValidationError> {
        let profile = sandbox_profile_from_value(std::env::var(SANDBOX_PROFILE_ENV).ok())?;
        validate_runtime_profile(profile)?;
        Ok(profile)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SandboxProfile::Development => "development",
            SandboxProfile::KubernetesRestricted => "kubernetes-restricted",
        }
    }

    pub fn model(self) -> SandboxModel {
        match self {
            SandboxProfile::Development => SandboxModel::development(),
            SandboxProfile::KubernetesRestricted => SandboxModel::kubernetes_restricted(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxModel {
    pub namespace_model: &'static str,
    pub filesystem_model: &'static str,
    pub network_model: &'static str,
    pub syscall_model: &'static str,
    pub resource_model: &'static str,
}

impl SandboxModel {
    fn development() -> Self {
        Self {
            namespace_model: "host namespaces",
            filesystem_model: "workspace-rooted paths with process home under workspace",
            network_model: "command environment disables network-dependent cargo and git prompts",
            syscall_model: "host default syscall policy",
            resource_model:
                "runner timeouts, process groups, bounded output, and rlimits where supported",
        }
    }

    fn kubernetes_restricted() -> Self {
        Self {
            namespace_model: "pod namespaces with non-root user and no privilege escalation",
            filesystem_model: "read-only root filesystem with writable data, artifacts, workspace, and tmp mounts",
            network_model: "NetworkPolicy denies egress for gate execution workloads",
            syscall_model: "RuntimeDefault seccomp profile with all Linux capabilities dropped",
            resource_model: "pod cpu and memory limits plus runner timeouts, process groups, bounded output, and rlimits",
        }
    }
}

fn sandbox_profile_from_value(value: Option<String>) -> Result<SandboxProfile, ValidationError> {
    match value.as_deref().map(str::trim) {
        None | Some("") | Some("development") => Ok(SandboxProfile::Development),
        Some("kubernetes-restricted") => Ok(SandboxProfile::KubernetesRestricted),
        Some(other) => Err(ValidationError::InvalidSandboxProfile(other.to_string())),
    }
}

fn validate_runtime_profile(profile: SandboxProfile) -> Result<(), ValidationError> {
    match profile {
        SandboxProfile::Development => Ok(()),
        SandboxProfile::KubernetesRestricted => validate_kubernetes_restricted_runtime(),
    }
}

#[cfg(target_os = "linux")]
fn validate_kubernetes_restricted_runtime() -> Result<(), ValidationError> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn validate_kubernetes_restricted_runtime() -> Result<(), ValidationError> {
    Err(ValidationError::InvalidSandboxProfile(
        "kubernetes-restricted requires a Linux container runtime".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_development_sandbox_profile() {
        assert_eq!(
            sandbox_profile_from_value(None).unwrap(),
            SandboxProfile::Development
        );
    }

    #[test]
    fn accepts_kubernetes_restricted_profile() {
        assert_eq!(
            sandbox_profile_from_value(Some("kubernetes-restricted".to_string())).unwrap(),
            SandboxProfile::KubernetesRestricted
        );
    }

    #[test]
    fn rejects_unknown_sandbox_profile() {
        let error = sandbox_profile_from_value(Some("privileged".to_string())).unwrap_err();

        assert!(matches!(error, ValidationError::InvalidSandboxProfile(_)));
    }

    #[test]
    fn kubernetes_profile_declares_kernel_controls() {
        let model = SandboxProfile::KubernetesRestricted.model();

        assert!(model.namespace_model.contains("pod namespaces"));
        assert!(model.filesystem_model.contains("read-only root filesystem"));
        assert!(model.network_model.contains("denies egress"));
        assert!(model.syscall_model.contains("seccomp"));
        assert!(model.resource_model.contains("cpu and memory limits"));
    }
}
