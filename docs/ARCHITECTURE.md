# ActionGuard Architecture

本文档描述 ActionGuard v0.2 的核心架构、数据模型和设计决策。

---

## 整体架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Frontend (Vue 3)                               │
│  App.vue │ SessionView │ ApprovalModal │ RiskPanel │ PolicyEditor           │
│         ─────────────────────────────────────────────────                     │
│         Event Bus (Tauri Events)                                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↑↓
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Backend (Tauri / Rust)                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ Bridge  │  │Policy   │  │ Risk    │  │ Storage │  │ Boundary│         │
│  │ /preexec│  │ Engine  │  │ Engine  │  │ (YAML)  │  │Registry │         │
│  │/resolve │  │         │  │         │  │         │  │         │         │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘         │
│         ↑            ↑            ↑            ↑            ↑              │
│         └────────────┴────────────┴────────────┴────────────┘              │
│                              Commands.rs                                     │
│                              main.rs (invoke_handler)                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↑↓
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLI (actionguard)                              │
│  protect │ capabilities │ policy-check │ report │ doctor │ undo │ boundary │
│  rule search │ rule install                                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↑↓
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Adapters (Language Agnostic)                        │
│  ag-hook.py (CodeBuddy) │ PS1 hooks (bash/zsh/fish) │ PSReadLine           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 数据模型（Action → Facts → Policy → Verdict）

核心是一条**确定性**的决策流水线。产品不是"规则引擎"——规则只是告诉它**什么危险**，Facts 告诉它**现在发生了什么**，Enforcement 决定**能不能真的挡住**，GUI 告诉用户**为什么挡、你要怎么办**。

```
                 ┌──────────────────────┐
                 │      User / Agent    │
                 └──────────┬───────────┘
                            │  actual operation
                 ┌──────────▼───────────┐
                 │   Action Extraction  │  适配器层：hook / PreToolUse / PS1
                 └──────────┬───────────┘
                            │
                 ┌──────────▼───────────┐
                 │      Facts Layer     │  Action + ActionContext（谁、在哪、什么状态）
                 └──────────┬───────────┘
                            │
                 ┌──────────▼───────────┐
                 │      Policy Engine   │  YAML 规则 → MatchSpec → first match wins
                 └──────────┬───────────┘
                            │
                 ┌──────────▼───────────┐
                 │        Verdict       │  Allow / Ask / Deny + Risk + Reason
                 └──────────────────────┘
```

Schema 定义在 `src-tauri/src/models.rs`，是整个系统的**稳定接口**。规则语言可以换（YAML → CEL → Rhai），但 Facts schema 一旦到处耦合才是真正难改的。

### Core Types（与 `models.rs` 一致）

```rust
// 动作来源 —— 回答「谁在做？」
enum ActionSourceType { Agent, Automation, Workflow, Human, Unknown }

// 动作 —— Facts 的主载体
struct Action {
    category: ActionCategory,      // File / Shell / Git / Package / Secret
    kind: ActionKind,              // Delete / Write / Exec / Install / …
    path: Option<String>,          // 目标路径
    command: Option<String>,       // 原始命令
    risk: RiskLevel,               // 分类器确定性评估
    sensitive: bool,               // 命中敏感资源（secrets / .env / credentials）
    outside: bool,                 // 越出受保护工作区
    source_type: ActionSourceType, // 谁发起的
}

// 动作上下文 —— 来自哪个边界
struct ActionContext {
    boundary: BoundaryType,
    source_type: ActionSourceType,
}

// 风险等级 —— 确定性、可解释，不是神秘分数
enum RiskLevel { Low, Medium, High, Critical }
// 建议映射：L0-SAFE → Low · L1-LOW → Low · L2-SENSITIVE → Medium
//           L3-HIGH → High · L4-CRITICAL → Critical

// Verdict
enum Decision { Allow, Ask, Deny }   // Ask 对应 YAML `action: confirm`

// 决策结果 —— 自带解释（Decision Explanation）
struct DecisionResult {
    decision: Decision,
    risk: RiskLevel,
    matched_rule: Option<String>,    // 命中的规则 id
    reason: String,                  // 为什么（BLOCKED / Rule / Facts / Reason）
}

// 策略来源 —— 决定优先级
enum PolicySource { Builtin, User }  // 未来扩展：Project
```

