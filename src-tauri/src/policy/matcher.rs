//! Rule matcher — decides which rule fires for a given `Action`.
//!
//! Evaluation order within a `PolicySet`:
//!   - Iterate rules in document order (user rules first, then built-ins).
//!   - The FIRST rule whose `MatchSpec` matches the action wins.
//!   - If no rule matches, the engine falls back to the deterministic risk
//!     engine (`risk::evaluate_action`) and returns `Decision::Allow` with
//!     whatever risk level the risk engine computes.

use crate::models::{
    Action, ActionCategory, Decision, DecisionResult, PolicySet, RiskLevel, Rule,
};
use dirs;

/// Expand `~` to the user's home directory for path matching.
fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy();
            if stripped.is_empty() {
                return home_str.to_string();
            }
            return format!("{}{}", home_str, stripped);
        }
    }
    path.to_string()
}

/// Decide what to do with an action. Returns the matched rule's decision
/// (or the fallback allow + risk engine result when nothing matched).
pub fn decide(action: &Action, policy: &PolicySet) -> DecisionResult {
    for rule in &policy.rules {
        if matches_spec(action, rule) {
            return DecisionResult {
                decision: rule.action,
                matched_rule: Some(rule.id.clone()),
                risk: rule.risk.unwrap_or(RiskLevel::Low),
                reason: rule.reason.clone().unwrap_or_default(),
            };
        }
    }
    decide_with_fallback(action)
}

/// Fallback path when no policy rule fires. Uses the deterministic risk
/// engine to compute the risk level, then returns Allow. The risk level
/// still surfaces in the UI so a HIGH-risk action that no rule covers will
/// still trip the approval gate (Phase C).
pub fn decide_with_fallback(action: &Action) -> DecisionResult {
    let r = crate::risk::evaluate_action(action);
    DecisionResult {
        decision: Decision::Allow,
        matched_rule: None,
        risk: r.level,
        reason: r.reasons.first().cloned().unwrap_or_default(),
    }
}

/// Does `action` match this rule's `MatchSpec`?
fn matches_spec(action: &Action, rule: &Rule) -> bool {
    let m = &rule.match_;

    // Category filter (exact match if Some).
    if let Some(cat) = m.category {
        if action.category != cat {
            return false;
        }
    }

    // Command filter. We match against the action's `kind` (canonical verb)
    // OR the first token of `target` for shell/git/package actions.
    if let Some(cmd) = &m.command {
        if !command_matches(action, cmd) {
            return false;
        }
    }

    // Path filter (wildcard). Empty for non-File categories.
    if let Some(pattern) = &m.path {
        let path = expand_tilde(action.path_str());
        if path.is_empty() || !wildcard_match(pattern, &path) {
            return false;
        }
    }

    // args_contains: every substring must appear somewhere in the target
    // (AND semantics). Use it for required arguments that are all mandatory.
    if let Some(needles) = &m.args_contains {
        // Expand ~ so `~/.aws/credentials` matches `.aws/credentials`.
        let target = expand_tilde(action.target_str()).to_lowercase();
        for n in needles {
            if !target.contains(&n.to_lowercase()) {
                return false;
            }
        }
    }

    // args_any: at least ONE substring must appear (OR semantics). Use it for
    // synonym flags such as `-9` / `-KILL` / `-SIGKILL` — with `args_contains`
    // those rules would silently never fire for every spelling but the first.
    if let Some(needles) = &m.args_any {
        let target = expand_tilde(action.target_str()).to_lowercase();
        if !needles.iter().any(|n| target.contains(&n.to_lowercase())) {
            return false;
        }
    }

    // Regex on the full target string (path for File, command for others).
    if let Some(pattern) = &m.regex {
        match regex::Regex::new(pattern) {
            Ok(re) => {
                let target = action.target_str();
                // Apply regex on the full command string for shell/git/package
                // (which includes the command + args), and on the path for File.
                let candidate = if action.category == ActionCategory::File {
                    path_or_target(action)
                } else {
                    target
                };
                if !re.is_match(candidate) {
                    return false;
                }
            }
            // A malformed regex in a rule should never silently fail to match.
            // Treat it as a non-match (the rule will be ignored) and surface
            // the issue via `lint_file`.
            Err(_) => return false,
        }
    }

    true
}

fn command_matches(action: &Action, expected: &str) -> bool {
    if action.category == ActionCategory::File {
        return false;
    }
    let target = action.target_str();
    let mut tokens = target.split_whitespace();
    let first = tokens.next().unwrap_or("");
    // For Git actions, the first token is always "git" — the meaningful
    // verb is the SECOND token (push, reset, clean, branch, …).
    if action.category == ActionCategory::Git {
        let second = tokens.next().unwrap_or("");
        return second.eq_ignore_ascii_case(expected);
    }
    first.eq_ignore_ascii_case(expected)
}

