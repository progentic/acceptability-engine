import { ApiClient } from "./api";
import type { RunDetail, RunListItem, RunStatus } from "./models";

export interface AppState {
  api: ApiClient;
  apiBase: string;
  runs: RunListItem[];
  selectedRunId: number | null;
  selectedRun: RunDetail | null;
  statusFilter: RunStatus | "ALL";
  loading: boolean;
  error: string | null;
  lastUpdated: Date | null;
}

export function createState(apiBase: string): AppState {
  return {
    api: new ApiClient(apiBase),
    apiBase,
    runs: [],
    selectedRunId: null,
    selectedRun: null,
    statusFilter: "ALL",
    loading: false,
    error: null,
    lastUpdated: null,
  };
}

export function selectedRunFromList(state: AppState): RunListItem | null {
  if (state.selectedRunId === null) {
    return null;
  }
  return state.runs.find((run) => run.run_id === state.selectedRunId) ?? null;
}

export function reviewRuns(state: AppState): RunListItem[] {
  return state.runs.filter((run) => run.status === "PENDING_HUMAN_REVIEW");
}

export function visibleRuns(state: AppState): RunListItem[] {
  if (state.statusFilter === "ALL") {
    return state.runs;
  }
  return state.runs.filter((run) => run.status === state.statusFilter);
}
