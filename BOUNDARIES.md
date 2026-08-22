# AI Automation Action Boundary Map

> ActionGuard is a **user-controlled safety boundary** for AI-powered automation.
> This is the **Action Boundary Map**: the vendor-neutral answer to the question
> **"Which Boundary Class does this automation expose?"** — *not* "which brands
> do we support".

ActionGuard attaches to automation by **boundary**, not by brand. A new product
is classified in minutes by asking one question:

> **Where does this automation expose an Action boundary we can reliably
> intercept?**

The six Boundary Classes below cover every mainstream AI automation
architecture we have surveyed. `src-tauri/src/models.rs` `BoundaryKind` is the
canonical mirror of these classes; `boundary.rs` `registry()` is the runtime
mirror of this file.

This file is a **research asset, not a feature matrix**. Every entry records the
six verifiable facts plus the date they were last true, and any verified claim
must be reproducible by `boundary test` (or a documented equivalent), with an
`action_id` from the ledger as evidence.

```bash
actionguard boundary list    # registry + local probes on this machine
actionguard boundary test    # non-destructive verification of each boundary
```

---

## Boundary Classes (A–F)

| Class | Name | What it means | Representative | ActionGuard stance |
|---|---|---|---|---|
| **A** | Tool Hook | Pre-action hook *inside* the automation tool; gives a complete Action before it runs | CodeBuddy `PreToolUse` | **Best.** Direct enforce. First official verified adapter. |
| **B** | Exec Approval | The automation defines its own execution boundary (exec policy / allowlist / host approvals) | OpenClaw `exec` approvals, Manus Desktop per-command approval | Research: attach as an independent, vendor-neutral policy layer *above* it. |
| **C** | Protected Shell | Only commands that go through the shell-hook path are controllable | Bash/Zsh/Fish preexec, PowerShell PSReadLine | Enforce where the hook is live; document bypass paths. |
| **D** | Runtime Sandbox | Lower-level runtime/process control; can govern more real resources | sandbox runtimes | Future L3 target — prefer adopting, not re-inventing. |
| **E** | System Enforcement | OS-level enforcement; independent of automation cooperation | Windows / macOS / Linux kernel & security APIs | Long-term endgame (L4). Not now. |
| **F** | Remote | Actions never land on the user's machine | Manus Cloud, remote browser, cloud workers | **Out of scope locally** — address-space limit, not a product choice. |

Mapping rule: **Tool Hook > Exec Approval > Protected Shell > Runtime Sandbox >
System Enforcement > Remote** — pick the first class that matches how the
automation actually executes.

---

## Status at a glance

| Automation | Status | Last verified |
|---|---|---|
| CodeBuddy | ✅ Core Verified | 2026-08-19 |
| Claude Code | ? Not verified | 2026-08-19 |
| Codex | ? Not verified | 2026-08-19 |
| Cursor | ? Not verified | 2026-08-19 |
| Windsurf | ? Not verified | 2026-08-19 |
| OpenClaw | ? Investigating | — |
| Manus Desktop | ? Investigating | 2026-08-19 |
| Manus Cloud | N/A Remote | 2026-08-19 |
| PowerShell — interactive | ✅ Core Verified (Phase C) | 2026-08-21 |
| PowerShell — scripts / `-Command` / piped | ⚠ Observe-only (known bypass) | 2026-08-21 |

- **Core Verified** — adapter attached, a policy denial proven by `boundary test` with ledger evidence.
- **Community Verified** — proven by a community contribution (PR) that passed the boundary test standard below; `actionguard boundary list` shows the contributor.
- **Not verified** — boundary researched (documented or binary-probed) but no ActionGuard enforcement proven.
- **Investigating** — active research, no conclusion yet.
- **N/A Remote** — actions never land on this machine; out of scope locally.
- **Observe-only** — hook covers only part of the execution path (known bypass).

## Community verification — the boundary test standard

ActionGuard does **not** chase every automation product. We maintain the
standard; the community verifies specific products. Anyone can contribute a
boundary claim, but a claim is not accepted on trust. Every contribution must
include reproducible evidence and gets a PR review before it is marked
**Community Verified**.

PR template (contribute to `boundaries/<product>.yml` + evidence):

```markdown
Automation:   Codex
Version:      x.x.x
OS:           Windows 11
Boundary:     Exec Approval (B) — built-in approval_policy
Test:         rm -rf ./actionguard-test
Expected:     DENY
Actual:       BLOCKED
Evidence:     test script + raw terminal output + action_id from the ledger
```

Rules of the standard:

1. **Measured, not assumed.** The `Actual` line must come from a real run, and
   the evidence must include enough output to reproduce it.
