//! `actionguard setup` / `actionguard uninstall` — auditable install & clean
//! uninstall for v0.3 ("Distribution & Enforcement Reliability").
//!
//! Setup walks a developer through the whole install:
//!   detect OS → detect shell → preview changes → confirm → create
//!   `~/.actionguard/{policy,rules,hooks}` → seed built-in rules → install the
//!   shell hook (marker-delimited) → detect boundaries → non-destructive
//!   self-check → final status.
//!
//! Uninstall removes exactly what setup wrote — and nothing else. The shell
//!   hook is removed by deleting the marker block (never by grep-ing for
//!   "actionguard"), and the local ledger is preserved unless the user opts out.
//!
//! AI tool hooks (Cursor, Claude Code) are detected and configured via the
//!   `--ai-tools` flag or automatically when detected during interactive setup.

use crate::policy;
use crate::shell_hooks;
use crate::storage;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Marker block open line. Everything between this and [`MARKER_CLOSE`] is
/// owned by ActionGuard and may be removed wholesale by `uninstall`.
pub const MARKER_OPEN: &str = "# >>> ActionGuard >>>";
/// Marker block close line.
pub const MARKER_CLOSE: &str = "# <<< ActionGuard <<<";

// ---------------------------------------------------------------------------
// rc-file resolution
// ---------------------------------------------------------------------------

/// Return the rc file for a shell, if ActionGuard knows where it lives.
pub fn rc_file_for_shell(shell: &str) -> Option<PathBuf> {
    rc_file_for_home(dirs::home_dir()?.as_path(), shell)
}

