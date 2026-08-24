//! Path-normalization bypass attempts.
//!
//! The question these tests answer: can an attacker rewrite a path so that
//! sensitive-resource detection or rule matching stops seeing it?
//!
//! Matrix coverage: dot-dot, backslash/slash, case variants, 8.3 short names,
//! trailing separators. Known blind spots are documented in README.md.

use actionguard_lib::models::{Action, ActionKind, AssetKind};
use actionguard_lib::risk::{detect_asset, is_sensitive_path};

// --- dot-dot (..) traversal ------------------------------------------------

#[test]
fn dotdot_escape_does_not_bypass_sensitive_detection() {
    // `..` must never hide an env file from the sensitive-resource detector.
    for p in [
        "..\\.env",
        "..\\..\\.env",
        "C:\\project\\..\\..\\.env",
        "../../.env",
        ".\\sub\\..\\.env",
        "C:/project/../../.env",
    ] {
        assert!(
            is_sensitive_path(p),
            "dot-dot escape should not bypass sensitive detection: {p}"
        );
    }
}

#[test]
fn dotdot_traversal_still_matches_asset_kind() {
    // And it must still be classified as the *right* asset kind.
    for p in [
        "..\\.env",
        "..\\..\\..\\id_rsa",
        "..\\.aws\\credentials",
        "C:\\x\\..\\.gnupg\\trustdb.gpg",
    ] {
        let asset = detect_asset(p).unwrap_or_else(|| panic!("asset should be found: {p}"));
        let expected = if p.ends_with(".env") {
            AssetKind::EnvFile
        } else if p.ends_with("id_rsa") {
            AssetKind::SshKey
        } else if p.contains("credentials") {
            AssetKind::AwsCreds
        } else {
            AssetKind::GpgKeychain
        };
        assert_eq!(
            std::mem::discriminant(&asset.kind),
            std::mem::discriminant(&expected),
            "asset kind for {p}"
        );
    }
}

// --- backslash / forward-slash equivalence ---------------------------------

#[test]
fn backslash_and_slash_equivalent_for_sensitive_paths() {
    assert_eq!(
        is_sensitive_path("C:\\proj\\.env"),
        is_sensitive_path("C:/proj/.env"),
    );
}

// --- case variants ----------------------------------------------------------

#[test]
fn case_variants_do_not_bypass_sensitive_detection() {
    // Windows filesystems are case-insensitive; the detector must be too.
    for p in [".ENV", ".Env", "c:\\proj\\.ENV", "C:\\PROJ\\.Env", "ID_RSA"] {
        assert!(
            is_sensitive_path(p),
            "case variant should be detected: {p}"
        );
    }
}

// --- 8.3 short names and trailing separators -------------------------------

#[test]
fn short_name_parent_dir_does_not_hide_env() {
    // Windows 8.3 short-name parent (PROJEC~1) must not hide a `.env` child.
    assert!(is_sensitive_path("C:\\Users\\dev\\PROJEC~1\\.env"));
}

#[test]
fn known_gap_trailing_separator_hides_asset() {
    // KNOWN GAP: `C:\proj\.env\` (trailing separator) defeats detect_asset —
    // the filename segment becomes empty after the split. On Windows a
    // trailing separator normally denotes a directory, so this is an edge
    // case, but an attacker-controlled path spellings can still reach it.
    // Asserting the current behavior documents the gap; it is NOT a promise.
    assert!(!is_sensitive_path("C:\\proj\\.env\\"));
    assert!(!is_sensitive_path("C:\\proj\\.env/"));
}

// --- rule matching on rewritten paths ---------------------------------------

#[test]
fn env_rule_fires_on_dotdot_path_variant() {
    // A user rule `path: "**/.env"` must still fire when the agent rewrites
    // the path through `..`. We exercise the full decide() path.
    use actionguard_lib::models::{
        ActionCategory, Decision, MatchSpec, PolicySet, PolicySource, RiskLevel, Rule,
    };
    use actionguard_lib::policy::decide;

    let rule = Rule {
        id: "test-protect-env".to_string(),
        match_: MatchSpec {
            category: Some(ActionCategory::File),
            command: None,
            path: Some("*.env".to_string()),
            args_contains: None,
            args_any: None,
            regex: None,
        },
        action: Decision::Deny,
        risk: Some(RiskLevel::Critical),
        reason: Some("env files are secrets".to_string()),
        source: PolicySource::User,
    };
    let policy = PolicySet {
        rules: vec![rule.clone()],
        ..Default::default()
    };

    for p in ["C:\\proj\\..\\.env", "..\\..\\.env", "sub\\..\\.env"] {
        let a = Action::new_file(p.to_string(), ActionKind::Modify);
        let r = decide(&a, &policy);
        assert_eq!(r.decision, Decision::Deny, "path variant must be denied: {p}");
        assert_eq!(r.matched_rule.as_deref(), Some("test-protect-env"));
    }
}

// --- documented blind spot (asserts current behavior, not a promise) --------

#[test]
fn wildcard_match_is_case_sensitive() {
    // KNOWN GAP: rule `path` matching is case-sensitive while `detect_asset`
    // is case-insensitive. On Windows this is an asymmetry an attacker can
    // exploit if they control the exact case. Asserting the current behavior
    // documents the gap; it is NOT a guarantee it stays this way.
    use actionguard_lib::models::{
        ActionCategory, Decision, MatchSpec, PolicySet, PolicySource, RiskLevel, Rule,
    };
    use actionguard_lib::policy::decide;

    let rule = Rule {
        id: "case-gap".to_string(),
        match_: MatchSpec {
            category: Some(ActionCategory::File),
            command: None,
            path: Some("*.env".to_string()),
            args_contains: None,
            args_any: None,
            regex: None,
        },
        action: Decision::Deny,
        risk: Some(RiskLevel::Critical),
        reason: Some("demonstrates case-sensitive matching".to_string()),
        source: PolicySource::User,
    };
    let policy = PolicySet {
        rules: vec![rule],
        ..Default::default()
    };

    // Matches (same case as pattern)
    let lower = Action::new_file("C:\\proj\\.env".to_string(), ActionKind::Modify);
    assert_eq!(decide(&lower, &policy).decision, Decision::Deny);

    // Does NOT match (different case) — documented gap, see README.md §Known blind spots
    let upper = Action::new_file("C:\\proj\\.ENV".to_string(), ActionKind::Modify);
    assert_ne!(decide(&upper, &policy).decision, Decision::Deny);
}
