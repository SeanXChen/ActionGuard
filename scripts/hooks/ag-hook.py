#!/usr/bin/env python3
"""ActionGuard <-> CodeBuddy PreToolUse adapter (v0.3, fail-closed by default).

An ACTION BOUNDARY adapter: it does not care which vendor produced the
command. It asks one question - "what does ActionGuard policy say?" - and
enforces the answer.

Reads a CodeBuddy PreToolUse JSON payload on stdin (fields: tool_name,
tool_input.command / cwd / session_id / permission_mode), evaluates the shell
command against the local ActionGuard engine (`actionguard policy-check`), and
maps the verdict to a CodeBuddy hook decision on stdout:

    ActionGuard allow   -> {"hookSpecificOutput":{"permissionDecision":"allow"}}
    ActionGuard confirm -> {"hookSpecificOutput":{"permissionDecision":"ask"}}
    ActionGuard deny    -> {"hookSpecificOutput":{"permissionDecision":"deny"}}

Failure policy (v0.3): FAIL-CLOSED by default. If ActionGuard is unreachable,
the payload is malformed, or evaluation times out, the command is DENIED and
the event is logged — a safety layer must block when it cannot evaluate.
Set AG_ALLOW_ON_FAILURE=1 to explicitly opt back into fail-open (allow on
failure).

Usage (CodeBuddy settings.json, Windows: hooks run under Git Bash):

    {
      "hooks": {
        "PreToolUse": [
          {
            "matcher": "execute_command|Bash",
            "hooks": [
              { "type": "command", "command": "python3 <repo>/scripts/hooks/ag-hook.py", "timeout": 10 }
            ]
          }
        ]
      }
    }

Log: ~/.actionguard/hook-adapter.log (one JSON line per evaluated action).
"""

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

SHELL_TOOLS = {"Bash", "execute_command", "Terminal"}
# Fail-closed by default; AG_ALLOW_ON_FAILURE=1 opts back into fail-open.
ALLOW_ON_FAILURE = os.environ.get("AG_ALLOW_ON_FAILURE", "0") == "1"

LOG_DIR = Path(os.environ.get("AG_LOG_DIR", Path.home() / ".actionguard"))
LOG_FILE = LOG_DIR / "hook-adapter.log"

# Resolution order for the ActionGuard CLI. When the engine is missing the
# adapter fails closed (deny) so commands are never silently unprotected.
_SCRIPT_DIR = Path(__file__).resolve().parent

# Dev-repo fallback: scripts/hooks/ag-hook.py -> repo root -> src-tauri/target.
# Derived from the script location instead of a hard-coded machine path, so it
# keeps working when the repo is checked out anywhere else.
#
# parents[i]: with _SCRIPT_DIR = <repo>/scripts/hooks,
#   parents[0] = <repo>/scripts, parents[1] = <repo>, parents[2] = the drive
#   root. parents[1] is the repo root — parents[2] was the bug that made the
#   engine unreachable from the hook (fail-closed denies on every command).
_DEFAULT_REPO_EXE = _SCRIPT_DIR.parents[1] / "src-tauri" / "target" / "debug" / "actionguard.exe"
_DEFAULT_UNIX_EXE = Path.home() / ".cargo/bin/actionguard"


def find_engine() -> str | None:
    env = os.environ.get("AG_CLI")
    if env and Path(env).exists():
        return env
    on_path = shutil.which("actionguard")
    if on_path:
        return on_path
    for cand in (_DEFAULT_REPO_EXE, _DEFAULT_UNIX_EXE):
        if cand.exists():
            return str(cand)
    return None


