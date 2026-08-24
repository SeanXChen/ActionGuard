use serde::{Deserialize, Serialize};

// ===========================================================================
// Action identity & kind
// ===========================================================================

/// File-system action verbs (kept for v0.1 backward-compat with .actions.json).
/// Stays File-only; non-File categories use the free-form `Action.kind` string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionKind {
    Create,
    Modify,
    Delete,
    Rename,
}

impl Default for ActionKind {
    fn default() -> Self {
        ActionKind::Modify
    }
}

/// Top-level action taxonomy. v0.2 ships File / Shell / Git / Package / Secret.
/// Future categories (Network, Browser, Cloud, Database, Communication,
/// Financial) are added here in later versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActionCategory {
    #[default]
    File,
    Shell,
    Git,
    Package,
    Secret,
}

impl ActionCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionCategory::File => "file",
            ActionCategory::Shell => "shell",
            ActionCategory::Git => "git",
            ActionCategory::Package => "package",
            ActionCategory::Secret => "secret",
        }
    }
}

// ===========================================================================
// Risk & decisions
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Low
    }
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

/// Policy decision. `Ask` corresponds to YAML `action: confirm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Ask,
    Deny,
}

impl Default for Decision {
    fn default() -> Self {
        Decision::Allow
    }
}

// ===========================================================================
// Boundary & enforcement — the vendor-neutral abstraction
// ===========================================================================

/// Where an action entered ActionGuard — i.e. which **Boundary Class** it
/// crossed. The boundary decides whether a policy decision can be *enforced*
/// before execution or only *observed* after the fact.
///
/// Boundary Classes (A–F) are the vendor-neutral way to classify any
/// automation product:
///   A. Tool Hook          → [`BoundaryKind::ToolHook`]
///   B. Execution Approval → [`BoundaryKind::ExecApproval`]
///   C. Protected Shell    → [`BoundaryKind::ProtectedShell`]
///   D. Runtime Sandbox    → [`BoundaryKind::RuntimeHook`]
///   E. System Enforcement → [`BoundaryKind::SystemLevel`]
///   F. Remote Automation  → [`BoundaryKind::Remote`]
///
/// ActionGuard is deliberately brand-agnostic: it does not care which agent
/// produced the action, only which boundary class it crossed. A new product
/// is mapped to a class, not to a bespoke integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryKind {
    /// **C. Protected Shell** — a protected interactive shell hook
    /// (bash/zsh/fish, PowerShell PSReadLine). Pre-action, enforceable.
    ProtectedShell,
    /// **A. Tool Hook** — an automation tool's hook/middleware that fires
    /// before a tool executes (e.g. CodeBuddy PreToolUse). Pre-action,
    /// enforceable.
    ToolHook,
    /// **B. Execution Approval** — the automation itself defines an
    /// execution boundary (exec policy / allowlist / host approvals, e.g.
    /// OpenClaw `exec` approvals, Manus Desktop per-command approval).
    /// ActionGuard can attach as an independent policy layer above it.
    ExecApproval,
    /// **D. Runtime Sandbox** — a runtime/process-level hook or sandbox
    /// (future L3). Pre-action, enforceable by design.
    RuntimeHook,
    /// No pre-action boundary — the action was recorded after the fact.
    /// Decisions can only be *observed*, never guaranteed.
    ObserveOnly,
    /// **E. System Enforcement** (future L4). Every local action crosses it.
    SystemLevel,
    /// **F. Remote Automation** — the action never lands on this machine
    /// (cloud worker / remote browser / remote sandbox). Outside the local
    /// ActionGuard boundary by address space, not by product choice.
    Remote,
}

impl Default for BoundaryKind {
    fn default() -> Self {
        // Most conservative default: if we do not know the boundary, we
        // cannot claim to have blocked anything.
        BoundaryKind::ObserveOnly
    }
}

