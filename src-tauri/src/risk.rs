use crate::models::{
    Action, ActionCategory, ActionKind, Asset, AssetKind, Counts, RiskLevel, RiskResult,
};

/// Deterministic, 100% rule-based risk engine. No AI / LLM. Every rule is auditable.
///
/// BATCH-LEVEL (`evaluate`):
///   CRITICAL: any sensitive asset touched (.env, *.pem, .aws/credentials, …)
///   HIGH:     > 20 files changed · > 3 deletions · path outside workspace ·
///             large change (>= 50) with deletions
///   MEDIUM:   10+ files modified · 5+ renames · 1-3 deletions
///   LOW:      everything else
///
/// SINGLE-ACTION (`evaluate_action`):
///   CRITICAL: touching a sensitive asset (a read of .env.production is
///             surfaced by the Secret category once shell monitoring lands;
///             for v0.2 Phase A any touch of a sensitive path is Critical)
///   HIGH:     deletion · path outside workspace
///   MEDIUM:   (none for single file actions — they're either Critical or Low)
///   LOW:      small modify/create
pub fn evaluate(actions: &[Action]) -> RiskResult {
    let mut counts = Counts::default();
    let mut sensitive: Vec<String> = Vec::new();
    let mut outside: Vec<String> = Vec::new();
    let mut asset: Option<Asset> = None;

    for a in actions {
        counts.add(a.action);
        if a.sensitive {
            sensitive.push(a.path_str().to_string());
            if asset.is_none() {
                if let Some(detected) = detect_asset(a.path_str()) {
                    asset = Some(detected);
                }
            }
        }
        if a.outside {
            outside.push(a.path_str().to_string());
        }
    }

    let mut reasons: Vec<String> = Vec::new();
    let mut level = RiskLevel::Low;

    let total = counts.total();

    // ---- CRITICAL rules (sensitive asset touched) -----------------
    if !sensitive.is_empty() {
        level = RiskLevel::Critical;
        reasons.push(format!("{} sensitive file(s) affected", sensitive.len()));
    }

    // ---- HIGH rules ------------------------------------------------
    if total > 20 {
        if level < RiskLevel::High {
            level = RiskLevel::High;
        }
        reasons.push(format!("More than 20 files changed ({total})"));
    }
    if counts.delete > 3 {
        if level < RiskLevel::High {
            level = RiskLevel::High;
        }
        reasons.push(format!("{} deletions (limit is 3)", counts.delete));
    }
    if !outside.is_empty() {
        if level < RiskLevel::High {
            level = RiskLevel::High;
        }
        reasons.push(format!(
            "{} path(s) outside the protected workspace",
            outside.len()
        ));
    }
    if total >= 50 && counts.delete >= 5 {
        if level < RiskLevel::High {
            level = RiskLevel::High;
        }
        reasons.push(format!(
            "Large change ({total} files) involving deletions ({} DELETEs)",
            counts.delete
        ));
    }

    // ---- MEDIUM rules ----------------------------------------------
    if level == RiskLevel::Low {
        if counts.modify >= 10 {
            level = RiskLevel::Medium;
            reasons.push(format!("{} files modified", counts.modify));
        } else if counts.rename >= 5 {
            level = RiskLevel::Medium;
            reasons.push(format!("{} renames", counts.rename));
        } else if counts.delete >= 1 && counts.delete <= 3 {
            level = RiskLevel::Medium;
            reasons.push(format!("{} file(s) deleted", counts.delete));
        }
    }

    // De-duplicate reasons while keeping order.
    reasons.sort();
    reasons.dedup();

    RiskResult {
        level,
        reasons,
        sensitive,
        outside,
        asset,
    }
}