/// Resolve an rc file relative to a supplied home directory.
///
/// Keeping the path mapping independent of the process environment makes it
/// deterministic to test and keeps [`rc_file_for_shell`] focused on resolving
/// the current user's home directory.
fn rc_file_for_home(home: &Path, shell: &str) -> Option<PathBuf> {
    match shell {
        "bash" => Some(home.join(".bashrc")),
        "zsh" => Some(home.join(".zshrc")),
        "fish" => Some(home.join(".config").join("fish").join("config.fish")),
        // PowerShell 7 keeps its profile under `Documents\PowerShell\`.
        "pwsh" => Some(home.join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1")),
        // Windows PowerShell 5.1 uses `Documents\WindowsPowerShell\`.
        "powershell" => Some(home.join("Documents").join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1")),
        _ => None,
    }
}

/// The source line that activates a hook file, for a given shell.
pub fn source_line(shell: &str) -> String {
    let hooks_dir = storage::data_dir().join("hooks");
    match shell {
        "fish" => format!("source {}", hooks_dir.join("fish.fish").display()),
        "powershell" | "pwsh" => format!(". {}", hooks_dir.join("powershell.ps1").display()),
        _ => format!("source {}", hooks_dir.join("posix.sh").display()),
    }
}

/// Hook file name for a shell, stored under `~/.actionguard/hooks/`.
pub fn hook_file_name(shell: &str) -> &'static str {
    match shell {
        "fish" => "fish.fish",
        "powershell" | "pwsh" => "powershell.ps1",
        _ => "posix.sh",
    }
}

// ---------------------------------------------------------------------------
// marker-block editing (install + uninstall share this)
// ---------------------------------------------------------------------------

/// Read the ActionGuard marker block (if any) out of an rc file.
/// Returns `(start_line, end_line_exclusive)` of the block, or `None`.
fn find_marker_block(rc: &Path) -> Result<Option<(usize, usize)>, String> {
    let raw = match std::fs::read_to_string(rc) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    let lines: Vec<&str> = raw.lines().collect();
    let mut start = None;
    for (i, l) in lines.iter().enumerate() {
        if l.trim() == MARKER_OPEN {
            start = Some(i);
        } else if l.trim() == MARKER_CLOSE {
            if let Some(s) = start {
                return Ok(Some((s, i + 1)));
            }
        }
    }
    Ok(None)
}

/// Insert (or replace) the ActionGuard marker block in an rc file. Never
/// touches lines outside the block. Returns `Ok(true)` when the file changed.
pub fn install_marker_block(rc: &Path, body: &str) -> Result<bool, String> {
    let block = format!("{MARKER_OPEN}\n{body}\n{MARKER_CLOSE}\n");
    let existing = find_marker_block(rc)?;

    let raw = std::fs::read_to_string(rc).unwrap_or_default();
    let updated = match existing {
        Some((s, e)) => {
            let mut lines: Vec<&str> = raw.lines().collect();
            lines.drain(s..e);
            lines.insert(s, block.trim_end());
            lines.join("\n") + "\n"
        }
        None => {
            if raw.is_empty() {
                block
            } else {
                // Keep the user's existing lines; append the block at the end.
                format!("{}\n{}\n", raw.trim_end(), block.trim_end())
            }
        }
    };

    if updated == raw {
        return Ok(false);
    }
    if let Some(parent) = rc.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(rc, updated).map_err(|e| format!("cannot write {}: {e}", rc.display()))?;
    Ok(true)
}

/// Remove the ActionGuard marker block (and nothing else) from an rc file.
/// Returns `Ok(true)` when something was removed.
pub fn remove_marker_block(rc: &Path) -> Result<bool, String> {
    let Some((s, e)) = find_marker_block(rc)? else {
        return Ok(false);
    };
    let raw = std::fs::read_to_string(rc).map_err(|e| format!("cannot read {}: {e}", rc.display()))?;
    let mut lines: Vec<&str> = raw.lines().collect();
    lines.drain(s..e);
    // Collapse the double newline left where the block used to sit.
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut prev_blank = false;
    for l in lines {
        let blank = l.trim().is_empty();
        if blank && prev_blank {
            continue;
        }
        out.push(l);
        prev_blank = blank;
    }
    std::fs::write(rc, out.join("\n") + "\n")
        .map_err(|e| format!("cannot write {}: {e}", rc.display()))?;
    Ok(true)
}

/// True when the marker block is present in an rc file.
pub fn marker_installed(rc: &Path) -> bool {
    find_marker_block(rc).ok().flatten().is_some()
}

// ---------------------------------------------------------------------------
// IDE/AI-tool hook detection and installation
// ---------------------------------------------------------------------------

/// Result of scanning for a supported AI tool's hook config file.
#[derive(Debug, Clone)]
pub struct AiToolHook {
    /// Display name e.g. "Cursor", "Claude Code"
    pub name: &'static str,
    /// Path to the tool's hook config file (~/.cursor/hooks.json etc.)
    pub config_path: PathBuf,
    /// Expected directory containing the hook script that needs to exist
    pub script_path: PathBuf,
    /// Hook script destination under ~/.actionguard/hooks/
    pub hook_dest: PathBuf,
    /// Whether the hook is already configured with an actionguard script
    pub is_configured: bool,
    /// Whether a legacy/old-path hook needs to be migrated to ~/.actionguard/hooks/
    pub needs_migration: bool,
}

fn command_is_actionguard(command: Option<&serde_json::Value>) -> bool {
    command
        .and_then(|value| value.as_str())
        .map(|command| command.contains("ag-") || command.contains("actionguard"))
        .unwrap_or(false)
}

fn cursor_hook_is_configured(config_path: &Path) -> bool {
    let Ok(raw) = std::fs::read_to_string(config_path) else {
        return false;
    };
    let Ok(config) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    config
        .get("hooks")
        .and_then(|hooks| hooks.get("beforeShellExecution"))
        .and_then(|hooks| hooks.as_array())
        .map(|entries| entries.iter().any(|entry| command_is_actionguard(entry.get("command"))))
        .unwrap_or(false)
}

fn claude_hook_is_configured(config_path: &Path) -> bool {
    let Ok(raw) = std::fs::read_to_string(config_path) else {
        return false;
    };
    let Ok(config) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    config
        .get("hooks")
        .and_then(|hooks| hooks.get("PreToolUse"))
        .and_then(|hooks| hooks.as_array())
        .map(|entries| {
            entries.iter().any(|entry| {
                entry
                    .get("hooks")
                    .and_then(|hooks| hooks.as_array())
                    .map(|hooks| hooks.iter().any(|hook| command_is_actionguard(hook.get("command"))))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Return the ActionGuard hook script for Cursor (Python adapter).
fn cursor_hook_script() -> &'static str {
    // Embedded Python adapter — same as scripts/hooks/ag-cursor-hook.py
    // This string is written verbatim to ~/.actionguard/hooks/cursor-hook.py
    r#"#!/usr/bin/env python3
"""
ActionGuard <-> Cursor IDE adapter (Class A Tool Hook).
Hook: ~/.cursor/hooks.json → beforeShellExecution.
See BOUNDARIES.md for the full protocol description.
"""
import json, os, shutil, subprocess, sys, time
from pathlib import Path

SHELL_TOOLS = {"execute_command", "Terminal"}
ALLOW_ON_FAILURE = os.environ.get("AG_ALLOW_ON_FAILURE", "0") == "1"
LOG_FILE = Path(os.environ.get("AG_LOG_DIR", Path.home() / ".actionguard")) / "hook-adapter.log"

_SCRIPT_DIR = Path(__file__).resolve().parent
_DEFAULT_REPO_EXE = _SCRIPT_DIR.parents[1] / "src-tauri" / "target" / "release" / "actionguard.exe"
_DEFAULT_UNIX_EXE = Path.home() / ".cargo" / "bin" / "actionguard"

def find_engine() -> str | None:
    env = os.environ.get("AG_CLI")
    if env and Path(env).exists(): return env
    on_path = shutil.which("actionguard")
    if on_path: return on_path
    if _DEFAULT_REPO_EXE.exists(): return str(_DEFAULT_REPO_EXE)
    return None

def log_event(event: dict) -> None:
    try:
        LOG_FILE.parent.mkdir(parents=True, exist_ok=True)
        event["ts"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
        with open(LOG_FILE, "a", encoding="utf-8") as fh:
            fh.write(json.dumps(event, ensure_ascii=False) + "\n")
    except OSError: pass

def emit(obj: dict) -> None:
    sys.stdout.buffer.write(json.dumps(obj, ensure_ascii=False).encode("utf-8") + b"\n")
    sys.stdout.buffer.flush()

def ask_engine(exe: str, cmd: str) -> dict:
    proc = subprocess.run([exe, "policy-check", cmd], capture_output=True, timeout=8)
    if proc.returncode != 0: raise RuntimeError(f"policy-check exited with status {proc.returncode}")
    raw_out = proc.stdout or b""
    for enc in ("utf-8", "gbk", "latin-1"):
        try: out = raw_out.decode(enc); break
        except UnicodeDecodeError: continue
    decision, matched, reason = None, "", ""
    for line in out.splitlines():
        line = line.strip()
        if line.startswith("decision:"): decision = line.split(":", 1)[1].strip()
        elif line.startswith("matched rule:"): matched = line.split(":", 1)[1].strip()
        elif line.startswith("reason:"): reason = line.split(":", 1)[1].strip()
    if decision not in {"allow", "confirm", "deny"}: raise RuntimeError("policy-check returned no valid decision")
    return {"decision": decision, "reason": reason, "matched": matched}

def fail(reason: str, payload: dict) -> int:
    log_event({"event": "fail", "reason": reason, "allow_on_failure": ALLOW_ON_FAILURE})
    if ALLOW_ON_FAILURE:
        emit({"confirmed": True, "reason": f"[AG-fail-open] {reason}"})
    else:
        emit({"denied": True, "reason": f"[ActionGuard-fail-closed] {reason}"})
    return 0

def main() -> int:
    if hasattr(sys.stdout, "reconfigure"): sys.stdout.reconfigure(encoding="utf-8")
    raw = sys.stdin.read() if not sys.stdin.isatty() else ""
    raw = raw.lstrip("\ufeff")
    try: payload = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as e: return fail(f"malformed stdin: {e}", {})
    tool_name = payload.get("tool_name", "")
    if tool_name not in SHELL_TOOLS:
        log_event({"event": "skipped", "tool": tool_name}); return 0
    tool_input = payload.get("tool_input", {}) or {}
    cmd = (tool_input.get("command") or tool_input.get("cmd") or tool_input.get("query") or "").strip()
    if not cmd: return fail("empty command", payload)
    exe = find_engine()
    if exe is None: return fail("actionguard engine not found", payload)
    try: verdict = ask_engine(exe, cmd)
    except Exception as e: return fail(f"engine error: {e}", payload)
    decision, reason, matched = verdict["decision"], verdict.get("reason", ""), verdict.get("matched", "")
    log_event({"event": "evaluate", "source": "cursor", "tool": tool_name, "command": cmd,
        "decision": decision, "matched": matched, "reason": reason, "allow_on_failure": ALLOW_ON_FAILURE})
    reason_str = f"[ActionGuard:{matched or decision}] {reason}"
    if decision == "deny": emit({"denied": True, "reason": reason_str})
    else: emit({"confirmed": True, "reason": reason_str})
    return 0

if __name__ == "__main__": sys.exit(main())
"#}

/// Return the ActionGuard hook script for Claude Code (Node.js-compatible JSON).
fn claude_hook_script() -> &'static str {
    r#"#!/usr/bin/env python3
"""
ActionGuard <-> Claude Code adapter (Class A Tool Hook).
Hook: ~/.claude/settings.json → hooks.PreToolUse.
See BOUNDARIES.md for the full protocol description.
"""
import json, os, shutil, subprocess, sys, time
from pathlib import Path

SHELL_TOOLS = {"Bash", "ExecuteCommand", "Terminal"}
ALLOW_ON_FAILURE = os.environ.get("AG_ALLOW_ON_FAILURE", "0") == "1"
LOG_FILE = Path(os.environ.get("AG_LOG_DIR", Path.home() / ".actionguard")) / "hook-adapter.log"

_SCRIPT_DIR = Path(__file__).resolve().parent
_DEFAULT_REPO_EXE = _SCRIPT_DIR.parents[1] / "src-tauri" / "target" / "release" / "actionguard.exe"
_DEFAULT_UNIX_EXE = Path.home() / ".cargo" / "bin" / "actionguard"

def find_engine() -> str | None:
    env = os.environ.get("AG_CLI")
    if env and Path(env).exists(): return env
    on_path = shutil.which("actionguard")
    if on_path: return on_path
    if _DEFAULT_REPO_EXE.exists(): return str(_DEFAULT_REPO_EXE)
    return None

def log_event(event: dict) -> None:
    try:
        LOG_FILE.parent.mkdir(parents=True, exist_ok=True)
        event["ts"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
        with open(LOG_FILE, "a", encoding="utf-8") as fh:
            fh.write(json.dumps(event, ensure_ascii=False) + "\n")
    except OSError: pass

def emit(obj: dict) -> None:
    sys.stdout.buffer.write(json.dumps(obj, ensure_ascii=False).encode("utf-8") + b"\n")
    sys.stdout.buffer.flush()

def ask_engine(exe: str, cmd: str) -> dict:
    proc = subprocess.run([exe, "policy-check", cmd], capture_output=True, timeout=8)
    if proc.returncode != 0: raise RuntimeError(f"policy-check exited with status {proc.returncode}")
    raw_out = proc.stdout or b""
    for enc in ("utf-8", "gbk", "latin-1"):
        try: out = raw_out.decode(enc); break
        except UnicodeDecodeError: continue
    decision, matched, reason = None, "", ""
    for line in out.splitlines():
        line = line.strip()
        if line.startswith("decision:"): decision = line.split(":", 1)[1].strip()
        elif line.startswith("matched rule:"): matched = line.split(":", 1)[1].strip()
        elif line.startswith("reason:"): reason = line.split(":", 1)[1].strip()
    if decision not in {"allow", "confirm", "deny"}: raise RuntimeError("policy-check returned no valid decision")
    return {"decision": decision, "reason": reason, "matched": matched}

def fail(reason: str, payload: dict) -> int:
    log_event({"event": "fail", "reason": reason, "allow_on_failure": ALLOW_ON_FAILURE})
    if ALLOW_ON_FAILURE:
        emit({"hookSpecificOutput": {"permissionDecision": "allow", "permissionDecisionReason": f"[AG-fail-open] {reason}"}})
    else:
        emit({"hookSpecificOutput": {"permissionDecision": "deny", "permissionDecisionReason": f"[ActionGuard-fail-closed] {reason}"}})
    return 0

def main() -> int:
    if hasattr(sys.stdout, "reconfigure"): sys.stdout.reconfigure(encoding="utf-8")
    raw = sys.stdin.read() if not sys.stdin.isatty() else ""
    raw = raw.lstrip("\ufeff")
    try: payload = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as e: return fail(f"malformed stdin: {e}", {})
    tool_name = payload.get("hookName", payload.get("tool_name", ""))
    if tool_name not in SHELL_TOOLS:
        log_event({"event": "skipped", "tool": tool_name}); return 0
    tool_input = payload.get("toolInput", payload.get("tool_input", {})) or {}
    cmd = (tool_input.get("command") or tool_input.get("commandInput", {}).get("command") or "").strip()
    if not cmd: return fail("empty command", payload)
    exe = find_engine()
    if exe is None: return fail("actionguard engine not found", payload)
    try: verdict = ask_engine(exe, cmd)
    except Exception as e: return fail(f"engine error: {e}", payload)
    decision, reason, matched = verdict["decision"], verdict.get("reason", ""), verdict.get("matched", "")
    log_event({"event": "evaluate", "source": "claude_code", "tool": tool_name, "command": cmd,
        "decision": decision, "matched": matched, "reason": reason, "allow_on_failure": ALLOW_ON_FAILURE})
    reason_str = f"[ActionGuard:{matched or decision}] {reason}"
    if decision == "deny": emit({"hookSpecificOutput": {"permissionDecision": "deny", "permissionDecisionReason": reason_str}})
    elif decision == "confirm": emit({"hookSpecificOutput": {"permissionDecision": "ask", "permissionDecisionReason": reason_str}})
    else: emit({"hookSpecificOutput": {"permissionDecision": "allow", "permissionDecisionReason": reason_str}})
    return 0

if __name__ == "__main__": sys.exit(main())
"#
}

/// Scan for installed AI tools that have hook configuration files.
pub fn detect_ai_tools() -> Vec<AiToolHook> {
    let mut tools = Vec::new();
    let home = dirs::home_dir();

    // --- Cursor ---
    if let Some(ref h) = home {
        let cursor_dir = h.join(".cursor");
        let hooks_path = cursor_dir.join("hooks.json");
        let script_src = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("actionguard.exe")))
            .map(|p| {
                if p.exists() { p } else {
                    // Fallback: scripts/hooks/ag-cursor-hook.py in repo
                    let repo_exe = p.parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.join("scripts").join("hooks").join("ag-cursor-hook.py"))
                        .unwrap_or_default();
                    repo_exe
                }
            });
        let hook_dest = storage::data_dir().join("hooks").join("cursor-hook.py");
        let is_configured = cursor_hook_is_configured(&hooks_path);
        let needs_migration = hooks_path.exists() &&
            std::fs::read_to_string(&hooks_path).ok()
                .map(|s| s.contains("Action Guard/scripts/hooks/ag-cursor-hook") ||
                       s.contains("ag-cursor-hook.py"))
                .unwrap_or(false);

        if hooks_path.exists() || script_src.as_ref().map(|p| p.exists()).unwrap_or(false) {
            tools.push(AiToolHook {
                name: "Cursor",
                config_path: hooks_path,
                script_path: script_src.unwrap_or_default(),
                hook_dest,
                is_configured,
                needs_migration,
            });
        }
    }

    // --- Claude Code ---
    if let Some(ref h) = home {
        let claude_dir = h.join(".claude");
        let settings_path = claude_dir.join("settings.json");
        let hook_dest = storage::data_dir().join("hooks").join("claude-hook.py");
        let is_configured = claude_hook_is_configured(&settings_path);
        let needs_migration = settings_path.exists() &&
            std::fs::read_to_string(&settings_path).ok()
                .map(|s| s.contains("scripts/hooks/") || s.contains("ag-claude-hook.py"))
                .unwrap_or(false);

        if settings_path.exists() || claude_dir.exists() {
            tools.push(AiToolHook {
                name: "Claude Code",
                config_path: settings_path,
                script_path: PathBuf::new(),
                hook_dest,
                is_configured,
                needs_migration,
            });
        }
    }

    tools
}

/// Install the ActionGuard hook script for a given AI tool under ~/.actionguard/hooks/.
fn install_ai_hook_script(hook_dest: &Path, _name: &str, script: &str) -> Result<(), String> {
    let hooks_dir = hook_dest.parent().ok_or("invalid hook dest parent")?;
    std::fs::create_dir_all(hooks_dir)
        .map_err(|e| format!("cannot create {}: {e}", hooks_dir.display()))?;
    std::fs::write(hook_dest, script)
        .map_err(|e| format!("cannot write {}: {e}", hook_dest.display()))?;
    // Make the script executable (Unix-style; on Windows the .py extension is sufficient)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(hook_dest)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(hook_dest, perms)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Write or update the Cursor hooks.json to include the ActionGuard beforeShellExecution hook.
fn install_cursor_hooks_json(config_path: &Path, hook_script: &Path) -> Result<bool, String> {
    let script_cmd = if cfg!(windows) {
        format!("python {}", hook_script.display())
    } else {
        format!("python3 {}", hook_script.display())
    };

    let existing: serde_json::Value = if config_path.exists() {
        let raw = std::fs::read_to_string(config_path)
            .map_err(|e| format!("cannot read {}: {e}", config_path.display()))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("cannot parse {}: {e}", config_path.display()))?
    } else {
        serde_json::json!({})
    };

    // Merge hooks.beforeShellExecution
    let mut hooks = existing.get("hooks")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    let mut before_shell = hooks.get("beforeShellExecution")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    // Check if we already have an ActionGuard entry
    let has_ag_entry = before_shell.iter().any(|entry| {
        entry.get("command")
            .and_then(|c| c.as_str())
            .map(|s| s.contains("ag-") || s.contains("actionguard"))
            .unwrap_or(false)
    });

    if has_ag_entry {
        // Check if any existing ActionGuard entry needs updating to the new path
        let new_cmd = script_cmd.clone();
        let needs_update = before_shell.iter().any(|entry| {
            entry.get("command")
                .and_then(|c| c.as_str())
                .map(|s| s.contains("ag-") && !s.contains(&*hook_script.display().to_string()))
                .unwrap_or(false)
        });
        if !needs_update {
            return Ok(false); // already using the right path
        }
        // Update all existing ActionGuard entries to the new script path
        for entry in &mut before_shell {
            let is_ag = entry.get("command")
                .and_then(|c| c.as_str())
                .map(|s| s.contains("ag-") || s.contains("actionguard"))
                .unwrap_or(false);
            if is_ag {
                if let Some(cmd_val) = entry.get_mut("command") {
                    *cmd_val = serde_json::json!(new_cmd);
                }
            }
        }
    } else {
        // Add new ActionGuard entry
        let new_entry = serde_json::json!({ "command": script_cmd });
        before_shell.push(new_entry);
    }

    hooks.insert("beforeShellExecution".to_string(), serde_json::json!(before_shell));

    let mut final_obj = existing.as_object().cloned().unwrap_or_default();
    final_obj.insert("hooks".to_string(), serde_json::Value::Object(hooks));

    // Create parent dir if needed
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }

    let pretty = serde_json::to_string_pretty(&serde_json::Value::Object(final_obj))
        .map_err(|e| format!("JSON error: {e}"))?;
    std::fs::write(config_path, pretty)
        .map_err(|e| format!("cannot write {}: {e}", config_path.display()))?;

    Ok(true)
}

/// Write or update the Claude Code settings.json to include the ActionGuard PreToolUse hook.
fn install_claude_settings_json(config_path: &Path, hook_script: &Path) -> Result<bool, String> {
    let script_cmd = if cfg!(windows) {
        format!("python {}", hook_script.display())
    } else {
        format!("python3 {}", hook_script.display())
    };

    let new_hook = serde_json::json!({
        "matcher": "execute_command|Bash|Terminal",
        "hooks": [{
            "type": "command",
            "command": script_cmd,
            "timeout": 10
        }]
    });

    let existing: serde_json::Value = if config_path.exists() {
        let raw = std::fs::read_to_string(config_path)
            .map_err(|e| format!("cannot read {}: {e}", config_path.display()))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("cannot parse {}: {e}", config_path.display()))?
    } else {
        serde_json::json!({})
    };

    let mut hooks_map = existing.get("hooks")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    let pre_tool_use = hooks_map.get("PreToolUse")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    let already_configured = pre_tool_use.iter().any(|entry| {
        entry.get("hooks")
            .and_then(|h| h.as_array())
            .map(|arr| arr.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|s| s.contains("ag-") || s.contains("actionguard"))
                    .unwrap_or(false)
            }))
            .unwrap_or(false)
    });

    if already_configured {
        return Ok(false);
    }

    let mut new_list = pre_tool_use;
    new_list.push(new_hook);
    hooks_map.insert("PreToolUse".to_string(), serde_json::json!(new_list));

    let mut final_obj = existing.as_object().cloned().unwrap_or_default();
    final_obj.insert("hooks".to_string(), serde_json::Value::Object(hooks_map));

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }

    let pretty = serde_json::to_string_pretty(&serde_json::Value::Object(final_obj))
        .map_err(|e| format!("JSON error: {e}"))?;
    std::fs::write(config_path, pretty)
        .map_err(|e| format!("cannot write {}: {e}", config_path.display()))?;

    Ok(true)
}

