import { invoke } from "@tauri-apps/api/core";
import type {
  Action,
  ActiveStatsPayload,
  AppConfig,
  ApprovalRequest,
  ApprovalResolution,
  BatchData,
  CoverageItem,
  CoveragePayload,
  Decision,
  ExecutionPath,
  LedgerEntry,
  LedgerFilter,
  Rule,
  SessionDetails,
  SessionInfo,
  SessionMode,
  SessionSummary,
  UndoResult,
} from "./types";

export const api = {
  getStartupArgs: () =>
    invoke<{ workspace: string | null; mode: SessionMode }>("get_startup_args"),
  chooseWorkspace: () => invoke<string | null>("choose_workspace"),
  startSession: (workspace: string, mode?: SessionMode) =>
    invoke<SessionInfo>("start_session", { workspace, mode: mode ?? "protected" }),
  stopSession: () => invoke<SessionSummary>("stop_session"),
  undoActiveSession: () => invoke<SessionSummary>("undo_active_session"),
  allowBatch: () => invoke<void>("allow_batch"),
  denyBatch: () => invoke<SessionSummary>("deny_batch"),
  undoSession: (sessionId: string) =>
    invoke<UndoResult>("undo_session", { sessionId }),
  listSessions: () => invoke<SessionSummary[]>("list_sessions"),
  getSession: (sessionId: string) =>
    invoke<SessionDetails>("get_session", { sessionId }),
  getActiveSession: () => invoke<SessionInfo | null>("get_active_session"),
  getPendingBatch: () => invoke<BatchData | null>("get_pending_batch"),
  getConfig: () => invoke<AppConfig>("get_config"),
  updateConfig: (config: AppConfig) => invoke<void>("update_config", { config }),
  // --- v0.2 additions ---
  getActiveStats: () => invoke<ActiveStatsPayload | null>("get_active_stats"),
  getLedger: (filter: LedgerFilter = {}) => {
    const args: Record<string, unknown> = {};
    if (filter.session_id !== undefined) args.sessionId = filter.session_id;
    if (filter.category !== undefined) args.category = filter.category;
    if (filter.risk !== undefined) args.risk = filter.risk;
    if (filter.limit !== undefined) args.limit = filter.limit;
    return invoke<LedgerEntry[]>("get_ledger", args);
  },
  // --- v0.2 Approval gate ---
  listPendingApprovals: () =>
    invoke<ApprovalRequest[]>("list_pending_approvals"),
  resolveApproval: (resolution: ApprovalResolution) =>
    invoke<void>("resolve_approval", { resolution }),
  previewLearnRule: (action: Action, decision: Decision) =>
    invoke<Rule>("preview_learn_rule", { action, decision }),
  // --- v0.2 Execution Path Matrix ---
  getEnforcementPaths: () => {
    console.log("[API] getEnforcementPaths called");
    return invoke<ExecutionPath[]>("get_enforcement_paths");
  },
  // --- Coverage dashboard ---
  getCoverage: () => invoke<CoveragePayload>("get_coverage"),
  // --- Autostart (uses tauri-plugin-autostart JS API) ---
  getAutostartEnabled: () => invoke<boolean>("plugin:autostart|isEnabled"),
  setAutostartEnabled: (enabled: boolean) =>
    enabled
      ? invoke("plugin:autostart|enable")
      : invoke("plugin:autostart|disable"),
};
