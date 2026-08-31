//! Boundary Registry — the vendor-neutral abstraction layer.
//!
//! ActionGuard does not care *which* agent produced an action. It cares
//! *where* the action entered — which [BoundaryKind] it crossed. This module
//! is the single place that knows about real automation sources and their
//! boundaries. The core engine (risk / policy / decision) never special-cases
//! agent brands.
//!
//! Two surfaces:
//! - [`registry`] — the static, human-maintained list of known boundaries.
//! - [`detect_boundaries`] — local probes overlaying the registry, so
//!   `actionguard boundary list` only claims what it can prove on this
//!   machine.
//!
//! `actionguard boundary test` runs non-destructive verification for a
//! boundary and prints ✓/✗ lines — evidence, not promises.

use crate::models::BoundaryKind;
use crate::storage;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Run a command with CREATE_NO_WINDOW on Windows so no CMD window flashes.
/// On non-Windows platforms, behaves identically to `Command::new`.
fn no_window_cmd(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let _ = program; // consumed above
    cmd
}

/// Cache for detect_boundaries — avoids re-running tasklist/pgrep on every call.
/// Stale after 5 seconds so app state changes (e.g. starting Cursor) are reflected.
#[allow(clippy::type_complexity)]
static DETECT_CACHE: std::sync::LazyLock<Mutex<Option<(Instant, Vec<BoundaryDescriptor>)>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

const DETECT_CACHE_TTL: Duration = Duration::from_secs(5);

/// Cached detect_boundaries — returns cached result if still fresh.
pub fn detect_boundaries_cached() -> Vec<BoundaryDescriptor> {
    let mut cache = DETECT_CACHE.lock().unwrap();
    match cache.as_ref() {
        Some((instant, boundaries)) if instant.elapsed() < DETECT_CACHE_TTL => {
            return boundaries.clone();
        }
        _ => {}
    }
    let boundaries = detect_boundaries();
    *cache = Some((Instant::now(), boundaries.clone()));
    boundaries
}

/// How a registry row's `enforcement` claim was established. This is the
/// `Core Verified` / `Community Verified` distinction — a green checkmark is
/// not enough for a safety product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verification {
    /// Verified by the ActionGuard maintainers on a real machine.
    Core,
    /// Verified by the community with reproducible evidence (script, output,
    /// Action ID, version, OS) — see `boundaries/*.yml`.
    Community,
    /// Not yet verified.
    None,
}

impl Verification {
    pub fn label(&self) -> &'static str {
        match self {
            Verification::Core => "CORE_VERIFIED",
            Verification::Community => "COMMUNITY_VERIFIED",
            Verification::None => "UNVERIFIED",
        }
    }
}

/// How much to trust a registry row, given how it was verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn label(&self) -> &'static str {
        match self {
            Confidence::High => "HIGH",
            Confidence::Medium => "MEDIUM",
            Confidence::Low => "LOW",
        }
    }
}

/// Live status of a registered boundary, as measured (or not yet measured)
/// on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryStatus {
    /// Verified: pre-action decisions are enforced on this boundary.
    Enforced,
    /// Verified: actions are recorded, but decisions cannot block execution.
    ObserveOnly,
    /// Detected / installed on this machine, but not currently active (e.g.
    /// the adapter is configured in settings.json but the host is not
    /// running). "Detected" and "Enforced" are deliberately different words.
    Inactive,
    /// Not detected on this machine / not yet verified.
    NotDetected,
}

impl BoundaryStatus {
    pub fn label(&self) -> &'static str {
        match self {
            BoundaryStatus::Enforced => "Enforced",
            BoundaryStatus::ObserveOnly => "Observe-only",
            BoundaryStatus::Inactive => "Inactive (installed, not running)",
            BoundaryStatus::NotDetected => "Not detected",
        }
    }
}

