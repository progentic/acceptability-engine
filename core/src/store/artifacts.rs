use super::types::{AttemptId, GateRunId, RunId};
use crate::error::StoreError;
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};
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

    pub fn delete_artifact(&self, storage_uri: &str) -> Result<ArtifactDeleteOutcome, StoreError> {
        let absolute_path = self.checked_artifact_path(storage_uri)?;
        remove_artifact_file(&absolute_path)
    }

    pub fn validate_artifact_uri(&self, storage_uri: &str) -> Result<(), StoreError> {
        self.checked_artifact_path(storage_uri).map(|_| ())
    }

    pub fn artifact_exists(&self, storage_uri: &str) -> Result<bool, StoreError> {
        let absolute_path = self.checked_artifact_path(storage_uri)?;
        Ok(absolute_path.exists())
    }

    #[cfg(test)]
    pub fn root_for_tests(&self) -> &Path {
        self.root.as_ref()
    }

    fn checked_artifact_path(&self, storage_uri: &str) -> Result<PathBuf, StoreError> {
        let relative_path = artifact_relative_path_from_uri(storage_uri)?;
        let absolute_path = self.root.join(relative_path);
        reject_symlinked_parent(&self.root, &absolute_path)?;
        Ok(absolute_path)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArtifactDeleteOutcome {
    Deleted,
    Missing,
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

fn artifact_relative_path_from_uri(storage_uri: &str) -> Result<PathBuf, StoreError> {
    let Some(relative) = storage_uri.strip_prefix("artifact://") else {
        return Err(StoreError::InvalidArtifactUri(storage_uri.to_string()));
    };
    let path = Path::new(relative);
    if relative.is_empty() {
        return Err(StoreError::InvalidArtifactUri(storage_uri.to_string()));
    }
    if relative.contains('\\') {
        return Err(StoreError::InvalidArtifactUri(storage_uri.to_string()));
    }
    if path.components().all(safe_relative_component) {
        return Ok(path.to_path_buf());
    }
    Err(StoreError::InvalidArtifactUri(storage_uri.to_string()))
}

fn safe_relative_component(component: Component<'_>) -> bool {
    matches!(component, Component::Normal(_))
}

fn reject_symlinked_parent(root: &Path, artifact_path: &Path) -> Result<(), StoreError> {
    reject_symlink_component(root)?;
    let Some(parent) = artifact_path.parent() else {
        return Ok(());
    };
    let relative_parent = parent.strip_prefix(root).map_err(|_| {
        StoreError::InvalidArtifactUri(artifact_path.to_string_lossy().into_owned())
    })?;
    let mut current = root.to_path_buf();
    for component in relative_parent.components() {
        current.push(component.as_os_str());
        reject_symlink_component(&current)?;
    }
    Ok(())
}

fn reject_symlink_component(path: &Path) -> Result<(), StoreError> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|source| StoreError::ArtifactDeleteFailed { source })?;
    if !metadata.file_type().is_symlink() {
        return Ok(());
    }
    Err(StoreError::InvalidArtifactUri(
        path.to_string_lossy().into_owned(),
    ))
}

fn remove_artifact_file(path: &Path) -> Result<ArtifactDeleteOutcome, StoreError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(ArtifactDeleteOutcome::Deleted),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(ArtifactDeleteOutcome::Missing)
        }
        Err(source) => Err(StoreError::ArtifactDeleteFailed { source }),
    }
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

    #[test]
    fn deletes_artifact_file_under_root() {
        let root = test_root("delete");
        let store = ArtifactStore::new(root.clone());
        let descriptor = store
            .write_artifact(ArtifactInput {
                run_id: RunId::new(7),
                attempt_id: None,
                gate_run_id: None,
                kind: "retention",
                label: "retained",
                content_type: "text/plain",
                summary: "retention test",
                bytes: b"delete-me",
            })
            .unwrap();

        let outcome = store.delete_artifact(&descriptor.storage_uri).unwrap();

        assert_eq!(outcome, ArtifactDeleteOutcome::Deleted);
        assert!(!artifact_file_exists(&root, &descriptor.storage_uri));
    }

    #[test]
    fn rejects_artifact_uri_traversal() {
        let root = test_root("traversal");
        let store = ArtifactStore::new(root);
        let result = store.delete_artifact("artifact://../escape");

        assert!(matches!(result, Err(StoreError::InvalidArtifactUri(_))));
    }

    #[test]
    fn rejects_artifact_uri_backslashes() {
        let root = test_root("backslash");
        let store = ArtifactStore::new(root);
        let result = store.delete_artifact("artifact://runs\\7\\artifact.bin");

        assert!(matches!(result, Err(StoreError::InvalidArtifactUri(_))));
    }

    #[test]
    fn validates_artifact_uri_without_deleting_file() {
        let root = test_root("validate");
        let store = ArtifactStore::new(root.clone());
        let descriptor = store
            .write_artifact(ArtifactInput {
                run_id: RunId::new(7),
                attempt_id: None,
                gate_run_id: None,
                kind: "retention",
                label: "retained",
                content_type: "text/plain",
                summary: "retention test",
                bytes: b"validate-me",
            })
            .unwrap();

        store
            .validate_artifact_uri(&descriptor.storage_uri)
            .unwrap();

        assert!(artifact_file_exists(&root, &descriptor.storage_uri));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_artifact_root() {
        let target_root = test_root("symlink-target");
        let symlink_root = test_root("symlink-root");
        std::fs::create_dir_all(&target_root).unwrap();
        std::fs::create_dir_all(symlink_root.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&target_root, &symlink_root).unwrap();
        let store = ArtifactStore::new(symlink_root);

        let result = store.delete_artifact("artifact://runs/7/artifact.bin");

        assert!(matches!(result, Err(StoreError::InvalidArtifactUri(_))));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_artifact_parent() {
        let root = test_root("symlink-parent-root");
        let target = test_root("symlink-parent-target");
        let runs_dir = root.join("runs");
        std::fs::create_dir_all(&runs_dir).unwrap();
        std::fs::create_dir_all(&target).unwrap();
        std::os::unix::fs::symlink(&target, runs_dir.join("7")).unwrap();
        let store = ArtifactStore::new(root);

        let result = store.delete_artifact("artifact://runs/7/artifact.bin");

        assert!(matches!(result, Err(StoreError::InvalidArtifactUri(_))));
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
