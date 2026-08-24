# Contributing to ActionGuard

Thank you for your interest in ActionGuard. All contribution types are welcome: **boundary verification reports**, policy rules, code, and documentation.

> 📖 中文版见 [docs/CONTRIBUTING.md](./docs/CONTRIBUTING.md)

---

## The most valuable contribution: Boundary Verification

ActionGuard runs on **honest, measured data** — not claims. The project's core asset is its boundary registry: what actually happens when an AI-powered automation action hits a boundary on a real machine.

The single most valuable thing you can contribute is a **verification report**: install ActionGuard on a real AI automation tool (CodeBuddy, Claude Code, Codex, Cursor, Windsurf, OpenClaw, Manus, ...), run a boundary test, and record what actually happened.

### How to run a boundary test

```bash
actionguard setup --yes          # install the shell hook for your shell
actionguard boundary test "Protected Shell (bash/zsh/fish)"
actionguard policy-check "sudo rm -rf /" --explain
actionguard doctor                # answer: is this machine protected right now?
```

Then exercise a real automation tool (e.g. ask your agent to run `rm -rf` on a test file, `git push --force`, or read a secret) and record the outcome.

### Verification report template (PR body — must be filled)

```markdown
## Boundary Verification Report

- **Automation**: [tool name]
- **Version**: [exact version]
- **OS**: [OS + shell]
- **Execution path**: [how the action was triggered: agent prompt / CLI / script]
- **Boundary**: [boundary name, e.g. "Protected Shell (bash/zsh/fish)"]
- **Test**: [exact command or scenario]
- **Expected**: [what policy says should happen]
- **Actual**: [what actually happened]
- **ActionGuard version**: [engine version]
- **Evidence**: [screenshot / recording / log]

> ⚠️ Claims without evidence are closed without merge.
```

### Verification standards

1. **Measured, not assumed** — must be a real execution, not a code-review guess.
2. **One boundary per report** — one command, one scenario, one entry.
3. **Version + OS required** — behavior differs across versions.
4. **Maintainer review only** — maintainers merge verification reports.
5. **Community Verified → Core Verified** — community reports are merged as **Community Verified**; a maintainer must reproduce on their own machine before upgrading to **Core Verified**.

Reports update `boundaries/<tool>.yml` and `SECURITY_TEST_MATRIX.md`.

---

## The second most valuable contribution: Boundary Discovery

ActionGuard's built-in baseline covers what we **know**. What we don't know yet is worth more: an AI-powered automation behavior that ActionGuard does not understand — a boundary we haven't modeled.

**Found a dangerous agent action that ActionGuard doesn't handle? Report it.** You don't need to write YAML, and you don't need to read the codebase.

We reward **boundary discovery**, not rule writing. Contributions are tiered:

| Tier | What you do | Required skill | Credit |
|------|-------------|----------------|--------|
| 1 · Report | "ActionGuard didn't stop X." Action + why it's dangerous, no YAML needed | None | Credited in the boundary backlog |
| 2 · Repro | Action, environment, expected, actual, reproduction steps | Basic | Credited + linked to the fix |
| 3 · Policy + Test | Policy rule + golden test (`tests/golden/`) | YAML + Rust test | Credited + maintainer review |

Every accepted discovery becomes a row in [`docs/BOUNDARY_BACKLOG.md`](./docs/BOUNDARY_BACKLOG.md).
Every new rule ships with a golden test — **a rule without a test is not merged**.
The queue is curated by maintainers; we review *what you found*, not how well you can write YAML.

---

## Other contribution types

| Type | Where | Notes |
|------|-------|-------|
| Policy rules | `src-tauri/rules/*.yml` (builtin) or a community package | See `docs/CONTRIBUTING.md` for the YAML schema |
| Code | `src-tauri/` (Rust), `src/` (Vue 3 + TS) | Open an issue first to discuss non-trivial changes |
| Documentation | `README.md`, `docs/`, translations | Direct PR is fine |

For rule format, commit message conventions, build instructions, and the code contribution workflow, see the full guide in [docs/CONTRIBUTING.md](./docs/CONTRIBUTING.md).

---

## Reporting security issues

**Do not open a public issue for a vulnerability.** Report privately — see [SECURITY.md](./SECURITY.md).

---

## License

By submitting a PR you agree that your contribution is released under the [Apache-2.0](./LICENSE) license.