### 策略优先级（Policy Precedence）

核心安全属性：**被保护对象不能决定自己的保护边界。**

| 层级 | 可以收紧 | 可以放松 |
|------|----------|----------|
| Built-in safety | ❌ | ❌ |
| User policy | ✅ | ✅ |
| Project policy（计划） | ✅ | ❌ |
| Agent / session request | ❌ | ❌ |

当前实现：`User > Builtin`（用户是机器所有者，可以覆盖内置规则）。未来加入 Project 后保持不变量：**Project policy can make ActionGuard stricter, but never weaker.**

### 决策缓存（Decision Cache）

同一个（rule + facts）组合不应重复打扰用户——这是对抗「打扰疲劳」最直接的一层：

- `Allow once` —— 只放行当前动作
- `Allow for session` —— 本会话内同类动作不再询问
- `Remember / learn_rule` —— 持久化为一条 User rule，之后直接决策（已实现：`deny --always`、GUI「Always deny」；GUI「Always allow」待补）

### Storage Layout

```
~/.actionguard/
├── sessions/
│   ├── <id>.json            # 会话摘要（SessionSummary，统计聚合）
│   └── <id>.ledger.json     # 追加式 NDJSON ledger（每条最终化的 Action 一行）
├── snapshots/               # 文件系统快照（undo 用）
│   └── <session-id>/
├── policies.user.yml        # 用户策略（User rules）
├── rules/                   # 内置规则（编译进二进制，include_str!）
└── <hook descriptor>        # 当前 hook 会话信息（端口 + secret）
```

---

## 四层设计

### L1 — Observation（观察）

- **目标**: 100% 动作可见，0% 漏报
- **实现**: 适配器层（hook、PS1、PreToolUse）捕获所有命令
- **保证**: 即使后续层全部失效，审计日志仍然完整

### L2 — Interception（拦截）

- **目标**: 在动作执行前阻止危险操作
- **实现**: 
  - Bridge `/preexec`: 受保护 shell 会话中挂起命令
  - ag-hook.py: CodeBuddy PreToolUse 返回 `permissionDecision: "deny"`
  - `actionguard protect`: 启动带审批的 shell
- **决策**: Policy Engine → Allow / Warn / Ask / Deny

### L3 — Policy（策略）

- **目标**: AI-native 规则引擎
- **实现**: 
  - YAML 规则文件（内置 + 用户自定义 + 社区安装）
  - 分类器：命令 → ActionCategory + RiskLevel
  - 规则匹配：patterns + regex
  - 决策输出：Allow / Warn / Ask / Deny

### L4 — Audit（审计）

- **目标**: 完整记录、可追溯、可回滚
- **实现**:
  - 每条动作写入 `actions.log`
  - Undo 系统：快照 + Git 引用修复 + 目录重建
  - Diff 报告生成

---

## 边界适配器架构

```
┌─────────────────────────────────────────────────────────────────┐
│                     ActionGuard Core (Rust)                     │
│                    policy-check / protect                       │
└─────────────────────────────────────────────────────────────────┘
                              ↑
                              │ stdin / stdout / HTTP
┌─────────────────────────────┼───────────────────────────────────┐
│                             │                                   │
│  ┌──────────────┐    ┌──────┴──────┐    ┌──────────────┐      │
│  │ CodeBuddy    │    │ bash/zsh    │    │ Claude Code  │      │
│  │ ag-hook.py   │    │ PS1 hook    │    │ (planned)    │      │
│  │ PreToolUse   │    │ PROMPT_CMD  │    │              │      │
│  └──────────────┘    └─────────────┘    └──────────────┘      │
│                                                                │
│  Boundary Adapter: vendor-neutral, replaceable, testable       │
└─────────────────────────────────────────────────────────────────┘
```

设计原则：
- **厂商无关**: 适配器不知道后端是 CodeBuddy 还是 Claude Code
- **可替换**: 任何支持 pre-action hook 的工具都可以接入
- **可测试**: 每个适配器独立验证，不依赖真实工具

