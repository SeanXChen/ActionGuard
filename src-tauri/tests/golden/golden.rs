//! Golden corpus helpers — 走真实 pipeline：raw command → classify → decide。
//! 与 `setup.rs` / `doctor.rs` 的 self-check 使用同一路径，避免测试伪造输入。

use actionguard_lib::models::{Action, DecisionResult, PolicySet};
use actionguard_lib::policy::classify::{classify_shell_command, kind_for};
use actionguard_lib::policy::{decide, load_policy_set};

/// 内置策略集（与 bypass / self-check 一致：包含本地 `policies.user.yml`；
/// 用户文件若翻转某条 Golden 断言，本身就是安全相关变更）。
pub fn builtin() -> PolicySet {
    load_policy_set()
}

/// Shell / Git / Package 命令：分类 → 决策（真实 pipeline）。
pub fn decide_cmd(line: &str) -> DecisionResult {
    let mut a =
        Action::new_shell_from_source(line.to_string(), None, "agent", Some("golden".to_string()));
    let parsed = classify_shell_command(line);
    a.category = parsed.category;
    a.kind = Some(kind_for(&parsed).to_string());
    decide(&a, &builtin())
}

/// File 类动作（secrets 规则走这里）。
pub fn decide_file(path: &str, kind: actionguard_lib::models::ActionKind) -> DecisionResult {
    let a = Action::new_file(path.to_string(), kind);
    decide(&a, &builtin())
}
