use crate::models::{
    Action, ActionCategory, AppConfig, PolicyFile, RiskLevel, SessionDetails, SessionSummary,
};
use crate::policy::loader::strip_bom;
use anyhow::Result;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub const APP_DIR_NAME: &str = ".actionguard";

pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR_NAME)
}

pub fn sessions_dir() -> PathBuf {
    data_dir().join("sessions")
}

pub fn snapshots_dir() -> PathBuf {
    data_dir().join("snapshots")
}

pub fn snapshot_dir(session_id: &str) -> PathBuf {
    snapshots_dir().join(session_id)
}

pub fn ensure_dirs() -> Result<()> {
    // Migrate ~/.agentguard/ → ~/.actionguard/ (v0.2 rename)
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let old_dir = home.join(".agentguard");
    if old_dir.is_dir() && !data_dir().exists() {
        let _ = fs::rename(&old_dir, data_dir());
    }

    fs::create_dir_all(sessions_dir())?;
    fs::create_dir_all(snapshots_dir())?;
    Ok(())
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

pub fn load_config() -> AppConfig {
    match fs::read_to_string(config_path()) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(cfg: &AppConfig) -> Result<()> {
    ensure_dirs()?;
    let s = serde_json::to_string_pretty(cfg)?;
    fs::write(config_path(), s)?;
    Ok(())
}

pub fn alloc_session_num(cfg: &mut AppConfig) -> u32 {
    let n = cfg.next_session_num;
    cfg.next_session_num += 1;
    n
}

pub fn save_session(summary: &SessionSummary) -> Result<()> {
    ensure_dirs()?;
    let path = sessions_dir().join(format!("{}.json", summary.id));
    let s = serde_json::to_string_pretty(summary)?;
    fs::write(path, s)?;
    Ok(())
}

pub fn load_session(id: &str) -> Result<SessionSummary> {
    let path = sessions_dir().join(format!("{id}.json"));
    let s = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&s)?)
}

pub fn list_sessions() -> Result<Vec<SessionSummary>> {
    ensure_dirs()?;
    let mut out = Vec::new();
    for entry in fs::read_dir(sessions_dir())? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(s) = fs::read_to_string(&path) {
            if let Ok(sum) = serde_json::from_str::<SessionSummary>(&s) {
                out.push(sum);
            }
        }
    }
    out.sort_by_key(|a| std::cmp::Reverse(a.num));
    Ok(out)
}

/// Attach the persisted action list. V0 keeps the action list inside the
/// session JSON file for simplicity (a sidecar `<id>.actions.json`).
pub fn save_actions(id: &str, actions: &[Action]) -> Result<()> {
    ensure_dirs()?;
    let path = sessions_dir().join(format!("{id}.actions.json"));
    let s = serde_json::to_string_pretty(actions)?;
    fs::write(path, s)?;
    Ok(())
}