impl BoundaryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            BoundaryKind::ProtectedShell => "protected_shell",
            BoundaryKind::ToolHook => "tool_hook",
            BoundaryKind::ExecApproval => "exec_approval",
            BoundaryKind::RuntimeHook => "runtime_hook",
            BoundaryKind::ObserveOnly => "observe_only",
            BoundaryKind::SystemLevel => "system_level",
            BoundaryKind::Remote => "remote",
        }
    }

    /// True when a deny/ask decision can be enforced *before* execution.
    /// Observe-only and remote boundaries cannot be pre-empted locally.
    pub fn can_enforce(&self) -> bool {
        !matches!(
            self,
            BoundaryKind::ObserveOnly | BoundaryKind::Remote
        )
    }

    /// Human label = the Boundary Class name (A–F). Keeps `boundary list`
    /// aligned with the public Boundary Classes map.
    pub fn label(&self) -> &'static str {
        match self {
            BoundaryKind::ProtectedShell => "Protected Shell (C)",
            BoundaryKind::ToolHook => "Tool Hook (A)",
            BoundaryKind::ExecApproval => "Exec Approval (B)",
            BoundaryKind::RuntimeHook => "Runtime Sandbox (D)",
            BoundaryKind::ObserveOnly => "Observe only",
            BoundaryKind::SystemLevel => "System Enforcement (E)",
            BoundaryKind::Remote => "Remote (F)",
        }
    }
}

/// What actually happened to the action at the boundary. Deliberately
/// distinct from `Decision`: **a policy decision is not an outcome.**
/// `Decision::Deny` + `EnforcementStatus::Bypassed` is the honest way to
/// record "ActionGuard said no, but the executor got around the boundary".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementStatus {
    /// The decision was applied before execution (deny → command did not run).
    Enforced,
    /// The action was recorded, but the boundary could not block it.
    Observed,
    /// The execution path bypassed the boundary entirely.
    Bypassed,
    /// No observation or enforcement path exists for this action.
    Unsupported,
}

impl EnforcementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnforcementStatus::Enforced => "enforced",
            EnforcementStatus::Observed => "observed",
            EnforcementStatus::Bypassed => "bypassed",
            EnforcementStatus::Unsupported => "unsupported",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            EnforcementStatus::Enforced => "Enforced",
            EnforcementStatus::Observed => "Observed",
            EnforcementStatus::Bypassed => "Bypassed",
            EnforcementStatus::Unsupported => "Unsupported",
        }
    }
}

// ===========================================================================
// Capability Tier Model — what ActionGuard can *actually* do on a path
// ===========================================================================

/// Capability Tier — the honest scale of what ActionGuard can do on a given
/// execution path. This is the "Detection ≠ Protection" model made concrete:
///
/// - **L1 Observe**  — ActionGuard can *see* the action (record it to the
///   ledger) but cannot stop it before execution. Decisions are advisory.
/// - **L2 Pre-action** — ActionGuard can *block* the action *before* it
///   executes (shell preexec hooks, tool hooks, exec-approval layers).
///   A `Deny` means the command does not run.
/// - **L3 Runtime** — process-level enforcement / sandboxing at runtime
///   (planned, not implemented in v0.3).
/// - **L4 System** — OS-level, vendor-independent enforcement
///   (planned, not implemented in v0.3).
///
/// Tiers are strictly increasing: a path that supports L2 also satisfies
/// L1 (it observes everything it can block). The GUI and CLI surface this so
/// users never mistake "detected" for "protected".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityTier {
    /// Observe only — record, never pre-empt.
    L1Observe,
    /// Pre-action block — deny before execution.
    L2PreAction,
    /// Runtime sandbox / process-level control (future L3).
    L3Runtime,
    /// OS-level, vendor-independent enforcement (future L4).
    L4System,
}

