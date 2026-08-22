//! v0.2 Policy engine — the third layer of the ActionGuard safety stack.
//!
//! Layered on top of `risk` (Classify) and underneath `approval` (Human Gate).
//! Given an `Action`, the policy engine returns a `DecisionResult` carrying:
//!   - `decision`: Allow / Ask / Deny
//!   - `matched_rule`: the id of the rule that fired (if any)
//!   - `risk`: risk level implied by the matched rule (falls back to risk engine)
//!   - `reason`: human-readable reason for the decision
//!
//! Rule sources, evaluated in priority order:
//!   1. User rules (`~/.actionguard/policies.user.yml`) — highest priority
//!   2. Built-in community rules (`rules/*.yml` baked in via `include_str!`)
//!
//! Within a single source, rules are evaluated in document order and the
//! FIRST match wins. This lets specific rules shadow broad ones.

pub mod classify;
pub mod loader;
pub mod matcher;

// Public API surface for the CLI (Phase D) + Approval gate (Phase C). These
// re-exports are part of the lib's stable interface even when the current
// crate-internal callers don't use all of them yet.
#[allow(unused_imports)]
pub use loader::{load_policy_set, lint_file};
#[allow(unused_imports)]
pub use matcher::{decide, decide_with_fallback};
// Re-export the data types the policy API uses so callers don't have to
// know they live in `crate::models`.
#[allow(unused_imports)]
pub use crate::models::{DecisionResult, PolicySet, Rule};