pub fn load_actions(id: &str) -> Vec<Action> {
    let path = sessions_dir().join(format!("{id}.actions.json"));
    match fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn load_session_details(id: &str) -> Result<SessionDetails> {
    let summary = load_session(id)?;
    Ok(SessionDetails {
        summary,
        actions: load_actions(id),
    })
}

/// Simple glob helper supporting `*` (any run of chars) on the file name.
fn wildcard_match(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let (pl, nl) = (p.len(), n.len());
    let mut dp = vec![vec![false; nl + 1]; pl + 1];
    dp[0][0] = true;
    for i in 1..=pl {
        if p[i - 1] == '*' {
            dp[i][0] = dp[i - 1][0];
        }
    }
    for i in 1..=pl {
        for j in 1..=nl {
            match p[i - 1] {
                '*' => dp[i][j] = dp[i - 1][j] || dp[i][j - 1],
                c => dp[i][j] = dp[i - 1][j - 1] && c == n[j - 1],
            }
        }
    }
    dp[pl][nl]
}

/// Should this workspace-relative path be ignored (watcher + snapshot + undo)?
/// Applies to any path segment (directory or file) matching an ignore pattern.
pub fn is_ignored(rel: &str, cfg: &AppConfig) -> bool {
    if rel.is_empty() || rel == "." {
        return false;
    }
    for seg in rel.split(['/', '\\']) {
        if cfg.ignore_patterns.iter().any(|p| wildcard_match(p, seg)) {
            return true;
        }
    }
    false
}

// ===========================================================================
// v0.2 — Shell hook files, Action Ledger, user policy file
// ===========================================================================

/// Per-session hook descriptor file: two lines (`port` and `secret`) that the
/// shell init script reads to find the active ActionGuard bridge listener.
pub fn hook_file(id: &str) -> PathBuf {
    sessions_dir().join(format!("{id}.hook"))
}

/// Symlink that always points at the currently-active session's hook file.
/// Shells read this instead of guessing the session id.
pub fn current_hook_symlink() -> PathBuf {
    sessions_dir().join("current.hook")
}

/// Per-session audit artifact written when a session ends. The shell hooks
/// key off `current_closed_marker()`; this one exists for history/debugging.
pub fn closed_sentinel(id: &str) -> PathBuf {
    sessions_dir().join(format!("{id}.closed"))
}

/// Sentinel that marks a *deliberate* session stop. Stale hooks read this
/// when `current.hook` is missing to tell "the user ended the session"
/// (allow — the terminal must not be bricked) apart from an unexpected
/// failure (fail-closed: deny). Written by `teardown_current_hook`, cleared
/// by `point_current_hook` when a fresh session starts.
pub fn current_closed_marker() -> PathBuf {
    sessions_dir().join("current.closed")
}

/// Path to the user-edited YAML policy file. Built-in rules live in `rules/`
/// at the repo root and are baked into the binary via `include_str!`.
pub fn user_policy_path() -> PathBuf {
    data_dir().join("policies.user.yml")
}

/// Append-only NDJSON ledger: one line per finalized Action. Read mid-session
/// without re-serializing the whole action list. Used by the Ledger UI and CLI.
pub fn ledger_path(id: &str) -> PathBuf {
    sessions_dir().join(format!("{id}.ledger.json"))
}

/// Write the hook descriptor (port + secret) atomically.
pub fn write_hook_file(id: &str, port: u16, secret: &str) -> Result<()> {
    ensure_dirs()?;
    let path = hook_file(id);
    let tmp = path.with_extension("hook.tmp");
    let body = format!("{port}\n{secret}\n");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Point `current.hook` at this session's hook file. On Windows this is a
/// real file copy (no symlink privileges for unprivileged users); on Unix
/// it's a symlink. Both are read with `fs::read_to_string`.
pub fn point_current_hook(id: &str) -> Result<()> {
    ensure_dirs()?;
    let target = hook_file(id);
    let link = current_hook_symlink();
    // A fresh session must not be masked by a stale "session ended" marker:
    // hooks should fail-closed if THIS session's bridge dies unexpectedly.
    let _ = fs::remove_file(current_closed_marker());
    #[cfg(unix)]
    {
        let _ = fs::remove_file(&link);
        std::os::unix::fs::symlink(&target, &link)?;
    }
    #[cfg(not(unix))]
    {
        // Windows: fall back to a copy so we don't require Developer Mode.
        fs::copy(&target, &link)?;
    }
    Ok(())
}

/// Remove the `current.hook` link and drop the closed markers so stale hooks
/// know the stop was deliberate: they allow the next command instead of
/// fail-closing on a missing descriptor.
pub fn teardown_current_hook(id: &str) {
    let _ = fs::remove_file(current_hook_symlink());
    let _ = fs::write(closed_sentinel(id), b"closed");
    let _ = fs::write(current_closed_marker(), b"closed");
}

/// Append a single Action to the per-session NDJSON ledger.
pub fn append_ledger(id: &str, action: &Action) -> Result<()> {
    ensure_dirs()?;
    let path = ledger_path(id);
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    // Stamp the owning session id onto the record so each NDJSON line is
    // self-contained — future cloud-sync can upload lines as-is, keyed by
    // (session_id, action id), without re-deriving context from file names.
    let mut a = action.clone();
    a.session_id = Some(id.to_string());
    let line = serde_json::to_string(&a)?;
    writeln!(f, "{line}")?;
    Ok(())
}

/// Optional filters for the ledger loader.
#[derive(Debug, Clone, Default)]
pub struct LedgerFilter {
    pub category: Option<ActionCategory>,
    pub risk: Option<RiskLevel>,
    pub limit: Option<usize>,
}

/// Load the per-session ledger, applying the optional filter. Lines that
/// fail to parse are skipped (forward-compat with future schema additions).
pub fn load_ledger(id: &str, filter: &LedgerFilter) -> Vec<Action> {
    let path = ledger_path(id);
    let Ok(f) = fs::File::open(&path) else {
        return Vec::new();
    };
    let reader = BufReader::new(f);
    let mut out: Vec<Action> = Vec::new();
    for line in reader.lines().map_while(|l| l.ok()) {
        let Ok(a) = serde_json::from_str::<Action>(&line) else {
            continue;
        };
        if let Some(c) = filter.category {
            if a.category != c {
                continue;
            }
        }
        if let Some(r) = filter.risk {
            if a.risk != Some(r) {
                continue;
            }
        }
        out.push(a);
        if let Some(limit) = filter.limit {
            if out.len() >= limit {
                break;
            }
        }
    }
    out
}

/// Load the user policy file. Returns an empty default if missing.
///
/// If the file exists but fails to parse, warn loudly (once per process) and
/// return an empty set — silently ignoring a malformed user policy would let
/// ActionGuard *appear* protected while user deny rules are not in effect.
pub fn load_policies_user() -> PolicyFile {
    static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    match fs::read_to_string(user_policy_path()) {
        Ok(s) => match serde_yaml::from_str(strip_bom(&s)) {
            Ok(pf) => pf,
            Err(e) => {
                if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    eprintln!(
                        "actionguard: WARNING {} is invalid YAML — user rules are NOT loaded: {e}",
                        user_policy_path().display()
                    );
                    eprintln!(
                        "actionguard: run `actionguard policy-lint {}` to see the error",
                        user_policy_path().display()
                    );
                }
                PolicyFile::default()
            }
        },
        Err(_) => PolicyFile::default(),
    }
}

/// Save the user policy file (pretty-printed YAML).
pub fn save_policies_user(policy: &PolicyFile) -> Result<()> {
    ensure_dirs()?;
    let s = serde_yaml::to_string(policy)?;
    fs::write(user_policy_path(), s)?;
    Ok(())
}


