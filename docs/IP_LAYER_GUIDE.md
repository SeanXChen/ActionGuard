# IP Layer Guide for Contributors

> **Purpose**: Help contributors understand what information is safe to公开 and what should be protected
> **Audience**: Developers, documentation writers, community members
> **Last updated**: v0.3 (2026-08-30)

---

## Quick Reference

### Can公开 (Safe to share publicly)

| Category | Examples |
|----------|----------|
| **Product concept** | "ActionGuard adds an independent safety layer between AI automation and your computer" |
| **Problem statement** | "AI can execute actions faster than humans can review them" |
| **Features** | Allow/Ask/Deny, Ledger, Trust Zone, Coverage Ladder |
| **Verified results** | "Deny enforced on Cursor via `beforeShellExecution` hook" |
| **Installation** | How to install, how to configure |
| **Use cases** | Real-world examples of dangerous actions that were blocked |
| **Architecture (high-level)** | Boundary classes A–F, Policy engine concept |
| **How to use** | Documentation for users |

### Do NOT公开 (Patent-sensitive or implementation secrets)

| Category | Why | What NOT to say |
|----------|-----|-----------------|
| **Novel enforcement algorithms** | May be patentable | "We use a specific algorithm to dynamically select enforcement boundaries based on..." |
| **Unpublished mechanisms** | Core IP | "Our new X enforcement method works by..." |
| **Complete implementation details** | Trade secrets | "The hook works by intercepting at step 1, then 2, then 3..." |
| **Experimental research** | Not yet validated | "We are testing Y approach which could revolutionize..." |

### Gray Area — Ask First

| Category | What to do |
|----------|------------|
| **New boundary implementations** | Discuss with maintainer before documenting in detail |
| **Enforcement research results** | Wait for official announcement |
| **Performance optimization techniques** | General approaches are fine; specific novel methods need review |

---

## The Four Layers Explained

### Layer 1: Marketing (GitHub, Social Media, Forums)

**Purpose**: Attract users, explain value proposition

**Safe to share**:
- Product screenshots and demos
- "This tool saved me from accidentally running `rm -rf /`"
- Installation instructions
- Comparison with other security tools
- "ActionGuard is the only vendor-neutral AI safety boundary"

**Not safe to share**:
- Internal architecture diagrams that reveal novel mechanisms
- "Our new enforcement algorithm does X"

### Layer 2: Product Documentation (README, User Guides)

**Purpose**: Help users understand and use the product

**Safe to document**:
- All current features and how to use them
- Verified enforcement status per boundary
- How to write policy rules
- Trust Zone configuration
- Coverage Ladder interpretation

**Be careful with**:
- Detailed implementation explanations (keep high-level)
- "Why" behind design decisions (fine), "How exactly it works" (ask first)

### Layer 3: Architecture Documentation (docs/)

**Purpose**: Help developers understand the system

**Safe to document**:
- High-level architecture diagrams
- Boundary model (A–F classes)
- Policy engine concept
- Facts Schema (general description)
- Data flow diagrams (without implementation specifics)

**Be careful with**:
- Specific algorithms or data structures
- Novel combinations of techniques
- Research directions or hypotheses

### Layer 4: Source Code (src/, src-tauri/)

**Purpose**: Enable contribution and inspection

**The source is open** — but this doesn't mean everything is documented:
- Code can be read and used under the license
- Implementation details in code are visible to anyone who looks
- But code comments should not disclose patentable innovations
- If you discover something novel in the code, discuss with maintainer before blog post

---

## Decision Tree

When writing documentation, ask:

```
Is this information...
│
├─ Already in source code (public)?
│   └─ Yes → OK to document
│
├─ About a shipped feature?
│   └─ Yes → OK to document (high-level)
│
├─ About a new mechanism discovered during development?
│   └─ Yes → Discuss with maintainer before documenting
│
├─ About experimental research?
│   └─ Yes → Do NOT document until validated + decision made
│
└─ About a potential patent candidate?
    └─ Yes → Do NOT document without maintainer approval
```

---

## Examples

### ✅ Good Documentation

```markdown
## Coverage Ladder

ActionGuard shows you which execution paths are protected:

- **Tool Hook**: Highest quality enforcement via AI tool's own hook
- **Protected Shell**: Enforces actions through shell hooks
- **Observed**: Actions are logged but not blocked

This helps you understand your actual protection level.
```

### ❌ Too Detailed Documentation

```markdown
## How Hook Enforcement Works

1. The hook intercepts at the pre-execution stage
2. It sends the action to the policy engine via HTTP
3. The engine normalizes the action using our proprietary
   normalization algorithm (Patent pending)
4. The decision is enforced by modifying the return value
5. The enforcement status is verified by checking if the
   command actually executed

[This level of detail may expose patentable innovations]
```

### ✅ Good Bug Report

```markdown
**Bug**: Cursor hook not working after update

**Steps to reproduce**:
1. Run `actionguard setup`
2. Open Cursor
3. Try to run `rm -rf /`

**Expected**: Should be denied
**Actual**: Command executed

[Standard bug report — no IP concerns]
```

### ❌ IP-Relevant Bug Report

```markdown
**Feature Request**: Add support for subprocess interception

I noticed that commands run via `subprocess.run()` bypass the hook.
I think you could fix this by intercepting at the syscall level
using eBPF, similar to how X does it. Here's my implementation...

[This could be a new enforcement mechanism — discuss first]
```

---

## What If I'm Unsure?

**Ask the maintainer** before posting, writing documentation, or sharing.

When in doubt:
1. Don't post immediately
2. Check with the maintainer
3. Wait for guidance

This protects both you and the project.

---

## Related Documents

- [IP_STRATEGY.md](../IP_STRATEGY.md) — Full IP strategy
- [PATENT_CANDIDATES.md](../PATENT_CANDIDATES.md) — Patent candidate details
- [CONTRIBUTING.md](./CONTRIBUTING.md) — General contribution guidelines
- [SECURITY_MODEL.md](../SECURITY_MODEL.md) — Security concepts

---

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2026-08-30 | v0.3 | Initial creation |