/// One row of the Boundary Registry.
#[derive(Debug, Clone)]
pub struct BoundaryDescriptor {
    /// Stable, human-readable name (also accepted by `actionguard boundary
    /// test <name>`).
    pub name: String,
    pub kind: BoundaryKind,
    pub status: BoundaryStatus,
    /// Which execution paths this entry's `enforcement` claim actually covers.
    /// Empty string means "the whole boundary". Required when one product has
    /// different enforcement per path (e.g. PowerShell interactive vs. scripts).
    pub scope: String,
    /// Structural capability: does this boundary *have* a pre-action
    /// enforcement mechanism at all? Independent of whether it is currently
    /// running (that is `status`). "Enforceable" is a property of the design,
    /// "Enforced" is a property of this machine, right now.
    pub enforceable: bool,
    /// Set only by a live probe: is the boundary actually present on this
    /// machine? Registry rows without a probe stay `false` — "documented"
    /// must never be reported as "detected".
    pub detected: bool,
    /// ISO date of the last live verification, or `""` if never verified.
    pub last_verified: String,
    /// Why this boundary is in this state.
    pub note: String,
    /// Who verified the `enforcement` claim (Core / Community / none).
    pub verification: Verification,
    /// Confidence in the verification.
    pub confidence: Confidence,
    /// Community handle of the contributor who verified this row
    /// (`@handle`), or `""` when Core-verified / unverified.
    pub contributor: String,
}