/// Install hook scripts and update config files for all detected AI tools.
/// Returns (installed_count, errors).
pub fn install_all_ai_hooks() -> (usize, Vec<String>) {
    let tools = detect_ai_tools();
    let mut installed = 0;
    let mut errors = Vec::new();

    for tool in tools {
        let already_done = tool.is_configured && !tool.needs_migration;
        if already_done {
            println!("  ✓ {} — already configured", tool.name);
            continue;
        }

        let needs_setup = !tool.is_configured || tool.needs_migration;

        if needs_setup {
            // Write the hook script
            let script = match tool.name {
                "Cursor" => cursor_hook_script(),
                "Claude Code" => claude_hook_script(),
                _ => {
                    println!("  ? {} — not supported yet", tool.name);
                    continue;
                }
            };

            if let Err(e) = install_ai_hook_script(&tool.hook_dest, tool.name, script) {
                errors.push(format!("{} hook script: {e}", tool.name));
                continue;
            }
            println!("  ✓ {} — installed hook script → {}", tool.name, tool.hook_dest.display());

            // Update the config file (append our entry, don't clobber existing ones)
            let updated = match tool.name {
                "Cursor" => install_cursor_hooks_json(&tool.config_path, &tool.hook_dest),
                "Claude Code" => install_claude_settings_json(&tool.config_path, &tool.hook_dest),
                _ => continue,
            };

            match updated {
                Ok(true) => {
                    println!("  ✓ {} — hooks.json updated ({})", tool.name, tool.config_path.display());
                    installed += 1;
                }
                Ok(false) => {
                    println!("  ✓ {} — already has ActionGuard entry", tool.name);
                }
                Err(e) => {
                    errors.push(format!("{} config: {e}", tool.name));
                }
            }
        }
    }

    (installed, errors)
}