impl CapabilityTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            CapabilityTier::L1Observe => "L1",
            CapabilityTier::L2PreAction => "L2",
            CapabilityTier::L3Runtime => "L3",
            CapabilityTier::L4System => "L4",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CapabilityTier::L1Observe => "L1 Observe",
            CapabilityTier::L2PreAction => "L2 Pre-action",
            CapabilityTier::L3Runtime => "L3 Runtime (future)",
            CapabilityTier::L4System => "L4 System (future)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CapabilityTier::L1Observe => {
                "records the action, never blocks it before execution"
            }
            CapabilityTier::L2PreAction => {
                "blocks a deny decision before the command executes"
            }
            CapabilityTier::L3Runtime => {
                "process-level sandboxing at runtime (planned)"
            }
            CapabilityTier::L4System => {
                "OS-level, vendor-independent enforcement (planned)"
            }
        }
    }

    /// The tier implied by observe/block capability of an execution path.
    pub fn from_capabilities(observe: bool, block: bool) -> Option<Self> {
        if !observe {
            return None; // not covered at all
        }
        Some(if block {
            CapabilityTier::L2PreAction
        } else {
            CapabilityTier::L1Observe
        })
    }
}

/// How an action entered ActionGuard. Every boundary adapter stamps this
/// context onto the action before handing it to the core. The core never
/// special-cases agent brands — it reads `boundary` and enforces accordingly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContext {
    /// "agent" | "automation" | "workflow" | "human" | "unknown"
    #[serde(default = "default_source_type")]
    pub source_type: String,
    /// Human-readable source identifier (e.g. "codebuddy", "manus-desktop").
    /// `None` when the source is unknown — never assume a brand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    #[serde(default)]
    pub boundary: BoundaryKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// Session mode — the "technical honesty" switch.
///
/// - `Observe`  (Mode A): record every action to the ledger, but NEVER block.
///   Use when you want to see what an agent does without slowing it down.
///
/// - `Protected` (Mode B): block high-risk actions BEFORE they execute.
///   Shell commands go through the bridge + approval gate.
///   File changes are still recorded (same as Observe).
///
/// Default is `Protected` for backward compat with v0.1/v0.2 sessions that
/// predate the mode field (serde defaults to Protected when missing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    Observe,
    Protected,
}

impl Default for SessionMode {
    fn default() -> Self {
        SessionMode::Protected
    }
}

impl SessionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionMode::Observe => "observe",
            SessionMode::Protected => "protected",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskResult {
    pub level: RiskLevel,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub sensitive: Vec<String>,
    #[serde(default)]
    pub outside: Vec<String>,
    /// v0.2: populated when the action touches a recognized sensitive asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<Asset>,
}

// ===========================================================================
// Evidence & sensitive assets
// ===========================================================================

/// Populated for Shell + Package side-effects and Secret reads.
/// Values are never surfaced — only redacted key NAMES.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Evidence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packages_added: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lockfile_modified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lockfile_diff: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contains_keys: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    #[default]
    EnvFile,
    SshKey,
    PemKey,
    AwsCreds,
    GpgKeychain,
    GitDir,
    CredentialsJson,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Asset {
    pub kind: AssetKind,
    pub matched_pattern: String,
    #[serde(default)]
    pub contains: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absolute_path: Option<String>,
}

// ===========================================================================
// Action — the unified struct (replaces v0.1 FileChange)
// ===========================================================================

