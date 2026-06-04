import { escapeHtml, setHtml } from "../dom";
import { formatDate } from "../format";
import type { AppState } from "../state";
import { reviewRuns } from "../state";

export function renderReviewQueue(state: AppState): void {
  const runs = reviewRuns(state);
  if (runs.length === 0) {
    setHtml("review-queue", `<div class="empty-state">No runs awaiting review.</div>`);
    return;
  }
  setHtml("review-queue", runs.map(reviewItem).join(""));
}

function reviewItem(run: ReturnType<typeof reviewRuns>[number]): string {
  return `
    <button class="review-item" data-run-id="${run.run_id}">
      <strong>${escapeHtml(run.contract_id)}</strong>
      <span>#${run.run_id} · ${formatDate(run.created_at)}</span>
    </button>
  `;
}
