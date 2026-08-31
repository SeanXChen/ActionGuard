# ActionGuard

### Give AI room to work. Keep control of what it can do.

**Local. Deterministic. Vendor-neutral.** · No cloud · No SDK · No model in the decision path.

---

## Why does AI automation need a safety boundary?

AI used to *answer*.
Now it can *act*.

You ask it to help with a task. It reads your files. Edits your code.
Runs commands. Installs packages. Changes Git state. Operates your computer.

The more useful AI becomes, the more authority it receives.

**The problem isn't that AI is malicious.**
**The problem is that AI is increasingly authorized to act.**

A useful agent may make a wrong assumption.
A prompt may be misunderstood.
A tool may behave unexpectedly.
A workflow may go further than you intended.

When AI can only talk, the cost is usually a bad answer.
When AI can act, the cost can be a changed file, a deleted directory,
a leaked secret, or an irreversible operation.

### The automation should not be the final authority over its own actions.

Your AI tools have their own permission models. But those models are
**written by the same vendor** that makes the automation — and the
automation itself is the final judge of what it may do.

ActionGuard adds an independent decision layer:

```
          AI Agent
               │
         "I want to do X"
               │
      Agent's own permissions
               │
               ▼
      ┌────────────────────┐
      │    ActionGuard     │
      │                    │
      │   Allow / Ask /    │
      │       Deny         │
      └──────────┬─────────┘
                  │
                  ▼
               Your OS
```

The automation decides what it *wants* to do.
**You decide what it is *allowed* to do.**

---

## Why not just use the automation's built-in permissions?

They help. But they answer a different question.

| | Built-in permissions | ActionGuard |
|---|---|---|
| **Who sets the rules?** | The vendor of that AI tool | You |
| **Scope** | Only inside that one product | All AI tools you use |
| **Final judge of the AI's own actions?** | Yes — the AI itself | No — independent layer |

> **Sandbox controls where AI can go.**
> **ActionGuard controls what it can do there.**

```
Agent can access your project      →   git reset --hard
Agent can write to your workspace  →   rm .env
Agent can install packages         →   npm publish
```

These three things can all be true at once:
- The AI has *access* to your project.
- Built-in permissions are configured.
- You still don't want the AI to do any of the above without asking.

**Access is capability. Consequence is policy.**
Having *access* to a resource does not entitle an automation to perform
consequential *actions* on it.

And your AI tools will change. Your safety policy shouldn't have to.

```
        Your policy
              │
              ▼
       ActionGuard
              │
     ┌────────┼────────┬─────────┐
     ▼        ▼        ▼         ▼
   Codex   Cursor   Claude   OpenClaw
```

---

## What ActionGuard lets you define

| AI wants to… | ActionGuard can… |
|---|---|
| Read ordinary project files | Allow |
| Modify files | Allow / Ask |
| Run a high-impact command | Ask |
| Access protected locations | Deny |
| Perform destructive operations | Deny |
| Act outside a verified boundary | Observe / block |

**ActionGuard lets AI work — without giving it unlimited authority.**

---

## Verified today

> **Measured on a real machine, not marketed.** Status verified **2026-08-26 against ActionGuard v0.3**; full per-boundary detail and dates live in [BOUNDARIES.md](./BOUNDARIES.md). Every claim is reproducible via `actionguard boundary test` and recorded in [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md).

| Boundary | Status | Setup required |
|---|---|---|
| CodeBuddy PreToolUse hook | ✅ Enforced | Automatic — via `actionguard setup` |
| Cursor `beforeShellExecution` hook | 🔬 Adapter available — not independently verified | Install via `actionguard setup` (adapter exists; reproducible boundary test + ledger evidence not yet recorded) |
| Protected shell — bash / zsh / fish | ✅ Enforced | Automatic — via `actionguard setup` |
| PowerShell — interactive (PSReadLine, Windows) | ✅ Enforced | Automatic — via `actionguard setup` |
| PowerShell — scripts / `-Command` / piped | ⚠️ Observe-only | — |
| Direct subprocess spawn (`os.system`, absolute paths) | ⚠️ Observe-only | — |
| Claude Code `PreToolUse` hook | 🔬 Documented | Requires `~/.claude/settings.json` hook config |
| Codex | 🔬 Investigating | — |
| OpenClaw | 🔬 Investigating | — |
| Manus Desktop (My Computer) | 🔬 Investigating | — |
| Manus Cloud | N/A — remote | Out of scope |