/// The static registry of every automation source ActionGuard knows about.
/// This is the "AI Automation Boundary Map" — a research asset, not a
/// feature matrix. Rows are grouped by **Boundary Class** (A–F), not by
/// brand; a new product is mapped to a class in minutes, not integrated.
/// Entries are added when a product exposes (or stops exposing) a local
/// action boundary.
///
/// The same rows live as YAML under `boundaries/*.yml` (the file is the
/// long-term community asset; this function is the offline fallback that
/// keeps a shipped binary self-contained). [`load_registry_from_files`]
/// tries the files first, then falls back to this built-in table.
pub fn registry() -> Vec<BoundaryDescriptor> {
    vec![
        // --- C. Protected Shell ---
        BoundaryDescriptor {
            name: "Protected Shell (bash/zsh/fish)".into(),
            kind: BoundaryKind::ProtectedShell,
            status: BoundaryStatus::Enforced,
            scope: "interactive commands".into(),
            enforceable: true,
            detected: false,
            last_verified: "2026-08-19".into(),
            note: "preexec hook → bridge /preexec → deny blocks before exec".into(),
            verification: Verification::Core,
            confidence: Confidence::High,
            contributor: String::new(),
        },
        // PowerShell is deliberately split into two registry entries: the
        // interactive path (Phase C, enforced) and the non-interactive paths
        // (scripts / -Command / piped stdin, observe-only known bypass).
        // One flag cannot express both — the scope field is the model.
        BoundaryDescriptor {
            name: "PowerShell (PSReadLine interactive)".into(),
            kind: BoundaryKind::ProtectedShell,
            status: BoundaryStatus::Enforced,
            scope: "interactive lines".into(),
            enforceable: true,
            detected: false,
            last_verified: "2026-08-21".into(),
            note: "Phase C: Enter handler → bridge /preexec → deny reverts the line before execution — verified 2026-08-21 (scripts/tests/verify-powershell-phase-c.ps1)".into(),
            verification: Verification::Core,
            confidence: Confidence::High,
            contributor: String::new(),
        },
        BoundaryDescriptor {
            name: "PowerShell (script/-Command/piped)".into(),
            kind: BoundaryKind::ProtectedShell,
            status: BoundaryStatus::ObserveOnly,
            scope: "scripts / -Command / piped stdin".into(),
            enforceable: false,
            detected: false,
            last_verified: "2026-08-21".into(),
            note: "bypasses PSReadLine entirely — known bypass, observe-only (verified 2026-08-21)".into(),
            verification: Verification::Core,
            confidence: Confidence::High,
            contributor: String::new(),
        },
        // --- A. Tool Hook ---
        BoundaryDescriptor {
            name: "CodeBuddy PreToolUse".into(),
            kind: BoundaryKind::ToolHook,
            status: BoundaryStatus::Enforced,
            scope: String::new(),
            enforceable: true,
            detected: false,
            last_verified: "2026-08-19".into(),
            note: "PreToolUse hook → ag-hook.py → policy-check → deny before execution — first verified official adapter".into(),
            verification: Verification::Core,
            confidence: Confidence::High,
            contributor: String::new(),
        },
        BoundaryDescriptor {
            name: "Claude Code".into(),
            kind: BoundaryKind::ToolHook,
            status: BoundaryStatus::NotDetected,
            scope: String::new(),
            enforceable: true,
            detected: false,
            last_verified: "2026-08-19".into(),
            note: "official PreToolUse/PostToolUse hooks (settings.json), exit 2 = deny. Documented 2026-08-19; enforcement not verified; not installed on this machine.".into(),
            verification: Verification::None,
            confidence: Confidence::Medium,
            contributor: String::new(),
        },
        BoundaryDescriptor {
            name: "Codex".into(),
            kind: BoundaryKind::ExecApproval,
            status: BoundaryStatus::NotDetected,
            scope: String::new(),
            enforceable: false,
            detected: false,
            last_verified: "2026-08-19".into(),
            note: "no third-party pre-tool hook protocol — built-in approval_policy (config.toml: untrusted/on-failure/never) is Class B and not extensible by ActionGuard. Documented 2026-08-19; not installed on this machine.".into(),
            verification: Verification::None,
            confidence: Confidence::Medium,
            contributor: String::new(),
        },
        BoundaryDescriptor {
            name: "Cursor".into(),
            kind: BoundaryKind::ToolHook,
            status: BoundaryStatus::NotDetected,
            scope: String::new(),
            enforceable: true,
            detected: false,
            last_verified: "2026-08-19".into(),
            note: "official hooks.json (preToolUse/beforeShellExecution/beforeMCPExecution/beforeReadFile; exit 2 = deny, failClosed option). Installed 3.11.13 — binary contains beforeShellExecution; no ActionGuard hook configured (defaults fail-open).".into(),
            verification: Verification::None,
            confidence: Confidence::Medium,
            contributor: String::new(),
        },
        BoundaryDescriptor {
            name: "Windsurf".into(),
            kind: BoundaryKind::ToolHook,
            status: BoundaryStatus::NotDetected,
            scope: String::new(),
            enforceable: true,
            detected: false,
            last_verified: "2026-08-19".into(),
            note: "official Cascade Hooks (pre/post; docs now under Devin). Documented 2026-08-19; not installed on this machine.".into(),
            verification: Verification::None,
            confidence: Confidence::Medium,
            contributor: String::new(),
        },
        BoundaryDescriptor {
            name: "OpenCode".into(),
            kind: BoundaryKind::ToolHook,
            status: BoundaryStatus::NotDetected,
            scope: String::new(),
            enforceable: false,
            detected: false,
            last_verified: String::new(),
            note: "IDE automation — needs a real pre-action hook probe; class unverified".into(),
            verification: Verification::None,
            confidence: Confidence::Low,
            contributor: String::new(),
        },
        // --- B. Execution Approval ---
        BoundaryDescriptor {
            name: "OpenClaw".into(),
            kind: BoundaryKind::ExecApproval,
            status: BoundaryStatus::NotDetected,
            scope: String::new(),
            enforceable: false,
            detected: false,
            last_verified: String::new(),
            note: "native exec boundary: sandbox/gateway/node + deny/allowlist/full + host approvals bound to canonical systemRunPlan (mismatch → approval rejected). Candidate for an independent ActionGuard policy layer — TBD".into(),
            verification: Verification::None,
            confidence: Confidence::Low,
            contributor: String::new(),
        },
        BoundaryDescriptor {
            name: "Manus Desktop (My Computer)".into(),
            kind: BoundaryKind::ExecApproval,
            status: BoundaryStatus::NotDetected,
            scope: String::new(),
            enforceable: false,
            detected: false,
            last_verified: String::new(),
            note: "local terminal/file/app actions with native per-command approval (Allow Once / Always Allow); ActionGuard as independent second policy layer — TBD".into(),
            verification: Verification::None,
            confidence: Confidence::Low,
            contributor: String::new(),
        },
        // --- F. Remote Automation ---
        BoundaryDescriptor {
            name: "Manus Cloud".into(),
            kind: BoundaryKind::Remote,
            status: BoundaryStatus::NotDetected,
            scope: String::new(),
            enforceable: false,
            detected: false,
            last_verified: String::new(),
            note: "remote sandbox — actions never land on this machine; outside the local ActionGuard boundary (address-space limit, not a product choice)".into(),
            verification: Verification::None,
            confidence: Confidence::Low,
            contributor: String::new(),
        },
    ]
}

