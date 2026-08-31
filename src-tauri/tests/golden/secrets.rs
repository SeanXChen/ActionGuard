//! secrets.yml 黄金回归用例（19 条规则 + P1 扩展）。
//! 本套件如实断言当前行为——包括两处已知缺陷（见 Backlog B013 / B014），
//! 修复它们时这里必须同步更新。

use actionguard_lib::models::{ActionKind, Decision, RiskLevel};
use crate::golden::{decide_cmd, decide_file};

#[test]
fn writes_to_secret_files_are_denied() {
    assert_decision!(
        decide_file(".env", ActionKind::Modify),
        Decision::Deny,
        "deny-write-env",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file(".env.production", ActionKind::Modify),
        Decision::Deny,
        "deny-write-env-variant",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file(".env.local", ActionKind::Modify),
        Decision::Deny,
        "deny-write-env-variant",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file("id_rsa", ActionKind::Modify),
        Decision::Deny,
        "deny-write-ssh-id",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file("id_ed25519", ActionKind::Modify),
        Decision::Deny,
        "deny-write-ssh-id-ed25519",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file(".aws/credentials", ActionKind::Modify),
        Decision::Deny,
        "deny-write-aws-creds",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file(".gnupg/pubring.kbx", ActionKind::Modify),
        Decision::Deny,
        "deny-write-gnupg",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file("credentials.json", ActionKind::Modify),
        Decision::Deny,
        "deny-write-credentials",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file("credentials.yml", ActionKind::Modify),
        Decision::Deny,
        "deny-write-credentials",
        RiskLevel::Critical
    );
}

#[test]
fn deletes_of_private_keys_are_denied() {
    assert_decision!(
        decide_file("server.pem", ActionKind::Delete),
        Decision::Deny,
        "deny-delete-pem",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_file("tls.key", ActionKind::Delete),
        Decision::Deny,
        "deny-delete-key",
        RiskLevel::Critical
    );
}

#[test]
fn reading_env_exposes_secrets_asks() {
    // P1: read-* rules come before confirm-* in secrets.yml — they match first.
    assert_decision!(
        decide_cmd("cat .env"),
        Decision::Ask,
        "read-env-file",
        RiskLevel::Critical
    );
    // 已知缺陷 B013：read-env-prod / read-env-local 被 read-env-file 遮蔽
    //（first-match-wins），这里如实锁定当前行为。
    assert_decision!(
        decide_cmd("cat .env.production"),
        Decision::Ask,
        "read-env-file",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat .env.local"),
        Decision::Ask,
        "read-env-file",
        RiskLevel::Critical
    );
}

#[test]
fn reading_keys_and_credentials_asks() {
    // P1: read-* rules match first; more specific rules (ssh, env, etc.) take precedence.
    assert_decision!(
        decide_cmd("cat server.pem"),
        Decision::Ask,
        "read-pem-key",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat tls.key"),
        Decision::Ask,
        "read-private-key",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat ~/.ssh/id_rsa"),
        Decision::Ask,
        "read-ssh-private-key",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat ~/.ssh/id_ed25519"),
        Decision::Ask,
        "read-ssh-ed25519-key",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat credentials.json"),
        Decision::Ask,
        "read-credentials-json",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat credentials.yml"),
        Decision::Ask,
        "read-credentials-yml",
        RiskLevel::Critical
    );
}

#[test]
fn reading_aws_credentials_is_covered() {
    // P1: read-aws-credentials rule covers cat ~/.aws/credentials — B014 is fixed.
    assert_decision!(
        decide_cmd("cat ~/.aws/credentials"),
        Decision::Ask,
        "read-aws-credentials",
        RiskLevel::Critical
    );
}

#[test]
fn benign_reads_are_not_interrupted() {
    assert_fallback_allow!(decide_cmd("cat config.yaml"));
    assert_fallback_allow!(decide_cmd("cat README.md"));
}
