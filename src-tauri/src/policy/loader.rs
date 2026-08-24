//! Loader for built-in + user policy rules.
//!
//! Built-in community rules live under `src-tauri/rules/*.yml` and are
//! compiled into the binary via `include_str!`. User rules live in
//! `~/.actionguard/policies.user.yml` and can override built-ins by using
//! the same `id`.

use crate::models::{PolicyFile, PolicySource, PolicySet, Rule};
use crate::storage;
use anyhow::{Context, Result};

// Built-in community rule files. Order matters for the loader's "first match
// wins" semantics within the builtin set — secrets are evaluated first so a
// deny on .env shadows any later allow on a generic file write.
const BUILTIN_RULES: &[(&str, &str)] = &[
    ("secrets", include_str!("../../rules/secrets.yml")),
    ("shell", include_str!("../../rules/shell.yml")),
    ("git", include_str!("../../rules/git.yml")),
    ("node", include_str!("../../rules/node.yml")),
    ("python", include_str!("../../rules/python.yml")),
];

/// Public view of the built-in rule files `(name, yaml)` — used by
/// `actionguard setup` to seed `~/.actionguard/rules/` so users can inspect
/// and contribute rules without digging into the repo.
pub fn builtin_rule_files() -> &'static [(&'static str, &'static str)] {
    BUILTIN_RULES
}

/// Strip a leading UTF-8 BOM (`U+FEFF`).
///
/// Windows editors (notepad, PowerShell 5.1 `Set-Content -Encoding UTF8`)
/// write a BOM by default. serde_yaml rejects the stream with
/// `missing field … at line 1 column 2`, which would silently disable user
/// rules. Measured 2026-08-19 during real-agent enforcement tests.
pub fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{FEFF}').unwrap_or(s)
}

/// Parse a YAML rules file. Returns an empty policy on parse failure so a
/// malformed built-in file does not abort startup.
fn parse(name: &str, body: &str) -> PolicyFile {
    match serde_yaml::from_str::<PolicyFile>(strip_bom(body)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "actionguard policy: failed to parse builtin rule file `{}`: {}",
                name, e
            );
            PolicyFile::default()
        }
    }
}

/// Load the complete policy set: user rules first (highest priority), then
/// built-in rules in the order declared in `BUILTIN_RULES`.
///
/// Called once at startup and again when the user edits
/// `policies.user.yml`. The result is wrapped in `Arc` for cheap cloning
/// into the bridge thread.
///
/// Precedence: User → Project → Builtin.
///
/// Security invariant: only User may relax the boundary. Project rules
/// (v0.3) will be inserted here, between User and Builtin, and their
/// `action` clamped to Deny/Ask at load time — a Project rule can make
/// ActionGuard stricter, never weaker.
pub fn load_policy_set() -> PolicySet {
    let mut rules: Vec<Rule> = Vec::new();

    // 1) User rules — highest priority.
    let user = storage::load_policies_user();
    for mut r in user.rules {
        r.source = PolicySource::User;
        rules.push(r);
    }

    // 2) Built-in rules in declared order.
    for (name, body) in BUILTIN_RULES {
        let parsed = parse(name, body);
        for mut r in parsed.rules {
            r.source = PolicySource::Builtin;
            rules.push(r);
        }
    }

    PolicySet { rules }
}

/// Lint a YAML rules file: parse + structural checks. Used by
/// `actionguard policy-lint`.
pub fn lint_file(path: &std::path::Path) -> Result<PolicyFile> {
    let body = std::fs::read_to_string(path)
        .with_context(|| format!("read policy file {}", path.display()))?;
    let parsed: PolicyFile = serde_yaml::from_str(strip_bom(&body))
        .with_context(|| format!("parse policy file {}", path.display()))?;
    for r in &parsed.rules {
        if r.id.is_empty() {
            anyhow::bail!("rule with empty id");
        }
        if r.match_.category.is_none()
            && r.match_.command.is_none()
            && r.match_.path.is_none()
            && r.match_.args_contains.is_none()
            && r.match_.args_any.is_none()
            && r.match_.regex.is_none()
        {
            anyhow::bail!("rule `{}` has no match criteria", r.id);
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_rules_load_without_error() {
        let set = load_policy_set();
        assert!(!set.rules.is_empty(), "built-in rules should load");
        assert!(set.rules.iter().any(|r| r.id.starts_with("deny-rm")));
        assert!(set.rules.iter().any(|r| r.id.starts_with("confirm-reset-hard")));
        assert!(set.rules.iter().any(|r| r.id.starts_with("allow-npm-install")));
        assert!(set.rules.iter().any(|r| r.id.starts_with("allow-pip-install")));
        assert!(set.rules.iter().any(|r| r.id.starts_with("deny-write-env")));
    }

    #[test]
    fn user_rules_not_present_yet_all_builtin() {
        let set = load_policy_set();
        // With no user file on disk, every loaded rule is Builtin.
        assert!(set.rules.iter().all(|r| r.source == PolicySource::Builtin));
    }

    #[test]
    fn strip_bom_removes_only_leading_utf8_bom() {
        assert_eq!(strip_bom("\u{FEFF}version: 1"), "version: 1");
        assert_eq!(strip_bom("version: 1"), "version: 1");
        assert_eq!(strip_bom(""), "");
        // BOM not at the start must be left alone.
        assert_eq!(strip_bom("a\u{FEFF}b"), "a\u{FEFF}b");
    }

    #[test]
    fn bom_prefixed_yaml_parses_to_policy() {
        // Reproduces the real failure: `missing field scope at line 1 column 2`
        // when Windows writes a BOM. Parsing must succeed after stripping.
        let yaml = "\u{FEFF}version: 1\nscope: shell\nrules:\n  - id: bom-test\n    match:\n      command: ls\n    action: deny\n    risk: medium\n    reason: bom\n";
        let pf: PolicyFile = serde_yaml::from_str(strip_bom(yaml)).unwrap();
        assert_eq!(pf.scope, "shell");
        assert_eq!(pf.rules.len(), 1);
        assert_eq!(pf.rules[0].id, "bom-test");
    }
}
