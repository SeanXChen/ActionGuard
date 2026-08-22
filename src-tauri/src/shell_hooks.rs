//! v0.2 Shell hook init scripts.
//!
//! One generator per shell: bash, zsh, fish, powershell. Each script:
//!
//!   1. Reads `~/.actionguard/sessions/current.hook` to discover the bridge
//!      port + per-session secret.
//!   2. Installs a pre-execution hook that POSTs each command to the bridge.
//!   3. If the bridge responds `deny`, aborts the command via a
//!      shell-specific mechanism.
//!
//! Failure policy — FAIL-CLOSED by default: if the bridge is unreachable,
//! the response is unparseable, or no active session exists, the command is
//! BLOCKED. `AG_ALLOW_ON_FAILURE=1` explicitly opts back into fail-open.
//! The one deliberate exception: after a clean session stop (the
//! `current.closed` sentinel) the hook allows the next command so the
//! protected terminal is not bricked.
//!
//! Best-effort blocking:
//!   - bash: SIGINT to self on `deny` (DEBUG trap can't directly abort).
//!   - zsh: `preexec` returns 1 (zsh honors this and skips the command).
//!   - fish: `commandline -f cancel` from `fish_preexec`.
//!   - powershell: PSReadLine `Enter` key handler. On `deny` we do NOT call
//!     `AcceptLine()` (that is what executes the line) — instead we call
//!     `RevertLine()` and swallow the keypress, so the command never enters
//!     the execution pipeline. Scope: interactive PSReadLine sessions only;
//!     non-interactive PowerShell (scripts, `-Command`, piped stdin) does not
//!     pass through PSReadLine and is NOT intercepted (see Execution Path
//!     Matrix in SECURITY_TEST_MATRIX.md).

use std::path::PathBuf;

/// Where the init script for a given shell + session is written to disk.
/// The terminal wrapper sources this file at startup.
pub fn init_script_path(session_id: &str, shell: &str) -> PathBuf {
    crate::storage::sessions_dir().join(format!("{session_id}.init.{shell}"))
}

/// Generate the init script for the requested shell.
pub fn generate(shell: &str) -> String {
    match shell {
        "bash" | "zsh" => posix_script(shell),
        "fish" => fish_script(),
        "powershell" | "pwsh" => powershell_script(),
        _ => posix_script("bash"),
    }
}

/// Write the init script to disk for a given session. Returns the path so
/// the terminal wrapper knows where to find it.
pub fn write_for_session(session_id: &str, shell: &str) -> std::io::Result<PathBuf> {
    let path = init_script_path(session_id, shell);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, generate(shell))?;
    Ok(path)
}

