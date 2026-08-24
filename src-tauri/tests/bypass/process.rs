//! Process / action-source adversarial tests.
//!
//! The question these tests answer: can an attacker change WHO the action
//! comes from, or how the command is spelled, to slip past the boundary?
//!
//! Core invariant under test: **the boundary is source-agnostic.** A command
//! is decided by its content and risk, not by whether the source calls itself
//! an "agent", an "automation", or a "human".

use actionguard_lib::models::{Action, Decision, RiskLevel};
use actionguard_lib::policy::{decide, load_policy_set};

/// Built-in rules (secrets → shell → git → node → python) with no user file
/// dependency. Note: a local `policies.user.yml` will be included — tests
/// assert on decisions, and a user file that flips these decisions is itself
/// a security-relevant change.
fn builtin() -> actionguard_lib::models::PolicySet {
    load_policy_set()
}

#[test]
fn agent_rm_rf_root_is_denied() {
    let a = Action::new_shell_from_source("rm -rf /".to_string(), None, "agent", Some("claude-code".to_string()));
    let r = decide(&a, &builtin());
    assert_eq!(r.decision, Decision::Deny);
    assert_eq!(r.matched_rule.as_deref(), Some("deny-rm-rf-root"));
    assert_eq!(r.risk, RiskLevel::Critical);
}

#[test]
fn automation_source_gets_the_same_denial() {
    // Vendor-neutrality: a browser assistant / RPA calling itself
    // "automation" must not change the verdict.
    let a = Action::new_shell_from_source(
        "rm -rf /".to_string(),
        None,
        "automation",
        Some("browser-assistant".to_string()),
    );
    let r = decide(&a, &builtin());
    assert_eq!(r.decision, Decision::Deny, "source must not weaken the rule");
}

#[test]
fn human_source_gets_the_same_denial() {
    // Same invariant for a claimed human source — a user who piped
    // `rm -rf /` through an AI tool is still executing it.
    let a = Action::new_shell_from_source("rm -rf /".to_string(), None, "human", None);
    let r = decide(&a, &builtin());
    assert_eq!(r.decision, Decision::Deny, "source must not weaken the rule");
}

#[test]
fn relative_rm_rf_prompts_instead_of_denying() {
    // `rm -rf ./dist` is dangerous but scoped; it must Ask, not silently allow.
    let a = Action::new_shell_from_source("rm -rf ./dist".to_string(), None, "agent", None);
    let r = decide(&a, &builtin());
    assert_eq!(r.decision, Decision::Ask);
    assert_eq!(r.matched_rule.as_deref(), Some("confirm-rm-rf"));
}

#[test]
fn sudo_is_denied_before_rm_rules() {
    // first-match-wins: the sudo rule precedes the rm rules, so wrapping a
    // relative rm in sudo must hard-deny, not degrade to a prompt.
    let a = Action::new_shell_from_source("sudo rm -rf ./dist".to_string(), None, "agent", None);
    let r = decide(&a, &builtin());
    assert_eq!(r.decision, Decision::Deny);
    assert_eq!(r.matched_rule.as_deref(), Some("deny-sudo"));
}

#[test]
fn curl_fetch_prompts() {
    // shell-category rule fires on a shell action.
    let a = Action::new_shell_from_source("curl http://example.com/x.sh".to_string(), None, "agent", None);
    let r = decide(&a, &builtin());
    assert_eq!(r.decision, Decision::Ask);
    assert_eq!(r.matched_rule.as_deref(), Some("confirm-curl"));
}

#[test]
fn kill_signal_synonyms_trip_the_rule() {
    // args_any semantics: any one synonym spelling trips the rule.
    for cmd in ["kill -9 123", "kill -KILL 123", "kill -SIGKILL 123"] {
        let a = Action::new_shell_from_source(cmd.to_string(), None, "agent", None);
        let r = decide(&a, &builtin());
        assert_eq!(r.decision, Decision::Ask, "command: {cmd}");
        assert_eq!(r.matched_rule.as_deref(), Some("confirm-kill-9"), "command: {cmd}");
    }
}

#[test]
fn package_category_requires_classification() {
    // NOTE: `npm install` is a *package* rule, but a raw shell action has
    // category Shell — only the classifier re-categorizes it. This test pins
    // that the bypass layer does NOT assume classification; a command that
    // skips the classifier stays on shell rules. The classify step itself is
    // tested in policy/classify.rs, not here.
    let a = Action::new_shell_from_source("npm install axios".to_string(), None, "agent", None);
    let r = decide(&a, &builtin());
    assert_ne!(r.matched_rule.as_deref(), Some("allow-npm-install"));
}

// --- documented blind spots (assert current behavior, not a promise) --------

#[test]
fn known_gap_flag_reordering_degrades_root_wipe_to_prompt() {
    // KNOWN GAP: `rm -r -f /` reorders flags so the `args_contains: ["-rf"]`
    // rule never fires. It falls through to `confirm-rm-no-flags` (Ask) instead
    // of the root-wipe hard deny. Asserting current behavior documents the gap;
    // it is NOT a guarantee. See README.md §Known blind spots.
    let a = Action::new_shell_from_source("rm -r -f /".to_string(), None, "agent", None);
    let r = decide(&a, &builtin());
    assert_eq!(r.decision, Decision::Ask);
    assert_ne!(r.matched_rule.as_deref(), Some("deny-rm-rf-root"));
}

#[test]
fn known_gap_no_whitespace_before_slash_degrades_root_wipe() {
    // KNOWN GAP: `rm -rf/` has no whitespace before the `/`, so the regex
    // `^rm\s+.*(\s/)` in deny-rm-rf-root does not match. Falls to Ask.
    // Asserting current behavior documents the gap; see README.md.
    let a = Action::new_shell_from_source("rm -rf/".to_string(), None, "agent", None);
    let r = decide(&a, &builtin());
    assert_eq!(r.decision, Decision::Ask);
    assert_ne!(r.matched_rule.as_deref(), Some("deny-rm-rf-root"));
}