// ---------------------------------------------------------------------------
// setup — one-command install
// ---------------------------------------------------------------------------

/// Run the full `actionguard setup` flow. `assume_yes` skips the confirmation
/// prompt (CI / scripting). Returns a process exit code.
pub fn run_setup(assume_yes: bool) -> i32 {
    let shell = shell_hooks::detect_default_shell();
    let os = os_name();
    let rc = rc_file_for_shell(shell);

    println!("ActionGuard Setup");
    println!("──────────────────────────────");
    println!();
    println!("Detected:");
    println!("  OS:        {os}");
    println!("  Shell:     {shell}");
    if let Some(rc) = &rc {
        println!("  rc file:   {}", rc.display());
    }
    println!();

    // --- Plan ---
    let hooks_dir = storage::data_dir().join("hooks");
    let rules_dir = storage::data_dir().join("rules");
    let will_create = vec![
        storage::data_dir(),
        storage::data_dir().join("policy"),
        hooks_dir.clone(),
        rules_dir.clone(),
    ];
    let mut will_modify: Vec<String> = Vec::new();
    if let Some(rc) = &rc {
        if marker_installed(rc) {
            will_modify.push(format!("{} (update ActionGuard hook block)", rc.display()));
        } else {
            will_modify.push(format!("{} (append ActionGuard hook block)", rc.display()));
        }
    }

    println!("Will create:");
    for p in &will_create {
        println!("  + {}", p.display());
    }
    println!();
    println!("Will modify:");
    for m in &will_modify {
        println!("  ~ {m}");
    }
    println!();
    println!("No existing lines will be overwritten.");

    if !assume_yes && !confirm("Continue?") {
        println!("Setup aborted. Nothing was changed.");
        return 1;
    }

    // --- Execute ---
    let mut errors: Vec<String> = Vec::new();

    // 1) Directories.
    for d in &will_create {
        if let Err(e) = std::fs::create_dir_all(d) {
            errors.push(format!("cannot create {}: {e}", d.display()));
        }
    }

    // 2) Seed built-in rules so users can inspect / contribute.
    for (name, yaml) in policy::loader::builtin_rule_files() {
        let target = rules_dir.join(format!("{name}.yml"));
        if target.exists() {
            continue; // never clobber an edited copy
        }
        if let Err(e) = std::fs::write(&target, yaml) {
            errors.push(format!("cannot write {}: {e}", target.display()));
        }
    }

    // 3) Write the shell hook file.
    let hook_path = hooks_dir.join(hook_file_name(shell));
    let script = shell_hooks::generate(shell);
    if let Err(e) = std::fs::write(&hook_path, script) {
        errors.push(format!("cannot write {}: {e}", hook_path.display()));
    }

    // 4) Install the rc marker block.
    if let Some(rc) = &rc {
        match install_marker_block(rc, &source_line(shell)) {
            Ok(_) => {}
            Err(e) => errors.push(e),
        }
    }

    // 4b) Detect and install AI tool hooks (Cursor, Claude Code)
    println!();
    println!("Scanning for AI tools...");
    let (ai_installed, ai_errors) = install_all_ai_hooks();
    errors.extend(ai_errors);
    if ai_installed > 0 {
        println!("  {ai_installed} AI tool(s) configured.");
    }

    // 5) Non-destructive self-check through the policy engine.
    let policy_ok = self_check(&mut errors);

    // --- Report ---
    println!();
    println!("ActionGuard Setup complete.");
    println!();
    print_boundary_summary();
    println!();
    if !errors.is_empty() {
        println!("Warnings:");
        for e in &errors {
            println!("  ⚠ {e}");
        }
        println!();
        println!("Run:");
        println!("  actionguard doctor        — detailed status");
        return 1;
    }
    println!("Run:");
    println!("  actionguard doctor        — verify this machine is really protected");
    println!("  actionguard boundary test — non-destructive boundary verification");
    if policy_ok {
        0
    } else {
        println!();
        println!("⚠ Policy self-check failed — see warnings above.");
        1
    }
}

