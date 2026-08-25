# ActionGuard

### Give AI room to work. Keep control of what it can do.

**Protect your machine before AI-powered automation acts.**

**Local. Deterministic. Vendor-neutral.** · No cloud · No SDK · No model in the decision path.

**Blocks high-impact file, shell, Git, package, and secret actions on supported boundaries.**

AI-powered automation can read your files, run your shell, rewrite your Git history, install packages, and touch your secrets. ActionGuard is an **independent, user-controlled safety boundary** between that automation and your machine: it evaluates consequential actions and says **Allow / Ask / Deny** — and on supported boundaries, it enforces that decision **before** anything happens.

```
            AI-powered automation
                      │
                      ▼
              ┌──────────────┐
              │  ActionGuard │
              │              │
              │ Allow / Ask  │
              │    / Deny    │
              └──────┬───────┘
                     │
                     ▼
                  Machine
```

**No SDK. No cloud. No model in the decision path.**

**We don't try to secure the AI. We control what it can do to your machine.**

```
$ actionguard protect

  ✓ Policy loaded
  ✓ Boundary detected
  ✓ Enforcement active

  AI-powered automation attempted:  sudo rm -rf /

  ✗ DENIED
```

<p align="center">
  <a href="#try-it-now"><strong>Try ActionGuard →</strong></a>
  &nbsp;·&nbsp;
  <a href="#verified-today">View verified boundaries</a>
  &nbsp;·&nbsp;
  <a href="https://github.com/SeanXChen/ActionGuard">GitHub</a>
</p>

> **The automation should not be the final authority over its own actions.**
>
> ActionGuard provides an independent, deterministic policy layer that sits at the action boundary — so the thing that *wants* to act is never the only thing that *decides* whether it may.

---

## 📋 Contents

