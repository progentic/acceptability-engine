# Phase 31 Sandbox Hardening

## Sandbox Architecture

Phase 31 defines two sandbox profiles.

| Profile | Purpose | Production Status |
| :--- | :--- | :--- |
| `development` | Local development with Rust runner hardening. | Not production containment. |
| `kubernetes-restricted` | Kubernetes Restricted-style runtime containment plus Rust runner hardening. | Production containment baseline for this repository. |

The engine validates `AH_SANDBOX_PROFILE` at startup. Unknown profiles fail
closed. The `kubernetes-restricted` profile requires a Linux container runtime.

## Namespace Model

The Rust runner does not create portable namespaces directly.

The `kubernetes-restricted` profile relies on pod/runtime namespaces supplied
by Kubernetes and the container runtime. The deployment manifest sets a non-root
security context and disables privilege escalation.

The `development` profile uses host namespaces and must not be treated as
production containment.

## Filesystem Model

The Kubernetes deployment uses a read-only root filesystem.

Writable paths are explicit mounts:

- `/data`
- `/artifacts`
- `/workspaces`
- `/tmp`

Gate command `HOME` remains under the selected workspace. SQLite descriptors
remain authoritative evidence; filesystem artifacts remain payloads.

## Network Model

The Kubernetes deployment adds a NetworkPolicy for the engine pod.

Ingress is limited to port `8080`. Egress is denied by default.

This profile is compatible with local workspace mode. Git materialization that
requires outbound clone access needs a deliberate future egress policy and is
still blocked by D25-001 candidate-change acquisition.

## Syscall Model

The Kubernetes deployment uses `RuntimeDefault` seccomp and drops all Linux
capabilities. The container denies privilege escalation and runs as a non-root
user.

The Rust runner still applies command environment scrubbing before gate command
execution.

## Resource Model

The Kubernetes deployment sets pod CPU and memory requests and limits.

The Rust runner adds:

- process-group timeout cleanup
- bounded stdout and stderr capture
- CPU limit through `RLIMIT_CPU` where supported
- address-space limit through `RLIMIT_AS` where supported
- process-count limit through `RLIMIT_NPROC` where supported

Unsupported rlimits fail open only when the platform reports that specific
limit as unsupported. Other rlimit failures remain process errors.

## Containment Evidence

| Evidence | Source |
| :--- | :--- |
| Sandbox profile validation | `sandbox_profile::tests::rejects_unknown_sandbox_profile` |
| Kernel-control declaration | `sandbox_profile::tests::kubernetes_profile_declares_kernel_controls` |
| Minimal command environment | `gates::sandbox::tests::applies_minimal_sandbox_environment` |
| Proxy stripping | `gates::process::tests::execute_with_timeout_uses_sandbox_environment` |
| Output cap | `gates::process::tests::rejects_output_above_limit` |
| Timeout cleanup | `gates::process::tests::timeout_kills_descendant_processes` |
| Process timeout | `gates::process::tests::test_process_timeout` |
| Runner rlimit wiring | `gates::sandbox_runner::tests::sandbox_runner_invokes_resource_limit_configuration` |
| Kubernetes restricted controls | `deploy/kubernetes.yaml` |
| Compose local hardening | `compose.yaml` |

## Escape Review

The deployment model rejects common privilege escalation paths:

- container runs as non-root
- privilege escalation is disabled
- Linux capabilities are dropped
- root filesystem is read-only
- writable paths are explicit
- RuntimeDefault seccomp is enabled
- pod egress is denied in the restricted Kubernetes profile

This phase does not claim VM isolation or a custom Rust namespace/chroot/seccomp
implementation.

## Deviation Register

| ID | Status | Disposition |
| :--- | :--- | :--- |
| D25-002 | Narrowed for `kubernetes-restricted` profile | Production containment is now defined as deployment-enforced Kubernetes restricted controls plus Rust runner hardening. Full closure requires validation on a Kubernetes runtime enforcing the manifest. |
| D31-001 | Accepted limitation | The `development` profile is not production containment. |
| D31-002 | Accepted limitation | Non-Kubernetes production deployments must provide equivalent namespace, filesystem, network, syscall, and resource controls outside this repository. |
| D31-003 | Accepted limitation | Git materialization with denied egress requires future controlled egress design and remains constrained by D25-001. |

## Validation Evidence

- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- Compose YAML shape validation through Ruby YAML
- Kubernetes manifest shape validation through the same Ruby YAML check used by CI

`docker compose config` could not be run because Docker is unavailable in the
local validation environment. Compose YAML was shape-validated by Ruby. Full
Compose validation remains required in an environment with Docker installed.

## Conclusion

Phase 31 narrows D25-002 for the documented `kubernetes-restricted` deployment
profile by defining the runtime controls and validating the manifest shape. Full
closure requires runtime enforcement validation on Kubernetes. The engine still
does not claim VM isolation or portable kernel namespace construction inside
Rust.
