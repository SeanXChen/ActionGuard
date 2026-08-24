//! shell.yml 黄金回归用例（15 条规则 + 回退路径）。
//! 规则顺序为 first-match-wins；sudo 与 rm 的交互顺序被本套件锁定。

use actionguard_lib::models::{Decision, RiskLevel};
use crate::golden::decide_cmd;

#[test]
fn rm_rf_root_is_denied() {
    assert_decision!(
        decide_cmd("rm -rf /"),
        Decision::Deny,
        "deny-rm-rf-root",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("rm -rf ~/x"),
        Decision::Deny,
        "deny-rm-rf-root",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("rm -rf $HOME/x"),
        Decision::Deny,
        "deny-rm-rf-root",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("rm -rf *"),
        Decision::Deny,
        "deny-rm-rf-root",
        RiskLevel::Critical
    );
}

#[test]
fn privilege_escalation_is_denied() {
    assert_decision!(
        decide_cmd("sudo apt-get update"),
        Decision::Deny,
        "deny-sudo",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("su -"),
        Decision::Deny,
        "deny-su",
        RiskLevel::Critical
    );
    // sudo 前缀：command=sudo（首 token），deny-rm-rf-root 的 ^rm regex
    // 不适用，由 deny-sudo 兜底 —— 结果仍是 Deny（不弱化）。
    assert_decision!(
        decide_cmd("sudo rm -rf /"),
        Decision::Deny,
        "deny-sudo",
        RiskLevel::Critical
    );
}

#[test]
fn rm_in_workspace_asks() {
    assert_decision!(
        decide_cmd("rm -rf ./dist"),
        Decision::Ask,
        "confirm-rm-rf",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("rm -rf build/"),
        Decision::Ask,
        "confirm-rm-rf",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("rm -fr temp"),
        Decision::Ask,
        "confirm-rm-fr",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("rm -r old"),
        Decision::Ask,
        "confirm-rm-r",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("rm -f file.txt"),
        Decision::Ask,
        "confirm-rm-f",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("rm old.txt"),
        Decision::Ask,
        "confirm-rm-no-flags",
        RiskLevel::Medium
    );
}

#[test]
fn remote_download_asks() {
    assert_decision!(
        decide_cmd("curl http://example.com/init.sh"),
        Decision::Ask,
        "confirm-curl",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("wget https://example.com/pkg.tar.gz"),
        Decision::Ask,
        "confirm-wget",
        RiskLevel::Medium
    );
}

#[test]
fn permissions_and_system_ops() {
    assert_decision!(
        decide_cmd("chmod 777 script.sh"),
        Decision::Ask,
        "confirm-chmod",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("chown root:root file"),
        Decision::Ask,
        "confirm-chown",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("shutdown now"),
        Decision::Deny,
        "deny-shutdown",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("reboot"),
        Decision::Deny,
        "deny-reboot",
        RiskLevel::Critical
    );
}

#[test]
fn kill_variants_ask() {
    assert_decision!(
        decide_cmd("kill -9 1234"),
        Decision::Ask,
        "confirm-kill-9",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("kill -KILL 5678"),
        Decision::Ask,
        "confirm-kill-9",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("kill -SIGKILL 9999"),
        Decision::Ask,
        "confirm-kill-9",
        RiskLevel::High
    );
}

#[test]
fn benign_shell_is_not_interrupted() {
    assert_fallback_allow!(decide_cmd("echo hello"));
    assert_fallback_allow!(decide_cmd("ls -la"));
    assert_fallback_allow!(decide_cmd("git status"));
}
