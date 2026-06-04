import { escapeHtml, setHtml } from "../dom";
import { formatDate, statusTone } from "../format";
import type { AppState } from "../state";
import { visibleRuns } from "../state";

export function renderRunList(state: AppState): void {
  const runs = visibleRuns(state);
  if (runs.length === 0) {
    setHtml("run-list", emptyRuns());
    return;
  }
  setHtml("run-list", runs.map((run) => runButton(state, run)).join(""));
}

function emptyRuns(): string {
  return `<div class="empty-state">No runs match the current filter.</div>`;
}

function runButton(state: AppState, run: ReturnType<typeof visibleRuns>[number]): string {
  const selected = state.selectedRunId === run.run_id ? " selected" : "";
  return `
    <button class="run-item${selected}" data-run-id="${run.run_id}">
      <span class="status-dot ${statusTone(run.status)}"></span>
      <span>
        <strong>${escapeHtml(run.contract_id)}</strong>
        <small>#${run.run_id} · ${formatDate(run.created_at)}</small>
      </span>
      <em>${escapeHtml(run.status.replaceAll("_", " "))}</em>
    </button>
  `;
}
