# Adversarial Bypass Tests

> **ActionGuard continuously tests its enforcement boundary against known bypass techniques.**

This directory treats the enforcement boundary as an attack surface. Every test asserts a behavior that would be a **regression** if it broke — and every test that documents a *known blind spot* pins the current behavior so the gap stays visible instead of silently changing.

Run locally:

```bash
cargo test --test bypass_runner
```

CI runs this automatically via `cargo test`.

## Matrix

| Area | File | Technique | Status |
|------|------|-----------|--------|
| Path | `path.rs` | `..` traversal (dot-dot escape) | ✅ protected |
| Path | `path.rs` | backslash / forward-slash equivalence | ✅ protected |
| Path | `path.rs` | case variants (`.ENV`, `ID_RSA`) | ✅ protected (`detect_asset`) |
| Path | `path.rs` | 8.3 short-name parent (`PROJEC~1`) | ✅ protected |
| Path | `path.rs` | rule `path` matching on rewritten path | ✅ protected |
| Path | `path.rs` | **rule `path` matching is case-SENSITIVE** | ⚠️ known gap |
| Path | `path.rs` | **trailing separator (`.env\`) hides asset** | ⚠️ known gap |
| Process | `process.rs` | source spoofing (`agent`/`automation`/`human`) | ✅ protected (source-agnostic) |
| Process | `process.rs` | `rm -rf /` root wipe | ✅ hard-denied |
| Process | `process.rs` | `sudo` prefix before rm rules | ✅ hard-denied |
| Process | `process.rs` | **flag reordering `rm -r -f /`** | ⚠️ known gap (Ask, not Deny) |
| Process | `process.rs` | **no whitespace `rm -rf/`** | ⚠️ known gap (Ask, not Deny) |
| Config | `config.rs` | user rule priority over built-in | ✅ invariant |
| Config | `config.rs` | first-match-wins determinism | ✅ invariant |
| Config | `config.rs` | empty match spec rejected by lint | ✅ invariant |
| Config | `config.rs` | source field never read by matcher | ✅ invariant |

## Planned expansions (not yet implemented)

These are real bypass classes we have NOT yet tested. Each is a known risk, not a promise:

- **Junction / symlink** — a link pointing at `.env` (path-level alias for a sensitive target). Requires OS-level path resolution.
- **8.3 short name of the FILE itself** — `ENVFILE~1` for `.env` (not just the parent dir).
- **Environment variable expansion** — `$HOME/.env` in a rule path.
- **Shell nesting** — `bash -c "rm -rf /"`, `powershell -Command ...`, `cmd /c ...` wrappers.
- **Child / grandchild processes** — `node -e "exec('rm -rf /')"` where the direct caller is `node.exe`, not the agent.
- **TOCTOU** — the file changes between policy evaluation and enforcement.
- **Config tampering** — the agent (or a prompt injection) rewrites `policies.user.yml` or the rule files.
- **Approval integrity** — synthetic approval input or a race between prompt and resolution.
- **Guard liveness** — killing the hook process, then running the action.

## Principles

1. **A test that documents a gap is better than no test.** The gap gets a name and a pinned behavior; fixing it later is a one-line diff to the assertion.
2. **The boundary is source-agnostic.** Verdicts come from action content + risk, never from the source's self-declared identity.
3. **User rules can relax built-ins** (the machine owner decides), but there is currently **no project/agent policy level** — so nothing below the owner can weaken the boundary. When Project policy arrives, the invariant is: *Project policy can make ActionGuard stricter, but never weaker.*
