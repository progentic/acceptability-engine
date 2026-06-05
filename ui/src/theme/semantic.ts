import type { RunStatus } from "../models";
import { colors } from "./colors";

export const semantic = {
  status: {
    queued: colors.accent.slate,
    running: colors.accent.slate,
    pendingReview: colors.accent.ochre,
    approved: colors.accent.sage,
    rejected: colors.danger.red,
    failedInternal: colors.danger.red,
  },

  action: {
    primary: colors.brand.rust,
    primaryHover: colors.brand.rustHover,
  },

  text: {
    primary: colors.ui.charcoal,
    secondary: colors.ui.muted,
    onAction: colors.ui.eggshell,
  },

  surface: {
    background: colors.ui.eggshell,
    card: colors.ui.surface,
    border: colors.ui.border,
  },
} as const;

const STATUS_CLASS: Record<RunStatus, string> = {
  QUEUED: "status-queued",
  RUNNING: "status-running",
  PENDING_HUMAN_REVIEW: "status-pending-review",
  APPROVED: "status-approved",
  REJECTED: "status-rejected",
  FAILED_INTERNAL: "status-failed-internal",
};

const CSS_VARIABLES: Record<string, string> = {
  "--action-primary": semantic.action.primary,
  "--action-primary-hover": semantic.action.primaryHover,
  "--status-approved": semantic.status.approved,
  "--status-failed-internal": semantic.status.failedInternal,
  "--status-pending-review": semantic.status.pendingReview,
  "--status-queued": semantic.status.queued,
  "--status-rejected": semantic.status.rejected,
  "--status-running": semantic.status.running,
  "--surface-background": semantic.surface.background,
  "--surface-border": semantic.surface.border,
  "--surface-card": semantic.surface.card,
  "--text-on-action": semantic.text.onAction,
  "--text-primary": semantic.text.primary,
  "--text-secondary": semantic.text.secondary,
};

export function statusClass(status: RunStatus | string): string {
  return STATUS_CLASS[status as RunStatus] ?? STATUS_CLASS.RUNNING;
}

export function applySemanticTheme(root: HTMLElement): void {
  for (const [name, value] of Object.entries(CSS_VARIABLES)) {
    root.style.setProperty(name, value);
  }
}