/// Non-destructive self-check: run the exact policy path a shell command would
/// hit and require a hard deny for `sudo rm -rf /`. Never executes anything.
fn self_check(errors: &mut Vec<String>) -> bool {
    let set = policy::load_policy_set();
    let mut action = crate::models::Action::new_shell(
        "sudo rm -rf /".to_string(),
        None,
        Some("setup-self-check".to_string()),
    );
    let parsed = policy::classify::classify_shell_command("sudo rm -rf /");
    action.category = parsed.category;
    action.kind = Some(policy::classify::kind_for(&parsed).to_string());
    let r = crate::risk::evaluate_action(&action);
    action.risk = Some(r.level);
    let decision = policy::decide(&action, &set);
    let ok = decision.decision == crate::models::Decision::Deny;
    let decision_label = match decision.decision {
        crate::models::Decision::Deny => "DENY",
        crate::models::Decision::Ask => "ASK",
        crate::models::Decision::Allow => "ALLOW",
    };
    println!("Self-check (no execution):");
    println!(
        "  sudo rm -rf / → {decision_label} (rule: {})",
        decision.matched_rule.as_deref().unwrap_or("(none)")
    );
    if !ok {
        errors.push(
            "self-check: `sudo rm -rf /` did not resolve to DENY — built-in rules may be missing"
                .to_string(),
        );
    }
    ok
}

