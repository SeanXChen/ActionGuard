//! git.yml 黄金回归用例（14 条规则的正 / 负 / 边界）。
//! 规则顺序为 first-match-wins，顺序变更会被本套件抓出来。

use actionguard_lib::models::{Decision, RiskLevel};
use crate::golden::decide_cmd;

#[test]
fn push_allow_baseline() {
    assert_decision!(
        decide_cmd("git push origin main"),
        Decision::Allow,
        "allow-push",
        RiskLevel::Low
    );
    assert_decision!(
        decide_cmd("git push origin feature/login"),
        Decision::Allow,
        "allow-push",
        RiskLevel::Low
    );
}

#[test]
fn force_push_shared_branch_is_denied() {
    // Tier 0：不可逆的共享分支覆盖。
    for branch in ["main", "master", "develop", "trunk"] {
        let cmd = format!("git push --force origin {branch}");
        assert_decision!(
            decide_cmd(&cmd),
            Decision::Deny,
            "deny-push-force-shared-branch",
            RiskLevel::Critical
        );
    }
    // 短参数 `-f` 同罪。
    assert_decision!(
        decide_cmd("git push -f origin main"),
        Decision::Deny,
        "deny-push-f-shared-branch",
        RiskLevel::Critical
    );
}

#[test]
fn force_push_feature_branch_asks() {
    // 非共享分支：Ask（确认），不 Deny。
    assert_decision!(
        decide_cmd("git push --force origin feature/login"),
        Decision::Ask,
        "confirm-push-force",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("git push -f origin feature/x"),
        Decision::Ask,
        "confirm-push-force",
        RiskLevel::High
    );
}

#[test]
fn force_with_lease_asks() {
    // --force-with-lease 不得命中 shared-branch deny（regex 要求 --force 为整 token）。
    assert_decision!(
        decide_cmd("git push --force-with-lease origin main"),
        Decision::Ask,
        "confirm-push-force-with-lease",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("git push --force-with-lease origin feature/x"),
        Decision::Ask,
        "confirm-push-force-with-lease",
        RiskLevel::Medium
    );
}

#[test]
fn destructive_reset_asks() {
    assert_decision!(
        decide_cmd("git reset --hard HEAD~1"),
        Decision::Ask,
        "confirm-reset-hard",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("git reset --hard"),
        Decision::Ask,
        "confirm-reset-hard",
        RiskLevel::High
    );
}

#[test]
fn clean_matrix() {
    assert_decision!(
        decide_cmd("git clean -fd"),
        Decision::Ask,
        "confirm-clean-fd",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("git clean -df"),
        Decision::Ask,
        "confirm-clean-df",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("git clean -f"),
        Decision::Ask,
        "confirm-clean-f",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("git clean -d"),
        Decision::Ask,
        "confirm-clean-d",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("git clean -x"),
        Decision::Ask,
        "confirm-clean-ndx",
        RiskLevel::High
    );
}

#[test]
fn branch_delete_matrix() {
    assert_decision!(
        decide_cmd("git branch -D old-branch"),
        Decision::Ask,
        "confirm-branch-delete-D",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("git branch -d merged-branch"),
        Decision::Ask,
        "confirm-branch-delete-d",
        RiskLevel::Low
    );
}

#[test]
fn benign_git_is_not_interrupted() {
    assert_decision!(
        decide_cmd("git pull origin main"),
        Decision::Allow,
        "allow-pull",
        RiskLevel::Low
    );
    assert_decision!(
        decide_cmd("git fetch origin"),
        Decision::Allow,
        "allow-fetch",
        RiskLevel::Low
    );
    // git status 无规则覆盖 → 回退路径（不能误配 allow-fetch）。
    assert_fallback_allow!(decide_cmd("git status"));
}

#[test]
fn non_git_commands_fall_through_to_fallback() {
    // 不是 git 开头（但含 git 字样）：必须走回退，而不是误配 git 规则。
    assert_fallback_allow!(decide_cmd("echo git push --force origin main"));
}