/// A single thing an agent did (or tried to do) on the machine.
///
/// v0.1 called this `FileChange` and only carried filesystem events. v0.2
/// generalizes it across File / Shell / Git / Package / Secret categories.
/// All new fields are `#[serde(default)]` so v0.1 `.actions.json` files still
/// deserialize cleanly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    // --- identity ---
    #[serde(default)]
    pub id: String,
    /// Owning session id, stamped by `storage::append_ledger`. Stored on the
    /// line (not just the file name) so each ledger record is self-contained:
    /// a future cloud-sync layer can upload lines as-is, keyed by
    /// (session_id, id), without re-deriving context from file paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Type of the source that produced this action. Defaults to "agent"
    /// for backward compat with v0.1/v0.2 ledgers. Future sources can set
    /// "automation" (browser-assistant, RPA), "workflow" (custom pipelines),
    /// "human" (manual shell entry), etc. The `agent` field above is the
    /// source's *name* (e.g. "claude-code"); this field is its *type*.
    #[serde(default = "default_source_type", skip_serializing_if = "is_default_source_type")]
    pub source_type: String,
    #[serde(default)]
    pub category: ActionCategory,
    /// Free-form verb: "create" | "modify" | "delete" | "rename" | "execute"
    /// | "install" | "read" | "push" | "reset" | ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    // --- target ---
    /// File path (workspace-relative) for category=File. Empty for non-File.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// For non-File categories: the canonical command/argument string
    /// (e.g. "git reset --hard HEAD~10").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    // --- file-specific (kept for v0.1 backward compat with .actions.json) ---
    pub action: ActionKind,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub outside: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    // --- classification (populated at ingest by `classify`) ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<RiskLevel>,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_rule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<Decision>,
    /// "allowed" | "blocked" | "executed" | "restored"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Boundary the action entered through. `None` on legacy records
    /// (pre-boundary schema) → treated as observe-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<BoundaryKind>,
    /// Enforcement outcome at the boundary — distinct from `decision`.
    /// A `Deny` + `Bypassed` pair records a decision that could not be
    /// applied, which is exactly the evidence safety analysis needs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<EnforcementStatus>,

    // --- asset / side-effect evidence ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<Asset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Evidence>,

    // --- user behavior (approval popup telemetry, local-only) ---
    // Set only on the *resolution* ledger row of an approval-gated action.
    // `Some(true)` = the user allowed an action the policy wanted to gate
    // (override); `Some(false)` = the user confirmed the gate. `None` on
    // every other row. Feeds User Override Rate in `actionguard stats`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_override: Option<bool>,
    /// When the user dismissed the popup. Lets us compute human wait time
    /// as `resolved_at − timestamp`. `None` unless `user_override` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
}

/// v0.1 backward-compat alias. Existing imports keep compiling.
pub type FileChange = Action;

impl Action {
    /// Convenience constructor for a file-system action.
    #[allow(dead_code)]
    pub fn new_file(path: String, action: ActionKind) -> Self {
        Self {
            id: new_id(),
            session_id: None,
            timestamp: now_str(),
            agent: None,
            source_type: default_source_type(),
            category: ActionCategory::File,
            kind: Some(action.as_str().to_string()),
            path: Some(path),
            target: None,
            cwd: None,
            action,
            sensitive: false,
            outside: false,
            from: None,
            risk: None,
            reasons: Vec::new(),
            matched_rule: None,
            decision: None,
            result: None,
            boundary: None,
            enforcement: None,
            asset: None,
            evidence: None,
            user_override: None,
            resolved_at: None,
        }
    }

    /// v0.1 alias for callers that used `FileChange::new(path, action)`.
    #[allow(dead_code)]
    pub fn new(path: String, action: ActionKind) -> Self {
        Self::new_file(path, action)
    }

    /// Build a Shell-category action from a raw command string.
    #[allow(dead_code)]
    pub fn new_shell(cmd: String, cwd: Option<String>, agent: Option<String>) -> Self {
        Self {
            id: new_id(),
            session_id: None,
            timestamp: now_str(),
            agent,
            source_type: default_source_type(),
            category: ActionCategory::Shell,
            kind: Some("execute".to_string()),
            path: None,
            target: Some(cmd),
            cwd,
            action: ActionKind::Modify,
            sensitive: false,
            outside: false,
            from: None,
            risk: None,
            reasons: Vec::new(),
            matched_rule: None,
            decision: None,
            result: None,
            boundary: None,
            enforcement: None,
            asset: None,
            evidence: None,
            user_override: None,
            resolved_at: None,
        }
    }