def log_event(event: dict) -> None:
    try:
        LOG_DIR.mkdir(parents=True, exist_ok=True)
        event["ts"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
        with open(LOG_FILE, "a", encoding="utf-8") as fh:
            fh.write(json.dumps(event, ensure_ascii=False) + "\n")
    except OSError:
        pass


def emit(obj: dict) -> None:
    """Write the hook verdict as UTF-8 bytes, bypassing console encoding."""
    sys.stdout.buffer.write(
        json.dumps(obj, ensure_ascii=False).encode("utf-8") + b"\n"
    )
    sys.stdout.buffer.flush()


def ask_engine(exe: str, cmd: str) -> dict:
    """Return {'decision': 'allow'|'confirm'|'deny', 'reason': str, 'matched': str}."""
    proc = subprocess.run(
        [exe, "policy-check", cmd],
        capture_output=True,
        timeout=8,
    )
    # The engine's stdout encoding varies by platform (UTF-8 on Unix, GBK on
    # Windows). Decode by trial; never let a decode error crash the adapter.
    raw_out = proc.stdout or b""
    out = ""
    for enc in ("utf-8", "gbk", "latin-1"):
        try:
            out = raw_out.decode(enc)
            break
        except UnicodeDecodeError:
            continue
    if not out:
        out = raw_out.decode("utf-8", errors="replace")
    decision = "allow"
    matched = ""
    reason = ""
    for line in out.splitlines():
        line = line.strip()
        if line.startswith("decision:"):
            decision = line.split(":", 1)[1].strip()
        elif line.startswith("matched rule:"):
            matched = line.split(":", 1)[1].strip()
        elif line.startswith("reason:"):
            reason = line.split(":", 1)[1].strip()
    return {"decision": decision, "reason": reason, "matched": matched}


def fail(reason: str, payload: dict) -> int:
    """Handle an evaluation failure.

    Fail-closed by default: the command is DENIED so a broken or unreachable
    engine can never silently leave the user unprotected. AG_ALLOW_ON_FAILURE=1
    opts back into fail-open (allow + log), for environments that prefer
    availability over enforcement.
    """
    log_event(
        {
            "event": "fail",
            "reason": reason,
            "payload": payload,
            "allow_on_failure": ALLOW_ON_FAILURE,
        }
    )
    if ALLOW_ON_FAILURE:
        emit({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "ActionGuard unavailable; allowed (fail-open, AG_ALLOW_ON_FAILURE=1) - " + reason,
            }
        })
    else:
        emit({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": "ActionGuard unavailable; denied (fail-closed) - " + reason,
            }
        })
    return 0


def main() -> int:
    # Force UTF-8 on stdout so the JSON verdict is byte-identical on every
    # platform, even when Windows pipes would otherwise fall back to GBK.
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    raw = sys.stdin.read() if not sys.stdin.isatty() else ""
    # Windows files/editors frequently carry a UTF-8 BOM; strip it defensively.
    raw = raw.lstrip("\ufeff")
    try:
        payload = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as e:
        return fail(f"malformed stdin JSON: {e}", {"raw": raw[:200]})

    tool_name = payload.get("tool_name", "")
    if tool_name not in SHELL_TOOLS:
        log_event({"event": "skipped", "tool": tool_name, "payload": payload})
        return 0  # not a shell action; do not police it

    tool_input = payload.get("tool_input", {}) or {}
    cmd = (
        tool_input.get("command")
        or tool_input.get("cmd")
        or tool_input.get("query")
        or ""
    ).strip()
    cwd = payload.get("cwd") or ""
    session_id = payload.get("session_id") or ""

    if not cmd:
        return fail("empty command", payload)

    exe = find_engine()
    if exe is None:
        return fail("actionguard engine not found", payload)

    try:
        verdict = ask_engine(exe, cmd)
    except (subprocess.SubprocessError, OSError) as e:
        return fail(f"engine error: {e}", payload)

    decision = verdict["decision"]
    reason = verdict.get("reason") or "ActionGuard policy"
    matched = verdict.get("matched") or ""

    log_event(
        {
            "event": "evaluate",
            "session_id": session_id,
            "cwd": cwd,
            "tool": tool_name,
            "command": cmd,
            "decision": decision,
            "matched": matched,
            "reason": reason,
            "allow_on_failure": ALLOW_ON_FAILURE,
        }
    )

    # Map ActionGuard verdicts onto CodeBuddy hook decisions.
    if decision == "deny":
        perm = "deny"
    elif decision == "confirm":
        perm = "ask"  # pause for human approval in the UI
    else:
        perm = "allow"

    emit({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": perm,
            "permissionDecisionReason": f"[ActionGuard:{matched or decision}] {reason}",
        }
    })
    return 0


if __name__ == "__main__":
    sys.exit(main())