/// Per-ingest classification for a single action.
/// For Phase A this only meaningfully classifies File-category actions; Shell
/// and Package classification lands in Phase B (classify_shell / classify_package).
pub fn evaluate_action(a: &Action) -> RiskResult {
    match a.category {
        ActionCategory::File => evaluate_file_action(a),
        // Shell / Git / Package / Secret rules land in Phase B; for now
        // we still surface asset detection on Secret-category reads.
        ActionCategory::Secret => evaluate_secret_action(a),
        _ => evaluate_file_action(a),
    }
}

fn evaluate_file_action(a: &Action) -> RiskResult {
    let path = a.path_str();
    let mut sensitive: Vec<String> = Vec::new();
    let mut outside: Vec<String> = Vec::new();
    let mut asset: Option<Asset> = None;

    if a.sensitive {
        sensitive.push(path.to_string());
        asset = detect_asset(path);
    }
    if a.outside {
        outside.push(path.to_string());
    }

    let mut reasons: Vec<String> = Vec::new();
    let mut level = RiskLevel::Low;

    // CRITICAL: sensitive asset touched
    if !sensitive.is_empty() {
        level = RiskLevel::Critical;
        let kind = asset
            .as_ref()
            .map(|x| format!("{:?}", x.kind))
            .unwrap_or_else(|| "sensitive".to_string());
        reasons.push(format!("Sensitive asset touched ({kind})"));
    } else if a.outside {
        level = RiskLevel::High;
        reasons.push("Path outside the protected workspace".to_string());
    } else if matches!(a.action, ActionKind::Delete) {
        level = RiskLevel::High;
        reasons.push("File deleted".to_string());
    }

    RiskResult {
        level,
        reasons,
        sensitive,
        outside,
        asset,
    }
}

fn evaluate_secret_action(a: &Action) -> RiskResult {
    // Secret reads are always Critical in v0.2. Evidence (redacted key names)
    // is populated by `evidence::collect_evidence` in Phase C.
    let path = a.target_str();
    let asset = detect_asset(path);
    RiskResult {
        level: RiskLevel::Critical,
        reasons: vec!["Agent attempting to read sensitive asset".to_string()],
        sensitive: vec![path.to_string()],
        outside: Vec::new(),
        asset,
    }
}

// ===========================================================================
// Sensitive asset detection
// ===========================================================================

/// Is this file name considered sensitive?
/// Matches: .env, .env.*, *.pem, *.key, credentials.*, id_rsa, id_ed25519,
/// *.pfx, *.p12, *.ppk, *.gpg, *.asc, .aws/credentials, .gnupg/**, .ssh/id_*,
/// .git/config, .npmrc, .pypirc, .netrc
pub fn is_sensitive_path(path: &str) -> bool {
    detect_asset(path).is_some()
}

