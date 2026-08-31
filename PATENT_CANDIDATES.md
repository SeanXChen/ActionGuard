# ActionGuard Patent Candidates

> **Status**: Living document — updated at each major version milestone
> **Last updated**: v0.3 (2026-08-30)
> **Purpose**: Track technically novel mechanisms that may qualify as patent-eligible inventions
> **Note**: "Candidate" means "worth evaluating", not "will be granted"

---

## Overview

This document tracks **patent candidates** — technically novel mechanisms that may,
after prior-art search and professional evaluation, qualify as patent-eligible inventions.

**Three tiers of candidates:**

| Tier | Description | Action |
|------|-------------|--------|
| 🔴 High Priority | Novel mechanism, clear technical problem, no obvious prior art | Prior-art search + patent attorney evaluation |
| 🟡 Medium Priority | Interesting technical approach, needs verification | Continue development + document experiments |
| 🟢 Monitor | Worth watching, too early to evaluate | Track in changelog, revisit at next milestone |

**Non-candidates** (documented for clarity, not because they're weak):

| What it's NOT | Why |
|---------------|-----|
| GUI design (tabs, colors, Allow/Deny buttons) | Design patents have limited commercial value for a security tool |
| "AI security software" as a category | Too abstract — must be tied to a specific technical mechanism |
| Policy rules (YAML files) | Copyrightable, not patentable |
| The "Detected ≠ Blocked ≠ Enforced" phrase | Product concept, not a technical implementation |

---

## Candidate 01 — Vendor-Neutral AI Action Safety Boundary

### Problem Statement

AI automation tools (Cursor, Claude Code, Codex, OpenClaw, Manus, etc.) each expose different execution interfaces. Existing security approaches either:

- Build a separate security layer per vendor (Cursor Security, Codex Security, etc.)
- Rely on the automation's own built-in permissions (which are written by the same vendor)

**Neither approach provides a unified, independent safety layer across heterogeneous automation tools.**

### Technical Approach

A computer-implemented method for enforcing a unified security policy across AI automation tools without modifying the automation tools themselves, comprising:

1. Detecting an action at an execution boundary that is independent of the automation tool's vendor
2. Normalizing the detected action into a vendor-neutral representation
3. Evaluating the normalized action against a policy engine
4. Enforcing the policy decision at the same execution boundary where the action was detected
5. Verifying that the enforcement was actually applied and recording the enforcement result

### Novel Elements

- **Boundary-first, not brand-first**: Classification by execution path (shell, hook, OS process) rather than by AI tool brand
- **Vendor-neutral normalization**: Different automation tools' actions mapped to a unified `Action Fact` schema
- **Independent enforcement**: The safety boundary is controlled by the user/machine owner, not by the automation vendor

### Current Implementation Status

- ✅ Basic concept implemented in v0.2: `boundary.rs` + `BoundaryKind` enum (A–F)
- ✅ Protected Shell boundary: `bash/zsh/fish` preexec hooks + PowerShell PSReadLine
- ✅ Tool Hook boundary: `beforeShellExecution` (Cursor), `PreToolUse` (CodeBuddy, Claude Code)
- ⚠️ Enforcement verification: partially implemented (`EnforcementStatus` enum)
- ❌ Dynamic boundary selection: not yet implemented (currently static per-boundary)

### Verification Evidence

- `SECURITY_TEST_MATRIX.md`: Verified enforcement on Class A (Tool Hook) and Class C (Protected Shell)
- Ledger evidence: `action_id` + `enforcement_status` per decision
- Boundary test: `actionguard boundary test` reproducible verification

### Prior Art Considerations

- Need to search: General-purpose AI security frameworks
- Need to search: Vendor-agnostic security policy enforcement
- Need to search: Cross-platform execution boundary interception

### Potential Claims (Draft)

1. A method for enforcing a security policy on AI automation actions, comprising: detecting an action at a first execution boundary associated with a first AI automation tool; converting the detected action into a normalized representation; evaluating the normalized representation against a policy rule set; and enforcing a policy decision at a second execution boundary that is independent of the first AI automation tool.

2. A system for providing vendor-neutral security for AI automation, comprising: a plurality of boundary adapters, each configured to detect actions from a corresponding AI automation tool; a normalization layer configured to convert detected actions into a unified action representation; a policy engine configured to evaluate the unified action representation against a policy rule set; and an enforcement layer configured to apply policy decisions at execution boundaries that are independent of the AI automation tools.

### Priority: 🔴 High Priority

**Rationale**: This is the core differentiating technical concept. The "attach by boundary, not by brand" principle, if implemented with a specific algorithm for dynamic boundary selection and enforcement verification, could constitute a novel and non-obvious technical solution.

---

## Candidate 02 — Normalized Action Facts Schema

### Problem Statement

AI automation tools produce actions in heterogeneous formats (CLI commands, API calls, file operations, etc.). A security policy must evaluate these actions without being tied to the specific format of any one tool.

### Technical Approach

A computer-implemented method for normalizing heterogeneous AI automation actions into a unified schema, comprising:

1. Extracting action facts from the raw action representation (actor, action, target, resource, context, origin, execution_path, risk, decision, enforcement)
2. Mapping the extracted facts to a normalized schema
3. Using the normalized schema as input to a policy engine
4. Preserving the mapping between normalized facts and original action for audit purposes

### Novel Elements

- **Standardized action representation**: Not just "file path" but a structured schema including actor, execution path, resource type, context
- **Bidirectional mapping**: Original action ↔ Normalized facts (for audit trail)
- **Execution path tracking**: Tracking not just what was done, but how it was executed

### Current Implementation Status

- ✅ Basic implementation in v0.2: `Action` struct in `models.rs`
- ✅ Fact extraction in `classify.rs`
- ✅ `docs/FACTS_SCHEMA.md`: Detailed schema documentation
- ⚠️ Schema evolution: Not yet versioned or formally specified

### Verification Evidence

- `docs/FACTS_SCHEMA.md`: Complete schema documentation
- Unit tests: `src-tauri/tests/`

### Prior Art Considerations

- Need to search: Event normalization in SIEM systems
- Need to search: Cross-platform action abstraction layers

### Priority: 🟡 Medium Priority

**Rationale**: The schema itself may not be novel (similar to SIEM normalization). The novel element could be the specific combination of fields relevant to AI automation security (especially `execution_path`, `origin`, `credential_detected`).

---

## Candidate 03 — Decision vs. Enforcement State Separation

### Problem Statement

Existing security systems conflate "we decided to block this" with "we actually prevented this". An action can be "blocked" by policy but still execute if the enforcement mechanism has a gap.

### Technical Approach

A computer-implemented method for verifying that a policy decision was actually enforced, comprising:

1. Making a policy decision (Allow / Ask / Deny) based on action facts
2. Applying the decision at an enforcement boundary
3. Verifying the enforcement result (Enforced / Observed / Bypassed / Unsupported)
4. Recording both the decision and the verification result

### Novel Elements

- **Explicit enforcement verification**: Not just "blocked" but "verified as blocked"
- **Enforcement status taxonomy**: Enforced, Observed, Bypassed, Unsupported
- **Separation of concerns**: Policy decision is independent of enforcement capability

### Current Implementation Status

- ✅ Implemented in v0.2: `EnforcementStatus` enum in `models.rs`
- ✅ Ledger records both `decision` and `enforcement`
- ⚠️ Verification mechanism: Currently limited to "did the command execute or not"

### Verification Evidence

- `SECURITY_MODEL.md`: Full explanation of `Detected ≠ Blocked ≠ Enforced`
- Ledger entries: Both decision and enforcement status recorded

### Prior Art Considerations

- Need to search: Enforcement verification in security systems
- Need to search: "Observed" vs "Enforced" distinction in access control

### Priority: 🟡 Medium Priority

**Rationale**: The concept is important for product differentiation, but the specific implementation (checking if command executed) may be obvious. The novel element could be the formal taxonomy and the systematic verification approach.

---

## Candidate 04 — Execution-Boundary Coverage Assessment

### Problem Statement

Users want to know "how protected am I right now?" Existing solutions show a binary "protected/not protected" status without explaining which execution paths are covered.

### Technical Approach

A computer-implemented method for dynamically assessing and presenting enforcement coverage across multiple execution boundaries, comprising:

1. Enumerating available execution boundaries on the current system
2. Determining the enforcement capability for each boundary (Enforced / Observed / Unsupported)
3. Calculating a coverage score based on enforcement capability and action frequency
4. Presenting the coverage assessment to the user in human-readable format

### Novel Elements

- **Dynamic boundary discovery**: Automatically detecting available execution boundaries
- **Coverage modeling**: Quantifying protection coverage across heterogeneous boundaries
- **Action frequency weighting**: Prioritizing coverage of frequently-used execution paths

### Current Implementation Status

- ✅ Implemented in v0.2: `actionguard coverage` command
- ✅ Coverage Ladder in GUI
- ❌ Dynamic boundary discovery: Currently static (configured boundaries only)
- ❌ Action frequency weighting: Not yet implemented

### Verification Evidence

- `actionguard coverage` output: Verified coverage assessment
- GUI screenshot: Coverage Ladder visualization

### Prior Art Considerations

- Need to search: Security coverage assessment tools
- Need to search: Attack surface analysis systems

### Priority: 🟢 Monitor

**Rationale**: The current implementation is relatively straightforward (counting configured vs. total boundaries). More novel would be a dynamic discovery mechanism that probes the system for available execution paths.

---

## Candidate 05 — Action Chain Risk Evaluation

### Problem Statement

Individual AI actions may appear benign, but a sequence of actions (an "action chain") can constitute a high-risk behavior. For example: read credential → package credential → network send.

### Technical Approach

A computer-implemented method for evaluating the risk of an action chain, comprising:

1. Tracking related actions within a session or time window
2. Detecting sequences of actions that individually appear benign but collectively constitute high risk
3. Evaluating the action chain against a chain-specific risk model
4. Enforcing policy decisions based on the chain-level risk assessment

### Novel Elements

- **Temporal correlation**: Linking related actions across time
- **Chain pattern detection**: Identifying sequences of actions that together are risky
- **Escalating risk model**: Risk level increases as the chain progresses

### Current Implementation Status

- ❌ Not implemented in v0.2
- ⚠️ Partial: Session tracking in ledger
- 🔲 Planned: v0.4+ (Action chain detection)

### Verification Evidence

None yet — future work.

### Prior Art Considerations

- Need to search: Attack chain detection in cybersecurity
- Need to search: Multi-step transaction risk assessment

### Priority: 🟢 Monitor

**Rationale**: This is a promising direction but not yet implemented. Monitor for patents in the broader cybersecurity field; if no blocking prior art, this could become a high-priority candidate once implemented.

---

## Candidate 06 — Trust/Context-Aware AI Action Policy

### Problem Statement

AI actions should be evaluated not just by what they are, but by who is running them, where they are running, and what resources they are accessing.

### Technical Approach

A computer-implemented method for evaluating AI action risk based on contextual trust factors, comprising:

1. Establishing a trust context for an AI automation session (workspace, user identity, resource scope)
2. Evaluating actions against the trust context (e.g., "read .env in workspace" vs "read .env outside workspace")
3. Dynamically adjusting policy enforcement based on trust context changes
4. Recording the trust context for audit purposes

### Novel Elements

- **Trust context modeling**: Beyond "is this command dangerous" to "is this command dangerous in this context"
- **Dynamic policy adjustment**: Policy strictness adjusts based on trust context
- **Resource scope tracking**: Defining what resources an AI automation has access to

### Current Implementation Status

- ⚠️ Partial: Workspace tracking in sessions
- ❌ Trust context modeling: Not yet implemented
- ❌ Dynamic policy adjustment: Not yet implemented

### Verification Evidence

- Session ledger: Workspace context recorded
- Trust Zone (GUI): User-defined trust directories

### Prior Art Considerations

- Need to search: Context-aware access control
- Need to search: Trust models in security systems

### Priority: 🟢 Monitor

**Rationale**: Similar concepts exist in traditional security (RBAC, ABAC). The novel element would be the specific application to AI automation with the boundary-agnostic approach.

---

## Enforcement Research Matrix

> **This section tracks the ongoing research into optimal enforcement mechanisms.**

The following matrix guides the enforcement capability research:

| Path | Current Status | Lower-Layer Solution | Bypass Resistance |
|------|---------------|---------------------|-------------------|
| CodeBuddy Hook | ✅ Enforced | — | High |
| Cursor Hook | ✅ Enforced | — | High |
| PowerShell interactive | ✅ Enforced | — | High |
| PowerShell script | ⚠️ Observe-only | Process / OS boundary | Medium |
| subprocess (absolute path) | ⚠️ Observe-only | Process / OS boundary | Low |
| child process | ⚠️ Observe-only | Process / OS boundary | Low |
| network | ❌ Unsupported | OS network boundary | — |
| secret file | ⚠️ Observe-only | OS file boundary | Medium |

**Research goal**: Move subprocess and absolute path from "Observe-only" to "Enforced" without requiring Agent modification.

---

## Patent Filing Recommendations

### Immediate Actions

1. **Candidate 01** — Vendor-Neutral AI Action Safety Boundary
   - Conduct formal prior-art search
   - Engage patent attorney for preliminary evaluation
   - Document reduction to practice (experiments, test results)

2. **Candidate 03** — Decision vs. Enforcement State Separation
   - Review for potential combination with Candidate 01
   - Document the specific taxonomy (Enforced / Observed / Bypassed / Unsupported)

### Future Actions

- At v0.4: Re-evaluate Candidates 05 and 06 based on implementation
- At v0.5: If enforcement research succeeds, re-evaluate Candidates 01 with new data

### What NOT to Do

- ❌ Do not file patents on current Hook/Shell implementation (too early, may not be optimal)
- ❌ Do not file patents on GUI design (low commercial value)
- ❌ Do not file patents without prior-art search (wastes resources)
- ❌ Do not公开 complete technical implementation details before filing

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-08-30 | v0.3 | Initial creation from IP strategy review |
