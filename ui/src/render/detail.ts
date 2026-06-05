import { escapeHtml, setHtml } from "../dom";
import { formatBytes, formatDate, formatDuration, gateName } from "../format";
import type {
  AttemptGateDetail,
  AttemptSummary,
  EvidenceBundleSummary,
  RunDetail,
} from "../models";
import type { AppState } from "../state";
import { statusClass } from "../theme/semantic";

export function renderRunDetail(state: AppState): void {
  if (!state.selectedRun) {
    setHtml("run-detail", emptyDetail());
    return;
  }
  setHtml("run-detail", detailMarkup(state.selectedRun));
}

function emptyDetail(): string {
  return `
    <section class="detail-empty">
      <h2>Select a run</h2>
      <p>Run evidence, gate output, and attempts appear here.</p>
    </section>
  `;
}

function detailMarkup(detail: RunDetail): string {
  return `
    <section class="detail-header">
      <div>
        <p class="eyebrow">Run #${detail.summary.run_id}</p>
        <h2>${escapeHtml(detail.summary.contract_id)}</h2>
      </div>
      <span class="status-pill ${statusClass(detail.summary.status)}">
        ${escapeHtml(detail.summary.status.replaceAll("_", " "))}
      </span>
    </section>
    ${gateRail(detail)}
    <div class="detail-split">
      ${attemptPanel(detail.attempts)}
      ${evidencePanel(detail.evidence)}
    </div>
    ${gateDetails(detail.gates)}
  `;
}

function gateRail(detail: RunDetail): string {
  const gates = detail.summary.gates;
  if (gates.length === 0) {
    return `<section class="panel"><div class="empty-state">No gate evidence recorded.</div></section>`;
  }
  return `
    <section class="gate-rail">
      ${gates.map(gateSummary).join("")}
    </section>
  `;
}

function gateSummary(gate: RunDetail["summary"]["gates"][number]): string {
  const state = gate.passed ? "passed" : "failed";
  return `
    <article class="gate-tile ${state}">
      <span>${gate.gate_num}</span>
      <strong>${escapeHtml(gateName(gate.gate_num))}</strong>
      <small>${formatDuration(gate.duration_ms)}</small>
    </article>
  `;
}

function attemptPanel(attempts: AttemptSummary[]): string {
  return `
    <section class="panel">
      <div class="panel-head"><h2>Attempts</h2></div>
      <div class="attempt-list">
        ${attempts.map(attemptItem).join("") || `<div class="empty-state">No attempts.</div>`}
      </div>
    </section>
  `;
}

function attemptItem(attempt: AttemptSummary): string {
  return `
    <article class="attempt-item">
      <strong>Attempt ${attempt.attempt_number}</strong>
      <span class="attempt-status ${statusClass(attempt.status)}">
        ${escapeHtml(attempt.status)} · ${formatDate(attempt.created_at)}
      </span>
    </article>
  `;
}

function evidencePanel(evidence: EvidenceBundleSummary[]): string {
  return `
    <section class="panel">
      <div class="panel-head"><h2>Evidence</h2></div>
      <div class="evidence-list">
        ${evidence.map(evidenceItem).join("") || `<div class="empty-state">No evidence.</div>`}
      </div>
    </section>
  `;
}

function evidenceItem(evidence: EvidenceBundleSummary): string {
  return `
    <article class="evidence-item">
      <strong>${escapeHtml(evidence.label)}</strong>
      <span>${escapeHtml(evidence.kind)} · ${formatBytes(evidence.byte_len)}</span>
      <small>${escapeHtml(evidence.storage_uri ?? evidence.summary)}</small>
    </article>
  `;
}

function gateDetails(gates: AttemptGateDetail[]): string {
  return `
    <section class="panel gate-details">
      <div class="panel-head"><h2>Latest Gate Output</h2></div>
      ${gates.map(gateDetail).join("") || `<div class="empty-state">No gate output.</div>`}
    </section>
  `;
}

function gateDetail(gate: AttemptGateDetail): string {
  return `
    <article class="gate-detail">
      <header>
        <strong>${gate.gate_num}. ${escapeHtml(gateName(gate.gate_num))}</strong>
        <span>${gate.passed ? "Passed" : "Failed"} · ${formatDuration(gate.duration_ms)}</span>
      </header>
      <p>${escapeHtml(gate.message)}</p>
      ${testMetrics(gate)}
      ${outputBlock("stdout", gate.stdout, gate.stdout_truncated)}
      ${outputBlock("stderr", gate.stderr, gate.stderr_truncated)}
    </article>
  `;
}

function testMetrics(gate: AttemptGateDetail): string {
  if (gate.test_passed === null && gate.test_failed === null) {
    return "";
  }
  return `
    <div class="test-metrics">
      <span>${gate.test_passed ?? 0} passed</span>
      <span>${gate.test_failed ?? 0} failed</span>
      <span>${gate.test_ignored ?? 0} ignored</span>
      <span>${gate.parse_errors ?? 0} parse errors</span>
    </div>
  `;
}

function outputBlock(label: string, value: string | null, truncated: boolean): string {
  if (!value) {
    return "";
  }
  const suffix = truncated ? " · truncated" : "";
  return `
    <details>
      <summary>${label}${suffix}</summary>
      <pre>${escapeHtml(value)}</pre>
    </details>
  `;
}