/// Bash + zsh share the POSIX-y syntax. `zsh` honors `preexec` returning 1
/// as a real abort; bash uses the SIGINT trick because its DEBUG trap can't
/// directly skip the next command.
fn posix_script(shell: &str) -> String {
    let preexec_hook = if shell == "zsh" {
        // zsh: returning non-zero from preexec aborts the command cleanly.
        // `zsh` only fires `preexec` when a function is REGISTERED via
        // `add-zsh-hook` (or happens to be named exactly `preexec`, which we
        // deliberately avoid so we never clobber the user's own hook). Without
        // this registration the function was dead code and zsh users got zero
        // enforcement.
        r#"
autoload -Uz add-zsh-hook
__actionguard_preexec() {
    __actionguard_check "$1" || return 1
}
add-zsh-hook preexec __actionguard_preexec
"#        .to_string()
    } else {
        // bash: DEBUG trap fires before each command. There's no clean way
        // to abort; the best-effort path sends SIGINT to ourselves.
        r#"
__actionguard_preexec() {
    if ! __actionguard_check "$BASH_COMMAND"; then
        kill -INT $$
    fi
}
trap __actionguard_preexec DEBUG
"#        .to_string()
    };

    format!(
        r#"# actionguard v0.2 shell hook ({shell})
# Auto-generated. Replaces nothing in your shell rc — only installs a
# preexec hook that talks to the ActionGuard bridge.
__actionguard_hook_path="${{HOME}}/.actionguard/sessions/current.hook"
__actionguard_closed_path="${{HOME}}/.actionguard/sessions/current.closed"

# Fail-closed by default: if the bridge is unreachable, broken, or there is
# no active session, the command is BLOCKED. Set AG_ALLOW_ON_FAILURE=1 to
# explicitly opt back into fail-open (allow on failure).
__actionguard_block() {{
    if [ "${{AG_ALLOW_ON_FAILURE:-0}}" = "1" ]; then
        return 0
    fi
    printf '\n  \033[31mactionguard: blocked\033[0m  %s\n  reason: protection unavailable (start a session, or set AG_ALLOW_ON_FAILURE=1 to allow)\n\n' "$1" >&2
    return 1
}}

__actionguard_check() {{
    local cmd="$1"
    [ -z "$cmd" ] && return 0

    # No active session? A clean-teardown sentinel means the user ended the
    # session deliberately — allow (the terminal must not be bricked).
    # Anything else means the hook is active with no policy engine: block.
    if [ ! -f "$__actionguard_hook_path" ]; then
        [ -f "$__actionguard_closed_path" ] && return 0
        __actionguard_block "$cmd"
        return $?
    fi

    # Read port + secret (two lines).
    local port secret
    {{ read port; read secret; }} < "$__actionguard_hook_path"
    if [ -z "$port" ] || [ -z "$secret" ]; then
        __actionguard_block "$cmd"
        return $?
    fi

    # POST the command to the bridge. `--max-time 3` prevents hanging on a
    # stuck bridge; on failure we fail-closed (block) by default, unless
    # AG_ALLOW_ON_FAILURE=1 explicitly opts back into fail-open.
    local body
    body=$(printf '{{"command":%s,"cwd":%s,"shell":"{shell}"}}' \
        "$(printf '%s' "$cmd" | python3 -c 'import sys,json;print(json.dumps(sys.stdin.read()))' 2>/dev/null || printf '"%s"' "$cmd")" \
        "$(printf '%s' "$PWD" | python3 -c 'import sys,json;print(json.dumps(sys.stdin.read()))' 2>/dev/null || printf '"%s"' "$PWD")")

    local resp
    resp=$(curl -s --max-time 3 \
        -X POST "http://127.0.0.1:$port/preexec" \
        -H "X-ActionGuard-Secret: $secret" \
        -H "Content-Type: application/json" \
        -d "$body" 2>/dev/null) || {{ __actionguard_block "$cmd"; return $?; }}

    # Extract the decision field. We use `sed` to avoid a hard jq dependency.
    local decision
    decision=$(printf '%s' "$resp" | sed -n 's/.*"decision":"\([^"]*\)".*/\1/p')
    if [ -z "$decision" ]; then
        __actionguard_block "$cmd"
        return $?
    fi

    if [ "$decision" = "deny" ]; then
        local reason
        reason=$(printf '%s' "$resp" | sed -n 's/.*"reason":"\([^"]*\)".*/\1/p')
        printf '\n  \033[31mactionguard: blocked\033[0m  %s\n  reason: %s\n\n' "$cmd" "$reason" >&2
        return 1
    fi

    return 0
}}
{preexec_hook}
# actionguard hook end
"#
    )
}

/// fish: use `fish_preexec` event + `commandline -f cancel` to abort.
fn fish_script() -> String {
    r#"# actionguard v0.2 shell hook (fish)
set -g __actionguard_hook_path "$HOME/.actionguard/sessions/current.hook"
set -g __actionguard_closed_path "$HOME/.actionguard/sessions/current.closed"

# Fail-closed by default: AG_ALLOW_ON_FAILURE=1 opts back into fail-open.
function __actionguard_block
    if test "$AG_ALLOW_ON_FAILURE" = "1"
        return 0
    end
    printf '\n  \033[31mactionguard: blocked\033[0m  %s\n  reason: protection unavailable (start a session, or set AG_ALLOW_ON_FAILURE=1 to allow)\n\n' $argv[1] >&2
    commandline -f cancel
    return 1
end

function __actionguard_check --on-event fish_preexec
    set -l cmd $argv[1]
    test -z "$cmd"; and return 0

    # No active session: a clean-teardown sentinel means the user ended the
    # session deliberately — allow. Otherwise the hook has no policy engine:
    # fail-closed unless AG_ALLOW_ON_FAILURE=1.
    test -f "$__actionguard_hook_path"; or begin
        test -f "$__actionguard_closed_path"; and return 0
        __actionguard_block "$cmd"
        return 0
    end

    # Read port + secret (two lines).
    set -l lines (cat "$__actionguard_hook_path")
    test (count $lines) -ge 2; or begin
        __actionguard_block "$cmd"
        return 0
    end
    set -l port $lines[1]
    set -l secret $lines[2]
    if test -z "$port"; or test -z "$secret"
        __actionguard_block "$cmd"
        return 0
    end

    # POST the command to the bridge.
    set -l body (printf '{"command":%s,"cwd":%s,"shell":"fish"}' \
        (printf '%s' "$cmd" | python3 -c 'import sys,json;print(json.dumps(sys.stdin.read()))' 2>/dev/null || printf '"%s"' "$cmd") \
        (printf '%s' "$PWD" | python3 -c 'import sys,json;print(json.dumps(sys.stdin.read()))' 2>/dev/null || printf '"%s"' "$PWD"))

    set -l resp (curl -s --max-time 3 \
        -X POST "http://127.0.0.1:$port/preexec" \
        -H "X-ActionGuard-Secret: $secret" \
        -H "Content-Type: application/json" \
        -d "$body" 2>/dev/null)
    test -n "$resp"; or begin
        __actionguard_block "$cmd"
        return 0
    end

    # Extract decision.
    set -l decision (echo "$resp" | sed -n 's/.*"decision":"\([^"]*\)".*/\1/p')
    test -n "$decision"; or begin
        __actionguard_block "$cmd"
        return 0
    end

    if test "$decision" = "deny"
        set -l reason (echo "$resp" | sed -n 's/.*"reason":"\([^"]*\)".*/\1/p')
        printf '\n  \033[31mactionguard: blocked\033[0m  %s\n  reason: %s\n\n' "$cmd" "$reason" >&2
        commandline -f cancel
    end
end
# actionguard hook end
"#.to_string()
}

