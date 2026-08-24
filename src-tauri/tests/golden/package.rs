//! node.yml + python.yml 黄金回归用例（20 条规则）。
//! 注意：规则对 command（npm/yarn/pnpm/pip/…）精确匹配，cross-tool 混淆会被本套件抓出。

use actionguard_lib::models::{Decision, RiskLevel};
use crate::golden::decide_cmd;

#[test]
fn npm_publish_is_denied() {
    assert_decision!(
        decide_cmd("npm publish"),
        Decision::Deny,
        "deny-npm-publish",
        RiskLevel::Critical
    );
}

#[test]
fn npm_uninstall_variants_ask() {
    assert_decision!(
        decide_cmd("npm uninstall axios"),
        Decision::Ask,
        "confirm-npm-uninstall",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("npm rm axios"),
        Decision::Ask,
        "confirm-npm-uninstall",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("npm remove axios"),
        Decision::Ask,
        "confirm-npm-uninstall",
        RiskLevel::Medium
    );
}

#[test]
fn global_installs_ask() {
    assert_decision!(
        decide_cmd("npm install -g typescript"),
        Decision::Ask,
        "confirm-npm-install-global",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("npm i -g cowsay"),
        Decision::Ask,
        "confirm-npm-install-global",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("npm add --global nodemon"),
        Decision::Ask,
        "confirm-npm-install-global",
        RiskLevel::High
    );
}

#[test]
fn local_installs_are_allowed() {
    assert_decision!(
        decide_cmd("npm install axios"),
        Decision::Allow,
        "allow-npm-install",
        RiskLevel::Low
    );
    assert_decision!(
        decide_cmd("npm ci"),
        Decision::Allow,
        "allow-npm-install",
        RiskLevel::Low
    );
    // `npm rm` 里的 "i"（axios 也含 i）不得误入 install-global regex。
    assert_decision!(
        decide_cmd("npm rm axios"),
        Decision::Ask,
        "confirm-npm-uninstall",
        RiskLevel::Medium
    );
}

#[test]
fn yarn_and_pnpm() {
    assert_decision!(
        decide_cmd("yarn remove x"),
        Decision::Ask,
        "confirm-yarn-remove",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("yarn add x"),
        Decision::Allow,
        "allow-yarn-add",
        RiskLevel::Low
    );
    assert_decision!(
        decide_cmd("pnpm remove x"),
        Decision::Ask,
        "confirm-pnpm-remove",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("pnpm add x"),
        Decision::Allow,
        "allow-pnpm-add",
        RiskLevel::Low
    );
}

#[test]
fn npx_rm_rf_is_denied() {
    assert_decision!(
        decide_cmd("npx rm -rf /"),
        Decision::Deny,
        "deny-npx-rm-rf",
        RiskLevel::Critical
    );
    assert_decision!(
        decide_cmd("npx rm -rf ./dist"),
        Decision::Deny,
        "deny-npx-rm-rf",
        RiskLevel::Critical
    );
    assert_fallback_allow!(decide_cmd("npx cowsay hi"));
}

#[test]
fn pip_upgrade_asks() {
    assert_decision!(
        decide_cmd("pip install -U requests"),
        Decision::Ask,
        "confirm-pip-install-upgrade",
        RiskLevel::High
    );
    assert_decision!(
        decide_cmd("pip install --upgrade requests"),
        Decision::Ask,
        "confirm-pip-install-upgrade",
        RiskLevel::High
    );
}

#[test]
fn pip_baseline() {
    assert_decision!(
        decide_cmd("pip install requests"),
        Decision::Allow,
        "allow-pip-install",
        RiskLevel::Low
    );
    assert_decision!(
        decide_cmd("pip uninstall x"),
        Decision::Ask,
        "confirm-pip-uninstall",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("pip3 install x"),
        Decision::Allow,
        "allow-pip3-install",
        RiskLevel::Low
    );
    assert_decision!(
        decide_cmd("pip3 uninstall x"),
        Decision::Ask,
        "confirm-pip3-uninstall",
        RiskLevel::Medium
    );
}

#[test]
fn poetry_uv_conda() {
    assert_decision!(
        decide_cmd("poetry remove x"),
        Decision::Ask,
        "confirm-poetry-remove",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("poetry add x"),
        Decision::Allow,
        "allow-poetry-add",
        RiskLevel::Low
    );
    assert_decision!(
        decide_cmd("uv remove x"),
        Decision::Ask,
        "confirm-uv-remove",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("uv add x"),
        Decision::Allow,
        "allow-uv-add",
        RiskLevel::Low
    );
    assert_decision!(
        decide_cmd("conda remove x"),
        Decision::Ask,
        "confirm-conda-remove",
        RiskLevel::Medium
    );
    assert_decision!(
        decide_cmd("conda install x"),
        Decision::Allow,
        "allow-conda-install",
        RiskLevel::Low
    );
}
