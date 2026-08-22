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
//! hook is removed by deleting the marker block (never by grep-ing for
//! "actionguard"), and the local ledger is preserved unless the user opts out.

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
    let home = dirs::home_dir()?;
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

    let raw = match std::fs::read_to_string(rc) {
        Ok(s) => s,
        Err(_) => String::new(),
    };
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

/// Print the boundary summary in the exact style requested: Protected /
/// Observe-only / Not detected.
fn print_boundary_summary() {
    println!("Protected:");
    for d in crate::boundary::detect_boundaries() {
        if d.status == crate::boundary::BoundaryStatus::Enforced {
            println!("  ✓ {}", d.name);
        }
    }
    println!();
    println!("Observe only:");
    for d in crate::boundary::detect_boundaries() {
        if d.status == crate::boundary::BoundaryStatus::ObserveOnly {
            println!("  ⚠ {}", d.name);
        }
    }
    let undetected: Vec<_> = crate::boundary::detect_boundaries()
        .iter()
        .filter(|d| d.status == crate::boundary::BoundaryStatus::NotDetected)
        .map(|d| d.name.clone())
        .collect();
    if !undetected.is_empty() {
        println!();
        println!("Not detected:");
        for n in undetected {
            println!("  ○ {n}");
        }
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
        assert!(rc_file_for_shell("bash").unwrap().ends_with(".bashrc"));
        assert!(rc_file_for_shell("zsh").unwrap().ends_with(".zshrc"));
        assert!(rc_file_for_shell("fish").unwrap().ends_with("config.fish"));
        assert!(rc_file_for_shell("powershell")
            .unwrap()
            .to_string_lossy()
            .contains("Microsoft.PowerShell_profile.ps1"));
        // PowerShell 7 and Windows PowerShell 5.1 use different profile dirs.
        let ps7 = rc_file_for_shell("pwsh").unwrap();
        assert!(ps7.to_string_lossy().contains("PowerShell"));
        assert!(!ps7.to_string_lossy().contains("WindowsPowerShell"));
        let ps5 = rc_file_for_shell("powershell").unwrap();
        assert!(ps5.to_string_lossy().contains("WindowsPowerShell"));
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
