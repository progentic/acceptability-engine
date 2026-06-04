import type { RunStatus } from "./models";

export function formatDate(seconds: number): string {
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(seconds * 1000));
}

export function formatDuration(value: number | null): string {
  if (value === null) {
    return "n/a";
  }
  if (value < 1000) {
    return `${value} ms`;
  }
  return `${(value / 1000).toFixed(1)} s`;
}

export function formatBytes(value: number | null): string {
  if (value === null) {
    return "n/a";
  }
  if (value < 1024) {
    return `${value} B`;
  }
  return `${(value / 1024).toFixed(1)} KiB`;
}

export function gateName(gateNumber: number): string {
  return (
    {
      1: "Contract",
      2: "Workspace",
      3: "Boundary",
      4: "Formatting",
      5: "Static",
      6: "Build",
      7: "Tests",
      8: "Supply",
      9: "Sandbox",
    }[gateNumber] ?? `Gate ${gateNumber}`
  );
}

export function statusTone(status: RunStatus | string): string {
  if (status === "APPROVED") {
    return "positive";
  }
  if (status === "REJECTED" || status === "FAILED_INTERNAL") {
    return "negative";
  }
  if (status === "PENDING_HUMAN_REVIEW") {
    return "review";
  }
  return "active";
}

export function isLiveStatus(status: RunStatus): boolean {
  return status === "QUEUED" || status === "RUNNING";
}
