import { parseRunProgressEvent } from "./api";
import { byId, setText } from "./dom";
import { isLiveStatus } from "./format";
import type { ContractSubmission, RunStatus } from "./models";
import { renderRunDetail } from "./render/detail";
import { renderLayout } from "./render/layout";
import { renderMetrics } from "./render/metrics";
import { renderReviewQueue } from "./render/review";
import { renderRunList } from "./render/runs";
import { createState, selectedRunFromList, type AppState } from "./state";
import { applySemanticTheme } from "./theme/semantic";
import "./styles.css";

const RUN_POLL_MS = 5000;
const DETAIL_POLL_MS = 2000;

const state = createState("/api");

applySemanticTheme(document.documentElement);
renderLayout();
bindEvents(state);
void refreshRuns(state);
window.setInterval(() => void refreshRuns(state), RUN_POLL_MS);
window.setInterval(() => void refreshSelectedRun(state), DETAIL_POLL_MS);

function bindEvents(appState: AppState): void {
  byId("connect-api").addEventListener("click", () => reconnect(appState));
  byId("status-filter").addEventListener("change", (event) => updateFilter(appState, event));
  byId("run-list").addEventListener("click", (event) => selectRunFromEvent(appState, event));
  byId("review-queue").addEventListener("click", (event) => selectRunFromEvent(appState, event));
  byId("submit-form").addEventListener("submit", (event) => submitRun(appState, event));
}

async function reconnect(appState: AppState): Promise<void> {
  closeProgressSocket(appState);
  appState.apiBase = byId<HTMLInputElement>("api-base").value.trim() || "/api";
  appState.api = createState(appState.apiBase).api;
  await refreshRuns(appState);
}

function updateFilter(appState: AppState, event: Event): void {
  const select = event.target as HTMLSelectElement;
  appState.statusFilter = select.value as RunStatus | "ALL";
  render(appState);
}

async function selectRunFromEvent(appState: AppState, event: Event): Promise<void> {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-run-id]");
  if (!button) {
    return;
  }
  appState.selectedRunId = Number(button.dataset.runId);
  await refreshSelectedRun(appState);
}

async function submitRun(appState: AppState, event: Event): Promise<void> {
  event.preventDefault();
  await runRequest(appState, async () => {
    const contract = readContractForm();
    const response = await appState.api.submitContract(contract);
    appState.selectedRunId = response.run_id;
    clearContractForm();
    await syncRuns(appState);
  });
}

async function refreshRuns(appState: AppState): Promise<void> {
  await runRequest(appState, () => syncRuns(appState));
}

async function refreshSelectedRun(appState: AppState): Promise<void> {
  await runRequest(appState, () => syncSelectedRun(appState));
}

async function syncRuns(appState: AppState): Promise<void> {
  appState.runs = await appState.api.listRuns();
  appState.selectedRunId ??= appState.runs[0]?.run_id ?? null;
  await syncSelectedRun(appState);
}

async function syncSelectedRun(appState: AppState): Promise<void> {
  const selected = selectedRunFromList(appState);
  if (!selected) {
    appState.selectedRun = null;
    render(appState);
    return;
  }
  if (!isLiveStatus(selected.status) && appState.selectedRun?.summary.run_id === selected.run_id) {
    closeProgressSocket(appState);
    render(appState);
    return;
  }
  appState.selectedRun = await appState.api.getRunDetail(selected.run_id);
  syncProgressSocket(appState, selected.run_id, selected.status);
  render(appState);
}

function syncProgressSocket(appState: AppState, runId: number, status: RunStatus): void {
  if (!isLiveStatus(status)) {
    closeProgressSocket(appState);
    return;
  }
  if (appState.progressRunId === runId && appState.progressSocket) {
    return;
  }
  closeProgressSocket(appState);
  appState.progressRunId = runId;
  appState.progressSocket = appState.api.openRunProgress(
    runId,
    appState.progressSequences[runId],
  );
  const socket = appState.progressSocket;
  appState.progressSocket.addEventListener("message", (event) =>
    handleProgressEvent(appState, event),
  );
  appState.progressSocket.addEventListener("close", () =>
    clearClosedProgressSocket(appState, socket),
  );
}

function handleProgressEvent(appState: AppState, event: MessageEvent<string>): void {
  const progress = parseRunProgressEvent(event.data);
  appState.progressSequences[progress.run_id] = Math.max(
    appState.progressSequences[progress.run_id] ?? 0,
    progress.sequence,
  );
  void refreshRuns(appState);
}

function closeProgressSocket(appState: AppState): void {
  appState.progressSocket?.close();
  appState.progressSocket = null;
  appState.progressRunId = null;
}

function clearClosedProgressSocket(appState: AppState, socket: WebSocket): void {
  if (appState.progressSocket !== socket) {
    return;
  }
  appState.progressSocket = null;
  appState.progressRunId = null;
}

async function runRequest(appState: AppState, operation: () => Promise<void>): Promise<void> {
  appState.loading = true;
  appState.error = null;
  renderStatus(appState);
  try {
    await operation();
    appState.lastUpdated = new Date();
  } catch (error) {
    appState.error = error instanceof Error ? error.message : String(error);
  } finally {
    appState.loading = false;
    render(appState);
  }
}

function render(appState: AppState): void {
  renderMetrics(appState);
  renderRunList(appState);
  renderRunDetail(appState);
  renderReviewQueue(appState);
  renderStatus(appState);
}

function renderStatus(appState: AppState): void {
  setText("connection-state", connectionText(appState));
  setText("last-updated", updateText(appState));
}

function connectionText(appState: AppState): string {
  if (appState.loading) {
    return "Syncing";
  }
  if (appState.error) {
    return appState.error;
  }
  return `Connected to ${appState.apiBase}`;
}

function updateText(appState: AppState): string {
  if (!appState.lastUpdated) {
    return "Never updated";
  }
  return `Updated ${appState.lastUpdated.toLocaleTimeString()}`;
}

function readContractForm(): ContractSubmission {
  return {
    id: inputValue("contract-id"),
    repo_url: inputValue("repo-url"),
    base_sha: inputValue("base-sha"),
    scopes: scopeValues(),
    requires_human_review: byId<HTMLInputElement>("human-review").checked,
  };
}

function inputValue(id: string): string {
  return byId<HTMLInputElement>(id).value.trim();
}

function scopeValues(): string[] {
  return inputValue("scopes")
    .split(",")
    .map((scope) => scope.trim())
    .filter(Boolean);
}

function clearContractForm(): void {
  byId<HTMLFormElement>("submit-form").reset();
}
