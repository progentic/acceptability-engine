import { escapeHtml, setHtml } from "../dom";
import type { AppState } from "../state";

const STATUSES = [
  "QUEUED",
  "RUNNING",
  "PENDING_HUMAN_REVIEW",
  "APPROVED",
  "REJECTED",
  "FAILED_INTERNAL",
];

export function renderMetrics(state: AppState): void {
  const cells = STATUSES.map((status) => metricCell(status, countStatus(state, status))).join("");
  setHtml("metrics-row", cells);
}

function countStatus(state: AppState, status: string): number {
  return state.runs.filter((run) => run.status === status).length;
}

function metricCell(label: string, value: number): string {
  return `
    <article class="metric">
      <span>${escapeHtml(label.replaceAll("_", " "))}</span>
      <strong>${value}</strong>
    </article>
  `;
}