/// Print the boundary summary — Coverage Ladder style.
/// Shows quality tiers rather than flat lists.
fn print_boundary_summary() {
    use crate::models::BoundaryKind;
    let all = crate::boundary::detect_boundaries();

    let mut enforced = Vec::new();      // actively blocking
    let mut observe = Vec::new();       // observe-only
    let mut inactive = Vec::new();      // installed but not running
    let mut not_detected = Vec::new();  // no boundary available

    for b in &all {
        match b.status {
            crate::boundary::BoundaryStatus::Enforced => enforced.push(b),
            crate::boundary::BoundaryStatus::ObserveOnly => observe.push(b),
            crate::boundary::BoundaryStatus::Inactive => inactive.push(b),
            crate::boundary::BoundaryStatus::NotDetected => not_detected.push(b),
        }
    }

    println!("Protection Coverage:");
    if enforced.is_empty() {
        println!("  ○ No enforced boundaries — unknown AI apps are not protected");
    } else {
        for b in &enforced {
            let tier = match b.kind {
                BoundaryKind::ToolHook => "High-quality",
                BoundaryKind::ExecApproval => "High-quality",
                BoundaryKind::ProtectedShell => "Generic",
                _ => "Other",
            };
            println!("  ✓ {} ({tier})", b.name);
        }
    }

    if !inactive.is_empty() {
        println!();
        println!("Inactive (installed but not running):");
        for b in &inactive {
            println!("  ○ {}", b.name);
        }
    }

    if !observe.is_empty() {
        println!();
        println!("Observe-only (no enforcement mechanism on this path):");
        for b in &observe {
            println!("  ⚠ {}", b.name);
        }
    }

    if !not_detected.is_empty() {
        println!();
        println!("Generic fallback (unknown apps go through shell if active):");
        let has_generic_shell = enforced.iter().any(|b| b.kind == BoundaryKind::ProtectedShell);
        if has_generic_shell {
            println!("  ✓ Protected shell is active — actions via shell are blocked.");
        } else {
            println!("  ○ No shell boundary detected — actions via shell are observe-only.");
        }
        println!("  Unknown apps without dedicated hooks: {}", not_detected.len());
        println!("  Run `actionguard coverage` for the full breakdown.");
    }
}

