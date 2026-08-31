# ActionGuard IP Strategy

> **Purpose**: Guide intellectual property decisions for ActionGuard
> **Scope**: Patents, trade secrets, open source strategy, and public disclosure
> **Owner**: ActionGuard project maintainer
> **Last updated**: v0.3 (2026-08-30)

---

## Strategic Framework

### Core Principle

> **User growth first. IP protection as defense, not obstacle.**

ActionGuard's primary goal is to protect users from harmful AI automation. Every IP decision should serve this goal:

- Patents should **protect** the technology, not **block** adoption
- Open source should **grow** the community, not **compromise** core IP
- Public disclosure should **educate** users, not **expose** patentable innovations

### Three-Layer Architecture

```
              ActionGuard
                   │
       ┌───────────┼───────────┐
       ↓           ↓           ↓
   Product      Open Source    IP / Patent
   Layer        Implementation Layer
       │           │           │
   User growth   Community    Technical
   + trust       + credibility 壁垒
       │           │           │
       └───────────┴───────────┘
                   │
              Commercial
              Viability
```

---

## Patent Strategy

### Decision Framework

Before filing a patent, evaluate:

1. **Novelty**: Does this solve a technical problem that isn't solved elsewhere?
2. **Non-obviousness**: Would a person skilled in the art consider this obvious?
3. **Utility**: Does this have a specific, practical application?
4. **Competitive value**: Would a competitor find this valuable to copy?
5. **Timeline**: Is now the right time, or should we wait for more validation?

### Filing Sequence

```
Discover technical innovation
         ↓
Document reduction to practice
         ↓
Prior-art search (DIY or professional)
         ↓
Patent attorney evaluation
         ↓
Decision: File / Defer / Abandon
         ↓
(if filing) File appropriate application based on jurisdiction
         ↓
Public disclosure
```

**Note**: Patent strategy depends on jurisdiction. Before public disclosure:
1. Identify potentially patentable technical solutions
2. Conduct prior-art review
3. Evaluate novelty / inventiveness / utility
4. Consult qualified patent professional when warranted
5. File an appropriate application (type varies by jurisdiction) before public disclosure

Common application types:
- **Provisional application** (US): 12-month grace period, not examined
- **Utility application** (US): Full examination
- ** Invention patent** (China): Requires full examination
- **PCT application**: International coverage before national phase

### What We Will Not Patent

- ❌ GUI design (tabs, colors, button styles)
- ❌ Policy rules (YAML files) — copyrightable, not patentable
- ❌ General AI security concepts — too abstract
- ❌ Current Hook/Shell implementation — may not be the optimal solution

### What We May Patent

- ✅ Novel enforcement mechanisms (especially OS-level or process-level)
- ✅ Vendor-neutral policy enforcement across heterogeneous AI tools
- ✅ Action chain risk evaluation
- ✅ Enforcement verification methodology

See [PATENT_CANDIDATES.md](./PATENT_CANDIDATES.md) for details.

---

## Public Disclosure Strategy

### The Four Layers

| Layer | What to公开 | What NOT to公开 |
|-------|-------------|----------------|
| **Marketing** | Product concept, benefits, use cases | Technical implementation details |
| **Product** | What it does, how to use it | How it works internally |
| **Architecture** | High-level design, boundary model | Specific algorithms, data structures |
| **Implementation** | Source code (open source) | Patent-core mechanisms (if not yet filed) |

### Marketing Layer (GitHub, V2EX, Reddit, HN)

✅ **Can公开**:
- Product concept: "Local safety boundary for AI automation"
- Problem statement: "AI can act faster than humans can review"
- Benefits: "Allow/Ask/Deny", "Ledger", "Trust Zone"
- Verified claims: "Enforced on Cursor, Claude Code, CodeBuddy"
- Screenshots and demos
- Installation instructions

❌ **Should NOT公开**:
- Complete step-by-step implementation of core enforcement mechanisms
- Specific algorithms for dynamic boundary selection
- Novel combinations of existing techniques that could constitute patentable inventions
- Internal architecture details that reveal patentable innovations

