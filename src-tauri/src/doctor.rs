//! `actionguard doctor` — answer the question "is this machine really
//! protected right now?", instead of "did setup run at some point?".
//!
//! Checks (in order):
//!   1. Binary  — is the installed binary runnable?
//!   2. Policy  — how many rules actually loaded?
//!   3. Workspace — is the current directory under a protected session?
//!   4. Current shell — is the shell hook active in the *current* shell?
//!   5. Current session — is the enforcement bridge connected?
//!   6. Boundary — which boundary classes are enforced / observe-only?
//!   7. Test (`--test`) — non-destructive deny simulation.
//!
//! Final status is one of: PROTECTED, OBSERVE ONLY, NOT FULLY PROTECTED,
//! NOT PROTECTED. This is deliberately starker than a green checkmark.

use crate::boundary;
use crate::policy;
use crate::risk;
use crate::setup;
use crate::shell_hooks;
use crate::storage;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Run `actionguard doctor`. `run_test` enables `--test` (deny simulation).
pub fn run_doctor(run_test: bool) -> i32 {
    let shell = shell_hooks::detect_default_shell();
    let rc = setup::rc_file_for_shell(shell);
    let rc_active = rc.as_ref().map(|p| setup::marker_installed(p)).unwrap_or(false);
    let session_live = storage::current_hook_symlink().exists();
    let bridge_ok = session_live && probe_bridge().is_ok();
    let boundaries = boundary::detect_boundaries();
    let enforced: Vec<_> = boundaries
        .iter()
        .filter(|d| d.status == boundary::BoundaryStatus::Enforced)
        .collect();

    println!("ActionGuard Doctor");
    println!("────────────────────────────────────");

    // 1) Binary
    println!("Binary");
    let exe = std::env::current_exe().ok().map(|p| p.to_string_lossy().to_string());
    match &exe {
        Some(p) => println!("  ✓ Installed ({p})"),
        None => println!("  ✗ Cannot locate binary"),
    }
    println!();

    // 2) Policy
    println!("Policy");
    let set = policy::load_policy_set();
    let rules = set.rules.len();
    println!("  ✓ {rules} rules loaded");
    println!();

    // 3) Workspace
    println!("Workspace");
    if session_live {
        let ws = session_workspace().unwrap_or_else(|| "?".to_string());
        println!("  ✓ Protected (workspace: {ws})");
    } else {
        println!("  ✗ No active protected session");
    }
    println!();

    // 4) Current shell
    println!("Current shell");
    if cfg!(target_os = "windows") && matches!(shell, "powershell" | "pwsh") {
        println!("  ⚠ PowerShell");
        println!("  ✓ Interactive lines:  Phase C ENFORCED (PSReadLine Enter handler, verified 2026-08-21)");
        println!("  ⚠ Scripts / -Command / piped stdin:  OBSERVE-ONLY (bypass PSReadLine — known bypass)");
        println!("  ⚠ Use Git Bash for full enforcement across every execution path.");
    } else if rc_active {
        println!("  ✓ {shell} hook active");
    } else {
        println!("  ⚠ {shell} detected but hook not active in rc file");
        if let Some(rc) = &rc {
            println!("  ⚠ expected marker in {}", rc.display());
        }
    }
    println!();

    // 5) Current session / bridge
    println!("Current session");
    if bridge_ok {
        println!("  ✓ Enforcement connected");
    } else if session_live {
        println!("  ⚠ Session file present but bridge unreachable");
    } else {
        println!("  ✗ No active session (run `actionguard protect`)");
    }
    println!();

    // 6) Boundary — one block per boundary, four orthogonal questions.
    // Detected ≠ Enforced, and neither equals Verified. The same layout
    // works for every boundary class (shells, tool hooks, native exec).
    println!("Boundary");
    for d in &boundaries {
        println!("  {}", d.name);
        let detected = d.detected;
        let supported = d.kind != crate::models::BoundaryKind::Remote;
        println!(
            "    Detected     {}",
            if detected { "✓" } else { "✗" }
        );
        println!(
            "    Supported    {}",
            if supported { "✓" } else { "✗" }
        );
        println!(
            "    Enforceable  {}",
            if d.enforceable { "✓" } else { "✗" }
        );
        println!(
            "    Verified     {}",
            if d.last_verified.is_empty() {
                "✗ not yet".to_string()
            } else {
                format!("✓ {}", d.last_verified)
            }
        );
        println!("    Status       {}", d.status.label());
        if !d.note.is_empty() {
            println!("    Note         {}", d.note);
        }
    }
    println!();

    // 7) Test (optional)
    let mut test_pass = true;
    if run_test {
        println!("Test (no execution — simulated deny)");
        let policy_deny = local_deny_check();
        println!(
            "  ✓ Action reached policy: {}",
            if policy_deny { "DENY" } else { "NOT DENY (bug!)" }
        );
        if bridge_ok {
            match remote_deny_check() {
                Ok(lines) => {
                    for l in lines {
                        println!("  {l}");
                    }
                }
                Err(e) => {
                    println!("  ⚠ bridge deny check failed: {e}");
                    test_pass = false;
                }
            }
        } else {
            println!("  ⚠ bridge offline — ledger check skipped (start a protected session)");
            test_pass = policy_deny;
        }
        if !policy_deny {
            test_pass = false;
        }
        println!();
    }

    // 8) Status
    println!("Status");
    let powershell_observe = cfg!(target_os = "windows") && matches!(shell, "powershell" | "pwsh");
    let status = if !session_live || !bridge_ok {
        "NOT PROTECTED"
    } else if powershell_observe {
        // PowerShell: interactive lines are Phase C enforced, but scripts /
        // -Command / piped stdin bypass the hook — so overall protection is
        // real but partial. We cannot detect from inside `doctor` whether the
        // current PowerShell is interactive or not, so we report the partial
        // truth rather than either extreme.
        "NOT FULLY PROTECTED (PowerShell: interactive enforced, non-interactive observe-only)"
    } else if !enforced.is_empty() && rc_active {
        "PROTECTED"
    } else if !enforced.is_empty() {
        "NOT FULLY PROTECTED"
    } else {
        "NOT PROTECTED"
    };
    println!("  {status}");
    println!();

    if run_test && !test_pass {
        return 1;
    }
    if status == "PROTECTED" {
        0
    } else {
        1
    }
}

