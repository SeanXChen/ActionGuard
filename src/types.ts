// ===========================================================================
// Action identity & kind
// ===========================================================================

export type ActionKind = "create" | "modify" | "delete" | "rename";

/** Top-level action taxonomy. v0.2 ships File / Shell / Git / Package / Secret. */
export type ActionCategory = "file" | "shell" | "git" | "package" | "secret";

/** Policy decision. `ask` corresponds to YAML `action: confirm`. */
export type Decision = "allow" | "ask" | "deny";

// ===========================================================================
// Risk
// ===========================================================================

export type RiskLevel = "low" | "medium" | "high" | "critical";

export interface RiskResult {
  level: RiskLevel;
  reasons: string[];
  sensitive: string[];
  outside: string[];
  asset?: Asset | null;
}

// ===========================================================================
// Evidence & sensitive assets
// ===========================================================================

export type AssetKind =
  | "env_file"
  | "ssh_key"
  | "pem_key"
  | "aws_creds"
  | "gpg_keychain"
  | "git_dir"
  | "credentials_json"
  | "other";

export interface Asset {
  kind: AssetKind;
  matched_pattern: string;
  contains?: string[];
  absolute_path?: string | null;
}

export interface Evidence {
  packages_added?: string[];
  lockfile_modified?: boolean;
  lockfile_diff?: string[];
  contains_keys?: string[];
  install_size_bytes?: number;
  global?: boolean;
}

// ===========================================================================
// Action — the unified struct (replaces v0.1 FileChange)
// ===========================================================================

export interface Action {
  id?: string;
  timestamp?: string;
  agent?: string | null;
  category?: ActionCategory;
  kind?: string | null;
  path?: string | null;
  target?: string | null;
  cwd?: string | null;
  action: ActionKind;
  sensitive?: boolean;
  outside?: boolean;
  from?: string | null;
  risk?: RiskLevel | null;
  reasons?: string[];
  matched_rule?: string | null;
  decision?: Decision | null;
  result?: string | null;
  asset?: Asset | null;
  evidence?: Evidence | null;
}

/** v0.1 backward-compat alias. */
export type FileChange = Action;

// ===========================================================================
// Counts (File-only) + new category/risk counts
// ===========================================================================

export interface Counts {
  create: number;
  modify: number;
  delete: number;
  rename: number;
}

export interface CategoryCounts {
  file: number;
  shell: number;
  git: number;
  package: number;
  secret: number;
}

export interface RiskCounts {
  low: number;
  medium: number;
  high: number;
  critical: number;
}

/** v0.3 — enforcement outcome split (Detection ≠ Protection). */
export interface EnforcementCounts {
  enforced: number;
  observed: number;
  bypassed: number;
  unsupported: number;
}

// ===========================================================================
// Batch
// ===========================================================================

export interface BatchData {
  counts: Counts;
  actions: Action[];
  risk: RiskResult;
}

// ===========================================================================
// Sessions
// ===========================================================================

export type SessionStatus = "active" | "completed" | "denied";

/// v0.2 Mode A/B: Observe = record only, Protected = block high-risk.
export type SessionMode = "observe" | "protected";

export interface SessionInfo {
  id: string;
  num: number;
  workspace: string;
  started_at: string;
  snapshot_files: number;
  mode: SessionMode;
}

export interface SessionSummary {
  id: string;
  num: number;
  workspace: string;
  started_at: string;
  ended_at?: string | null;
  duration_secs: number;
  counts: Counts;
  total: number;
  risk: RiskLevel;
  status: SessionStatus;
  undone: boolean;
  sensitive_count: number;
  outside_count: number;
  // --- v0.2 additive (old sessions load with zeros) ---
  category_counts?: CategoryCounts;
  risk_counts?: RiskCounts;
  actions_protected?: number;
  actions_blocked?: number;
  mode?: SessionMode;
  // --- v0.3 additive ---
  enforcement_counts?: EnforcementCounts;
}

export interface SessionDetails {
  summary: SessionSummary;
  actions: Action[];
}

export interface UndoResult {
  restored_files: number;
  deleted_files: number;
  removed_dirs: number;
  skipped: number;
}

// ===========================================================================
// Config
// ===========================================================================

export interface AppConfig {
  next_session_num: number;
  ignore_patterns: string[];
  detect_outside: boolean;
  // --- v0.2 additive ---
  shell_blocking?: boolean;
  approval_timeout_secs?: number;
  default_agent?: string;
}

export interface CountsPayload {
  total: Counts;
  batch: Counts;
  risk_level: RiskLevel;
}

// ===========================================================================
// v0.2 — Live session stats + Action Ledger (UI/CLI)
// ===========================================================================

/** UI/CLI-friendly view of an Action. Carries a derived `time_hms`. */
export interface LedgerEntry {
  id: string;
  timestamp: string;
  time_hms: string;
  agent: string;
  category: ActionCategory;
  kind: string;
  target: string;
  risk: RiskLevel;
  decision: Decision;
  result: string;
  reasons: string[];
  asset?: Asset | null;
  evidence?: Evidence | null;
}

export interface ActiveStatsPayload {
  session_id: string;
  session_num: number;
  workspace: string;
  started_at: string;
  total_actions: number;
  category_counts: CategoryCounts;
  risk_counts: RiskCounts;
  actions_protected: number;
  actions_blocked: number;
  enforcement_counts: EnforcementCounts;
  awaiting_review: boolean;
}

export interface LedgerFilter {
  session_id?: string | null;
  category?: ActionCategory | null;
  risk?: RiskLevel | null;
  limit?: number | null;
}

// ===========================================================================
// v0.2 — Policy + Approval gate
// ===========================================================================

export type PolicySource = "builtin" | "user";

export interface MatchSpec {
  category?: ActionCategory | null;
  command?: string | null;
  path?: string | null;
  args_contains?: string[] | null;
  regex?: string | null;
}

export interface Rule {
  id: string;
  match: MatchSpec;
  action: Decision;
  risk?: RiskLevel | null;
  reason?: string | null;
  source: PolicySource;
}

export interface ApprovalRequest {
  id: string;
  session_id: string;
  action: Action;
  decision_due_at: string;
  timeout_secs: number;
}

export interface ApprovalResolution {
  approval_id: string;
  decision: Decision;
  learn_rule?: Rule | null;
}

// ===========================================================================
// v0.2 — Execution Path Matrix (platform capability surfacing)
// ===========================================================================

/** v0.3 — Capability Tier Model (L1 observe … L4 system). */
export type CapabilityTier = "l1_observe" | "l2_pre_action" | "l3_runtime" | "l4_system";

export interface ExecutionPath {
  path: string;
  observe: boolean;
  block: boolean;
  note: string;
  /** v0.3 — tier implied by observe/block; null = not covered. */
  tier: CapabilityTier | null;
}
