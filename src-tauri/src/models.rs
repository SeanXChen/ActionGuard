use serde::{Deserialize, Serialize};

// ===========================================================================
// Action identity & kind
// ===========================================================================

/// File-system action verbs (kept for v0.1 backward-compat with .actions.json).
/// Stays File-only; non-File categories use the free-form `Action.kind` string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActionKind {
    #[default]
    Modify,
    Create,
    Delete,
    Rename,
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

/// Data sensitivity class — the semantic layer that drives credential
/// exfiltration detection.
///
/// Unlike a path-based rule, `DataClass` says *what kind of data* an action
/// touches, not *where* it lives. This allows rules to express intent:
///   "Deny reading any Credential / Critical data"
/// instead of:
///   "Deny reading ~/.ssh/id_rsa, ~/.aws/credentials, …"
///
/// These map to the "Secret Classes" from the v0.2 adversarial analysis:
///   Read → Collect → Transform → Transmit
/// v0.2 covers Read (enforced via credential rules); Collection/Transform/Transmit
/// are observed and correlated in the session risk engine (Phase P1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataClass {
    /// No sensitive data involved (ordinary source code, configs, docs, etc.)
    #[default]
    Ordinary,
    /// Credentials: tokens, keys, secrets used to authenticate or authorize.
    /// Subclasses tracked via `credential_type` on `LedgerEntry` when known.
    Credential,
    /// Personally identifiable information, health records, financial data,
    /// legal documents — anything that creates regulatory exposure.
    PersonalData,
    /// Internal business logic, proprietary algorithms, M&A data,
    /// unreleased product plans.
    Proprietary,
    /// System-level secrets: SSH keys, GPG keys, CA certificates,
    /// host credentials.
    SystemSecret,
    /// Shell command history — often contains tokens, credentials, server names.
    ShellHistory,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    #[default]
    Allow,
    Ask,
    Deny,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryKind {
    #[default]
    ObserveOnly,
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
    /// **E. System Enforcement** (future L4). Every local action crosses it.
    SystemLevel,
    /// **F. Remote Automation** — the action never lands on this machine
    /// (cloud worker / remote browser / remote sandbox). Outside the local
    /// ActionGuard boundary by address space, not by product choice.
    Remote,
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

// ===========================================================================
// v0.3 — Contextual Facts Schema 2.0
// ===========================================================================

/// What class of resource an action targets — drives semantic policy rules
/// instead of path-based string matching.
///
/// v0.2 used paths like `~/.ssh/**` to detect credential access.
/// v0.3 uses `target_class: credential` to express the same intent:
///   "Deny reading any Critical Credential resource"
/// instead of:
///   "Deny reading ~/.ssh/id_rsa, ~/.aws/credentials, …"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetClass {
    /// Ordinary source code, configs, docs — no special sensitivity.
    #[default]
    SourceCode,
    /// Credentials: tokens, keys, secrets used to authenticate.
    Credential,
    /// System-level secrets: SSH keys, GPG keys, CA certificates.
    SystemSecret,
    /// Configuration files that may contain embedded secrets.
    Config,
    /// Build artifacts, caches, generated files.
    BuildArtifact,
    /// Package manager manifests and lock files.
    PackageManifest,
    /// Git repository metadata (.git directory).
    GitRepo,
    /// User data: personal files, downloads, documents.
    UserData,
    /// External resources: remote services, APIs, cloud resources.
    ExternalResource,
    /// Network endpoints and connections.
    NetworkEndpoint,
    /// Other / unclassified.
    Unknown,
}

impl TargetClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetClass::SourceCode => "source_code",
            TargetClass::Credential => "credential",
            TargetClass::SystemSecret => "system_secret",
            TargetClass::Config => "config",
            TargetClass::BuildArtifact => "build_artifact",
            TargetClass::PackageManifest => "package_manifest",
            TargetClass::GitRepo => "git_repo",
            TargetClass::UserData => "user_data",
            TargetClass::ExternalResource => "external_resource",
            TargetClass::NetworkEndpoint => "network_endpoint",
            TargetClass::Unknown => "unknown",
        }
    }

    /// Default sensitivity level for each target class.
    pub fn default_sensitivity(&self) -> SensitivityLevel {
        match self {
            TargetClass::SystemSecret | TargetClass::Credential => SensitivityLevel::Critical,
            TargetClass::ExternalResource | TargetClass::NetworkEndpoint => SensitivityLevel::High,
            TargetClass::Config => SensitivityLevel::Medium,
            _ => SensitivityLevel::Low,
        }
    }
}

/// Who owns the resource being accessed — critical for detecting
/// third-party resource modification (e.g. the gym booking scenario:
/// "cancel someone else's reservation").
///
/// In local contexts, ownership is inferred from path ownership.
/// In remote/external contexts, ownership may be `Unknown` until the
/// source system provides provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Ownership {
    /// The current user owns this resource.
    #[default]
    SelfOwned,
    /// A third party owns this resource (other user, external service).
    ThirdParty,
    /// Shared resource (group-owned, team-shared).
    Shared,
    /// Cannot be determined from available context.
    Unknown,
}

