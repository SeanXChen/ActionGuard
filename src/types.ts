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
  reasons: readonly string[];
  sensitive: readonly string[];
  outside: readonly string[];
  asset?: Asset | null;
}

// ===========================================================================
// v0.3 — Contextual Facts Schema 2.0
// ===========================================================================

/** What class of resource an action targets — drives semantic policy rules. */
export type TargetClass =
  | "source_code"
  | "credential"
  | "system_secret"
  | "config"
  | "build_artifact"
  | "package_manifest"
  | "git_repo"
  | "user_data"
  | "external_resource"
  | "network_endpoint"
  | "unknown";

/** Who owns the resource being accessed. */
export type Ownership = "self" | "third_party" | "shared" | "unknown";

/** Where the action has effect. */
export type Externality = "local" | "third_party" | "public" | "external_system";

/** Side effects an action may produce. */
export type SideEffect =
  | "none"
  | "destructive"
  | "irreversible"
  | "third_party_impact"
  | "external_call"
  | "system_modification"
  | "publication";

/** Whether an action's effects can be reversed. */
export type Reversibility = "reversible" | "difficult" | "irreversible" | "unknown";

/** How sensitive a resource or action is. */
export type SensitivityLevel = "low" | "medium" | "high" | "critical";

/** v0.3 — The consequence dimension of an action. */
export interface Consequence {
  side_effect?: SideEffect;
  externality?: Externality;
  reversibility?: Reversibility;
  is_chain_link?: boolean;
}

/** v0.3 — Target context. */
export interface TargetContext {
  class?: TargetClass;
  sensitivity?: SensitivityLevel;
  ownership?: Ownership;
  ownership_note?: string;
}

/** v0.3 — Provenance: where an action came from. */
export interface Provenance {
  boundary?: string;
  confidence?: "verified" | "inferred" | "heuristic";
  observed_at?: string;
}

/** v0.3 — Type of detected action chain. */
export type ActionChainType =
  | "credential_access"
  | "credential_collection"
  | "exfiltration"
  | "privilege_escalation"
  | "destructive_cascade"
  | "other";

/** v0.3 — Action correlation: links actions into sequences and chains. */
export interface ActionCorrelation {
  related_actions?: readonly string[];
  chain_type?: ActionChainType;
  chain_description?: string;
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
  contains?: readonly string[];
  absolute_path?: string | null;
}

export interface Evidence {
  packages_added?: readonly string[];
  lockfile_modified?: boolean;
  lockfile_diff?: readonly string[];
  contains_keys?: readonly string[];
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
  reasons?: readonly string[];
  matched_rule?: string | null;
  decision?: Decision | null;
  result?: string | null;
  asset?: Asset | null;
  evidence?: Evidence | null;
  // v0.2: data classification
  data_class?: string;
  credential_type?: string;
  // v0.3: contextual facts
  target_class?: TargetClass;
  target_sensitivity?: SensitivityLevel;
  ownership?: Ownership;
  externality?: Externality;
  side_effect?: SideEffect;
  reversibility?: Reversibility;
  consequence?: Consequence;
  target_context?: TargetContext;
  provenance?: Provenance;
  correlation?: ActionCorrelation;
  parent_action?: string;
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

/** v0.3 — Confidence level of a detection. */
export type ProvenanceConfidence = "verified" | "inferred" | "heuristic";

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
  // --- P1: credential exfiltration chain detection ---
  credential_sources_touched?: number;
  credential_types?: readonly string[];
  chain_detected?: boolean;
  // --- v0.3: Contextual Session Risk ---
  target_classes_touched?: number;
  target_class_types?: readonly string[];
  max_sensitivity?: SensitivityLevel;
  destructive_actions?: number;
  irreversible_actions?: number;
  third_party_actions?: number;
  detected_chains?: readonly ActionChainType[];
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
  reasons: readonly string[];
  asset?: Asset | null;
  evidence?: Evidence | null;
  // v0.2: data classification
  data_class?: string;
  credential_type?: string;
  // v0.3: chain detection
  chain_tag?: string;
  // v0.3: contextual facts
  target_class?: TargetClass;
  target_sensitivity?: SensitivityLevel;
  ownership?: Ownership;
  externality?: Externality;
  side_effect?: SideEffect;
  correlation?: ActionCorrelation;
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

/** Coverage item from `get_coverage`. */
export interface CoverageItem {
  name: string;
  status: "enforced" | "observe" | "inactive" | "not_detected";
  kind: string;
  note: string;
  quality: "high" | "generic" | "observe_only" | "none";
}

export interface CoveragePayload {
  items: CoverageItem[];
  enforced_count: number;
  observe_count: number;
  inactive_count: number;
  not_detected_count: number;
  has_generic_shell: boolean;
}
