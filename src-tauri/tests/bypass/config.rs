//! Policy / configuration adversarial tests.
//!
//! The question these tests answer: can a policy file be written so that the
//! boundary stops enforcing? Two invariants are under test:
//!
//! 1. **Priority** — user rules are evaluated before built-ins, so a user
//!    rule can override a built-in (the machine owner is allowed to relax).
//! 2. **Determinism** — first match wins within a source; rule order is
//!    significant and auditable.

use actionguard_lib::models::{
    Action, ActionCategory, Decision, MatchSpec, PolicySet, PolicySource, RiskLevel, Rule,
};
use actionguard_lib::policy::decide;

fn deny_rule(id: &str, cmd: &str, arg: &str) -> Rule {
    Rule {
        id: id.to_string(),
        match_: MatchSpec {
            category: Some(ActionCategory::Shell),
            command: Some(cmd.to_string()),
            path: None,
            args_contains: Some(vec![arg.to_string()]),
            args_any: None,
            regex: None,
        },
        action: Decision::Deny,
        risk: Some(RiskLevel::High),
        reason: Some("test rule".to_string()),
        source: PolicySource::User,
    }
}

#[test]
fn user_rule_evaluated_before_builtin() {
    // A user rule that is FIRST in the set shadows a built-in for the same
    // command — the machine owner decides, not the community defaults.
    let user = deny_rule("user-deny-npm", "npm", "install");
    let policy = PolicySet {
        rules: vec![user],
        ..Default::default()
    };

    let a = Action::new_shell_from_source("npm install axios".to_string(), None, "agent", None);
    let r = decide(&a, &policy);
    assert_eq!(r.decision, Decision::Deny);
    assert_eq!(r.matched_rule.as_deref(), Some("user-deny-npm"));
}

#[test]
fn first_match_wins_within_user_rules() {
    // Specific rule before broad rule: the first matching rule decides.
    let specific = deny_rule("user-deny-npm-install", "npm", "install");
    let broad = deny_rule("user-deny-npm-all", "npm", "");
    let policy = PolicySet {
        rules: vec![specific, broad],
        ..Default::default()
    };

    let a = Action::new_shell_from_source("npm install axios".to_string(), None, "agent", None);
    let r = decide(&a, &policy);
    assert_eq!(r.matched_rule.as_deref(), Some("user-deny-npm-install"));

    // And the broad rule still catches npm invocations the specific one misses.
    let a2 = Action::new_shell_from_source("npm publish".to_string(), None, "agent", None);
    let r2 = decide(&a2, &policy);
    assert_eq!(r2.matched_rule.as_deref(), Some("user-deny-npm-all"));
}

#[test]
fn unmatched_action_falls_through_to_allow() {
    // No rule matches → Allow with the risk engine's evaluation.
    let policy = PolicySet::default();
    let a = Action::new_shell_from_source("echo hello".to_string(), None, "agent", None);
    let r = decide(&a, &policy);
    assert_eq!(r.decision, Decision::Allow);
}

#[test]
fn source_field_never_read_by_matcher() {
    // The matcher must ignore the action's claimed source. A rule that
    // matches a command applies to every source, and the absence of a
    // "source" match field means a policy cannot whitelist one origin.
    let rule = deny_rule("user-deny-rm", "rm", "-rf");
    let policy = PolicySet {
        rules: vec![rule],
        ..Default::default()
    };

    for src in ["agent", "automation", "workflow", "human", "unknown"] {
        let a = Action::new_shell_from_source("rm -rf x".to_string(), None, src, None);
        let r = decide(&a, &policy);
        assert_eq!(
            r.decision,
            Decision::Deny,
            "source '{src}' must not bypass the rule"
        );
    }
}

#[test]
fn lint_rejects_empty_match_spec() {
    // A rule with no criteria would match everything or nothing depending on
    // implementation — `actionguard policy-lint` must reject it.
    use actionguard_lib::policy::loader::lint_file;

    let path = std::env::temp_dir().join(format!(
        "actionguard-bypass-lint-{}.yml",
        std::process::id()
    ));
    std::fs::write(
        &path,
        "version: 1\nscope: test\nrules:\n  - id: empty\n    match: {}\n    action: deny\n",
    )
    .unwrap();
    let r = lint_file(&path);
    std::fs::remove_file(&path).ok();
    assert!(r.is_err(), "empty match spec must be rejected");
}