impl Ownership {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ownership::SelfOwned => "self",
            Ownership::ThirdParty => "third_party",
            Ownership::Shared => "shared",
            Ownership::Unknown => "unknown",
        }
    }
}

/// Where the action has effect — local machine vs external system.
/// Fundamental for detecting the gym booking scenario: "cancel reservation"
/// has `externality: third_party` even though the command runs locally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Externality {
    /// Action affects only local resources on this machine.
    #[default]
    Local,
    /// Action affects a third-party resource (other user's data, external API).
    ThirdParty,
    /// Action creates or modifies a public/external resource.
    Public,
    /// Action involves an external system (cloud, SaaS, remote service).
    ExternalSystem,
}

impl Externality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Externality::Local => "local",
            Externality::ThirdParty => "third_party",
            Externality::Public => "public",
            Externality::ExternalSystem => "external_system",
        }
    }
}

/// Side effects an action may produce — drives the "consequence" dimension
/// of contextual policy. E.g. "git push --force" and "git status" are both
/// Git actions but have completely different side effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SideEffect {
    /// No meaningful side effects.
    #[default]
    None,
    /// Modifies or destroys data (file delete, git reset, DB write).
    Destructive,
    /// Permanently removes data that cannot be easily recovered.
    Irreversible,
    /// Modifies a resource belonging to another user or system.
    ThirdPartyImpact,
    /// Creates or modifies network connections or external calls.
    ExternalCall,
    /// Modifies system configuration or installed software.
    SystemModification,
    /// Creates a new external artifact (commit, release, publication).
    Publication,
}

impl SideEffect {
    pub fn as_str(&self) -> &'static str {
        match self {
            SideEffect::None => "none",
            SideEffect::Destructive => "destructive",
            SideEffect::Irreversible => "irreversible",
            SideEffect::ThirdPartyImpact => "third_party_impact",
            SideEffect::ExternalCall => "external_call",
            SideEffect::SystemModification => "system_modification",
            SideEffect::Publication => "publication",
        }
    }
}

/// Whether an action's effects can be reversed — critical for risk escalation.
/// "AI modifies README" and "AI deletes .git" may have the same operation type
/// but vastly different reversibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    /// Effects can be trivially restored (e.g. re-download a package).
    #[default]
    Reversible,
    /// Reversible with some effort (e.g. git reset, file restore from backup).
    Difficult,
    /// Effects cannot be recovered (e.g. git push --force, cloud resource deletion).
    Irreversible,
    /// Reversibility cannot be determined.
    Unknown,
}

impl Reversibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Reversibility::Reversible => "reversible",
            Reversibility::Difficult => "difficult",
            Reversibility::Irreversible => "irreversible",
            Reversibility::Unknown => "unknown",
        }
    }
}

/// How sensitive a resource or action is — used for risk escalation when
/// `RiskLevel` alone is insufficient.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SensitivityLevel {
    /// Ordinary resource, no special sensitivity.
    #[default]
    Low,
    /// Sensitive but not critical (configs, package manifests).
    Medium,
    /// High-value target (credentials, system secrets, external resources).
    High,
    /// Critical: credentials, private keys, PII, production secrets.
    Critical,
}

impl SensitivityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SensitivityLevel::Low => "low",
            SensitivityLevel::Medium => "medium",
            SensitivityLevel::High => "high",
            SensitivityLevel::Critical => "critical",
        }
    }
}

/// v0.3 — The consequence dimension of an action's context.
/// Records what effects the action produces beyond its primary operation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Consequence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side_effect: Option<SideEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub externality: Option<Externality>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reversibility: Option<Reversibility>,
    /// True when this action is part of a detected chain (credential collection,
    /// exfiltration pattern, escalation sequence).
    #[serde(default)]
    pub is_chain_link: bool,
}

/// v0.3 — Target context: what resource class and ownership context the action affects.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TargetContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<TargetClass>,
    #[serde(default)]
    pub sensitivity: SensitivityLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ownership: Option<Ownership>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ownership_note: Option<String>,
}

/// v0.3 — Provenance: where an action came from.
/// Allows the ledger to answer "how do we know about this action?"
/// instead of just "what happened?".
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Provenance {
    /// The boundary class through which this action was detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<BoundaryKind>,
    /// Confidence level of this detection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<ProvenanceConfidence>,
    /// When the action was observed (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceConfidence {
    /// Observed through a verified enforcement path (tool hook, exec approval).
    #[default]
    Verified,
    /// Inferred from shell/preexec observation.
    Inferred,
    /// Heuristic or pattern-based detection.
    Heuristic,
}

impl ProvenanceConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProvenanceConfidence::Verified => "verified",
            ProvenanceConfidence::Inferred => "inferred",
            ProvenanceConfidence::Heuristic => "heuristic",
        }
    }
}

