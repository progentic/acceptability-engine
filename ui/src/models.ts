export type RunStatus =
  | "QUEUED"
  | "RUNNING"
  | "APPROVED"
  | "REJECTED"
  | "FAILED_INTERNAL"
  | "PENDING_HUMAN_REVIEW";

export interface ContractSubmission {
  id: string;
  repo_url: string;
  base_sha: string;
  scopes: string[];
  requires_human_review: boolean;
}

export interface SubmitResponse {
  run_id: number;
  status: RunStatus;
  reason: string | null;
}

export interface RunListItem {
  run_id: number;
  contract_id: string;
  status: RunStatus;
  created_at: number;
}

export interface RunStatusSummary extends RunListItem {
  gates: GateRunSummary[];
}

export interface GateRunSummary {
  gate_num: number;
  passed: boolean;
  message: string;
  exit_code: number | null;
  duration_ms: number | null;
}

export interface AttemptSummary {
  attempt_id: number;
  run_id: number;
  attempt_number: number;
  status: string;
  created_at: number;
}

export interface AttemptGateDetail extends GateRunSummary {
  gate_run_id: number;
  attempt_id: number;
  stdout: string | null;
  stdout_truncated: boolean;
  stderr: string | null;
  stderr_truncated: boolean;
  test_passed: number | null;
  test_failed: number | null;
  test_ignored: number | null;
  parse_errors: number | null;
}

export interface EvidenceBundleSummary {
  evidence_bundle_id: number;
  run_id: number;
  attempt_id: number | null;
  gate_run_id: number | null;
  review_decision_id: number | null;
  kind: string;
  label: string;
  storage_uri: string | null;
  sha256: string | null;
  byte_len: number | null;
  content_type: string | null;
  summary: string;
  created_at: number;
}

export interface RunDetail {
  summary: RunStatusSummary;
  attempts: AttemptSummary[];
  gates: AttemptGateDetail[];
  evidence: EvidenceBundleSummary[];
}

export interface ReviewDecisionRequest {
  reason: string;
}

export interface ReviewDecisionResponse {
  run_id: number;
  status: RunStatus;
  review_decision_id: number;
  evidence_bundle_id: number;
}
