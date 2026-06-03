use super::types::{AttemptId, GateRunId, RunId};
use crate::error::StoreError;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct ArtifactStore {
    root: Arc<PathBuf>,
}

pub struct ArtifactInput<'a> {
    pub run_id: RunId,
    pub attempt_id: Option<AttemptId>,
    pub gate_run_id: Option<GateRunId>,
    pub kind: &'a str,
    pub label: &'a str,
    pub content_type: &'a str,
    pub summary: &'a str,
    pub bytes: &'a [u8],
}

pub struct StoredArtifactDescriptor {
    pub kind: String,
    pub label: String,
    pub storage_uri: String,
    pub sha256: String,
    pub byte_len: i64,
    pub content_type: String,
    pub summary: String,
}

impl ArtifactStore {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: Arc::new(root),
        }
    }

    pub fn write_artifact(
        &self,
        input: ArtifactInput<'_>,
    ) -> Result<StoredArtifactDescriptor, StoreError> {
        let sha256 = sha256_hex(input.bytes);
        let relative_path = artifact_relative_path(&input, &sha256);
        let absolute_path = self.root.join(&relative_path);
        write_bytes(&absolute_path, input.bytes)?;
        Ok(stored_descriptor(input, relative_path, sha256))
    }
}

fn stored_descriptor(
    input: ArtifactInput<'_>,
    relative_path: PathBuf,
    sha256: String,
) -> StoredArtifactDescriptor {
    StoredArtifactDescriptor {
        kind: input.kind.to_string(),
        label: input.label.to_string(),
        storage_uri: artifact_uri(&relative_path),
        sha256,
        byte_len: input.bytes.len() as i64,
        content_type: input.content_type.to_string(),
        summary: input.summary.to_string(),
    }
}

fn artifact_relative_path(input: &ArtifactInput<'_>, sha256: &str) -> PathBuf {
    PathBuf::from(format!("runs/{}", input.run_id.get()))
        .join(attempt_segment(input.attempt_id))
        .join(gate_segment(input.gate_run_id))
        .join(artifact_file_name(input.kind, sha256))
}

fn attempt_segment(attempt_id: Option<AttemptId>) -> String {
    id_segment("attempt", attempt_id.map(AttemptId::get))
}

fn gate_segment(gate_run_id: Option<GateRunId>) -> String {
    id_segment("gate", gate_run_id.map(GateRunId::get))
}

fn id_segment(prefix: &str, value: Option<i64>) -> String {
    value
        .map(|id| format!("{prefix}-{id}"))
        .unwrap_or_else(|| format!("{prefix}-none"))
}

fn artifact_file_name(kind: &str, sha256: &str) -> String {
    format!("{}-{sha256}.bin", sanitize_segment(kind))
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    create_parent_dir(path)?;
    std::fs::write(path, bytes).map_err(|source| StoreError::ArtifactWriteFailed { source })
}

fn create_parent_dir(path: &Path) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|source| StoreError::ArtifactWriteFailed { source })?;
    }
    Ok(())
}

fn artifact_uri(relative_path: &Path) -> String {
    format!(
        "artifact://{}",
        relative_path.to_string_lossy().replace('\\', "/")
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sanitize_segment(value: &str) -> String {
    let sanitized: String = value.chars().map(safe_segment_char).collect();
    let trimmed = sanitized.trim_matches('-').to_string();
    if trimmed.is_empty() {
        return "artifact".to_string();
    }
    trimmed
}

fn safe_segment_char(value: char) -> char {
    if value.is_ascii_alphanumeric() {
        return value.to_ascii_lowercase();
    }
    '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_filesystem_artifact_descriptor() {
        let root = test_root("descriptor");
        let store = ArtifactStore::new(root.clone());

        let descriptor = store
            .write_artifact(ArtifactInput {
                run_id: RunId::new(7),
                attempt_id: Some(AttemptId::new(8)),
                gate_run_id: Some(GateRunId::new(9)),
                kind: "Gate Telemetry",
                label: "Gate 1 telemetry",
                content_type: "application/json",
                summary: "gate telemetry artifact captured",
                bytes: br#"{"passed":true}"#,
            })
            .unwrap();

        assert_eq!(descriptor.byte_len, 15);
        assert_eq!(descriptor.content_type, "application/json");
        assert!(descriptor.storage_uri.starts_with("artifact://runs/7/"));
        assert!(artifact_file_exists(&root, &descriptor.storage_uri));
    }

    fn artifact_file_exists(root: &Path, storage_uri: &str) -> bool {
        let relative_path = storage_uri.trim_start_matches("artifact://");
        root.join(relative_path).exists()
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join("acceptability-engine-artifact-tests")
            .join(name)
            .join(unique_suffix())
    }

    fn unique_suffix() -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        now.as_nanos().to_string()
    }
}