// ---------------------------------------------------------------------------
// uninstall — clean removal
// ---------------------------------------------------------------------------

/// Run the full `actionguard uninstall` flow. `assume_yes` skips the
/// confirmation prompt. Returns a process exit code.
pub fn run_uninstall(assume_yes: bool) -> i32 {
    let shell = shell_hooks::detect_default_shell();
    let rc = rc_file_for_shell(shell);

    println!("ActionGuard Uninstall");
    println!("──────────────────────────────");
    println!();
    println!("ActionGuard will remove:");
    println!("  ✓ ActionGuard shell hook block (marker-delimited, only ActionGuard's own lines)");
    let hooks_dir = storage::data_dir().join("hooks");
    if hooks_dir.exists() {
        println!("  ✓ {}", hooks_dir.display());
    }
    if let Some(rc) = &rc {
        println!("  ✓ shell integration entry in {}", rc.display());
    }
    println!();

    let keep_ledger = if assume_yes {
        true
    } else {
        println!("Preserve local ledger (sessions, snapshots, evidence)?");
        confirm("[Yes] keep ledger / [No] delete everything under ~/.actionguard")
    };

    if !assume_yes {
        let proceed = confirm("Proceed with uninstall?");
        if !proceed {
            println!("Uninstall aborted. Nothing was changed.");
            return 1;
        }
    }

    let mut errors: Vec<String> = Vec::new();

    // 1) Remove the rc marker block — exactly ActionGuard's own lines.
    if let Some(rc) = &rc {
        match remove_marker_block(rc) {
            Ok(true) => println!("  ✓ removed hook block from {}", rc.display()),
            Ok(false) => println!("  - no ActionGuard block in {}", rc.display()),
            Err(e) => errors.push(e),
        }
    }

    // 2) Remove the hooks directory we own.
    if hooks_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&hooks_dir) {
            errors.push(format!("cannot remove {}: {e}", hooks_dir.display()));
        } else {
            println!("  ✓ removed {}", hooks_dir.display());
        }
    }

    // 3) Ledger policy.
    if keep_ledger {
        println!("  ✓ ledger preserved at {}", storage::data_dir().display());
    } else if let Err(e) = std::fs::remove_dir_all(storage::data_dir()) {
        errors.push(format!("cannot remove {}: {e}", storage::data_dir().display()));
    } else {
        println!("  ✓ removed {}", storage::data_dir().display());
    }

    println!();
    if errors.is_empty() {
        println!("ActionGuard has been removed cleanly. Everything else on this machine is untouched.");
        0
    } else {
        println!("Uninstall finished with warnings:");
        for e in &errors {
            println!("  ⚠ {e}");
        }
        1
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Human-readable OS name.
pub fn os_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Windows"
    }
    #[cfg(target_os = "macos")]
    {
        "macOS"
    }
    #[cfg(target_os = "linux")]
    {
        "Linux"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "Unknown"
    }
}

