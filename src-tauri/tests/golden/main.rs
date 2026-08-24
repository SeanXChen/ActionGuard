//! Golden regression corpus — 每条规则的正面 / 负面 / 边界案例。
//!
//! 与 `bypass`（对抗性绕过视角）互补：这里是"规则行为矩阵"视角。
//! 目标：以后新增第 100 条规则时，跑一次本套件就能确认前 99 条没被搞坏。
//!
//! 运行：`cargo test --test golden`
//! CI：  `cargo test` 自动包含。
//!
//! 目录结构（新增规则时按域加文件）：
//!   golden/git.rs      — git.yml
//!   golden/shell.rs    — shell.yml
//!   golden/secrets.rs  — secrets.yml
//!   golden/package.rs  — node.yml + python.yml

mod golden;

/// 完整断言：decision + matched_rule + risk。
/// `$expr` 产出一个 `DecisionResult`（decide_cmd / decide_file）。
macro_rules! assert_decision {
    ($expr:expr, $want:path, $rule:expr, $risk:path) => {
        let r = $expr;
        assert_eq!(
            r.decision, $want,
            "[{}] decision: got {:?}, want {:?} (rule={:?}, reason={:?})",
            stringify!($expr), r.decision, $want, r.matched_rule, r.reason
        );
        assert_eq!(
            r.matched_rule.as_deref(),
            Some($rule),
            "[{}] rule: got {:?}",
            stringify!($expr),
            r.matched_rule
        );
        assert_eq!(r.risk, $risk, "[{}] risk: got {:?}", stringify!($expr), r.risk);
    };
}

/// 无规则命中的回退路径：Allow + matched_rule = None（risk 由风险引擎决定）。
macro_rules! assert_fallback_allow {
    ($expr:expr) => {
        let r = $expr;
        assert_eq!(
            r.decision, Decision::Allow,
            "{}: got {:?}",
            stringify!($expr),
            r.decision
        );
        assert_eq!(
            r.matched_rule, None,
            "{}: expected fallback (no rule), got {:?}",
            stringify!($expr),
            r.matched_rule
        );
    };
}

// 子模块必须在宏定义之后声明（macro_rules 的文本作用域）。
mod git;
mod package;
mod secrets;
mod shell;