---

## CLI 架构

```
actionguard
├── protect <workspace>      → start bridge → shell → approval gate
├── capabilities             → print Capability Tier Matrix
├── policy-check <cmd>       → policy engine → decision (no side effects)
├── report                   → read sessions → risk report
├── doctor                   → system diagnostics
├── undo                     → latest session → undo actions
├── boundary list            → Boundary Registry → table
└── rule
    ├── search <query>       → filter builtin rules
    └── install <file>       → lint → merge → save to user policy
```

---

## 关键设计决策

### 1. Detection ≠ Protection

所有输出明确区分「能检测」和「能阻止」。这是 Capability Tier Model 的核心——L1 不等于 L2。

### 2. Fail-Closed by Default

当引擎不可用时，适配器选择「阻止」而非「放行」。`ag-hook.py` 第 17-21 行：

> A safety layer must block when it cannot evaluate.

### 3. Honest Verification

README、文档、矩阵只标注已验证的能力。`Not verified` 不等于 `Not possible`——只是还没人测过。

### 4. Community-Driven Boundary Expansion

维护者只定标准，社区提 PR 提交验证。维护者实机验证后才标 **Core Verified**。

---

## 文件结构

```
actionguard/
├── src/                    # Vue 3 frontend
│   ├── App.vue
│   ├── components/
│   │   ├── ApprovalModal.vue
│   │   ├── RiskPanel.vue
│   │   └── SessionView.vue
│   ├── api.ts             # Tauri command wrappers
│   ├── models.ts          # TypeScript type definitions
│   └── store.ts           # Pinia store
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs        # Tauri app entry
│   │   ├── lib.rs         # Command registration
│   │   ├── commands.rs    # Tauri commands
│   │   ├── bridge.rs      # HTTP bridge for protected sessions
│   │   ├── models.rs      # Core data types（Action / Facts / Verdict schema）
│   │   ├── policy/
│   │   │   ├── mod.rs     # Policy engine（decide）
│   │   │   ├── matcher.rs # MatchSpec 匹配
│   │   │   └── loader.rs  # YAML 加载 + lint
│   │   ├── risk.rs        # Deterministic risk engine
│   │   ├── classify.rs    # Command classifier
│   │   ├── approval.rs    # Approval gate（含 learn_rule）
│   │   ├── storage.rs     # Persistence
│   │   ├── boundary.rs    # Boundary registry
│   │   └── bin/
│   │       └── cli.rs     # CLI binary
│   ├── rules/             # Built-in YAML rules
│   ├── Cargo.toml
│   └── tauri.conf.json
├── boundaries/             # Boundary Registry YAMLs
├── scripts/
│   ├── install.ps1
│   ├── install.sh
│   ├── e2e-windows.ps1
│   └── hooks/
│       └── ag-hook.py     # CodeBuddy adapter
├── docs/
│   ├── ARCHITECTURE.md
│   ├── CONTRIBUTING.md
│   ├── BOUNDARIES.md
│   ├── SECURITY_MODEL.md
│   └── SECURITY_TEST_MATRIX.md
├── .github/
│   ├── pull_request_template.md
│   └── workflows/
│       └── ci.yml
├── README.md
├── README.zh.md
├── CHANGELOG.md
├── LICENSE
├── package.json
└── vite.config.ts
```

---

## 版本演进

| 版本 | 主题 | 状态 |
|------|------|------|
| v0.1 | File Safety — protected workspace · file monitor · risk rules · undo | ✅ shipped |
| v0.2 | Action Safety — 4 layers · 5 categories · policy · gate · CLI | ✅ shipped |
| v0.2.x | Validation Phase — Facts schema 固化 · 对抗性 bypass 测试 · 决策缓存 · stats export · 反馈渠道 | 🚧 current |
| v0.3 | Enforcement Validation — Project policy（只能收紧）· Decision Explanation · 风险等级 L0–L4 | planned |
| v0.4 | 扩展自动化对象 — 桌面自动化 / 浏览器自动化 / 本地脚本 | planned |
| v0.5 | 策略源生态 — Filter List / 外部规则源 | planned |