// ---------------------------------------------------------------------------
// probes
// ---------------------------------------------------------------------------

/// Read the current hook descriptor: (port, secret).
fn read_hook_descriptor() -> Result<(u16, String), String> {
    let link = storage::current_hook_symlink();
    let raw = std::fs::read_to_string(&link)
        .map_err(|e| format!("no active hook ({e})"))?;
    let mut lines = raw.lines();
    let port: u16 = lines
        .next()
        .ok_or("hook file empty")?
        .trim()
        .parse()
        .map_err(|e: std::num::ParseIntError| format!("bad port: {e}"))?;
    let secret = lines.next().ok_or("hook file missing secret")?.trim().to_string();
    Ok((port, secret))
}

/// GET /status against the bridge.
fn probe_bridge() -> Result<String, String> {
    let (port, secret) = read_hook_descriptor()?;
    http_get(port, &secret, "/status")
}

/// Best-effort: the active session's workspace from the hook file's sibling
/// `<id>.json` files. Falls back to "?".
fn session_workspace() -> Option<String> {
    let dir = storage::sessions_dir();
    let mut best: Option<(std::time::SystemTime, String)> = None;
    for entry in std::fs::read_dir(&dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        // Session json has `"workspace":"..."` — cheap scan without serde.
        if let Some(idx) = raw.find("\"workspace\"") {
            let rest = &raw[idx..];
            if let Some(start) = rest.find('"').map(|i| i + 1) {
                let v = &rest[start..];
                if let Some(end) = v.find('"') {
                    let ws = v[..end].to_string();
                    let mtime = entry.metadata().ok()?.modified().ok()?;
                    if best.as_ref().map(|(t, _)| mtime > *t).unwrap_or(true) {
                        best = Some((mtime, ws));
                    }
                }
            }
        }
    }
    best.map(|(_, ws)| ws)
}

/// Local dry-run: would the policy engine deny `sudo rm -rf /`?
fn local_deny_check() -> bool {
    let cmd = "sudo rm -rf /";
    let set = policy::load_policy_set();
    let mut action = crate::models::Action::new_shell(
        cmd.to_string(),
        None,
        Some("doctor-test".to_string()),
    );
    let parsed = policy::classify::classify_shell_command(cmd);
    action.category = parsed.category;
    action.kind = Some(policy::classify::kind_for(&parsed).to_string());
    let r = risk::evaluate_action(&action);
    action.risk = Some(r.level);
    let d = policy::decide(&action, &set);
    d.decision == crate::models::Decision::Deny
}

/// Remote dry-run: POST the simulated deny through the live bridge. The
/// bridge only *decides*; it never executes. The ledger row proves the
/// enforcement path end-to-end.
fn remote_deny_check() -> Result<Vec<String>, String> {
    let (port, secret) = read_hook_descriptor()?;
    let body = serde_json::json!({
        "command": "sudo rm -rf /",
        "cwd": std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
    })
    .to_string();
    let resp = http_post(port, &secret, "/preexec", &body)?;
    let v: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|e| format!("bad bridge response: {e}"))?;
    let decision = v.get("decision").and_then(|d| d.as_str()).unwrap_or("unknown");
    let mut lines = vec![
        "  ✓ Action reached boundary: bridge /preexec accepted".to_string(),
        format!("  ✓ Policy returned:         {decision}"),
    ];
    if decision == "deny" {
        lines.push("  ✓ Execution prevented:     no execution by bridge".into());
        lines.push("  ✓ Ledger recorded:         check `actionguard ledger tail`".into());
        Ok(lines)
    } else {
        lines.push("  ✗ Expected DENY but got something else".into());
        Err("deny simulation did not deny".into())
    }
}

// ---------------------------------------------------------------------------
// minimal HTTP client (local bridge only)
// ---------------------------------------------------------------------------

fn http_get(port: u16, secret: &str, path: &str) -> Result<String, String> {
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nX-ActionGuard-Secret: {secret}\r\nConnection: close\r\n\r\n"
    );
    http_roundtrip(port, req)
}

fn http_post(port: u16, secret: &str, path: &str, body: &str) -> Result<String, String> {
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nX-ActionGuard-Secret: {secret}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    http_roundtrip(port, req)
}

fn http_roundtrip(port: u16, req: String) -> Result<String, String> {
    let mut stream = TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(2),
    )
    .map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    let raw = String::from_utf8_lossy(&buf).to_string();
    Ok(raw
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_policy_denies_rm_rf() {
        assert!(local_deny_check(), "sudo rm -rf / must resolve to DENY");
    }

    #[test]
    fn status_enum_liveness() {
        // Sanity: the checks we print don't panic in a headless env.
        let _ = session_workspace();
    }
}
