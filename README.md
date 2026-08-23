# ActionGuard

> **AI can act. You set the boundary.**
>
> **The agent should not be the final authority over its own actions.**

ActionGuard is a **local, user-controlled safety boundary for AI-powered automation**. It evaluates consequential actions — file changes, shell commands, Git operations, package installs, secret access — and, **on supported boundaries**, can enforce policy **before** they affect your machine or other protected resources.

It does not watch a brand. It watches the **boundary**: the point where an automation's action enters your system. CodeBuddy, Claude Code, OpenClaw, Manus, Codex — they are all just *action sources*. **An action source is only metadata; the boundary is the security primitive.**

ActionGuard is built around a simple principle: **the user should have an independent safety boundary around autonomous actions, regardless of which agent or automation tool produced them.**

> **Attach by boundary, not by brand.**

> **Honesty principle**: this README only claims capabilities that are **verified today** — measured on a real machine, recorded in the [Security Test Matrix](./SECURITY_TEST_MATRIX.md). Nothing "planned" is presented as "done".

> **Detection never implies enforcement.** "We logged it" is not "we stopped it". Every claim in the protection matrix below states which of the two is true for each path.

---

## 📋 Contents

- [What is ActionGuard?](#what-is-actionguard)
- [Why it exists](#why-it-exists)
- [Security posture](#security-posture)
- [10-second demo](#10-second-demo)
- [Current protection matrix](#current-protection-matrix)
- [Quick Start](#quick-start)
- [How it works](#how-it-works)
- [Known limitations](#known-limitations)
- [Contributing](#contributing)

---

## What is ActionGuard?

AI automation is now authorized to change your files, run your shell, install packages, and touch credentials. The tools trust the model. ActionGuard is the layer that does **not** — a deterministic policy engine sitting at the action boundary that says *yes*, *no*, or *ask a human* before anything consequential runs.

```
AI automation wants to run:

    sudo rm -rf /

ActionGuard

    CRITICAL
    DENY

  ✓ Policy matched
  ✓ Action blocked
  ✓ Evidence recorded
```

ActionGuard is built around an **extensible boundary model**, not vendor-specific integrations. Different automation systems may expose different enforcement points — tool hooks, execution approval layers, protected runtimes, and system-level boundaries. Today it works with **AI coding tools and any other automation system that exposes a supported action boundary** — it is not limited to one brand or one class of agent.

AI-powered automation is moving from *suggesting* actions to *executing* them. As agents gain access to shells, files, repositories, packages, and local applications, the security boundary around those actions becomes increasingly important.

Current scope (**v0.2**): **File, Shell, Git, Package, Secret** actions on your local machine. That's it. Browser, Network, API/SaaS, and remote automation are deliberately **not** in v0.2.

---

## Why it exists

- **The action boundary is the attack surface.** AI-powered automation is given `execute` permission by default, and nothing verifies what it actually does. Terminal execution is the first boundary ActionGuard protects in v0.2.
- **Detection is not protection.** "We logged it" is not "we stopped it". ActionGuard distinguishes the two on every path it supports — see the [matrix](#current-protection-matrix).
- **Security fragmentation is a safety gap.** As users adopt multiple automation tools, their security controls become fragmented across vendor-specific policies and approval systems — and none of them talk to each other. ActionGuard provides one independent policy layer that spans boundaries, instead of asking the user to trust N vendor-specific ones.
- **Vendor-specific controls create inconsistent safety boundaries.** Protection that works for one agent but not another leaves a gap. Because ActionGuard attaches to *boundaries* rather than brands, any tool exposing a hookable pre-action boundary can be enforced — and anything that can't is labeled explicitly, never implied.
- **Deterministic, not "AI-flavored."** No black-box model decides what is dangerous. Risk classification runs on explicit rules you can read and edit.
- **Honesty is a feature.** A safety product that overclaims makes users *less* safe. Every enforcement claim in this document is backed by a test record.

---

## Security posture

- **Local-first** — no telemetry, no account required. Everything stays on your machine.
- **Fail-closed by default** — every enforcement point denies when the policy engine is unreachable, the response is unparseable, or there is no active session. A cleanly-closed session is the one deliberate exception (the terminal is never bricked), and `AG_ALLOW_ON_FAILURE=1` explicitly opts back into fail-open.
- **Open verification** — enforcement claims are backed by reproducible tests in [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md), not by marketing.

---

## 10-second demo

Dry run first — decide without executing:

```bash
actionguard policy-check "git reset --hard HEAD~1" --explain
```

```text
Decision:    ASK
Risk:        HIGH
Rule:        git-reset-hard
Reason:      destructive repository rewrite
Boundary:    ProtectedShell
Mode:        DRY RUN
```

Then see real enforcement — inside a protected session the same command never runs:

```bash
actionguard protect ./my-project
# agent runs a destructive command, e.g.  sudo rm -rf /
```

```text
→ DENY
→ ENFORCED
→ command not executed
```

`policy-check` never executes anything. Enforcement only happens inside an active protected session.

---

## Current protection matrix

> Status measured on **2026-08-21 against ActionGuard v0.2**; each row keeps its own `Last verified`. `Enforced` = the action is gated **before** it executes. `Observe-only` = it is recorded, but not blocked. This matrix is also surfaced live on your machine — run `actionguard boundary list` or `actionguard capabilities`. Full registry: [BOUNDARIES.md](./BOUNDARIES.md).

| Boundary | Observe | Enforced | Verification | Last verified |
|---|---|---|---|---|
| CodeBuddy PreToolUse hook | ✅ | ✅ | **Core verified** — real `sudo rm -rf /` denied before execution | 2026-08-19 |
| Protected shell — bash | ✅ | ✅ | Core verified | 2026-08-19 |
| Protected shell — zsh | ✅ | ✅ | Core verified | 2026-08-19 |
| Protected shell — fish | ✅ | ✅ | Core verified | 2026-08-19 |
| PowerShell — interactive (PSReadLine) | ✅ | ✅ | **Phase C** — block + exit 126, marker survived; requires an active protected session | 2026-08-21 |
| PowerShell — scripts / `-Command` / piped stdin | ✅ | ❌ | **Observe-only** — bypass verified (marker deleted) | 2026-08-21 |
| Direct subprocess spawn (`os.system`, absolute paths) | ✅ | ❌ | **Observe-only** — known bypass | 2026-08-19 |
| Claude Code | ⏳ documented | ❓ | Documented — enforcement not verified | — |
| Cursor | ⏳ installed | ❓ | Documented — installed, not integrated | — |
| Codex | ❓ | ❓ | Documented — ExecApproval, not extensible, enforcement not verified | — |
| OpenClaw | ❓ | ❓ | Investigating — candidate independent policy layer (ExecApproval) | — |
| Manus Desktop (My Computer) | ❓ | ❓ | Investigating — candidate second policy layer (ExecApproval) | — |
| Manus Cloud | N/A | N/A | Remote — cannot be enforced by a local tool | — |

> Two layers of truth: the **Boundary Type** (where the action enters — tool hook, protected shell, exec approval, …) and the **Enforcement Status** (whether it can currently be blocked). Capability Tier details (L1–L4) live in [SECURITY_MODEL.md](./SECURITY_MODEL.md) — the README deliberately does not use them.

> **Honesty about bypasses.** ActionGuard only enforces actions that pass through a **supported boundary**. An action that bypasses a boundary is observed, recorded, and labeled **Bypassed / Unsupported** — never silently claimed as blocked. "We saw it but didn't stop it" is the correct, honest outcome for an unsupported path, not a product failure.

### Platform status

> **Build ≠ Verified.** CI compiles, tests, and runs the install lifecycle on all three platforms — but **enforcement claims are only made where a real-machine test record exists** (see [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md)). A green build on macOS or Linux is not a claim that those platforms are fully enforced.

| Platform | CI build | Lifecycle (setup → doctor → uninstall) | Real-machine enforcement verified |
|---|---|---|---|
| Windows | ✅ | ✅ | ✅ PowerShell interactive (Phase C), protected shells |
| Linux | ✅ | ✅ | ✅ Protected shells (bash/zsh/fish) |
| macOS | ✅ | ✅ | ⏳ **Build available — enforcement requires platform-specific verification** (Gatekeeper, permissions, shell hook behavior) |

macOS and Linux binaries are published so early adopters can help verify real enforcement. If you're on one of those platforms, a [boundary verification report](./CONTRIBUTING.md) is the most valuable contribution you can make.

---

## Quick Start

> **For non-Rust developers**: download the binary from **GitHub Releases** (no Rust toolchain needed). Releases ship with a `SHA256SUMS` file — verify before running, e.g. `sha256sum actionguard` or `Get-FileHash actionguard.exe -Algorithm SHA256`. Source build is below as a secondary path.

### 1. Install

**From GitHub Releases** — download the `actionguard` binary for your platform, verify its checksum, and add it to your `PATH`.

**From source**:

```bash
npm install
cargo build --release --bin actionguard
```

### 2. One-command setup

```bash
actionguard setup
```

Detects your OS and shell, previews every change, creates `~/.actionguard`, installs the built-in rule packs, installs the shell hook, and runs a self-check. **No root/admin privileges are required for the default local setup.**

### 3. Verify

```bash
actionguard doctor --test
```

Runs a non-destructive end-to-end boundary test and prints ✓/✗ per boundary — evidence, not promises.

### 4. Protect

```bash
actionguard protect ./my-project
```

Starts a protected session: high-risk actions now go through the policy engine and the approval gate (`allow` / `deny`) before they run.

### 5. Check a command without executing it

```bash
actionguard policy-check "git reset --hard HEAD~1" --explain
```

---

## How it works

```
  Action Source  (CodeBuddy, shell, script, agent …)
        │
        ▼
   Boundary      ← the entry point an action crosses (hook / shell / approval)
        │
        ▼
   ActionGuard Core
        ├── Classify    → category + risk (File / Shell / Git / Package / Secret)
        ├── Policy      → YAML rules → Allow / Ask / Deny
        ├── Approval    → ask a human (CLI or GUI); timeout defaults to deny
        └── Evidence    → append-only ledger, per-action record, stats
```

- **Observation and Enforcement are properties of the Boundary, not of the core.** A boundary is `Enforced` (blocks before execution), `Observe-only` (records after the fact), or `Not detected` — and the registry labels every boundary with exactly one.
- **Attach by boundary, not by brand.** The core engine never special-cases an agent brand. Adding a new automation source = mapping it to a boundary type, not integrating it.
- **Deterministic policy.** Decision values are `Allow`, `Ask`, `Deny`. `Warn` is not a decision — it is an annotation on a policy result, shown in the UI and ledger.
- **Fail-closed by default.** If the engine is unreachable, every enforcement point **blocks** rather than passes.

### Action categories (v0.2)

| Category | Examples | Risk engine |
|---|---|---|
| **File** | create / modify / delete / rename | sensitive paths, out-of-workspace writes |
| **Shell** | `rm`, `chmod`, `sudo`, `curl` | dangerous patterns, irreversible ops |
| **Git** | `reset --hard`, `clean -f`, force push | destructive refs |
| **Package** | `npm`, `pnpm`, `pip`, `cargo` | untrusted installs |
| **Secret** | `.env`, SSH keys, credentials | access or exfiltration |

These are the **current v0.2 action classes**. New classes (Browser, Network, API, Finance) are intentionally frozen while the boundary model is validated — not because we haven't thought of them.

Sensitive assets are **recognized** by the risk engine (home dir, `.git`, `~/.ssh`, `.env`); whether a change to them is actually **blocked** depends on the active boundary — see the [protection matrix](#current-protection-matrix). **Detection never implies enforcement.**

### Rules

Built-in rule packs (`shell.yml`, `git.yml`, `node.yml`, `python.yml`, `secrets.yml`) ship with the binary. Users can add YAML rules:

```yaml
id: risky-shell-operations
category: shell
severity: high
match:
  patterns:
    - "rm -rf /*"
    - ":(){ :|: & };:"
action:
  decision: ask
```

Manage with: `actionguard policy-list` · `policy-lint <file>` · `policy-edit` · `policy-path` · `rule search` · `rule install <file.yml>`.

### CLI at a glance

```bash
actionguard setup            # one-command install
actionguard doctor --test    # verify detected/enforceable boundaries
actionguard status           # session / hook state
actionguard protect <dir>    # protected session
actionguard policy-check <cmd> --explain
actionguard allow | deny [id]
actionguard stats | capabilities | boundary list
actionguard rule search <q> | rule install <file.yml>
```

The complete, generated command reference ships with the binary — run `actionguard --help`. This list is audited against `--help` (v0.2).

---

## Known limitations

These are real, measured limits — the same ones printed by `actionguard doctor`:

1. **PowerShell protection is interactive-only on Windows.** Interactive lines are enforced (Phase C — verified 2026-08-21 by `scripts/tests/verify-powershell-phase-c.ps1`); scripts, `-Command`, and piped stdin are observe-only. The registry models this as **two separate entries** (`PowerShell (PSReadLine interactive)` = enforced, `PowerShell (script/-Command/piped)` = observe-only), and `boundary list`, `capabilities`, and `doctor` all reflect the same split.
2. **Direct subprocesses can bypass shell hooks.** `os.system`, `/usr/bin/rm`, and other absolute-path or non-shell spawns are observed, not blocked.
3. **Remote automation is out of scope.** Actions executed on a different machine (e.g. Manus Cloud) cannot be enforced by a local tool.
4. **Only a few boundaries are verified.** CodeBuddy and the protected shells are Core verified. Cursor, Windsurf, OpenClaw, Codex, and Manus are documented or investigating — not yet enforced.
5. **No sandboxing yet.** Running actions in an isolated environment (container / network isolation) is future work.
6. **`undo` is not exposed on the CLI in v0.2.** The v0.1 snapshot/restore mechanism still exists behind the GUI flow; a CLI `undo` command is planned for v0.3.

---

## Contributing

Contributions are welcome — especially **boundary verification reports** (the asset this project runs on). See [CONTRIBUTING.md](./CONTRIBUTING.md) for report templates, rule format, code conventions, and security reporting. (中文版: [docs/CONTRIBUTING.md](./docs/CONTRIBUTING.md))

See also: [SECURITY_MODEL.md](./SECURITY_MODEL.md) · [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md) · [BOUNDARIES.md](./BOUNDARIES.md) · [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)

---

## License

[Apache-2.0](./LICENSE)

---

## Languages

- [English](./README.md)
- [简体中文](./README.zh.md)