### Architecture Layer (docs/)

✅ **Can公开**:
- High-level architecture diagrams
- Boundary model explanation (A–F classes)
- Policy engine concept
- Facts Schema (general description)
- Enforcement status taxonomy

❌ **Should NOT公开**:
- Specific implementation details of novel mechanisms
- Detailed algorithms for enforcement verification
- Proprietary combinations of techniques

### Implementation Layer (Source Code)

The source code is open source, which means:
- ✅ Anyone can read, use, and modify it
- ✅ Contributions are welcome
- ❌ This precludes patent protection on specific code elements
- ✅ But does NOT preclude patent protection on novel technical concepts implemented in the code

**Key insight**: You can patent the *concept* while open-sourcing the *implementation*, as long as you file the patent *before* open-sourcing.

---

## Prior Art Considerations

### What Counts as Public Disclosure

Under most patent regimes (including China, US, and EPO):

- GitHub repositories
- Blog posts and articles
- Forum posts (V2EX, Reddit, Hacker News)
- Conference talks and videos
- Demo videos
- Published books

**Once disclosed, the clock starts ticking.**

### Grace Periods

| Jurisdiction | Grace Period | Notes |
|--------------|--------------|-------|
| US | 1 year | From first public disclosure |
| China | 6 months | Limited to inventor's own disclosure |
| EPO | None | Absolute novelty required |
| WIPO | Varies | Depends on specific country |

**Recommendation**: File patent applications *before* any public disclosure, especially for international protection.

---

## Enforcement Research Strategy

### Current Gaps

The biggest enforcement gaps in v0.2:

1. **subprocess / absolute path execution**: Actions that don't go through a shell hook
2. **PowerShell scripts**: `-Command`, piped stdin, script files

### Research Directions

| Direction | Description | Priority |
|-----------|-------------|----------|
| OS-level enforcement | Use Windows AppContainer / Linux Landlock | High |
| Process-level enforcement | Restrict child process capabilities | High |
| Network boundary | Isolate network access for AI processes | Medium |
| Credential boundary | Prevent credential exfiltration | High |

### Patent Implications

If a novel enforcement mechanism is discovered through research:
1. Document the discovery thoroughly
2. Do NOT publish implementation details
3. Engage patent attorney for evaluation
4. File patent application if warranted

---

## Timeline and Milestones

| Phase | Timeframe | Actions |
|-------|-----------|---------|
| **v0.3** | Current | Finalize IP documentation, freeze current technical state |
| **v0.4** | Next | Evaluate enforcement research results, update patent candidates |
| **v0.5** | Future | If novel mechanisms discovered, file patents before public disclosure |
| **v1.0** | Future | Complete patent portfolio review |

---

## Action Items

### Immediate (v0.3)

- [x] Create `PATENT_CANDIDATES.md`
- [x] Create this `IP_STRATEGY.md`
- [x] Create `docs/IP_LAYER_GUIDE.md` for contributor guidance
- [ ] Review all public documentation for patent-sensitive content
- [ ] Engage patent attorney for preliminary evaluation of Candidate 01

### Near-term (v0.4)

- [ ] Conduct prior-art search for Candidate 01
- [ ] Evaluate enforcement research results
- [ ] Update patent candidates based on new implementation
- [ ] Decide whether to file provisional applications

### Long-term (v0.5+)

- [ ] File patents on validated innovations
- [ ] Review and update IP strategy annually
- [ ] Monitor competitor patents

---

## Resources

- [PATENT_CANDIDATES.md](./PATENT_CANDIDATES.md) — Detailed patent candidate documentation
- [docs/IP_LAYER_GUIDE.md](./docs/IP_LAYER_GUIDE.md) — Contributor guidance on what to公开
- [SECURITY_MODEL.md](./SECURITY_MODEL.md) — Security concepts
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) — Technical architecture

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-08-30 | v0.3 | Initial creation from IP strategy review |
