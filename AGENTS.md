# AGENTS.md

Instructions for AI coding agents working on ActionGuard.

You are modifying a **security product**. Do not infer enforcement from policy
decisions. Do not weaken safety defaults to make code simpler.

## Project

ActionGuard is a local, user-controlled safety boundary for AI-powered automation.

Core principle:

> Attach by boundary, not by brand.

The core engine must remain **automation-neutral**. Never special-case
individual products (CodeBuddy, Claude Code, Codex, Cursor, OpenClaw, Manus).
New automation sources are added via boundary adapters or registry entries
(`boundaries/*.yml`, `src-tauri/src/boundary.rs` `registry()`), never via
brand branches in the engine.

## Architecture

```
Action Source → Boundary → ActionGuard Core → Classify → Policy → Decision
             → Approval → Evidence (ledger)
```

The core safety invariant — using the project's own vocabulary
(see `SECURITY_MODEL.md`):

```
Detected ≠ Blocked ≠ Enforced
```

- **Detected** — recorded to the ledger, on any path.
- **Blocked** — policy decision was Deny.
- **Enforced** — a Deny that actually prevented execution (pre-action/L2 path only).

A policy decision of Deny does **not** mean the action was prevented. The
enforcement result must be recorded separately (`EnforcementStatus` in
`src-tauri/src/commands.rs` / `models.rs`).

## Boundary model

Boundaries are execution paths, not product brands. The six Boundary Classes
(A–F), mirrored by `BoundaryKind` in `src-tauri/src/models.rs`:

| Class | Name |
|---|---|
| A | Tool Hook (pre-action hook inside the automation) |
| B | Exec Approval (automation's own execution policy) |
| C | Protected Shell (preexec hook; PowerShell PSReadLine) |
| D | Runtime Sandbox (future L3 — do not invent now) |
| E | System Enforcement (future L4 — do not invent now) |
| F | Remote (actions never land on this machine — out of scope) |

Never invent new boundary classes. Add adapters/registry entries, not brand branches.

## Enforcement semantics

Allowed enforcement states (exact enum values):

- `Enforced` — action was actually prevented before execution
- `Observed` — action was seen but could not be pre-empted
- `Bypassed` — action executed on a path outside boundary coverage
- `Unsupported` — boundary class has no enforcement mechanism yet

**Detection never implies enforcement.** Do not mark a boundary `Enforced`
without reproducible evidence (a `boundary test` run, ledger `action_id`).

## Security rules

Do not weaken these defaults without an explicit security review:

- **Fail-closed by default.** If the bridge is unreachable, the response is
  unparsable, or there is no active session when a hook fires, the command is
  BLOCKED. This applies to *unexpected* failures only.
- **The only escapes are explicit:** `AG_ALLOW_ON_FAILURE=1` (opt back into
  fail-open) and the deliberate-stop sentinel (`current.closed`) that lets the
  command *after* a clean `actionguard stop` run. Never add new escape paths.
- **Approval timeout denies.** A stalled approval gate must not default to allow.
- **Policy parsing failures must never silently become ALLOW.**
- **Secrets must not be written to logs.**
- **Sensitive paths must not be exposed unnecessarily.**

## Current v0.2 scope

Action categories (`ActionCategory` in `src-tauri/src/models.rs`):

- File, Shell, Git, Package, Secret

Do not add Browser, Network, API/SaaS, Finance, or other new action classes
unless the task explicitly requires it.

## Windows / PowerShell

- **PowerShell interactive (PSReadLine)** is enforced **only while an active
  protected session exists** (Phase C, verified 2026-08-21).
- **PowerShell scripts, `-Command`, and piped stdin** are **observe-only** in
  v0.2 (known bypass, verified 2026-08-21).
- Do not change these enforcement statuses without a real end-to-end test.

## Testing

- Before committing Rust changes: `cargo test`.
- Before changing boundary logic, run the relevant boundary tests.
- For security-sensitive changes, update:
  - `SECURITY_TEST_MATRIX.md`
  - `BOUNDARIES.md`
  - `CHANGELOG.md`
- Do not mark a boundary as verified without reproducible evidence.

## Generated / do not commit

Do not modify or commit:

- `node_modules/`, `dist/`, `src-tauri/target/`
- local logs and local ActionGuard state
- secrets or credentials

## Code style

- Prefer small, deterministic changes.
- Do not introduce LLM/AI-based risk classification into the core policy
  engine unless explicitly requested.
- Avoid speculative refactors.
- Do not change public safety semantics just to simplify implementation.
- A rule change that alters policy defaults must update the rule tests and the
  affected registry/matrix entries.

## Definition of done

A change is not complete until:

1. the implementation is correct,
2. relevant tests pass,
3. security semantics remain accurate,
4. documentation matches actual behavior,
5. verification status is updated when applicable.