- [See it work — on a real machine](#see-it-work--on-a-real-machine)
- [Who is it for?](#who-is-it-for)
- [Why you need it](#why-you-need-it)
- [Why not just use the automation's built-in permissions?](#why-not-just-use-the-automations-built-in-permissions)
- [What ActionGuard protects](#what-actionguard-protects)
- [Try it now](#try-it-now)
- [Verified today](#verified-today)
- [How it works](#how-it-works)
- [What ActionGuard can — and cannot — enforce today](#what-actionguard-can--and-cannot--enforce-today)
- [Give feedback](#give-feedback)
- [Using ActionGuard in your company?](#using-actionguard-in-your-company)
- [Contributing](#contributing)
- [License](#license)

---

## See it work — on a real machine

> **AI attempts a destructive action. ActionGuard evaluates it before execution.**

```
   CodeBuddy / Protected shell
                │
                ▼
      sudo rm -rf /
                │
                ▼
        ActionGuard
                │
                ▼
    APPROVAL REQUIRED  →  DENY
```

*This is the real path a verified boundary takes on your machine — not a mockup. The
CodeBuddy PreToolUse hook, protected shells (bash/zsh/fish), and interactive PowerShell
below are all measured in [Verified today](#verified-today) and reproducible with
`actionguard doctor --test`.*

## Who is it for?

Anyone who lets AI-powered automation act on their computer:

- **Developers** using Codex, Claude Code, Cursor, CodeBuddy and other coding agents.
- **AI power users** letting automation handle files, scripts, applications, or local workflows — no programming required.
- **Anyone running autonomous automation** who wants a safety boundary between the automation and their machine.

ActionGuard is **designed for people using AI-powered automation** — such as Codex, Claude Code, Cursor, OpenClaw and Manus. Its **current verified enforcement** is: CodeBuddy PreToolUse hook, protected shells (bash/zsh/fish), and interactive PowerShell on Windows. Everything else is documented or under research — see [Verified today](#verified-today). Nothing is implied that isn't measured.

---

## Why you need it

AI can already modify files, run commands, and work for hours without supervision. The problem isn't giving AI access. **The problem is giving it unrestricted authority.**

- **The action boundary is the attack surface.** AI-powered automation is given `execute` permission by default. Built-in permissions and sandboxes control *access* — but they do not necessarily provide an independent policy for every consequential action.
- **Detection is not protection.** "We logged it" is not "we stopped it". ActionGuard distinguishes the two on every path it supports.
- **Approval fatigue is real.** When every step asks for permission, people start clicking "allow" reflexively. ActionGuard asks only when an action actually crosses your safety boundary.
- **Security fragmentation is a safety gap.** Controls scattered across N vendor-specific approval systems don't talk to each other. ActionGuard provides one independent policy layer that spans boundaries.
- **Deterministic, not "AI-flavored."** No black-box model decides what is dangerous. Risk classification runs on explicit rules you can read and edit.
- **Honesty is a feature.** A safety product that overclaims makes users *less* safe. Every enforcement claim in this document is backed by a test record.

> **Let it run.** ActionGuard only interrupts when an action crosses your safety boundary — so you can let your AI work while you step away, instead of babysitting every step.

---

## Why not just use the automation's built-in permissions?

Fair question. Built-in controls answer *"what is the automation allowed to access?"* — ActionGuard answers a different question: *"what is the automation allowed to **do** with that access?"*

> **Sandbox controls where AI can go.**
> **ActionGuard controls what it can do there.**
>
> *Access is capability. Consequence is policy.* Having *access* to a resource does not entitle an automation to perform consequential *actions* on it — that decision belongs to policy, not to the automation.

Three examples — the automation has full access in all three:

```
Agent can access your project      →   git reset --hard
Agent can write to your workspace  →   rm .env
Agent can install packages         →   npm publish
```

Built-in permissions are set by the vendor, apply only inside that one product, and the vendor's own automation is the final judge of its own actions. ActionGuard is **independent** — it is not written by the same vendor, it does not run inside the same process, and it applies one policy across every automation you use. That independence is the entire point: the automation should not be the final authority over its own actions.

---

## What ActionGuard protects

| | What it gates | Risk engine watches for |
|---|---|---|
| **Files** | create / modify / delete / rename | sensitive paths, out-of-workspace writes |
| **Shell** | `rm`, `chmod`, `sudo`, `curl` | dangerous patterns, irreversible ops |
| **Git** | `reset --hard`, `clean -f`, force push | destructive refs |
| **Packages** | `npm`, `pnpm`, `pip`, `cargo` | untrusted installs |
| **Secrets** | `.env`, SSH keys, credentials | access or exfiltration |

These are the **current v0.2 action classes**, enforced locally and recorded in an append-only ledger. Browser, network, API, and remote automation are deliberately **not** in v0.2.

---

## Try it now

> **Local-first. No account. No cloud telemetry.** Everything stays on your machine — nothing phones home.

**30 seconds, no Rust toolchain required** — install from **GitHub Releases**:

1. **Download** the installer for your OS (`ActionGuard_0.2.0_x64-setup.exe` on Windows).
2. **Verify** it against the `SHA256SUMS` file in the same release.
3. **Install**, open the app, and click **Protect this computer**.

`winget install` is on the roadmap — see [`docs/WINGET.md`](./docs/WINGET.md).

**For developers — the CLI:**

```bash
actionguard setup           # one-command install: shell hook + rule packs + self-check
actionguard doctor --test   # non-destructive end-to-end test — prints ✓/✗ per boundary
actionguard protect ./my-project   # start a protected session
```

**For AI users — the GUI:**

Prefer a point-and-click start? Open ActionGuard → **Protect this computer**. One button, plain-language counters, no commands.

### See your first decision

Prefer to see a decision without executing anything?

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

`policy-check` never executes anything. Enforcement only happens inside an active protected session — where the same command is refused before it runs:

```text
→ DENY
→ ENFORCED
→ command not executed
```

That is the whole loop: an action arrives, ActionGuard decides, the ledger records it, nothing harmful runs. No root/admin privileges are required for the default local setup.

---

## Verified today

> **Measured on a real machine, not marketed.** Status verified **2026-08-21 against ActionGuard v0.2**; full per-boundary detail and dates live in [BOUNDARIES.md](./BOUNDARIES.md). Every claim is reproducible via `actionguard boundary test` and recorded in [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md).

| Boundary | Status |
|---|---|
| CodeBuddy PreToolUse hook | ✅ Enforced |
| Protected shell — bash / zsh / fish | ✅ Enforced |
| PowerShell — interactive (PSReadLine, Windows) | ✅ Enforced |
| PowerShell — scripts / `-Command` / piped | ⚠️ Observe-only |
| Direct subprocess spawn (`os.system`, absolute paths) | ⚠️ Observe-only |
| Claude Code | 🔬 Documented — not verified |
| Cursor | 🔬 Documented — not verified |
| Codex | 🔬 Documented — not verified |
| OpenClaw | 🔬 Investigating |
| Manus Desktop (My Computer) | 🔬 Investigating |
| Manus Cloud | N/A — remote, out of scope |

`Enforced` means the action is gated **before** it executes. `Observe-only` means it is recorded but not blocked — and it is labeled exactly that, never implied as protection. Platform status (Windows / Linux / macOS) is in [BOUNDARIES.md](./BOUNDARIES.md).

---

## Can you break ActionGuard?

Every "Verified today" line above is a claim — and claims are meant to be
tested. Find a **new** way to bypass an enforced boundary that is not already
documented in [`SECURITY_MODEL.md`](./SECURITY_MODEL.md)? Report it and you get
credited.

- Report **privately** via GitHub Private vulnerability reporting
  ([`SECURITY.md`](./SECURITY.md)) — never a public issue.
- Every accepted report is credited in the advisory and changelog, unless you
  prefer to stay anonymous.
- The honest list of known limitations (subprocess / absolute-path execution)
  is **not** a vulnerability — it is documented and observable with
  `actionguard doctor`. New, undocumented bypasses are in scope.

No bounty program yet — this is a small project. Credit, a changelog entry,
and a thank-you in the next release notes. If you want, we fix it together.

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
- **Attach by boundary, not by brand.** The core engine never special-cases an automation brand. Adding a new automation source = mapping it to a boundary type, not integrating it.

### Action boundary — the question to ask about any automation tool

Replace *"can ActionGuard integrate with tool X?"* with *"does tool X have an action boundary we can reliably intercept?"* — Yes → build an adapter. No → observe. Unstable/undocumented → observe now, enforce later via system-level enforcement. This is why the boundary registry is per **execution path**, not per tool brand.

- **Deterministic policy.** Decision values are `Allow`, `Ask`, `Deny`. `Warn` is not a decision — it is an annotation on a policy result, shown in the UI and ledger.
- **Fail-closed by default.** If the engine is unreachable, every enforcement point **blocks** rather than passes.

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

## What ActionGuard can — and cannot — enforce today

- **Local-first** — no telemetry, no account required. Everything stays on your machine.
- **Fail-closed by default** — every enforcement point denies when the policy engine is unreachable, the response is unparseable, or there is no active session. A cleanly-closed session is the one deliberate exception (the terminal is never bricked), and `AG_ALLOW_ON_FAILURE=1` explicitly opts back into fail-open.
- **Open verification** — enforcement claims are backed by reproducible tests in [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md), not by marketing.

These limits are real and measured — the same ones printed by `actionguard doctor`:

1. **PowerShell protection is interactive-only on Windows.** Interactive lines are enforced (Phase C — verified 2026-08-21); scripts, `-Command`, and piped stdin are observe-only.
2. **Direct subprocesses can bypass shell hooks.** `os.system`, `/usr/bin/rm`, and other absolute-path or non-shell spawns are observed, not blocked.
3. **Remote automation is out of scope.** Actions executed on a different machine (e.g. Manus Cloud) cannot be enforced by a local tool.
4. **Only a few boundaries are verified.** CodeBuddy and the protected shells are Core verified. Cursor, Claude Code, Codex, OpenClaw, and Manus are documented or investigating — not yet enforced.
5. **No sandboxing yet.** Running actions in an isolated environment (container / network isolation) is future work.
6. **`undo` is not exposed on the CLI in v0.2.** The v0.1 snapshot/restore mechanism still exists behind the GUI flow; a CLI `undo` command is planned for v0.3.

> **Honesty about bypasses.** ActionGuard only enforces actions that pass through a **supported boundary**. An action that bypasses a boundary is observed, recorded, and labeled **Bypassed / Unsupported** — never silently claimed as blocked. "We saw it but didn't stop it" is the correct, honest outcome for an unsupported path, not a product failure.

---

## Give feedback

ActionGuard is local-first: **no telemetry** is built into the product. Instead, we listen actively — and what we need most is your real experience.

- 🛡 **Did it catch something?** Tell us about your first interception — what got blocked and what you thought: [open the feedback form](https://github.com/SeanXChen/ActionGuard/issues/new?template=feedback.yml) (takes 1 minute).
- 🐞 **Did something fail to install, start, or protect?** [File a bug report](https://github.com/SeanXChen/ActionGuard/issues/new?template=bug_report.yml) — it helps us find where users fall off.
- 💬 Prefer open conversation? [Start a discussion](https://github.com/SeanXChen/ActionGuard/discussions).

We track these signals in [docs/USER_VALIDATION.md](./docs/USER_VALIDATION.md).

---

## Using ActionGuard in your company?

ActionGuard is local-first, but we're actively exploring **team deployment and enterprise use cases** — shared policy, audit-friendly exports, managed rule packs.

If *"can this work for a team?"* is a question you're asking, we'd like to hear it: [start a discussion](https://github.com/SeanXChen/ActionGuard/discussions) or [file an enterprise-inquiry issue](https://github.com/SeanXChen/ActionGuard/issues/new).

---

## Contributing

Contributions are welcome — especially **boundary discovery** (a dangerous automation action ActionGuard doesn't handle yet — no YAML required) and **boundary verification reports** (the asset this project runs on). See [CONTRIBUTING.md](./CONTRIBUTING.md) for report templates, rule format, code conventions, and security reporting. (中文版: [docs/CONTRIBUTING.md](./docs/CONTRIBUTING.md))

See also: [SECURITY_MODEL.md](./SECURITY_MODEL.md) · [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md) · [BOUNDARIES.md](./BOUNDARIES.md) · [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) · [docs/FACTS_SCHEMA.md](./docs/FACTS_SCHEMA.md) · [docs/BOUNDARY_BACKLOG.md](./docs/BOUNDARY_BACKLOG.md)

---

## License

[Apache-2.0](./LICENSE)

---

## Languages

- [English](./README.md)
- [简体中文](./README.zh.md)
