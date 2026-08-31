# Security Policy

ActionGuard is a security product. Vulnerabilities in it can put the user's machine at risk — please report them **privately**, never through a public issue.

## Supported versions

| Version | Supported |
|---------|-----------|
| Latest release | ✅ |
| Older releases | ❌ |

Only the latest released version receives security fixes. Users are expected to update.

## Reporting a vulnerability

**Do not file a public issue or Discussion for a vulnerability.**

Please use **GitHub Private vulnerability reporting**:

1. Open the repository's *Security* tab.
2. Click **Report a vulnerability** (or use the private advisory form at `https://github.com/<OWNER>/<REPO>/security/advisories/new`).
3. Provide:

   - Affected version (e.g. `actionguard 0.3.0`)
   - Operating system and shell
   - Steps to reproduce (exact commands)
   - Expected behavior
   - Actual behavior
   - Any evidence (screenshots, logs) — do **not** paste secrets

What happens next:

- We acknowledge receipt within **48 hours**.
- We assess severity and keep you informed of the fix timeline.
- We coordinate disclosure: you get credit in the advisory and changelog, unless you prefer to stay anonymous.

## Scope

In scope:

- The Rust engine (`src-tauri/`): policy engine, boundary detection, shell hook, approval gate, ledger.
- The CLI (`actionguard`).
- The Tauri GUI (`src/` + `src-tauri/`).

Known, documented limitations (bypasses via subprocess / absolute-path execution) are **not** vulnerabilities — see `SECURITY_MODEL.md` and the `actionguard doctor` output for the honest list. If you find a *new* way to bypass an enforced boundary that is not documented, that **is** in scope.

## Security-conscious development

Maintainers follow:

- Rust `cargo clippy` with warnings denied.
- `cargo test` before every merge (CI enforces this).
- Dependabot for Rust, npm, and GitHub Actions dependencies.
- Secrets never committed — GitHub Secret Scanning (enabled by default on this public repository) plus manual review of every PR.

## Known dependency vulnerabilities

We track advisories for our dependency tree in GitHub's Dependabot alerts. Not every alert can be fixed by bumping a version — the table below records the ones that are **blocked upstream**, so the status is explicit and not left to guesswork.

| Advisory | Dependency | Affected | Status | Notes |
|----------|-----------|----------|--------|-------|
| GHSA (RUSTSEC-2025-0032) — `glib` | `glib` (transitive via Tauri → GTK) | >= 0.15.0, < 0.20.0 | **Blocked upstream** | Tauri 2.x depends on `gtk 0.18`, which pins `glib ^0.18`. The fix requires Tauri upstream to migrate to GTK4 bindings; there is no compatible `glib >= 0.20` we can select today. Actual exposure for ActionGuard is minimal: it is a local desktop app with no network service, and the vulnerable code path (`VariantStrIter` iterator) is not reachable from user-controlled input in our usage. We will re-run `cargo update` when Tauri releases a version that unblocks this dependency. |
