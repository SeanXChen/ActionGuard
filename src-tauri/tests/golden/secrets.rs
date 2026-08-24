//! secrets.yml 黄金回归用例（19 条规则）。
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
    assert_decision!(
        decide_cmd("cat .env"),
        Decision::Ask,
        "confirm-cat-env",
        RiskLevel::Critical
    );
    // 已知缺陷 B013：confirm-cat-env-prod / confirm-cat-env-local 被
    // confirm-cat-env 遮蔽（first-match-wins），这里如实锁定当前行为。
    assert_decision!(
        decide_cmd("cat .env.production"),
        Decision::Ask,
        "confirm-cat-env",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat .env.local"),
        Decision::Ask,
        "confirm-cat-env",
        RiskLevel::Critical
    );
}

#[test]
fn reading_keys_and_credentials_asks() {
    assert_decision!(
        decide_cmd("cat server.pem"),
        Decision::Ask,
        "confirm-cat-pem",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat tls.key"),
        Decision::Ask,
        "confirm-cat-key",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat ~/.ssh/id_rsa"),
        Decision::Ask,
        "confirm-cat-id-rsa",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat ~/.ssh/id_ed25519"),
        Decision::Ask,
        "confirm-cat-id-ed25519",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat credentials.json"),
        Decision::Ask,
        "confirm-cat-credentials-json",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("cat credentials.yml"),
        Decision::Ask,
        "confirm-cat-credentials-yml",
        RiskLevel::Critical
    );
}

#[test]
fn reading_aws_credentials_is_uncovered_today() {
    // 已知缺口 B014：写 .aws/credentials 被 deny-write-aws-creds 拦，
    // 但 `cat ~/.aws/credentials` 没有任何 confirm-cat-* 覆盖 → 回退放行。
    assert_fallback_allow!(decide_cmd("cat ~/.aws/credentials"));
}

#[test]
fn benign_reads_are_not_interrupted() {
    assert_fallback_allow!(decide_cmd("cat config.yaml"));
    assert_fallback_allow!(decide_cmd("cat README.md"));
}
