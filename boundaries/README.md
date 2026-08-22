# Action Boundary Registry — `boundaries/*.yml`

The vendor-neutral map of every automation source ActionGuard knows about.
Rows are grouped by **Boundary Class** (A–F), not by brand. A new product is
mapped to a class in minutes; it is never special-cased in the engine.

These files are the **community-facing asset**. A repo checkout drives
`actionguard boundary list` / `boundary test` straight from YAML. The same
rows are baked into the binary (`src-tauri/src/boundary.rs::registry`) so a
binary-only install keeps working without the repo.

## File format

One YAML file per product. Filename is cosmetic; the `name` field is the key.

```yaml
# boundaries/<product>.yml
name: My Product            # REQUIRED — must match the canonical registry name
class: tool_hook            # Boundary Class A–F (see below); also accepts
                            #   a/b/c/d/e/f, tool-hook, protected-shell, …
boundary:
  type: "PreToolUse hook"   # what the boundary actually is
  mechanism: "deny → no exec"  # how decisions are enforced
enforcement:
  action: enforced          # enforced | observe_only
  verification: core        # core | community | (omit → unverified)
  confidence: high          # high | medium | low
contributor: "@someone"     # REQUIRED when verification: community
last_verified: "2026-08-19" # ISO date of last live verification
note: |-
  Free-form evidence: why this boundary is in this state, how it was
  verified, known bypasses, reproduction steps.
```

## Boundary Classes (A–F)

| Class | Kind | Pre-action? | Enforceable? |
|-------|------|-------------|--------------|
| **A. Tool Hook** | `tool_hook` | Yes (tool middleware) | Yes |
| **B. Exec Approval** | `exec_approval` | Yes (agent-side policy) | Via independent layer |
| **C. Protected Shell** | `protected_shell` | Yes (preexec hook) | Yes |
| **D. Runtime Sandbox** | `runtime_hook` | Yes (process-level, future L3) | Yes |
| **E. System Enforcement** | `system_level` | Yes (OS-level, future L4) | Yes |
| — observe only | `observe_only` | No — recorded after the fact | No |
| **F. Remote** | `remote` | No — never lands on this machine | No |

## Verification policy

A green checkmark is not enough for a safety product.

- `verification: core` — verified by the ActionGuard maintainers on a real
  machine, with the evidence recorded in `note`.
- `verification: community` — verified by the community with reproducible
  evidence (script + output + ActionGuard version + OS) attached to a PR that
  touches this file. **Must** also set `contributor: "@handle"`.
- omitted — not yet verified. `actionguard boundary list` will show it as
  `? Not verified` / `Not detected`.

## Add or update a row

1. Copy an existing file whose class matches.
2. Fill in `name`, `boundary`, `enforcement`.
3. If you verified it live, set `verification` + `last_verified` (and
   `contributor` for `community`) and put the evidence in `note`.
4. Open a PR using the template in `BOUNDARIES.md` →
   "Community verification — the boundary test standard".
5. After review and merge, `actionguard boundary list` (repo checkout) shows
   `✓ Community Verified` + `contributor: @handle`. If the name matches a row
   that has a live probe in `src-tauri/src/boundary.rs`, the probe still
   overlays it.