/// v0.3 — Action correlation: links actions into sequences and chains.
/// Used to detect patterns like:
///   Read credential → Read credential → Archive → External send
/// which individually might be Low risk but collectively are Critical.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionCorrelation {
    /// IDs of related actions in the same session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_actions: Vec<String>,
    /// Type of correlation detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_type: Option<ActionChainType>,
    /// Human-readable description of the detected pattern.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionChainType {
    /// Credential access pattern: multiple credential sources touched.
    #[default]
    CredentialAccess,
    /// Aggregation pattern: credentials collected and prepared for transfer.
    CredentialCollection,
    /// Exfiltration: collected data prepared for or sent to external destination.
    Exfiltration,
    /// Privilege escalation: escalating access pattern.
    PrivilegeEscalation,
    /// Destructive cascade: multiple destructive operations in sequence.
    DestructiveCascade,
    /// Other correlated sequence.
    Other,
}

impl ActionChainType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionChainType::CredentialAccess => "credential_access",
            ActionChainType::CredentialCollection => "credential_collection",
            ActionChainType::Exfiltration => "exfiltration",
            ActionChainType::PrivilegeEscalation => "privilege_escalation",
            ActionChainType::DestructiveCascade => "destructive_cascade",
            ActionChainType::Other => "other",
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    #[default]
    Observe,
    Protected,
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
    /// What kind of data this action touches — drives credential exfiltration
    /// detection (Read → Collect → Transform → Transmit chain).
    /// v0.2: populated for Secret-category and credential-path actions.
    /// v0.3: correlated across session for chain detection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_class: Option<DataClass>,
    /// When `data_class` is `Credential`, the specific credential type
    /// detected (e.g. "ssh_private_key", "aws_credentials", "api_token").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_type: Option<String>,

    // --- v0.3: Contextual Facts ---
    /// Semantic class of the target resource (credential, config, source_code, etc.)
    /// instead of path-based matching.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_class: Option<TargetClass>,
    /// Sensitivity level of the target resource.
    #[serde(default)]
    pub target_sensitivity: SensitivityLevel,
    /// Who owns the resource being accessed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ownership: Option<Ownership>,
    /// Where the action has effect: local, third_party, external_system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub externality: Option<Externality>,
    /// What side effects the action may produce.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side_effect: Option<SideEffect>,
    /// Whether the action's effects can be reversed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reversibility: Option<Reversibility>,
    /// The consequence dimension of this action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consequence: Option<Consequence>,
    /// Target context: class, sensitivity, ownership.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_context: Option<TargetContext>,
    /// Provenance: where this action came from and how confident we are.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
    /// Action correlation: related actions and detected chains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation: Option<ActionCorrelation>,
    /// Parent action ID (for sub-actions or forked actions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_action: Option<String>,

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
            data_class: None,
            credential_type: None,
            // v0.3 contextual facts
            target_class: None,
            target_sensitivity: SensitivityLevel::Low,
            ownership: None,
            externality: Some(Externality::Local),
            side_effect: None,
            reversibility: None,
            consequence: None,
            target_context: None,
            provenance: None,
            correlation: None,
            parent_action: None,
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
            data_class: None,
            credential_type: None,
            // v0.3 contextual facts
            target_class: None,
            target_sensitivity: SensitivityLevel::Low,
            ownership: Some(Ownership::SelfOwned),
            externality: Some(Externality::Local),
            side_effect: None,
            reversibility: None,
            consequence: None,
            target_context: None,
            provenance: None,
            correlation: None,
            parent_action: None,
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

    // --- P1: credential exfiltration chain detection ---
    /// Number of distinct credential sources accessed in this session
    /// (e.g. SSH key + AWS credentials + shell history = 3).
    /// Populated by the session risk engine scanning `data_class` on ledger entries.
    #[serde(default)]
    pub credential_sources_touched: u32,
    /// List of credential types touched (e.g. ["ssh_private_key", "aws_credentials",
    /// "shell_history"]). Drives the session risk banner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub credential_types: Vec<String>,
    /// Set to true when the session risk engine detects a potential
    /// credential exfiltration chain: credential access(s) + aggregation + outbound.
    /// Triggers the "Credential collection detected" banner in the GUI.
    #[serde(default)]
    pub chain_detected: bool,

    // --- v0.3: Contextual Session Risk ---
    /// Number of distinct target classes accessed in this session.
    #[serde(default)]
    pub target_classes_touched: u32,
    /// Types of target classes accessed (e.g. ["credential", "config", "source_code"]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_class_types: Vec<String>,
    /// Highest sensitivity level touched in this session.
    #[serde(default)]
    pub max_sensitivity: SensitivityLevel,
    /// Number of destructive actions detected.
    #[serde(default)]
    pub destructive_actions: u32,
    /// Number of irreversible actions detected.
    #[serde(default)]
    pub irreversible_actions: u32,
    /// Number of third-party impacting actions detected.
    #[serde(default)]
    pub third_party_actions: u32,
    /// Chain types detected in this session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detected_chains: Vec<ActionChainType>,
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
    /// What kind of sensitive data this action touches. Drives exfiltration
    /// chain detection in the session risk engine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_class: Option<DataClass>,
    /// Credential type when data_class is Credential (e.g. "ssh", "aws", "api_token").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_type: Option<String>,
    /// P1 — set when this action is part of a detected exfiltration chain
    /// (credential access → aggregation → outbound). Feeds the session risk
    /// banner in the GUI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_tag: Option<String>,
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
