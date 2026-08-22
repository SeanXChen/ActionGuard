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

   - Affected version (e.g. `actionguard 0.2.0`)
   - Operating system and shell
   - Steps to reproduce (exact commands)
   - Expected behavior
   - Actual behavior
   - Any evidence (screenshots, logs) — do **not** paste secrets

What happens next:

- We acknowledge receipt within **48 hours**.
- We assess severity and keep you informed of the fix timeline.
- We coordinate disclosure: you get credit in the advisory and changelog, unless you prefer to stay anonymous.

> If private vulnerability reporting is not yet enabled on the repository, report via email instead: **[your security contact email / placeholder]**.

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
- Secrets never committed — see `docs/CONTRIBUTING.md` for pre-commit checks.