`Enforced` means the action is gated **before** it executes. `Observe-only` means it is recorded but not blocked — and it is labeled exactly that, never implied as protection.

> **Honesty about coverage.** ActionGuard does not claim to protect every execution path. It shows exactly where protection is enforced, where actions are observed, and where coverage is still being investigated. An action that bypasses a supported boundary is labeled **Bypassed / Unsupported** — never silently claimed as blocked.

---

## Try it now

> **Local-first. No account. No cloud telemetry.** Everything stays on your machine — nothing phones home.

**30 seconds, no Rust toolchain required** — install from **GitHub Releases**:

1. **Download** the installer for your OS (`ActionGuard_0.3.0_x64-setup.exe` on Windows).
2. **Verify** it against the `SHA256SUMS` file in the same release.
3. **Install**, open the app, and click **Protect this computer**.

`winget install` is on the roadmap — see [`docs/WINGET.md`](./docs/WINGET.md).

**For developers — the CLI:**

```bash
actionguard setup              # one-command install: shell + AI tool hooks + self-check
actionguard doctor --test      # non-destructive end-to-end test — prints ✓/✗ per boundary
actionguard protect ./my-project  # start a protected session
```

`actionguard setup` automatically detects and configures every supported boundary on this machine:
- Shell hooks (bash / zsh / fish / PowerShell)
- CodeBuddy PreToolUse hook
- Cursor `beforeShellExecution` hook (if Cursor is installed)
- Claude Code `PreToolUse` hook (if Claude Code is installed)

Running `actionguard setup` again is safe — it detects what is already configured and only adds what's missing.

**Coverage Ladder — no need to know which AI tool you're using:**

```bash
actionguard coverage        # Protection Coverage summary
actionguard coverage -v     # Full per-boundary breakdown
```

```
Protection Coverage — v0.3

  ✓ boundary tiers active (out of 12)

  ┌─ High-Quality Boundaries (Tool Hook / Exec Approval)
  │  ✓ Cursor
  └  enforced: 5  |  generic fallback: 0

  ┌─ Generic Boundaries (Protected Shell)
  │  ✓ PowerShell (PSReadLine interactive)
  └  enforced: 1

  ╔══════════════════════════════════════════════════════╗
  ║            6/12 boundaries enforced (50%)            ║
  ╚══════════════════════════════════════════════════════╝

  Generic shell boundary is active — actions from unknown AI apps
  (apps without a dedicated hook) are still protected via shell.
```

The key principle: **Generic Boundary First, Vendor Adapter Second.**
Hook adapters provide the highest quality enforcement. But if a tool has no dedicated hook, ActionGuard falls back to the protected shell — so every AI app is covered, not just the ones you've configured manually.

**For AI users — the GUI:**

Prefer a point-and-click start? Open ActionGuard → **Protect this computer**. One button, plain-language counters, no commands.

### See a decision without executing anything

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

`policy-check` never executes anything. Enforcement only happens inside an active protected session:

```
→ DENY
→ ENFORCED
→ command not executed
```

```
$ actionguard protect

  ✓ Policy loaded
  ✓ Boundary detected
  ✓ Enforcement active

  AI-powered automation attempted:  sudo rm -rf /

  ✗ DENIED
```

That is the whole loop: an action arrives, ActionGuard decides, the ledger records it, nothing harmful runs.

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
        ├── Approval   → ask a human (CLI or GUI); timeout defaults to deny
        └── Evidence   → append-only ledger, per-action record, stats