/// Detect the asset kind for a given path. Returns `None` for non-sensitive paths.
/// Patterns cover the v0.2 surface: env files, SSH keys, PEM keys, AWS creds,
/// GPG keyring, Git internals, credentials.json, plus .npmrc/.pypirc/.netrc.
pub fn detect_asset(path: &str) -> Option<Asset> {
    let normalized = path.replace('\\', "/");
    let name = match normalized.rsplit('/').next() {
        Some(n) if !n.is_empty() => n,
        _ => &normalized,
    };
    let lower = name.to_lowercase();

    // --- EnvFile: .env, .env.* ---
    if lower == ".env" || lower.starts_with(".env.") {
        return Some(asset(&lower, AssetKind::EnvFile));
    }

    // --- SshKey: id_rsa, id_ed25519, id_dsa, id_ecdsa ---
    if matches!(
        lower.as_str(),
        "id_rsa" | "id_ed25519" | "id_dsa" | "id_ecdsa"
    ) {
        return Some(asset(&lower, AssetKind::SshKey));
    }
    // Or any file under a .ssh/ directory matching id_*
    if normalized.contains("/.ssh/")
        && (lower.starts_with("id_") || lower == "config" || lower == "known_hosts")
    {
        return Some(asset(&lower, AssetKind::SshKey));
    }

    // --- PemKey: *.pem, *.key, *.pfx, *.p12, *.ppk, *.gpg, *.asc ---
    for ext in [".pem", ".key", ".pfx", ".p12", ".ppk"] {
        if lower.ends_with(ext) {
            return Some(asset(&lower, AssetKind::PemKey));
        }
    }

    // --- GpgKeychain: anything under .gnupg/ ---
    if normalized.contains("/.gnupg/")
        || normalized.starts_with(".gnupg/")
        || normalized == ".gnupg"
        || lower.ends_with(".gpg")
        || lower.ends_with(".asc")
    {
        return Some(asset(&lower, AssetKind::GpgKeychain));
    }

    // --- AwsCreds: .aws/credentials, .aws/config, ~/.aws/* ---
    if normalized.contains("/.aws/") {
        return Some(asset(&lower, AssetKind::AwsCreds));
    }
    // Top-level .aws at start of path
    if normalized.starts_with(".aws/") || normalized == ".aws" {
        return Some(asset(&lower, AssetKind::AwsCreds));
    }

    // --- GitDir: anything under .git/ ---
    if normalized.starts_with(".git/") || normalized.contains("/.git/") || normalized == ".git" {
        return Some(asset(&lower, AssetKind::GitDir));
    }

    // --- CredentialsJson: credentials.* (filename) ---
    if lower.starts_with("credentials.") || lower == "credentials" {
        return Some(asset(&lower, AssetKind::CredentialsJson));
    }

    // --- Other: .npmrc, .pypirc, .netrc ---
    if matches!(lower.as_str(), ".npmrc" | ".pypirc" | ".netrc") {
        return Some(asset(&lower, AssetKind::Other));
    }

    None
}