    /// Build a Shell-category action with an explicit source type.
    ///
    /// Use this for non-agent sources: `source_type = "automation"` for
    /// browser-assistant / RPA, `"workflow"` for custom pipelines,
    /// `"human"` for manual shell entry. The `name` parameter is the
    /// source's human-readable identifier (e.g. "claude-code",
    /// "browser-assistant", "ci-runner") and is stored in `agent` for
    /// backward compatibility with the v0.1/v0.2 ledger format.
    #[allow(dead_code)]
    pub fn new_shell_from_source(
        cmd: String,
        cwd: Option<String>,
        source_type: &str,
        name: Option<String>,
    ) -> Self {
        let mut a = Self::new_shell(cmd, cwd, name);
        a.source_type = source_type.to_string();
        a
    }

    /// Workspace-relative path as `&str` (empty when absent).
    pub fn path_str(&self) -> &str {
        self.path.as_deref().unwrap_or("")
    }

    /// Target string (command for non-File categories; falls back to path).
    pub fn target_str(&self) -> &str {
        self.target
            .as_deref()
            .or(self.path.as_deref())
            .unwrap_or("")
    }
}

impl ActionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionKind::Create => "create",
            ActionKind::Modify => "modify",
            ActionKind::Delete => "delete",
            ActionKind::Rename => "rename",
        }
    }
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ===========================================================================
// Counts (File-only, kept for v0.1 compat) + new category/risk counts
// ===========================================================================

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Counts {
    pub create: u32,
    pub modify: u32,
    pub delete: u32,
    pub rename: u32,
}

impl Counts {
    pub fn total(&self) -> u32 {
        self.create + self.modify + self.delete + self.rename
    }