2. **One boundary per row.** A contribution proves one boundary class on one
   OS/version. A second OS is a second contribution.
3. **ActionGuard version + automation version + OS are mandatory.** Boundary
   behavior changes with every release.
4. **The maintainer reviews and merges.** After merge the row is marked
   `verification: community` with `contributor: @handle`; `actionguard boundary
   list` then shows `✓ Community Verified` + the contributor.
5. **Core Verified stays separate.** Only the ActionGuard maintainers can mark
   `core`, always with a live `boundary test` + ledger evidence in `note`.

This is the same "by boundary, not by brand" principle applied to the
community: the standard is about the boundary, never about the brand.

---

## Quick table

| Automation | Class | Local actions | Action boundary | Enforcement | Last verified |
|---|---|---|---|---|---|
| CodeBuddy | **A** Tool Hook | ✅ | `PreToolUse` hook | **Enforced** | 2026-08-19 |
| OpenClaw | **B** Exec Approval | ✅ | `exec` policy + host approvals (canonical systemRunPlan) | TBD — investigate adapter | — |
| Manus Desktop (My Computer) | **B** Exec Approval | ✅ | native per-command approval | TBD (probe planned) | 2026-08-19 |
| Protected Shell (bash/zsh/fish) | **C** Protected Shell | ✅ | preexec hook | **Enforced** (interactive) | 2026-08-19 |
| PowerShell — interactive (PSReadLine) | **C** Protected Shell | ✅ | interactive line hook (Phase C) | **Enforced** — requires active session | 2026-08-21 |
| PowerShell — scripts / `-Command` / piped stdin | **C** Protected Shell | ✅ | no hookable boundary | Observe-only (known bypass) | 2026-08-21 |
| Claude Code | **A** Tool Hook (documented) | ✅ | official `PreToolUse`/`PostToolUse` hooks (settings.json), exit 2 = deny | Not installed here — documented only | 2026-08-19 |
| Codex | **B** Exec Approval | ✅ | built-in `approval_policy` — **no third-party hook protocol** | N/A via hook | 2026-08-19 |
| Cursor | **A** Tool Hook (probed) | ✅ | official `hooks.json` — `beforeShellExecution` / `preToolUse` | Not configured (defaults fail-open) | 2026-08-19 |
| Windsurf | **A** Tool Hook (documented) | ✅ | official Cascade Hooks (pre/post; docs under Devin) | Not installed here — documented only | 2026-08-19 |
| OpenCode | **A** unverified | ✅ | needs a real pre-action hook probe | TBD | — |
| Manus Cloud | **F** Remote | ❌ | remote sandbox | N/A from this machine | 2026-08-19 |

The `Last verified` column is what the compatibility matrix actually needs —
**Boundary class / Enforcement status / Last tested**. We never write "supports
Codex"; we write what was measured.

---

## Per-automation details

### CodeBuddy — Class A (Tool Hook) — first verified official adapter

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **YES** — Bash / shell via agent |
| 2 | Where does the action enter? | `PreToolUse` hook (stdin JSON → adapter → engine) |
| 3 | Is there a pre-action boundary? | **YES** — `PreToolUse` hook is pre-execution |
| 4 | Can ActionGuard observe it? | YES — ledger + hook adapter log |
| 5 | Can ActionGuard enforce it? | **YES** — verified: `sudo rm -rf /` blocked pre-execution (deny-sudo), 2026-08-19 |
| 6 | Last verified? | 2026-08-19 |

Caveat: only the hook path is enforced. CodeBuddy spawning its own subprocess
(`powershell -Command …`) is **observe-only** — same vendor, two paths.

### OpenClaw — Class B (Exec Approval) — research target

OpenClaw is **not** "an automation without a boundary". It has a mature
execution boundary of its own:

- `exec` can run in **sandbox / gateway / node** — gateway/node are real host
  execution.
- Host execution policy: **deny / allowlist / full**; approval modes
  **ask = off / on-miss / always**; optional human approval.
- An approval request is bound to a **canonical `systemRunPlan`** — if
  `command`, `cwd`, `agentId`, `sessionKey`, etc. change afterwards, the
  request is rejected as an **approval mismatch**.

This is close to the "Action Evidence" model ActionGuard wants. The point of
attaching is **not** "help OpenClaw block `rm`" — it is to stand at the
`policy/approval → actual execution` step as a **higher, vendor-neutral policy
layer**:

```
OpenClaw exec → ActionGuard Adapter → Policy → OpenClaw approval → OS
```

