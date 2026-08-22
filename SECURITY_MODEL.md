# ActionGuard Security Model

The threat model in one sentence:

> **Detection ≠ Protection.** ActionGuard records *every* action it sees, but
> it can only *stop* an action that crosses one of its pre-action boundaries —
> and only on the execution paths that boundary actually covers.

This document is the source of truth for what ActionGuard guarantees, what it
cannot, and how to reason about the gap. It is deliberately conservative: a
security product that over-promises is worse than one that under-sells.

---

## 1. Core model — three numbers, one gap

Every action that reaches the engine is classified and recorded. Whether it
is *stopped* depends on the boundary it crossed:

```
        detected                blocked by policy          stopped before execution
   (recorded, regardless   (decision = Deny)         (Deny on an L2 path)
    of path coverage)              │                          │
              ▼                    ▼                          ▼
        ┌─────────────┐    ┌──────────────┐    ┌──────────────────────────┐
        │  Detected   │ →  │    Blocked   │ →  │        Enforced          │
        │ (ledger)    │    │ (decision)   │    │ (execution prevented)    │
        └─────────────┘    └──────────────┘    └──────────────────────────┘
                    │                │                    │
             a Deny is only    any Deny on a      a Deny on an L2 path
             as strong as      non-L2 path ends   is a hard stop —
             the path it       as Observed /      the command does not run
             runs on           Bypassed
```

The CLI (`actionguard stats`, `actionguard session show`) and the GUI
dashboard surface this split explicitly:

| Metric | Meaning |
|--------|---------|
| **Detected** | Every action recorded to the ledger, on any path. |
| **Blocked** | Every action whose policy decision was `Deny`. |
| **Enforced** | Denies that were executed *before* the action ran (L2 paths only). |
| **Observed** | Actions ActionGuard saw but could not pre-empt (no pre-action boundary on that path). |
| **Bypassed** | Actions that executed on a path outside any boundary's coverage. |
| **Unsupported** | Actions whose boundary class has no enforcement mechanism yet. |

## 2. Capability Tiers — L1 → L4

`actionguard capabilities` prints the four-tier model and the live
execution-path matrix for the current machine.

| Tier | Name | What it means | v0.3 |
|------|------|---------------|------|
| **L1** | Observe | ActionGuard records the action but cannot stop it before execution. Decisions are advisory. | yes |
| **L2** | Pre-action | A `Deny` prevents execution. Shell preexec hooks, tool hooks, exec-approval layers. | yes |
| **L3** | Runtime | Process-level enforcement / sandboxing during execution. | planned |
| **L4** | System | OS-level, vendor-independent enforcement (Endpoint Security / fanotify). | planned |

A path that supports a higher tier trivially satisfies the lower tiers. The
matrix is decided **per execution path**, never per software brand:

```
Execution Path Matrix (Windows, this machine):
  PowerShell interactive (PSReadLine)    L2   deny reverts the line
  PowerShell scripts / -Command / piped  L1   bypasses PSReadLine
  cmd.exe interactive                     —    not covered
  bash/zsh/fish (WSL/MSYS2)             L2   preexec hook
  python -c "os.system(...)"              —    inner exec is not a shell line
  absolute-path binaries                  —    rules match by program name
```

> **Why the matrix exists.** The same product can run a command through an
> L2 path today and an uncovered path tomorrow. Brand-based guarantees are
> fiction; path-based guarantees are measurable.

## 3. Fail-closed by default

Since v0.3 the engine defaults to **fail-closed**:

- If the policy engine cannot be reached (bridge down, session gone)…
- If the decision response cannot be parsed…
- If there is no active session when a hook fires…

…the command is **blocked** and the user is told why, unless the operator
explicitly opts out per environment with `AG_ALLOW_ON_FAILURE=1`.

Two deliberate exceptions keep a terminal usable:

1. **Deliberate stop** — after a clean `actionguard stop`, the *next*
   command is allowed so the user is never locked out of a dead session
   (`current.closed` sentinel). Per-session audit markers still record the
   stop.