    pub fn add(&mut self, kind: ActionKind) {
        match kind {
            ActionKind::Create => self.create += 1,
            ActionKind::Modify => self.modify += 1,
            ActionKind::Delete => self.delete += 1,
            ActionKind::Rename => self.rename += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CategoryCounts {
    pub file: u32,
    pub shell: u32,
    pub git: u32,
    pub package: u32,
    pub secret: u32,
}

impl CategoryCounts {
    pub fn total(&self) -> u32 {
        self.file + self.shell + self.git + self.package + self.secret
    }

    pub fn add(&mut self, c: ActionCategory) {
        match c {
            ActionCategory::File => self.file += 1,
            ActionCategory::Shell => self.shell += 1,
            ActionCategory::Git => self.git += 1,
            ActionCategory::Package => self.package += 1,
            ActionCategory::Secret => self.secret += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RiskCounts {
    pub low: u32,
    pub medium: u32,
    pub high: u32,
    pub critical: u32,
}

impl RiskCounts {
    pub fn total(&self) -> u32 {
        self.low + self.medium + self.high + self.critical
    }

    pub fn add(&mut self, r: RiskLevel) {
        match r {
            RiskLevel::Low => self.low += 1,
            RiskLevel::Medium => self.medium += 1,
            RiskLevel::High => self.high += 1,
            RiskLevel::Critical => self.critical += 1,
        }
    }
}

/// Enforcement outcomes across a session. The concrete "Detection ≠
/// Protection" split: `total()` counts actions that crossed a boundary with
/// a known outcome, while `enforced` is the number ActionGuard *actually*
/// stopped before execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnforcementCounts {
    pub enforced: u32,
    pub observed: u32,
    pub bypassed: u32,
    pub unsupported: u32,
}

impl EnforcementCounts {
    pub fn total(&self) -> u32 {
        self.enforced + self.observed + self.bypassed + self.unsupported
    }

    pub fn add(&mut self, e: EnforcementStatus) {
        match e {
            EnforcementStatus::Enforced => self.enforced += 1,
            EnforcementStatus::Observed => self.observed += 1,
            EnforcementStatus::Bypassed => self.bypassed += 1,
            EnforcementStatus::Unsupported => self.unsupported += 1,
        }
    }
}

// ===========================================================================
// Batch (used by the existing file-aggregator flow)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchData {
    pub counts: Counts,
    pub actions: Vec<Action>,
    pub risk: RiskResult,
}

// ===========================================================================
// Sessions
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Completed,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub num: u32,
    pub workspace: String,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(default)]
    pub duration_secs: u64,
    pub counts: Counts,
    pub total: u32,
    pub risk: RiskLevel,
    pub status: SessionStatus,
    #[serde(default)]
    pub undone: bool,
    #[serde(default)]
    pub sensitive_count: u32,
    #[serde(default)]
    pub outside_count: u32,

    // --- v0.2 additive (all default; old sessions load with zeros) ---
    #[serde(default)]
    pub category_counts: CategoryCounts,
    #[serde(default)]
    pub risk_counts: RiskCounts,
    #[serde(default)]
    pub actions_protected: u32,
    #[serde(default)]
    pub actions_blocked: u32,
    /// v0.3 — enforcement outcome split (Detection ≠ Protection).
    /// Old sessions load with all-zero counts.
    #[serde(default)]
    pub enforcement_counts: EnforcementCounts,
    /// v0.2 Mode A/B: "observe" or "protected". Defaults to Protected for
    /// backward compat with sessions that predate the mode field.
    #[serde(default)]
    pub mode: SessionMode,
    /// v0.2 — approval popups that fired (interruptions). Every gated action
    /// shows one popup; this is the denominator of User Override Rate.
    /// Old sessions load with zeros.
    #[serde(default)]
    pub popups: u32,
    /// v0.2 — times the user allowed a gated action (override). This is the
    /// numerator of User Override Rate. A high rate means the policy is too
    /// sensitive or the popup is doing the user's thinking for them.
    #[serde(default)]
    pub overrides: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetails {
    #[serde(flatten)]
    pub summary: SessionSummary,
    pub actions: Vec<Action>,
}

// ===========================================================================
// Config
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub next_session_num: u32,
    pub ignore_patterns: Vec<String>,
    pub detect_outside: bool,
    // --- v0.2 additive ---
    #[serde(default = "default_true")]
    pub shell_blocking: bool,
    #[serde(default = "default_approval_timeout")]
    pub approval_timeout_secs: u32,
    #[serde(default = "default_agent")]
    pub default_agent: String,
}

fn default_true() -> bool {
    true
}
fn default_approval_timeout() -> u32 {
    60
}
fn default_agent() -> String {
    "shell-user".to_string()
}

/// Default source type for actions that don't specify one. Old v0.1/v0.2
/// ledgers predate the `source_type` field, so deserialization stamps them
/// "agent" — preserving the historical semantics.
fn default_source_type() -> String {
    "agent".to_string()
}

/// Predicate used by `skip_serializing_if` so we don't pollute the ledger
/// with `source_type: "agent"` on every entry (it's the default).
fn is_default_source_type(s: &str) -> bool {
    s == "agent"
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            next_session_num: 1,
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "target".to_string(),
                ".cache".to_string(),
            ],
            detect_outside: true,
            shell_blocking: true,
            approval_timeout_secs: 60,
            default_agent: "shell-user".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UndoResult {
    pub restored_files: u32,
    pub deleted_files: u32,
    pub removed_dirs: u32,
    pub skipped: u32,
}

// ===========================================================================
// Snapshots (unchanged from v0.1)
// ===========================================================================

/// Snapshot manifest persisted under ~/.actionguard/snapshots/<session_id>/manifest.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnapshotManifest {
    pub files: Vec<SnapshotFile>,
    pub dirs: Vec<String>,
    pub file_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFile {
    /// Path relative to the workspace.
    pub path: String,
    /// sha256 hex of the content. Empty string when the file was empty.
    pub hash: String,
    /// Size in bytes.
    pub size: u64,
}

// ===========================================================================
// Policy
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<ActionCategory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args_contains: Option<Vec<String>>,
    /// Any-of substring match: the rule fires when AT LEAST ONE of these
    /// substrings appears in the target. Use for flag synonyms (e.g. `-9`,
    /// `-KILL`, `-SIGKILL`) where `args_contains` (all-of) would silently
    /// disable the rule for every spelling except the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args_any: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
}

/// Where a rule came from — this drives both display and policy precedence.
///
/// Precedence (highest first): `User` → `Project` → `Builtin`.
///
/// Security invariant: the protected object (an agent) must not be able to
/// decide its own protection boundary. Project rules are therefore only
/// allowed to make the boundary **stricter** (deny/ask), never weaker —
/// enforced at load time once Project policy lands (v0.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PolicySource {
    #[default]
    Builtin,
    User,
    /// Project/team policy (`.actionguard.yml` in the workspace root).
    /// Reserved for v0.3 — only allowed to tighten, never to relax.
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    #[serde(rename = "match")]
    pub match_: MatchSpec,
    pub action: Decision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<RiskLevel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default)]
    pub source: PolicySource,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyFile {
    pub version: u32,
    pub scope: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Default)]