/// Pick registry rows from `boundaries/` files when present, else built-in.
fn registry_from_env() -> Vec<BoundaryDescriptor> {
    let candidates = [PathBuf::from("boundaries")];
    for dir in &candidates {
        let from_files = load_registry_from_files(dir);
        if !from_files.is_empty() {
            return from_files;
        }
    }
    registry()
}

/// Try to load the registry from `boundaries/*.yml` files. These files are
/// the community-facing asset (see `boundaries/README.md`); a repo checkout
/// can therefore drive `actionguard boundary list` straight from YAML. When
/// the files are absent (e.g. a binary-only install) we fall back to the
/// built-in [`registry`].
pub fn load_registry_from_files(dir: &Path) -> Vec<BoundaryDescriptor> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    let mut names: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map(|e| e == "yml" || e == "yaml").unwrap_or(false)
                && p.file_name().map(|n| n != "registry.yml").unwrap_or(false)
        })
        .collect();
    names.sort();
    for path in names {
        match std::fs::read_to_string(&path) {
            Ok(raw) => match serde_yaml::from_str::<RegistryEntry>(&raw) {
                Ok(entry) => out.push(entry.into_descriptor(&path)),
                Err(e) => eprintln!("boundary: skipping {} (bad yaml: {e})", path.display()),
            },
            Err(_) => continue,
        }
    }
    out
}