OpenClaw's own rules solve *OpenClaw's safety*. ActionGuard solves *one
consistent safety policy across every automation source* ("no AI automation may
read `.env`" — CodeBuddy, OpenClaw, Manus all pass through the same engine).

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **YES** — via `exec` (gateway/node host execution) |
| 2 | Where does the action enter? | `tools.exec.*` → exec policy → host approvals |
| 3 | Is there a pre-action boundary? | **YES** — its own (policy + approval + canonical systemRunPlan binding) |
| 4 | Can ActionGuard observe it? | TBD — an independent ActionGuard boundary is not yet verified |
| 5 | Can ActionGuard enforce it? | **TBD** — candidate for an independent policy layer above `exec` approval |
| 6 | Last verified? | — |

### Manus Desktop (My Computer) — Class B (Exec Approval)

Manus Desktop's *My Computer* can execute terminal commands locally, read and
write local files, and launch applications. Per the vendor: every local
terminal command currently requires explicit user approval
(`Allow Once` / `Always Allow`), and only user-authorized folders are
accessible.

Manus Desktop is **local desktop automation**; Manus Cloud is **remote** — the
two are different Boundary Classes and must not be conflated.

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **YES** — CLI, file read/write, app launch |
| 2 | Where does the action enter? | Local terminal behind native per-command approval |
| 3 | Is there a pre-action boundary? | YES (vendor's own approval). Is it hookable by ActionGuard? **TBD** |
| 4 | Can ActionGuard observe it? | Partially — side-effect file watcher sees writes; terminal capture TBD |
| 5 | Can ActionGuard enforce it? | **TBD** — ActionGuard as a second, user-owned layer behind Manus's own approval |
| 6 | Last verified? | 2026-08-19 (registry entry; no live test yet) |

Why it matters: Manus proving *"every terminal command needs approval"*
validates the **action-level approval** model — and creates the second-layer
question ActionGuard answers: *whose policy wins?* Manus asks "ok to run this
command?"; ActionGuard is the user's own layer that can say no *even after*
Manus approves (e.g. "never touch `.env`", "deleting > 5 files needs a human").

### Claude Code — Class A (Tool Hook, documented)

Probed 2026-08-19 from official documentation. The tool is **not installed on
this machine**, so the boundary is documented, not measured.

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **YES** — local CLI |
| 2 | Where does the action enter? | Tool calls via official hooks (`PreToolUse` / `PostToolUse`) in `settings.json` |
| 3 | Is there a pre-action boundary? | **YES** — `PreToolUse` fires before the tool runs; exit 2 denies the action (same protocol as the verified CodeBuddy adapter) |
| 4 | Can ActionGuard observe it? | Not verified here — not installed on this machine |
| 5 | Can ActionGuard enforce it? | Not verified — hook protocol is identical to CodeBuddy's verified one; adapter would be reusable, none written yet |
| 6 | Last verified? | 2026-08-19 (documentation research; no live probe) |

### Codex — Class B (Exec Approval, documented)

Probed 2026-08-19 from official documentation. The tool is **not installed on
this machine**, so the boundary is documented, not measured. Key finding:
Codex has **no third-party pre-tool hook protocol** — its approval mechanism is
built-in and not extensible by ActionGuard.

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **YES** — local CLI |
| 2 | Where does the action enter? | Built-in `approval_policy` (`~/.codex/config.toml`: untrusted / on-failure / never) |
| 3 | Is there a pre-action boundary? | Native approval is pre-action but **not extensible** by ActionGuard |
| 4 | Can ActionGuard observe it? | Only via a protected shell, if Codex happens to run under one |
| 5 | Can ActionGuard enforce it? | **No hook point** — ActionGuard cannot attach as a pre-action layer |
| 6 | Last verified? | 2026-08-19 (documentation research; no live probe) |

### Cursor — Class A (Tool Hook, probed on this machine)

Real probe on 2026-08-19: Cursor **3.11.13** installed on this machine
(`D:\cursor` + `%LOCALAPPDATA%\Programs\cursor`). The boundary mechanism is
confirmed in the installed binary (`beforeShellExecution` strings); no
ActionGuard hook is attached yet.

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **YES** — installed 3.11.13 on this machine |
| 2 | Where does the action enter? | Official `hooks.json` — `beforeShellExecution`, `preToolUse`, `beforeMCPExecution`, `beforeReadFile` (user `~/.cursor/hooks.json`; project `.cursor/hooks.json`) |
| 3 | Is there a pre-action boundary? | **YES** — `beforeShellExecution` / `preToolUse` can `permission: deny` (exit 2, Claude Code-compatible) or rewrite inputs; `failClosed` option |
| 4 | Can ActionGuard observe it? | Mechanism confirmed (binary probe); no ActionGuard hook attached, so nothing is observed yet |
| 5 | Can ActionGuard enforce it? | **Not configured** — `~/.cursor/hooks.json` absent; defaults **fail-open** |
| 6 | Last verified? | 2026-08-19 (install + binary probe; no hook attached) |

### Windsurf — Class A (Tool Hook, documented)

Probed 2026-08-19 from official documentation. The tool is **not installed on
this machine**, so the boundary is documented, not measured. Note: Windsurf has
merged into Devin — its Cascade Hooks docs now live under docs.devin.ai.

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **YES** — IDE/CLI |
| 2 | Where does the action enter? | Official Cascade Hooks (pre/post) covering command execution |
| 3 | Is there a pre-action boundary? | **YES** — pre hooks fire before command execution |
| 4 | Can ActionGuard observe it? | Not verified — not installed on this machine |
| 5 | Can ActionGuard enforce it? | Not verified — adapter only if a stable pre-action hook protocol is confirmed |
| 6 | Last verified? | 2026-08-19 (documentation research; no live probe) |

### OpenCode — Class A (IDE automation, unverified)

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **YES** |
| 2 | Where does the action enter? | IDE terminal / tool execution — hook presence TBD |
| 3 | Is there a pre-action boundary? | TBD — needs a real hook probe, not an assumption |
| 4 | Can ActionGuard observe it? | Child-process / file-watcher observation only, if no hook |
| 5 | Can ActionGuard enforce it? | TBD (adapter only if a pre-action hook exists) |
| 6 | Last verified? | — |

### Protected Shell (bash / zsh / fish) — Class C (Protected Shell)

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | YES (interactive shells) |
| 2 | Where does the action enter? | `preexec` hook → bridge `POST /preexec` |
| 3 | Is there a pre-action boundary? | **YES** — before execution |
| 4 | Can ActionGuard observe it? | YES |
| 5 | Can ActionGuard enforce it? | **YES** — deny blocks before exec |
| 6 | Last verified? | 2026-08-19 |

### PowerShell — Class C (Protected Shell, split by execution path)

Enforcement differs per execution path, so PowerShell is one product with **two
registry entries** (mirrored by `boundary list` / `capabilities` / `doctor`):

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | YES |
| 2 | Where does the action enter? | Interactive: PSReadLine Enter handler → bridge `/preexec`. Scripts / `-Command` / piped stdin: no hookable boundary |
| 3 | Is there a pre-action boundary? | Interactive: **YES** (Phase C). Scripts / `-Command` / piped: **NO** |
| 4 | Can ActionGuard observe it? | YES (both paths) |
| 5 | Can ActionGuard enforce it? | Interactive: **YES** — verified 2026-08-21 (denied command reverted, exit 126, marker survived). Scripts / `-Command` / piped: **Observe-only** — verified 2026-08-21 (same command executed via `-Command`). Enforcement of interactive lines requires an active protected session |
| 6 | Last verified? | 2026-08-21 (scripts/tests/verify-powershell-phase-c.ps1) |

### Manus Cloud — Class F (Remote)

| # | Question | Answer |
|---|---|---|
| 1 | Does it execute locally? | **NO** — remote sandbox |
| 2 | Where does the action enter? | Remote environment |
| 3 | Is there a pre-action boundary? | Remote approval — not reachable from this machine |
| 4 | Can ActionGuard observe it? | N/A |
| 5 | Can ActionGuard enforce it? | N/A |
| 6 | Last verified? | 2026-08-19 |

Not being able to control Manus Cloud is **not a product defect** — it is an
address-space boundary. The local ActionGuard cannot see actions that never
land on the machine.

---

## How to classify a new product (10 minutes)

1. **Remote-only execution?** → Class **F**. Done — record and move on.
2. **Pre-action hook / middleware in the tool?** → Class **A**. Best outcome.
3. **Automation has its own exec policy / approvals?** → Class **B** — research
   an independent policy layer, don't re-invent its approval.
4. **Only a shell path we control?** → Class **C** — enforce on that path.
5. **Runtime/sandbox control available?** → Class **D** — future L3.
6. **Everything else** → record as *unverified*; the class column stays a
   hypothesis until `boundary test` proves it.

## How to update this map

1. **Measure, don't assume.** Add/update an entry only when you have a
   reproducible probe or a documented test with ledger evidence
   (`action_id` + timestamp). Never write "supports X" — write the measured
   class / status / date.
2. PR the table **and** the per-automation details. Change the `Last verified`
   date; do not keep stale claims.
3. Mirrored at runtime — update `src-tauri/src/boundary.rs` `registry()` when
   you change this file, so `actionguard boundary list` agrees.
4. Community-driven: the compatibility matrix is a shared asset — anyone can PR
   a probe result for a product they actually run.

The map is a research asset first: *"what did this AI automation actually get
access to on the machine?"* — even for products ActionGuard does not yet
enforce.