2. **Explicit opt-out** — `AG_ALLOW_ON_FAILURE=1` restores the old
   fail-open behavior for environments that prefer availability.

See `src-tauri/src/shell_hooks.rs` (POSIX / fish / PowerShell) and
`scripts/hooks/ag-hook.py` for the exact decision tree.

## 4. Trust boundaries

ActionGuard sits *between* the automation and the machine. It does not
redefine the machine's security — it adds a decision layer in front of
dangerous actions.

```
   AI / agent / user
         │  action
         ▼
   ┌────────────────────┐   L2 (pre-action, e.g. shell preexec,
   │  ActionGuard hook   │   tool hook) — can hard-stop here
   └─────────┬──────────┘
             │  HTTP 127.0.0.1:<port> (per-session, secret-authenticated)
             ▼
   ┌────────────────────┐
   │  ActionGuard core   │  classify → policy → decision (Allow/Ask/Deny)
   └─────────┬──────────┘
             ▼
   ┌────────────────────┐
   │  Execution          │  L1 watcher observes post-hoc what ran
   └────────────────────┘
```

| Trust boundary | Property |
|----------------|----------|
| Hook → Core | Loopback-only listener, per-session random secret, commands are validated before evaluation. |
| Core → Policy | Rules are hot-reloaded, signed or hash-verified when shipped, mtime-checked per preexec. |
| Core → Ledger | Every decision is appended (append-only); session files are integrity-checked on load. |
| Approval gate | HIGH/CRITICAL need explicit human yes; headless TTY approval is logged and attributable. |

## 5. The six boundary classes (A–F)

| Class | Kind | Pre-action? | Enforceable? |
|-------|------|-------------|--------------|
| **A** | Tool Hook | Yes (tool middleware) | Yes |
| **B** | Exec Approval | Yes (agent-side policy) | Via independent layer |
| **C** | Protected Shell | Yes (preexec) | Yes |
| **D** | Runtime Sandbox | Yes (future L3) | Yes (planned) |
| **E** | System Enforcement | Yes (future L4) | Yes (planned) |
| **F** | Remote | No — never on this machine | No (address space) |

The live per-product registry is data-driven from `boundaries/*.yml`
(`actionguard boundary list`); per-product evidence lives in
[`BOUNDARIES.md`](./BOUNDARIES.md) and
[`SECURITY_TEST_MATRIX.md`](./SECURITY_TEST_MATRIX.md).

## 6. Known bypass vectors (v0.3)

ActionGuard is honest about what it cannot cover. Full table:
[`SECURITY_TEST_MATRIX.md`](./SECURITY_TEST_MATRIX.md).

| Vector | Status |
|--------|--------|
| Absolute-path binaries (`/usr/bin/rm`) | Bypassed (rules match program name) |
| Inner subprocess exec (`python -c os.system`) | Bypassed |
| PowerShell scripts / `-Command` / piped stdin | Observed only |
| File writes via non-shell APIs | Observed only (watcher is post-hoc) |
| Remote sandbox execution (Manus Cloud) | Outside local boundary by design |

## 7. What you should rely on

1. **L2 paths are real.** On a supported path, a `Deny` stops the command
   before it runs — verified by `e2e` tests and `boundary test`.
2. **The ledger is the truth.** Every action, decision, and enforcement
   outcome is recorded; never trust a counter that contradicts the ledger.
3. **`capabilities` tells you what is covered** on *your* machine, right now.
4. **Observe mode is a shield, not a wall.** If you audit only, expect
   bypasses; that is a feature of the mode, not a bug.

## 8. File layout

- `SECURITY_MODEL.md` — this document (model + guarantees).
- `SECURITY_TEST_MATRIX.md` — per-action enforcement evidence.
- `BOUNDARIES.md` — boundary-class design + per-product analysis.
- `boundaries/*.yml` — data-driven boundary registry (community-editable).
- `src-tauri/src/platform.rs` — execution-path matrix source of truth.
- `src-tauri/src/shell_hooks.rs`, `scripts/hooks/ag-hook.py` — fail-closed hook logic.