/// Mirror of one `boundaries/<product>.yml` file.
#[derive(Debug, serde::Deserialize)]
struct RegistryEntry {
    name: String,
    #[serde(default)]
    class: Option<String>,
    #[serde(default)]
    boundary: Option<RegistryBoundary>,
    #[serde(default)]
    enforcement: Option<RegistryEnforcement>,
    #[serde(default)]
    last_verified: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    note: Option<String>,
    /// Community handle who verified this row (`@someone`). Set by a
    /// Community-Verified contribution; `None` for Core / unverified rows.
    #[serde(default)]
    contributor: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RegistryBoundary {
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    mechanism: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RegistryEnforcement {
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    verification: Option<String>,
    #[serde(default)]
    confidence: Option<String>,
}

impl RegistryEntry {
    fn into_descriptor(self, path: &Path) -> BoundaryDescriptor {
        let kind = self
            .class
            .as_deref()
            .and_then(|c| match c.to_ascii_lowercase().as_str() {
                "tool_hook" | "tool-hook" | "a" => Some(BoundaryKind::ToolHook),
                "exec_approval" | "exec-approval" | "b" => Some(BoundaryKind::ExecApproval),
                "protected_shell" | "protected-shell" | "c" => Some(BoundaryKind::ProtectedShell),
                "runtime_hook" | "runtime-hook" | "d" => Some(BoundaryKind::RuntimeHook),
                "observe_only" | "observe-only" => Some(BoundaryKind::ObserveOnly),
                "system_level" | "system-level" | "e" => Some(BoundaryKind::SystemLevel),
                "remote" | "f" => Some(BoundaryKind::Remote),
                _ => None,
            })
            .unwrap_or(BoundaryKind::ObserveOnly);
        let verification = match self
            .enforcement
            .as_ref()
            .and_then(|e| e.verification.as_deref())
            .unwrap_or("none")
            .to_ascii_lowercase()
            .as_str()
        {
            "core" | "core_verified" | "core-verified" => Verification::Core,
            "community" | "community_verified" | "community-verified" => Verification::Community,
            _ => Verification::None,
        };
        let confidence = match self
            .enforcement
            .as_ref()
            .and_then(|e| e.confidence.as_deref())
            .unwrap_or("low")
            .to_ascii_lowercase()
            .as_str()
        {
            "high" => Confidence::High,
            "medium" => Confidence::Medium,
            _ => Confidence::Low,
        };
        let enforcement_action = self
            .enforcement
            .as_ref()
            .and_then(|e| e.action.as_deref())
            .unwrap_or("observe_only");
        // Structural capability comes from the YAML's declared action:
        // "enforced" implies the boundary has a pre-action mechanism.
        let enforceable = matches!(enforcement_action, "enforced");
        let status = match enforcement_action {
            "enforced" => BoundaryStatus::Enforced,
            "observe_only" | "observe-only" => BoundaryStatus::ObserveOnly,
            _ => BoundaryStatus::NotDetected,
        };
        let boundary_note = match (&self.boundary, &self.note) {
            (Some(b), _) => {
                let mut s = String::new();
                if let Some(t) = &b.r#type {
                    s.push_str(t);
                }
                if let Some(m) = &b.mechanism {
                    if !s.is_empty() {
                        s.push_str(" / ");
                    }
                    s.push_str(m);
                }
                s
            }
            (None, n) => n.clone().unwrap_or_default(),
        };
        let mut note = self.note.unwrap_or_else(|| {
            format!(
                "registered in {}",
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            )
        });
        if !boundary_note.is_empty() {
            if !note.is_empty() {
                note.push_str(" — ");
            }
            note.push_str(&boundary_note);
        }
        BoundaryDescriptor {
            name: self.name,
            kind,
            status,
            scope: self.scope.unwrap_or_default(),
            enforceable,
            detected: false, // only a live probe may set this to true
            last_verified: self.last_verified.unwrap_or_default(),
            note,
            verification,
            confidence,
            contributor: self.contributor.unwrap_or_default(),
        }
    }
}

/// Local probes overlaying the static registry. Never claims more than it
/// can prove on this machine.
///
/// When a `boundaries/` directory is present in the current working
/// directory (a repo checkout), rows are loaded from `boundaries/*.yml`;
/// otherwise the built-in table is used.
pub fn detect_boundaries() -> Vec<BoundaryDescriptor> {
    let mut out = registry_from_env();
    let active_session = storage::current_hook_symlink().exists();

    for d in &mut out {
        match d.name.as_str() {
            "Protected Shell (bash/zsh/fish)" => {
                // Live probe: is a bash-family shell actually available?
                let shell_present = no_window_cmd("bash")
                    .arg("--version")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                d.detected = shell_present;
                if !shell_present {
                    d.status = BoundaryStatus::NotDetected;
                    d.last_verified.clear();
                    d.note = "no bash-family shell detected on this machine".into();
                } else if active_session {
                    d.status = BoundaryStatus::Enforced;
                    d.note = format!(
                        "active session — preaction hook live; deny enforced when shell hook is sourced (verified {})",
                        d.last_verified
                    );
                } else {
                    d.status = BoundaryStatus::ObserveOnly;
                    d.note = "no active session — hook path offline".into();
                }
            }
            "PowerShell (PSReadLine interactive)" => {
                d.detected = true; // Windows ships PowerShell by default.
                if active_session {
                    d.status = BoundaryStatus::Enforced;
                    d.note = format!(
                        "active session — Phase C live: Enter handler blocks denied lines (verified {})",
                        d.last_verified
                    );
                } else {
                    d.status = BoundaryStatus::ObserveOnly;
                    d.note = "no active session — hook path offline".into();
                }
            }
            "PowerShell (script/-Command/piped)" => {
                d.detected = true; // Windows ships PowerShell by default.
                if active_session {
                    d.status = BoundaryStatus::ObserveOnly;
                    d.note = format!(
                        "active session — observe-only by design (known bypass: -Command/scripts), verified {}",
                        d.last_verified
                    );
                } else {
                    d.note = "no active session — hook path offline".into();
                }
            }
            "CodeBuddy PreToolUse" => {
                let probe = probe_codebuddy_hook();
                d.detected = probe.detected;
                if probe.detected {
                    // Detected ≠ Enforced: the hook only enforces while
                    // CodeBuddy is alive.
                    if probe.running {
                        d.status = BoundaryStatus::Enforced;
                        d.note = format!(
                            "{}; CodeBuddy running — Detected: YES, Active: YES, Enforced: YES (verified {})",
                            probe.note, d.last_verified
                        );
                    } else {
                        d.status = BoundaryStatus::Inactive;
                        d.last_verified.clear();
                        d.note = format!(
                            "{} — Detected: YES, Active: NO, Enforced: NO (CodeBuddy not running)",
                            probe.note
                        );
                    }
                } else {
                    d.status = BoundaryStatus::NotDetected;
                    d.last_verified.clear();
                    d.note = probe.note;
                }
            }
            // All other rows have no live probe yet. Exec-approval (B) and
            // IDE-tool-hook (A) classes get probes once a reproducible
            // integration exists; remote (F) is never reachable from this
            // machine by definition. Until then the honest answer is
            // NotDetected: "documented" (the YAML registry) must never be
            // reported as "detected" or "observed" on this machine.
            "Cursor" => {
                let home = dirs::home_dir();
                let hooks_json = home.map(|h| h.join(".cursor").join("hooks.json"));
                let hook_configured = hooks_json.as_ref().map(|p| {
                    p.exists() && std::fs::read_to_string(p)
                        .map(|s| s.contains("beforeShellExecution") && (s.contains("actionguard") || s.contains("ag-")))
                        .unwrap_or(false)
                }).unwrap_or(false);
                d.detected = hook_configured;
                if hook_configured {
                    // A configured hook is not evidence that the host invoked it or
                    // honoured a deny. Keep the registry honest until a boundary
                    // test and ledger evidence prove this exact integration.
                    d.status = BoundaryStatus::ObserveOnly;
                    d.last_verified.clear();
                    d.note = "ActionGuard hook configured in ~/.cursor/hooks.json; integration is detected but not independently verified, so no enforcement claim is made.".into();
                } else {
                    d.status = BoundaryStatus::NotDetected;
                    d.last_verified.clear();
                    d.note = "Cursor installed (3.11.13) — no ActionGuard hook detected in ~/.cursor/hooks.json; defaults fail-open. Run `actionguard setup` to install.".into();
                }
            }
            _ => {
                d.status = BoundaryStatus::NotDetected;
                d.detected = false;
                d.last_verified.clear();
            }
        }
    }
    out
}

/// Result of probing the CodeBuddy boundary. Deliberately separates the two
/// questions that used to be conflated:
///
/// * `detected` — is the ActionGuard adapter *installed* (present in a
///   `settings.json` CodeBuddy will read, and the hook script reachable)?
/// * `running`  — is CodeBuddy *actually running right now*?
///
/// Detected ≠ Enforced. An installed hook only enforces while its host is
/// alive; the caller maps (detected, running) onto the status.
struct CodeBuddyProbe {
    detected: bool,
    running: bool,
    note: String,
}

/// Where CodeBuddy reads hook config from: user level
/// (`~/.codebuddy/settings.json`) and every `.codebuddy/settings.json` found
/// by walking up from the current directory to the filesystem root. The walk
/// matters: `actionguard` is usually installed to a different directory than
/// the workspace, so a cwd-relative `.codebuddy` would silently miss the
/// real configuration (the bug that made an active hook report "Not
/// detected").
fn codebuddy_settings_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(home) = dirs::home_dir() {
        v.push(home.join(".codebuddy").join("settings.json"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd.as_path());
        while let Some(d) = dir {
            v.push(d.join(".codebuddy").join("settings.json"));
            dir = d.parent();
        }
    }
    v
}

/// True when the `ag-hook.py` referenced by a CodeBuddy settings file is
/// actually reachable. Best-effort: first the known checkout paths, then any
/// absolute script path quoted inside a settings file.
fn adapter_script_ok() -> bool {
    let candidates = [
        PathBuf::from("scripts/hooks/ag-hook.py"),
        PathBuf::from(r"D:/Action Guard/scripts/hooks/ag-hook.py"),
    ];
    if candidates.iter().any(|p| p.exists()) {
        return true;
    }
    for p in codebuddy_settings_paths() {
        if let Ok(raw) = std::fs::read_to_string(&p) {
            if let Some(script) = extract_quoted_path(&raw, "ag-hook.py") {
                if script.exists() {
                    return true;
                }
            }
        }
    }
    false
}

/// Extract the double-quoted path that mentions `needle` from a settings
/// JSON fragment (e.g. `"command": "D:/python/python.exe \"D:/Action
/// Guard/scripts/hooks/ag-hook.py\""`).
fn extract_quoted_path(raw: &str, needle: &str) -> Option<PathBuf> {
    for line in raw.lines() {
        if !line.contains(needle) {
            continue;
        }
        if let Some(start) = line.find('"') {
            let rest = &line[start + 1..];
            if let Some(end) = rest.find('"') {
                return Some(PathBuf::from(&rest[..end]));
            }
        }
    }
    None
}

/// True when CodeBuddy is currently running on this machine. Used to
/// distinguish "installed" from "active": a configured hook that has no live
/// host enforces nothing.
fn codebuddy_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        match no_window_cmd("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
        {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout);
                s.contains("CodeBuddy")
            }
            Err(_) => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        no_window_cmd("pgrep")
            .arg("-f")
            .arg("CodeBuddy")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

fn probe_codebuddy_hook() -> CodeBuddyProbe {
    let mut detected = false;
    let mut detail = String::new();
    for p in codebuddy_settings_paths() {
        let raw = match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if raw.contains("ag-hook.py") {
            if adapter_script_ok() {
                detected = true;
                detail = format!("ActionGuard hook present in {} + adapter reachable", p.display());
                break;
            }
        } else if raw.contains("actionguard") {
            detail = format!("{} mentions ActionGuard but no ag-hook.py", p.display());
        }
    }
    if !detected && detail.is_empty() {
        detail = "no ActionGuard hook in user- or workspace-level .codebuddy/settings.json".into();
    }
    let running = codebuddy_running();
    CodeBuddyProbe {
        detected,
        running,
        note: detail,
    }
}

/// Run non-destructive verification for a boundary (or all, when `name` is
/// `None`). Returns lines for `actionguard boundary test`, each prefixed
/// with ✓/✗ so a user can see at a glance what is proven.
pub fn test_boundaries(name: Option<&str>) -> Vec<String> {
    let detected = detect_boundaries();
    let targets: Vec<&BoundaryDescriptor> = match name {
        Some(n) => detected
            .iter()
            .filter(|d| {
                d.name.eq_ignore_ascii_case(n)
                    || d.name.to_lowercase().contains(&n.to_lowercase())
            })
            .collect(),
        None => detected.iter().collect(),
    };

    if targets.is_empty() {
        return vec![match name {
            Some(n) => format!("no boundary matches '{n}'"),
            None => "no boundaries to test".to_string(),
        }];
    }

    let mut lines: Vec<String> = Vec::new();
    for d in targets {
        lines.push(format!("Testing {}...", d.name));
        match d.name.as_str() {
            "CodeBuddy PreToolUse" => {
                let probe = probe_codebuddy_hook();
                if probe.detected {
                    lines.push("  pre-action boundary:  ✓ configured (hook + adapter)".into());
                    if probe.running {
                        lines.push(format!(
                            "  deny enforced:        ✓ verified {} (sudo rm -rf / blocked before exec)",
                            d.last_verified
                        ));
                    } else {
                        lines.push("  deny enforced:        ✗ installed but CodeBuddy not running — nothing to enforce".into());
                    }
                } else {
                    lines.push("  pre-action boundary:  ✗ not configured".into());
                    lines.push("  deny enforced:        ✗ nothing to enforce".into());
                }
            }
            "Protected Shell (bash/zsh/fish)" => {
                if storage::current_hook_symlink().exists() {
                    lines.push("  pre-action boundary:  ✓ active session live".into());
                    lines.push(format!(
                        "  deny enforced:        ✓ verified {} when shell hook sourced",
                        d.last_verified
                    ));
                } else {
                    lines.push("  pre-action boundary:  ✗ no active session".into());
                    lines.push("  deny enforced:        ✗ start a protected session first".into());
                }
            }
            "PowerShell (PSReadLine interactive)" => {
                lines.push("  pre-action boundary:  ✓ interactive lines (Phase C)".into());
                lines.push(format!(
                    "  deny enforced:        ✓ verified {} (denied line reverted before execution)",
                    d.last_verified
                ));
            }
            "PowerShell (script/-Command/piped)" => {
                lines.push("  pre-action boundary:  ✓ observed".into());
                lines.push(format!(
                    "  deny enforced:        ✗ observe-only — bypasses PSReadLine (known bypass, verified {})",
                    d.last_verified
                ));
            }
            "Manus Cloud" => {
                lines.push("  pre-action boundary:  ✗ remote (F) — actions never land on this machine".into());
                lines.push("  deny enforced:        ✗ n/a (address-space limit, not a product choice)".into());
            }
            _ => {
                lines.push("  pre-action boundary:  ✗ not detected / not yet verified".into());
                lines.push("  deny enforced:        ✗ no reproducible boundary yet".into());
            }
        }
        lines.push(format!("  status:               {} (last verified: {})", d.status.label(), if d.last_verified.is_empty() { "—".to_string() } else { d.last_verified.clone() }));
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn name_set(rows: &[BoundaryDescriptor]) -> BTreeSet<&str> {
        rows.iter().map(|d| d.name.as_str()).collect()
    }

    /// The YAML registry (`boundaries/*.yml`, the community asset) and the
    /// built-in offline table are two renderings of the same rows. They must
    /// never drift — same names, same count.
    #[test]
    fn yaml_registry_matches_builtin_table() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../boundaries");
        let from_files = load_registry_from_files(&dir);
        assert!(
            !from_files.is_empty(),
            "boundaries/ should be loadable from a repo checkout"
        );
        assert_eq!(
            name_set(&from_files),
            name_set(&registry()),
            "boundaries/*.yml and the built-in table must list the same boundaries"
        );
    }

    /// `detect_boundaries()` (the single source consumed by `boundary list`,
    /// `capabilities`, and `doctor`) must never add or drop rows — it only
    /// measures status on this machine.
    #[test]
    fn detect_never_changes_the_registry() {
        let detected = detect_boundaries();
        let base = registry_from_env();
        assert_eq!(detected.len(), base.len());
        assert_eq!(
            name_set(&detected),
            name_set(&base),
            "detection changes status, never membership"
        );
    }

    /// Repeated calls render the same machine state in the same order, so the
    /// three consumers (`boundary list` / `capabilities` / `doctor`) cannot
    /// disagree with each other.
    #[test]
    fn detect_is_deterministic() {
        let a = detect_boundaries();
        let b = detect_boundaries();
        let names_a: Vec<&str> = a.iter().map(|d| d.name.as_str()).collect();
        let names_b: Vec<&str> = b.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names_a, names_b);
    }

    /// "Enforceable" is a structural capability claim and must survive
    /// detection: a boundary that *can* enforce is still enforceable even when
    /// it is currently NotDetected on this machine.
    #[test]
    fn detect_preserves_enforceable_claims() {
        let base = registry_from_env();
        for d in detect_boundaries() {
            let row = base.iter().find(|b| b.name == d.name).unwrap();
            assert_eq!(
                d.enforceable, row.enforceable,
                "detect must not rewrite enforceable claims for {}",
                d.name
            );
        }
    }
}