/// Path used as the regex candidate for File actions. Falls back to the
/// target string if path is empty (defensive — shouldn't happen for File).
fn path_or_target(action: &Action) -> &str {
    let p = action.path_str();
    if !p.is_empty() {
        p
    } else {
        action.target_str()
    }
}

/// Simple wildcard matcher supporting `*` (any run of chars) and `?` (any
/// single char). Patterns are matched against the workspace-relative path
/// with `/` as the separator. `**` is treated as `*` for simplicity — the
/// rules files use `**` for documentation but the matcher is glob-style.
fn wildcard_match(pattern: &str, name: &str) -> bool {
    // Normalize backslashes (Windows paths) to forward slashes.
    let pat = pattern.replace('\\', "/");
    let nam = name.replace('\\', "/");
    // Treat `**` as `*` (the matcher is glob-style, not regex).
    let pat = pat.replace("**", "*");
    let p: Vec<char> = pat.chars().collect();
    let n: Vec<char> = nam.chars().collect();
    let (pl, nl) = (p.len(), n.len());
    let mut dp = vec![vec![false; nl + 1]; pl + 1];
    dp[0][0] = true;
    for i in 1..=pl {
        if p[i - 1] == '*' {
            dp[i][0] = dp[i - 1][0];
        }
    }
    for i in 1..=pl {
        for j in 1..=nl {
            match p[i - 1] {
                '*' => dp[i][j] = dp[i - 1][j] || dp[i][j - 1],
                '?' => dp[i][j] = dp[i - 1][j - 1],
                c => dp[i][j] = dp[i - 1][j - 1] && c == n[j - 1],
            }
        }
    }
    dp[pl][nl]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ActionCategory, ActionKind, MatchSpec, PolicySource};

    fn rule(id: &str, m: MatchSpec, action: Decision) -> Rule {
        Rule {
            id: id.to_string(),
            match_: m,
            action,
            risk: None,
            reason: None,
            source: PolicySource::Builtin,
        }
    }

    fn shell_action(target: &str) -> Action {
        Action::new_shell(target.to_string(), None, Some("claude-code".to_string()))
    }

    fn file_action(path: &str, kind: ActionKind) -> Action {
        Action::new_file(path.to_string(), kind)
    }

    #[test]
    fn deny_rm_rf_root() {
        let set = PolicySet {
            rules: vec![rule(
                "deny-rm-rf-root",
                MatchSpec {
                    category: Some(ActionCategory::Shell),
                    command: Some("rm".to_string()),
                    args_contains: Some(vec!["-rf".to_string()]),
                    args_any: None,
                    regex: Some(r"^rm\s+.*((\s/)|~(\/|$)|\*(\s|$)|\$HOME)".to_string()),
                    path: None,
                },
                Decision::Deny,
            )],
        };
        // Real root / home / glob deletions are hard-denied…
        for target in ["rm -rf /", "rm -rf /etc", "rm -rf ~", "rm -rf ~/x", "rm -rf *", "rm -rf $HOME"] {
            let r = decide(&shell_action(target), &set);
            assert_eq!(r.decision, Decision::Deny, "expected deny for `{target}`");
            assert_eq!(r.matched_rule.as_deref(), Some("deny-rm-rf-root"));
        }
        // …but a scoped relative path is NOT a root wipe and must fall through
        // to the ask rules (this is the over-broad regression we fixed).
        for target in ["rm -rf ./dist", "rm -rf build/", "rm -rf src/foo.ts"] {
            let r = decide(&shell_action(target), &set);
            assert_ne!(r.decision, Decision::Deny, "expected non-deny for `{target}`");
        }
    }

    #[test]
    fn args_any_matches_any_one_synonym() {
        // Mirrors confirm-kill-9: `-9` / `-KILL` / `-SIGKILL` are synonyms and
        // must each trip the rule — `args_contains` (all-of) would only match `-9`.
        let set = PolicySet {
            rules: vec![rule(
                "confirm-kill-9",
                MatchSpec {
                    category: Some(ActionCategory::Shell),
                    command: Some("kill".to_string()),
                    args_contains: None,
                    args_any: Some(vec!["-9".to_string(), "-kill".to_string(), "-sigkill".to_string()]),
                    regex: None,
                    path: None,
                },
                Decision::Ask,
            )],
        };
        for target in ["kill -9 123", "kill -KILL 123", "kill -SIGKILL 123"] {
            let r = decide(&shell_action(target), &set);
            assert_eq!(r.decision, Decision::Ask, "expected ask for `{target}`");
        }
        // Plain `kill 123` (no signal flag) must NOT match.
        let r = decide(&shell_action("kill 123"), &set);
        assert_ne!(r.decision, Decision::Ask);
    }

    #[test]
    fn command_filter_matches_first_token_only() {
        let set = PolicySet {
            rules: vec![rule(
                "allow-npm-install",
                MatchSpec {
                    category: Some(ActionCategory::Package),
                    command: Some("npm".to_string()),
                    args_contains: Some(vec!["install".to_string()]),
                    args_any: None,
                    regex: None,
                    path: None,
                },
                Decision::Allow,
            )],
        };
        // "npm install axios" should match (command=npm, args contains "install").
        // The bridge will classify the action's category as Package, not Shell,
        // so we construct one manually here to mirror the real flow.
        let mut a = Action::new_shell("npm install axios".to_string(), None, Some("claude-code".to_string()));
        a.category = ActionCategory::Package;
        let r = decide(&a, &set);
        assert_eq!(r.decision, Decision::Allow);
        assert_eq!(r.matched_rule.as_deref(), Some("allow-npm-install"));
    }

    #[test]
    fn deny_write_env_file() {
        let set = PolicySet {
            rules: vec![rule(
                "deny-write-env",
                MatchSpec {
                    category: Some(ActionCategory::File),
                    path: Some("*.env".to_string()),
                    command: None,
                    args_contains: None,
                    args_any: None,
                    regex: None,
                },
                Decision::Deny,
            )],
        };
        let a = file_action(".env", ActionKind::Modify);
        let r = decide(&a, &set);
        assert_eq!(r.decision, Decision::Deny);
        assert_eq!(r.matched_rule.as_deref(), Some("deny-write-env"));
    }

    #[test]
    fn no_match_falls_back_to_allow_with_risk() {
        let set = PolicySet { rules: vec![] };
        // A delete on a regular file: no rule, fallback path. Risk engine
        // returns HIGH for deletes, so the decision is Allow but risk=HIGH.
        let a = file_action("src/foo.ts", ActionKind::Delete);
        let r = decide(&a, &set);
        assert_eq!(r.decision, Decision::Allow);
        assert!(r.matched_rule.is_none());
        assert_eq!(r.risk, RiskLevel::High);
    }

    #[test]
    fn first_match_wins() {
        let set = PolicySet {
            rules: vec![
                rule(
                    "broad",
                    MatchSpec {
                        category: Some(ActionCategory::Shell),
                        command: Some("rm".to_string()),
                        args_contains: None,
                        args_any: None,
                        regex: None,
                        path: None,
                    },
                    Decision::Deny,
                ),
                rule(
                    "narrow",
                    MatchSpec {
                        category: Some(ActionCategory::Shell),
                        command: Some("rm".to_string()),
                        args_contains: Some(vec!["-rf".to_string()]),
                        args_any: None,
                        regex: None,
                        path: None,
                    },
                    Decision::Ask,
                ),
            ],
        };
        // `broad` matches first (just command=rm), so Deny wins even though
        // `narrow` would also match. Rule authors must order specific→broad.
        let a = shell_action("rm -rf dist");
        let r = decide(&a, &set);
        assert_eq!(r.decision, Decision::Deny);
        assert_eq!(r.matched_rule.as_deref(), Some("broad"));
    }

    #[test]
    fn wildcard_path_matches_directory_prefix() {
        assert!(wildcard_match(".aws/*", ".aws/credentials"));
        assert!(wildcard_match(".gnupg/**", ".gnupg/pubring.kbx"));
        assert!(!wildcard_match("*.env", "src/.envhelper"));
        // `*.env` matches files ENDING in `.env` (e.g. `prod.env`).
        assert!(wildcard_match("*.env", "prod.env"));
        assert!(!wildcard_match("*.env", ".env.production"));
        // `.env.*` matches files STARTING with `.env.` (e.g. `.env.production`).
        assert!(wildcard_match(".env.*", ".env.production"));
        assert!(wildcard_match(".env.*", ".env.local"));
        assert!(wildcard_match("credentials.*", "credentials.json"));
    }

    #[test]
    fn command_filter_never_matches_file_actions() {
        let set = PolicySet {
            rules: vec![rule(
                "weird-rule",
                MatchSpec {
                    category: Some(ActionCategory::File),
                    command: Some("rm".to_string()),
                    args_contains: None,
                    args_any: None,
                    regex: None,
                    path: None,
                },
                Decision::Deny,
            )],
        };
        // A file action with a command filter — should NOT match.
        let a = file_action("src/foo.ts", ActionKind::Delete);
        let r = decide(&a, &set);
        assert_eq!(r.decision, Decision::Allow);
        assert!(r.matched_rule.is_none());
    }
}