fn asset(pattern: &str, kind: AssetKind) -> Asset {
    Asset {
        kind,
        matched_pattern: pattern.to_string(),
        contains: Vec::new(),
        absolute_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ActionKind;

    fn change(path: &str, action: ActionKind) -> Action {
        Action::new(path.to_string(), action)
    }

    #[test]
    fn low_small_changes() {
        let actions = vec![
            change("src/a.ts", ActionKind::Modify),
            change("src/b.ts", ActionKind::Create),
        ];
        let r = evaluate(&actions);
        assert_eq!(r.level, RiskLevel::Low);
    }

    #[test]
    fn medium_ten_modifies() {
        let actions: Vec<Action> = (0..10)
            .map(|i| change(&format!("src/f{i}.ts"), ActionKind::Modify))
            .collect();
        let r = evaluate(&actions);
        assert_eq!(r.level, RiskLevel::Medium);
    }

    #[test]
    fn high_more_than_20() {
        let actions: Vec<Action> = (0..21)
            .map(|i| change(&format!("src/f{i}.ts"), ActionKind::Modify))
            .collect();
        let r = evaluate(&actions);
        assert_eq!(r.level, RiskLevel::High);
    }

    #[test]
    fn high_deletes() {
        let actions = vec![
            change("a.ts", ActionKind::Delete),
            change("b.ts", ActionKind::Delete),
            change("c.ts", ActionKind::Delete),
            change("d.ts", ActionKind::Delete),
        ];
        let r = evaluate(&actions);
        assert_eq!(r.level, RiskLevel::High);
    }

    #[test]
    fn critical_sensitive_env() {
        let mut a = change(".env", ActionKind::Modify);
        a.sensitive = true;
        let r = evaluate(&[a]);
        assert_eq!(r.level, RiskLevel::Critical);
        assert!(!r.sensitive.is_empty());
        assert!(matches!(
            r.asset.unwrap().kind,
            AssetKind::EnvFile
        ));
    }

    #[test]
    fn high_outside() {
        let mut a = change("../etc/hosts", ActionKind::Modify);
        a.outside = true;
        let r = evaluate(&[a]);
        assert_eq!(r.level, RiskLevel::High);
    }

    #[test]
    fn high_large_with_deletes() {
        let mut actions: Vec<Action> = (0..50)
            .map(|i| change(&format!("src/f{i}.ts"), ActionKind::Modify))
            .collect();
        for i in 0..6 {
            actions.push(change(&format!("legacy/g{i}.ts"), ActionKind::Delete));
        }
        let r = evaluate(&actions);
        assert_eq!(r.level, RiskLevel::High);
    }

    #[test]
    fn sensitive_names() {
        for p in [
            ".env",
            ".env.production",
            "keys/id_rsa",
            "cert.pem",
            "server.key",
            "credentials.json",
            "a/b/credentials.yml",
            "id_ed25519",
            "cert.pfx",
            ".aws/credentials",
            "subdir/.aws/config",
            ".gnupg/pubring.kbx",
            ".ssh/id_ecdsa",
            ".git/config",
            ".npmrc",
            ".netrc",
            ".pypirc",
        ] {
            assert!(is_sensitive_path(p), "should be sensitive: {p}");
        }
        for p in [
            "src/main.ts",
            "package.json",
            "README.md",
            "env.ts",
            "env.example",
            "src/.envhelper",
        ] {
            assert!(!is_sensitive_path(p), "should NOT be sensitive: {p}");
        }
    }

    #[test]
    fn detect_asset_kinds() {
        assert!(matches!(
            detect_asset(".env").map(|a| a.kind),
            Some(AssetKind::EnvFile)
        ));
        assert!(matches!(
            detect_asset("keys/id_rsa").map(|a| a.kind),
            Some(AssetKind::SshKey)
        ));
        assert!(matches!(
            detect_asset(".ssh/id_ed25519").map(|a| a.kind),
            Some(AssetKind::SshKey)
        ));
        assert!(matches!(
            detect_asset("cert.pem").map(|a| a.kind),
            Some(AssetKind::PemKey)
        ));
        assert!(matches!(
            detect_asset(".aws/credentials").map(|a| a.kind),
            Some(AssetKind::AwsCreds)
        ));
        assert!(matches!(
            detect_asset(".gnupg/trustdb.gpg").map(|a| a.kind),
            Some(AssetKind::GpgKeychain)
        ));
        assert!(matches!(
            detect_asset(".git/config").map(|a| a.kind),
            Some(AssetKind::GitDir)
        ));
        assert!(matches!(
            detect_asset("credentials.json").map(|a| a.kind),
            Some(AssetKind::CredentialsJson)
        ));
        assert!(matches!(
            detect_asset(".npmrc").map(|a| a.kind),
            Some(AssetKind::Other)
        ));
        assert!(matches!(
            detect_asset(".netrc").map(|a| a.kind),
            Some(AssetKind::Other)
        ));
        assert!(detect_asset("src/main.ts").is_none());
    }

    #[test]
    fn evaluate_action_env_critical() {
        let mut a = change(".env", ActionKind::Modify);
        a.sensitive = true;
        let r = evaluate_action(&a);
        assert_eq!(r.level, RiskLevel::Critical);
        assert!(matches!(r.asset.unwrap().kind, AssetKind::EnvFile));
    }

    #[test]
    fn evaluate_action_delete_high() {
        let a = change("src/old.ts", ActionKind::Delete);
        let r = evaluate_action(&a);
        assert_eq!(r.level, RiskLevel::High);
    }

    #[test]
    fn evaluate_action_outside_high() {
        let mut a = change("../etc/hosts", ActionKind::Modify);
        a.outside = true;
        let r = evaluate_action(&a);
        assert_eq!(r.level, RiskLevel::High);
    }

    #[test]
    fn evaluate_action_low_modify() {
        let a = change("src/foo.ts", ActionKind::Modify);
        let r = evaluate_action(&a);
        assert_eq!(r.level, RiskLevel::Low);
    }
}
