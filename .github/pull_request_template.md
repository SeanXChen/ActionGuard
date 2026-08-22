## What

<!-- One line: what this PR does. -->

## Type

- [ ] Bug fix
- [ ] Core / policy change
- [ ] Boundary Registry update / new boundary
- [ ] Docs
- [ ] CI / tooling
- [ ] Other

## Boundary Registry update

If this PR touches `boundaries/*.yml` or adds a verification claim, the
maintainers will **not** merge a bare claim. Attach the full evidence:

| Field | Value |
|---|---|
| Automation | e.g. `Codex` |
| Version | e.g. `0.64.0` |
| OS | e.g. `Windows 11 24H2` |
| Boundary | e.g. `Exec Approval (B)` |
| Test | e.g. `rm -rf ./actionguard-test` |
| Expected | `DENY` |
| Actual | `BLOCKED` / `NOT BLOCKED` |
| ActionGuard version | e.g. `0.3.0` |

Evidence must be reproducible — include the test script and the raw terminal
output (and the ledger `action_id` when applicable) in the PR description or
as an attached file. Claims without evidence are closed without merge.

<!-- Mark only if this row is already verified by an ActionGuard maintainer
     on a real machine — otherwise leave unchecked and the review decides.
- [ ] Core Verified (maintainer only, `boundary test` + ledger evidence)
-->

## Testing

<!-- What did you run? e.g. `cargo test`, `npm run build`, `actionguard boundary test "CodeBuddy"` -->