/// PowerShell: interactive pre-execution block via a PSReadLine `Enter`
/// handler (Phase C). Non-interactive PowerShell (scripts, `-Command`,
/// piped stdin) bypasses PSReadLine and is NOT intercepted.
fn powershell_script() -> String {
    r#"# actionguard v0.2 shell hook (PowerShell) — Phase C interactive enforcement
# Intercepts every line typed at the PSReadLine prompt. On a `deny` from the
# bridge the line is REVERTED (never executed) and the keypress is swallowed.
# Scope: interactive PowerShell only. Scripts / -Command / piped stdin bypass
# PSReadLine and are NOT intercepted (see Execution Path Matrix).
$ActionGuard__HookPath = "$env:USERPROFILE\.actionguard\sessions\current.hook"
$ActionGuard__ClosedPath = "$env:USERPROFILE\.actionguard\sessions\current.closed"

# Fail-closed by default: an unreachable/broken bridge BLOCKS the command.
# $env:AG_ALLOW_ON_FAILURE = "1" explicitly opts back into fail-open.
function Test-ActionGuardFailOpen {
    return ($env:AG_ALLOW_ON_FAILURE -eq "1")
}

function Write-ActionGuardUnavailable {
    param([string]$Cmd)
    Write-Host ""
    Write-Host "  actionguard: blocked  $Cmd" -ForegroundColor Red
    Write-Host "  reason: protection unavailable (start a session, or set AG_ALLOW_ON_FAILURE=1 to allow)" -ForegroundColor DarkGray
    Write-Host ""
}

function Invoke-ActionGuardCheck {
    param([string]$Cmd)
    if ([string]::IsNullOrEmpty($Cmd)) { return $true }

    if (-not (Test-Path $ActionGuard__HookPath)) {
        # No active session. A clean-teardown sentinel means the user ended
        # the session deliberately — allow (don't brick the terminal).
        # Otherwise the hook has no policy engine: fail-closed by default.
        if (Test-Path $ActionGuard__ClosedPath) { return $true }
        if (-not (Test-ActionGuardFailOpen)) {
            Write-ActionGuardUnavailable -Cmd $Cmd
            return $false
        }
        return $true
    }

    $lines = Get-Content $ActionGuard__HookPath -ErrorAction SilentlyContinue
    if ($lines.Count -lt 2) {
        if (-not (Test-ActionGuardFailOpen)) {
            Write-ActionGuardUnavailable -Cmd $Cmd
            return $false
        }
        return $true
    }
    $port = $lines[0]
    $secret = $lines[1]
    if ([string]::IsNullOrEmpty($port) -or [string]::IsNullOrEmpty($secret)) {
        if (-not (Test-ActionGuardFailOpen)) {
            Write-ActionGuardUnavailable -Cmd $Cmd
            return $false
        }
        return $true
    }

    $body = @{ command = $Cmd; cwd = (Get-Location).Path; shell = "powershell" } | ConvertTo-Json -Compress
    try {
        $resp = Invoke-RestMethod -Uri "http://127.0.0.1:$port/preexec" `
            -Method Post `
            -Headers @{ "X-ActionGuard-Secret" = $secret } `
            -ContentType "application/json" `
            -Body $body `
            -TimeoutSec 3
        if ($resp.decision -eq "deny") {
            Write-Host ""
            Write-Host "  actionguard: blocked  $Cmd" -ForegroundColor Red
            Write-Host "  reason: $($resp.reason)" -ForegroundColor DarkGray
            Write-Host ""
            return $false
        }
    } catch {
        # Bridge unreachable/crashed — fail-closed by default; the
        # AG_ALLOW_ON_FAILURE=1 escape hatch restores fail-open.
        if (-not (Test-ActionGuardFailOpen)) {
            Write-ActionGuardUnavailable -Cmd $Cmd
            return $false
        }
    }
    return $true
}

# Phase C: intercept the Enter key. If the bridge says `deny`, we do NOT
# call AcceptLine() (that is what runs the line) — instead we RevertLine()
# and return $true so PSReadLine swallows the keypress. The command is
# discarded before it can ever be executed.
if (Get-Module -ListAvailable -Name PSReadLine) {
    Import-Module PSReadLine
    Set-PSReadLineKeyHandler -Key Enter -BriefDescription ActionGuardPreExec `
        -LongDescription "ActionGuard: block denied commands before execution" `
        -ScriptBlock {
            $line = $null
            [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$null)
            if (-not [string]::IsNullOrEmpty($line)) {
                $allowed = Invoke-ActionGuardCheck -Cmd $line
                if (-not $allowed) {
                    # Denied: revert the input line. The command is gone;
                    # AcceptLine() is never called so nothing executes.
                    [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
                    return $true
                }
            }
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
        }
}
# actionguard hook end
"#.to_string()
}

/// Detect the user's preferred shell from $SHELL (Unix) or PowerShell on
/// Windows (prefers PowerShell 7 `pwsh` when installed, else Windows
/// PowerShell 5.1). Returns a canonical name (`bash` / `zsh` / `fish` /
/// `pwsh` / `powershell`).
pub fn detect_default_shell() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        // PowerShell 7 (`pwsh`) is the modern default when installed;
        // otherwise fall back to the built-in Windows PowerShell 5.1.
        if pwsh_available() {
            "pwsh"
        } else {
            "powershell"
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        match std::env::var("SHELL").ok().and_then(|s| {
            let lower = s.to_lowercase();
            if lower.ends_with("zsh") {
                Some("zsh")
            } else if lower.ends_with("fish") {
                Some("fish")
            } else if lower.ends_with("bash") {
                Some("bash")
            } else {
                None
            }
        }) {
            Some(s) => match s {
                "zsh" => "zsh",
                "fish" => "fish",
                _ => "bash",
            },
            None => "bash",
        }
    }
}

/// True when PowerShell 7 (`pwsh`) is on PATH and responds.
#[cfg(target_os = "windows")]
fn pwsh_available() -> bool {
    std::process::Command::new("pwsh")
        .arg("-NoLogo")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("$true")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsh_hook_registers_preexec() {
        // Regression: the preexec function must be REGISTERED via
        // add-zsh-hook, otherwise zsh never calls it and users get zero
        // enforcement (the function name alone is not enough).
        let script = generate("zsh");
        assert!(
            script.contains("autoload -Uz add-zsh-hook"),
            "zsh script must autoload add-zsh-hook"
        );
        assert!(
            script.contains("add-zsh-hook preexec __actionguard_preexec"),
            "zsh script must register __actionguard_preexec as a preexec hook"
        );
    }

    #[test]
    fn bash_hook_uses_trap_not_zsh_hook() {
        let script = generate("bash");
        assert!(
            !script.contains("add-zsh-hook preexec"),
            "bash hook must not depend on zsh's add-zsh-hook"
        );
        assert!(
            script.contains("__actionguard_check"),
            "bash hook must still define the shared checker"
        );
    }

    #[test]
    fn powershell_hook_has_enter_handler() {
        let script = generate("powershell");
        assert!(
            script.contains("Enter"),
            "powershell hook must install a PSReadLine Enter handler"
        );
    }

    #[test]
    fn posix_hook_fails_closed_by_default() {
        let script = generate("bash");
        assert!(
            script.contains("__actionguard_block"),
            "bash hook must define the fail-closed helper"
        );
        assert!(
            script.contains("AG_ALLOW_ON_FAILURE"),
            "bash hook must honor the AG_ALLOW_ON_FAILURE escape hatch"
        );
        assert!(
            script.contains("current.closed"),
            "bash hook must check the clean-teardown sentinel"
        );
        assert!(
            !script.contains("fail-open (return 0 = allow)"),
            "bash hook must not document fail-open as the default"
        );
    }

    #[test]
    fn fish_hook_fails_closed_by_default() {
        let script = generate("fish");
        assert!(script.contains("__actionguard_block"));
        assert!(script.contains("AG_ALLOW_ON_FAILURE"));
        assert!(script.contains("current.closed"));
        assert!(
            !script.contains("or return 0"),
            "fish hook must not silently allow on failure"
        );
    }

    #[test]
    fn powershell_hook_fails_closed_by_default() {
        let script = generate("powershell");
        assert!(script.contains("Test-ActionGuardFailOpen"));
        assert!(script.contains("AG_ALLOW_ON_FAILURE"));
        assert!(script.contains("current.closed"));
        assert!(
            !script.contains("Fail-open on bridge errors"),
            "powershell hook must not silently allow on bridge errors"
        );
    }
}
