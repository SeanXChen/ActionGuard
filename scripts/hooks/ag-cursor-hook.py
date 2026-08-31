#!/usr/bin/env python3
"""
ActionGuard <-> Cursor IDE adapter (v0.2, Class A Tool Hook).

HOOK TYPE: Cursor hooks.json — beforeShellExecution
The adapter is invoked BEFORE each shell command runs inside Cursor's terminal.
It calls `actionguard policy-check <cmd>`, reads the decision, and maps it to
the Cursor hook response:

    ActionGuard allow   -> return hook response with allowed: true
    ActionGuard confirm -> return hook response with confirmed: true (prompt)
    ActionGuard deny    -> return hook response with denied: true

FAIL-CLOSED BY DEFAULT. If the engine is unreachable or returns an error,
the command is DENIED and the event is logged to ~/.actionguard/hook-adapter.log.
Set AG_ALLOW_ON_FAILURE=1 to opt back into fail-open.

Log: ~/.actionguard/hook-adapter.log (one JSON line per evaluated action).
"""

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

# Cursor hooks run under node on Windows, so we use python (python3 from PATH).
SHELL_TOOLS = {"execute_command", "Terminal"}
ALLOW_ON_FAILURE = os.environ.get("AG_ALLOW_ON_FAILURE", "0") == "1"

LOG_DIR = Path(os.environ.get("AG_LOG_DIR", Path.home() / ".actionguard"))
LOG_FILE = LOG_DIR / "hook-adapter.log"

_SCRIPT_DIR = Path(__file__).resolve().parent
_DEFAULT_REPO_EXE = _SCRIPT_DIR.parents[1] / "src-tauri" / "target" / "release" / "actionguard.exe"
_DEFAULT_UNIX_EXE = Path.home() / ".cargo" / "bin" / "actionguard"


def find_engine() -> str | None:
    env = os.environ.get("AG_CLI")
    if env and Path(env).exists():
        return env
    on_path = shutil.which("actionguard")
    if on_path:
        return on_path
    # Fallback to the built release exe in the repo
    if _DEFAULT_REPO_EXE.exists():
        return str(_DEFAULT_REPO_EXE)
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
    """Write the hook verdict as UTF-8 bytes."""
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
    if proc.returncode != 0:
        raise RuntimeError(f"policy-check exited with status {proc.returncode}")
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

    decision = None
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
    if decision not in {"allow", "confirm", "deny"}:
        raise RuntimeError("policy-check returned no valid decision")
    return {"decision": decision, "reason": reason, "matched": matched}


def fail(reason: str, payload: dict) -> int:
    """Fail-closed by default: deny on engine failure."""
    log_event({
        "event": "fail",
        "reason": reason,
        "payload": payload,
        "allow_on_failure": ALLOW_ON_FAILURE,
    })
    if ALLOW_ON_FAILURE:
        emit({"confirmed": True, "reason": f"[AG-fail-open] {reason}"})
    else:
        emit({"denied": True, "reason": f"[ActionGuard-fail-closed] {reason}"})
    return 0


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    # Read stdin — Cursor passes the hook payload as JSON
    raw = sys.stdin.read() if not sys.stdin.isatty() else ""
    raw = raw.lstrip("\ufeff")
    try:
        payload = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as e:
        return fail(f"malformed stdin JSON: {e}", {"raw": raw[:200]})

    tool_name = payload.get("tool_name", "")
    if tool_name not in SHELL_TOOLS:
        log_event({"event": "skipped", "tool": tool_name})
        return 0  # not a shell action; passthrough

    tool_input = payload.get("tool_input", {}) or {}
    cmd = (
        tool_input.get("command")
        or tool_input.get("cmd")
        or tool_input.get("query")
        or ""
    ).strip()

    if not cmd:
        return fail("empty command", payload)

    exe = find_engine()
    if exe is None:
        return fail("actionguard engine not found", payload)

    try:
        verdict = ask_engine(exe, cmd)
    except (subprocess.SubprocessError, OSError, RuntimeError) as e:
        return fail(f"engine error: {e}", payload)

    decision = verdict["decision"]
    reason = verdict.get("reason") or "ActionGuard policy"
    matched = verdict.get("matched") or ""

    log_event({
        "event": "evaluate",
        "source": "cursor",
        "tool": tool_name,
        "command": cmd,
        "decision": decision,
        "matched": matched,
        "reason": reason,
        "allow_on_failure": ALLOW_ON_FAILURE,
    })

    # Map ActionGuard verdicts to Cursor hook response
    reason_str = f"[ActionGuard:{matched or decision}] {reason}"
    if decision == "deny":
        emit({"denied": True, "reason": reason_str})
    elif decision == "confirm":
        emit({"confirmed": True, "reason": reason_str})
    else:
        emit({"confirmed": True, "reason": reason_str})  # allow = confirmed in Cursor

    return 0


if __name__ == "__main__":
    sys.exit(main())
