import { listen } from "@tauri-apps/api/event";
import { reactive, readonly } from "vue";
import { api } from "./api";
import type {
  ActiveStatsPayload,
  ApprovalRequest,
  BatchData,
  Counts,
  CountsPayload,
  LedgerEntry,
  RiskLevel,
  SessionInfo,
  SessionMode,
  SessionSummary,
} from "./types";

export type View = "home" | "session" | "review" | "history";

export interface ParaStats {
  n: number;
  high: number;
  medium: number;
  critical: number;
  flagged: number;
  rate: number;
  actionsProtected: number;
  actionsBlocked: number;
}

interface StoreState {
  view: View;
  session: SessionInfo | null;
  // Legacy v0.1 file-only counters (still emitted by `actionguard://counts`).
  total: Counts;
  batch: Counts;
  riskLevel: RiskLevel;
  pendingBatch: BatchData | null;
  awaitingReview: boolean;
  lastEnded: SessionSummary | null;
  busy: boolean;
  error: string | null;
  sessions: SessionSummary[];
  paraStats: ParaStats;
  sessionsLoaded: boolean;
  // --- v0.2 additions ---
  activeStats: ActiveStatsPayload | null;
  ledger: LedgerEntry[];
  // v0.2 Approval gate: queue of pending approvals surfaced from the shell bridge.
  pendingApprovals: ApprovalRequest[];
  // v0.2 CLI startup args (--workspace / --observe passed via `actionguard protect`).
  startupArgs: { workspace: string | null; mode: SessionMode } | null;
}

function emptyPara(): ParaStats {
  return {
    n: 0,
    high: 0,
    medium: 0,
    critical: 0,
    flagged: 0,
    rate: 0,
    actionsProtected: 0,
    actionsBlocked: 0,
  };
}

const state = reactive<StoreState>({
  view: "home",
  session: null,
  total: { create: 0, modify: 0, delete: 0, rename: 0 },
  batch: { create: 0, modify: 0, delete: 0, rename: 0 },
  riskLevel: "low",
  pendingBatch: null,
  awaitingReview: false,
  lastEnded: null,
  busy: false,
  error: null,
  sessions: [],
  paraStats: emptyPara(),
  sessionsLoaded: false,
  activeStats: null,
  ledger: [],
  pendingApprovals: [],
  startupArgs: null,
});

let listening = false;

function computePara(list: SessionSummary[]): ParaStats {
  const n = list.length;
  const high = list.filter((s) => s.risk === "high").length;
  const medium = list.filter((s) => s.risk === "medium").length;
  const critical = list.filter((s) => s.risk === "critical").length;
  const flagged = high + medium + critical;
  const actionsProtected = list.reduce(
    (acc, s) => acc + (s.actions_protected ?? 0),
    0,
  );
  const actionsBlocked = list.reduce(
    (acc, s) => acc + (s.actions_blocked ?? 0),
    0,
  );
  return {
    n,
    high,
    medium,
    critical,
    flagged,
    rate: n ? Math.round((flagged / n) * 100) : 0,
    actionsProtected,
    actionsBlocked,
  };
}

async function refreshSessions() {
  try {
    const list = await api.listSessions();
    state.sessions = list;
    state.paraStats = computePara(list);
    state.sessionsLoaded = true;
  } catch {
    /* ignore */
  }
}

async function refreshActiveStats() {
  try {
    const stats = await api.getActiveStats();
    state.activeStats = stats;
  } catch {
    /* ignore */
  }
}

async function refreshLedger(limit = 50) {
  try {
    const entries = await api.getLedger({ limit });
    state.ledger = entries;
  } catch {
    /* ignore */
  }
}

async function refreshPendingApprovals() {
  try {
    state.pendingApprovals = await api.listPendingApprovals();
  } catch {
    /* ignore */
  }
}

async function resolveApproval(
  approvalId: string,
  decision: "allow" | "deny",
  learnRule: boolean,
  rulePreview?: import("./types").Rule,
) {
  const resolution: import("./types").ApprovalResolution = {
    approval_id: approvalId,
    decision,
    learn_rule: learnRule ? rulePreview ?? null : null,
  };
  await api.resolveApproval(resolution);
  // Optimistic removal — the `actionguard://approval/resolved` event also
  // fires from the backend, but removing locally first avoids a flash of
  // the resolved card while the event round-trips.
  state.pendingApprovals = state.pendingApprovals.filter(
    (p) => p.id !== approvalId,
  );
}

export function useStore() {
  async function init() {
    if (!listening) {
      listening = true;
      await listen<CountsPayload>("actionguard://counts", (e) => {
        state.total = e.payload.total;
        state.batch = e.payload.batch;
        state.riskLevel = e.payload.risk_level;
      });
      await listen<BatchData>("actionguard://batch", (e) => {
        state.pendingBatch = e.payload;
      });
      await listen<BatchData>("actionguard://risk", (e) => {
        state.pendingBatch = e.payload;
        state.awaitingReview = true;
        state.view = "review";
      });
      await listen<SessionSummary>("actionguard://ended", (e) => {
        state.lastEnded = e.payload;
        state.session = null;
        state.activeStats = null;
        state.ledger = [];
        state.pendingBatch = null;
        state.awaitingReview = false;
        state.pendingApprovals = [];
        state.total = { create: 0, modify: 0, delete: 0, rename: 0 };
        state.batch = { create: 0, modify: 0, delete: 0, rename: 0 };
        void refreshSessions();
      });
      // v0.2 Approval gate events from the shell bridge.
      await listen<ApprovalRequest>("actionguard://approval/request", (e) => {
        state.pendingApprovals = [
          ...state.pendingApprovals,
          e.payload,
        ];
      });
      await listen<{ approval_id: string }>(
        "actionguard://approval/resolved",
        (e) => {
          state.pendingApprovals = state.pendingApprovals.filter(
            (p) => p.id !== e.payload.approval_id,
          );
          void refreshActiveStats();
          void refreshLedger();
        },
      );
    }

    const [active, pending] = await Promise.all([
      api.getActiveSession(),
      api.getPendingBatch(),
      refreshSessions(),
    ]);
    // Read CLI startup args (from `actionguard protect <workspace> [--observe]`).
    state.startupArgs = await api.getStartupArgs();
    if (active) {
      state.session = active;
      state.view = "session";
      await Promise.all([
        refreshActiveStats(),
        refreshLedger(),
        refreshPendingApprovals(),
      ]);
      if (pending) {
        state.pendingBatch = pending;
        if (
          pending.risk.level === "high" ||
          pending.risk.level === "critical"
        ) {
          state.awaitingReview = true;
          state.view = "review";
        }
      }
    }
  }

  function setView(v: View) {
    state.view = v;
  }

  function navigate(view: View, session?: SessionInfo | null) {
    if (session !== undefined) state.session = session;
    state.view = view;
  }

  return {
    state: readonly(state),
    init,
    setView,
    navigate,
    clearError: () => (state.error = null),
    refreshSessions,
    refreshActiveStats,
    refreshLedger,
    refreshPendingApprovals,
    resolveApproval,
  };
}
