# User Validation — v0.3

> **Purpose.** v0.3 is released. The question is no longer "what features can we add" but **"does anyone actually hit a real problem, and where do users drop off?"** This document is the single place to track that evidence. It is a living table — update it when a signal changes, not when the mood strikes.
>
> **How data gets collected.** No telemetry is built into the product (local-first is a core promise). Signals come from:
>
> - [GitHub Discussions](https://github.com/SeanXChen/ActionGuard/discussions) — open-ended conversation
> - [Feedback issue form](../.github/ISSUE_TEMPLATE/feedback.yml) — structured user reports
> - [Bug report form](../.github/ISSUE_TEMPLATE/bug_report.yml) — install/protection failures
> - Community boundary-verification PRs (see `BOUNDARIES.md`)
> - Direct user interviews
> - `actionguard stats --export <path>` — **local** user-behavior telemetry: every approval popup (`popups`) and every time the user allowed a gated action (`overrides`). No network — the report is a JSON file on the user's machine. This is the only quantitative "is the boundary trusted?" signal we have.

---

## The three signals that matter most

| # | Signal | Why it matters |
|---|--------|----------------|
| ① | **Someone was actually protected by it.** | A real interception ("Deny — rm -rf, glad it stopped") is the product moment. Without one, there is no proof of pain. |
| ② | **Someone keeps it running.** | A security tool that people install, try once, and delete is not PMF. Daily use is the proxy for trust. |
| ③ | **Someone asks: "can you also protect my other AI tool?"** | The user voices the multi-agent / unified-boundary need *for us*. That request validates the strategy without us having to argue for it. |

If ① and ② stay empty, do not build more features — find out why people drop off instead.

---

## Validation table

| Metric | Current | Target / observation | Data source |
|--------|---------|----------------------|-------------|
| GitHub Stars | _trend_ | — | Repo page |
| Downloads | _trend_ | — | Releases page |
| Completed `setup` | _n_ | higher is better | Feedback form, discussions |
| `doctor` normal | _n_ | higher is better | Feedback form, bug reports |
| First real interception | _n_ | **core** | Feedback form §"first interception" |
| Continued use (running daily) | _n_ | **core** | Feedback form §"still using" |
| Users adding custom rules | _n_ | strong signal | Policy PRs, discussions |
| Multi-agent users (2–4 tools) | _n_ | validates unified layer | Feedback form §"which tools" |
| Community boundary PRs | _n_ | validates ecosystem | Pull requests |
| Paid / enterprise inquiries | _n_ | business signal | Discussions, email |
| Install / protection failures | _n_ | **must track** | Bug report form |
| User Override Rate (`stats --export`) | _n_ | **core** — high rate ⇒ policy too sensitive or popup UX is failing | Local telemetry |
| Popups per session (`stats --export`) | _n_ | low better (autonomy preserved) | Local telemetry |
| Overrides per session (`stats --export`) | _n_ | **core** — numerator of Override Rate | Local telemetry |

---

## What to record when a signal arrives

### Install / protection failures (bug reports)

For each report, tag which **stage of the chain** failed:

> Download → install → `setup` → `doctor` → hook active → first interception

If many people fail at the same stage, that stage is a **product problem**, not a user problem. Examples of stage-specific failures to log: hook not taking effect, unclear how to start, `doctor` anomalies, PowerShell/shell environment issues, permissions, SmartScreen / Gatekeeper / Defender blocks.

### First-interception reactions

Capture the tone, not just the fact:

- Did the user find it useful, or "this should have been blocked anyway"?
- Did they share a screenshot unprompted?
- Did they start writing their own rules?
- Did they keep ActionGuard running afterward?

No "glad it caught that" moment after install → re-examine whether the pain point is real.

### What users are protecting

Do not assume `.env`, `rm -rf`, or `git reset --hard` are the top worries. Tally from feedback form §"what are you protecting against":

| User concern | Count |
|--------------|-------|
| AI deleting files | ? |
| AI reading secrets | ? |
| AI running dangerous shell | ? |
| AI modifying Git | ? |
| AI installing packages | ? |
| Prompt injection | ? |
| Other | ? |

This tally decides the product's core positioning.

### Single-agent vs multi-agent

Track which tools users name. The important number is **how many tools the average user runs at once**:

- If many users run **2–4 tools** → the vendor-neutral unified safety boundary claim is being validated by real data.
- If almost everyone runs **one tool** → do not push "unified policy layer" as the headline feature yet.

### What users ask for

Three kinds of requests, in order of value:

- **A — high-frequency strong demand**: "Can you block Cursor?" / "Support Claude Code?" / "Protect PowerShell?"
- **B — users who contribute**: "I tested it on OpenClaw, here are the results." These grow the Boundary Registry on their own.
- **C — users willing to pay**: "Is there a team plan?" / "Can I manage my company's machines centrally?" / "Centralized audit?" These are real commercialization signals.

### Retention (the one that matters)

Distinguish between:

- Downloaded → played → blocked `rm -rf` → starred → uninstalled (**not PMF**)
- Installed → protection on → daily dev → added own rules → hit one real risk → kept using (**PMF signal**)

---

## Update log

| Date | Change |
|------|--------|
| 2026-08-23 | Created for the v0.2 validation phase. No user data yet — first real signal is the goal. |
| 2026-08-24 | v0.2.1 telemetry: `stats --export` now records approval popups / user overrides (User Override Rate), plus per-action ledger rows for local analysis. README + GUI added a restrained enterprise-deployment signal ("Using ActionGuard in your company?"). |