/// Read a yes/no answer from stdin. Anything starting with `y`/`Y` is yes.
fn confirm(prompt: &str) -> bool {
    print!("{prompt} [y/N] ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(0) => false,
        Ok(_) => line.trim().starts_with('y') || line.trim().starts_with('Y'),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rc_file_maps_shells() {
        let home = Path::new("C:/Users/ActionGuard");
        assert!(rc_file_for_home(home, "bash").unwrap().ends_with(".bashrc"));
        assert!(rc_file_for_home(home, "zsh").unwrap().ends_with(".zshrc"));
        assert!(rc_file_for_home(home, "fish").unwrap().ends_with("config.fish"));
        assert!(rc_file_for_home(home, "powershell")
            .unwrap()
            .to_string_lossy()
            .contains("Microsoft.PowerShell_profile.ps1"));
        // PowerShell 7 and Windows PowerShell 5.1 use different profile dirs.
        let ps7 = rc_file_for_home(home, "pwsh").unwrap();
        assert!(ps7.to_string_lossy().contains("PowerShell"));
        assert!(!ps7.to_string_lossy().contains("WindowsPowerShell"));
        let ps5 = rc_file_for_home(home, "powershell").unwrap();
        assert!(ps5.to_string_lossy().contains("WindowsPowerShell"));
    }

    #[test]
    fn cursor_hook_install_preserves_unrelated_configuration() {
        let dir = std::env::temp_dir().join(format!("ag-cursor-config-test-{}", std::process::id()));
        let config = dir.join("hooks.json");
        let hook = dir.join("cursor-hook.py");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            &config,
            r#"{"theme":"dark","hooks":{"beforeShellExecution":[{"command":"other-hook"}]}}"#,
        )
        .unwrap();

        assert!(install_cursor_hooks_json(&config, &hook).unwrap());
        let updated: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        assert_eq!(updated["theme"], "dark");
        assert_eq!(updated["hooks"]["beforeShellExecution"].as_array().unwrap().len(), 2);
        assert!(!cursor_hook_is_configured(&dir.join("missing.json")));
        assert!(cursor_hook_is_configured(&config));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn invalid_hook_configuration_is_not_overwritten() {
        let dir = std::env::temp_dir().join(format!("ag-invalid-config-test-{}", std::process::id()));
        let config = dir.join("hooks.json");
        let hook = dir.join("cursor-hook.py");
        std::fs::create_dir_all(&dir).unwrap();
        let invalid = "{not valid JSON";
        std::fs::write(&config, invalid).unwrap();

        assert!(install_cursor_hooks_json(&config, &hook).is_err());
        assert_eq!(std::fs::read_to_string(&config).unwrap(), invalid);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn source_line_matches_hook_file() {
        let bash = source_line("bash");
        assert!(bash.contains("posix.sh"));
        let fish = source_line("fish");
        assert!(fish.contains("fish.fish"));
    }

    #[test]
    fn marker_block_roundtrip() {
        let dir = std::env::temp_dir().join(format!("ag-setup-test-{}", std::process::id()));
        let rc = dir.join(".bashrc");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&rc, "export FOO=1\n").unwrap();

        // Install.
        assert!(install_marker_block(&rc, "source ~/.actionguard/hooks/posix.sh").unwrap());
        let raw = std::fs::read_to_string(&rc).unwrap();
        assert!(raw.contains(MARKER_OPEN));
        assert!(raw.contains("source ~/.actionguard/hooks/posix.sh"));
        assert!(raw.contains("export FOO=1")); // untouched

        // Idempotent replace (update body, keep position).
        assert!(install_marker_block(&rc, "source ~/.actionguard/hooks/posix.sh # v2").unwrap());
        let raw = std::fs::read_to_string(&rc).unwrap();
        assert!(raw.contains("# v2"));
        assert!(!raw.contains("# v2 # v2"));

        // Remove exactly the block.
        assert!(remove_marker_block(&rc).unwrap());
        let raw = std::fs::read_to_string(&rc).unwrap();
        assert!(!raw.contains(MARKER_OPEN));
        assert!(!raw.contains("actionguard"));
        assert!(raw.contains("export FOO=1"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