```

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

The complete, generated command reference ships with the binary — run `actionguard --help`.

---

## What ActionGuard can — and cannot — enforce today

- **Local-first** — no telemetry, no account required. Everything stays on your machine.
- **Fail-closed by default** — every enforcement point denies when the policy engine is unreachable, the response is unparseable, or there is no active session.
- **Open verification** — enforcement claims are backed by reproducible tests in [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md), not by marketing.

These limits are real and measured:

1. **PowerShell protection is interactive-only on Windows.** Interactive lines are enforced (Phase C — verified 2026-08-21); scripts, `-Command`, and piped stdin are observe-only.
2. **Direct subprocesses can bypass shell hooks.** `os.system`, `/usr/bin/rm`, and other absolute-path or non-shell spawns are observed, not blocked.
3. **Remote automation is out of scope.** Actions executed on a different machine cannot be enforced by a local tool.
4. **No sandboxing yet.** Running actions in an isolated environment (container / network isolation) is future work.

---

## Can you break ActionGuard?

Every "Verified today" claim above is meant to be tested. Find a **new** way to bypass an enforced boundary that is not already documented?

- Report **privately** via GitHub Private vulnerability reporting ([`SECURITY.md`](./SECURITY.md)) — never a public issue.
- Every accepted report is credited in the changelog.

The honest list of known limitations (subprocess / absolute-path execution) is **not** a vulnerability — it is documented and observable with `actionguard doctor`. New, undocumented bypasses are in scope.

---

## Give feedback

ActionGuard is local-first: **no telemetry** is built into the product. Instead, we listen actively.

- 🛡 **Did it catch something?** Tell us about your first interception: [open the feedback form](https://github.com/SeanXChen/ActionGuard/issues/new?template=feedback.yml) (takes 1 minute).
- 🐞 **Did something fail to install, start, or protect?** [File a bug report](https://github.com/SeanXChen/ActionGuard/issues/new?template=bug_report.yml).
- 💬 Prefer open conversation? [Start a discussion](https://github.com/SeanXChen/ActionGuard/discussions).

We track these signals in [docs/USER_VALIDATION.md](./docs/USER_VALIDATION.md).

---

## Using ActionGuard in your company?

ActionGuard is local-first, but we're actively exploring **team deployment and enterprise use cases** — shared policy, audit-friendly exports, managed rule packs.

If *"can this work for a team?"* is a question you're asking, we'd like to hear it: [start a discussion](https://github.com/SeanXChen/ActionGuard/discussions) or [file an enterprise-inquiry issue](https://github.com/SeanXChen/ActionGuard/issues/new).

---

## Contributing

Contributions are welcome — especially **boundary discovery** (a dangerous automation action ActionGuard doesn't handle yet) and **boundary verification reports**. See [CONTRIBUTING.md](./CONTRIBUTING.md) for report templates, rule format, and security reporting.

### Intellectual Property

ActionGuard takes a thoughtful approach to IP protection while maintaining open development:

- **[IP_STRATEGY.md](./IP_STRATEGY.md)** — Our overall IP strategy framework
- **[PATENT_CANDIDATES.md](./PATENT_CANDIDATES.md)** — Technical innovations under evaluation
- **[docs/IP_LAYER_GUIDE.md](./docs/IP_LAYER_GUIDE.md)** — Guidance on what to公开 vs. protect

**For contributors**: If you discover a new enforcement mechanism or have ideas about technical innovations, please discuss with maintainers before documenting publicly.

See also: [SECURITY_MODEL.md](./SECURITY_MODEL.md) · [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md) · [BOUNDARIES.md](./BOUNDARIES.md) · [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) · [docs/FACTS_SCHEMA.md](./docs/FACTS_SCHEMA.md) · [docs/BOUNDARY_BACKLOG.md](./docs/BOUNDARY_BACKLOG.md)

---

## License

[Apache-2.0](./LICENSE)

---

## Languages

- [English](./README.md)
- [简体中文](./README.zh.md)