pub struct PolicySet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct DecisionResult {
    pub decision: Decision,
    pub matched_rule: Option<String>,
    pub risk: RiskLevel,
    pub reason: String,
}

impl Default for DecisionResult {
    fn default() -> Self {
        Self {
            decision: Decision::Allow,
            matched_rule: None,
            risk: RiskLevel::Low,
            reason: String::new(),
        }
    }
}

// ===========================================================================
// Ledger (UI DTO) + approvals
// ===========================================================================

/// Computed view of an `Action` for the UI/CLI. Carries a convenience
/// `time_hms` field so the frontend doesn't have to parse timestamps.
#[derive(Debug, Clone, Serialize)]
pub struct LedgerEntry {
    pub id: String,
    pub timestamp: String,
    pub time_hms: String,
    pub agent: String,
    pub category: ActionCategory,
    pub kind: String,
    pub target: String,
    pub risk: RiskLevel,
    pub decision: Decision,
    pub result: String,
    pub reasons: Vec<String>,
    pub asset: Option<Asset>,
    pub evidence: Option<Evidence>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub session_id: String,
    pub action: Action,
    pub decision_due_at: String,
    pub timeout_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResolution {
    pub approval_id: String,
    pub decision: Decision,
    #[serde(default)]
    pub learn_rule: Option<Rule>,
}

// ===========================================================================
// Tests — v0.2 user-behavior telemetry fields
// ===========================================================================

#[cfg(test)]
mod telemetry_tests {
    use super::*;

    #[test]
    fn old_ledger_rows_deserialize_without_telemetry_fields() {
        // Pre-v0.2 ledger rows have no user_override / resolved_at. They must
        // deserialize as None, and re-serializing must NOT emit the fields
        // (keeps old rows byte-stable when re-written).
        let old_row = r#"{"id":"x","timestamp":"2026-01-01 00:00:00","action":"modify","sensitive":false,"outside":false}"#;
        let a: Action = serde_json::from_str(old_row).unwrap();
        assert_eq!(a.user_override, None);
        assert_eq!(a.resolved_at, None);
        let out = serde_json::to_string(&a).unwrap();
        assert!(
            !out.contains("user_override") && !out.contains("resolved_at"),
            "None fields must be skipped: {out}"
        );
    }

    #[test]
    fn resolution_row_roundtrips_override_fields() {
        // A resolution row (the row the bridge appends after the popup) must
        // carry user_override + resolved_at through serialize/deserialize.
        let mut a = Action::new_shell("npm install axios".to_string(), None, None);
        a.user_override = Some(true);
        a.resolved_at = Some(now_str());
        let out = serde_json::to_string(&a).unwrap();
        let back: Action = serde_json::from_str(&out).unwrap();
        assert_eq!(back.user_override, Some(true));
        assert!(back.resolved_at.is_some());
    }

    #[test]
    fn confirmed_gate_and_override_are_distinct() {
        // `Some(false)` = user confirmed the gate (agreed with the popup).
        // `Some(true)` = user overrode it. These are the two sides of
        // User Override Rate.
        let mut confirmed = Action::new_shell("git push".to_string(), None, None);
        confirmed.user_override = Some(false);
        assert!(!confirmed.user_override.unwrap());

        let mut overrode = Action::new_shell("git push".to_string(), None, None);
        overrode.user_override = Some(true);
        assert!(overrode.user_override.unwrap());
    }
}
