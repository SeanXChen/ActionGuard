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

## 数据模型

### Core Types

```rust
// 动作
struct Action {
    command: String,
    timestamp: u64,
    category: ActionCategory,
    risk_level: RiskLevel,
}

// 风险
enum RiskLevel { None, Low, Medium, High, Critical }

// Action 分类
enum ActionCategory {
    File,
    Command,
    Network,
    Secret,
    Irreversible,
}

// 会话
struct Session {
    id: String,
    workspace: PathBuf,
    actions: Vec<Action>,
    risk_score: f64,
}

// 策略
struct Policy {
    rules: Vec<Rule>,
    allowed_patterns: Vec<String>,
    denied_patterns: Vec<String>,
    auto_approve_threshold: RiskLevel,
}

// 命令分类结果
struct CommandClassification {
    category: ActionCategory,
    risk_level: RiskLevel,
    reason: String,
    patterns_matched: Vec<String>,
}
```

### Storage Layout

```
~/.actionguard/
├── sessions/              # 运行时会话数据
│   └── <session-id>/
│       ├── actions.log
│       ├── ledger.json    # 等待审批的动作
│       └── meta.json
├── snapshots/             # 文件系统快照（用于 undo）
├── rules/                 # 内置规则副本
│   ├── git.yml
│   ├── node.yml
│   ├── python.yml
│   ├── secrets.yml
│   └── shell.yml
├── policy/                # 用户策略（空）
├── hook-adapter.log       # 适配器日志
└── current.hook           # 当前 hook 会话信息
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
│   │   ├── policy.rs      # Policy engine
│   │   ├── risk.rs        # Risk engine
│   │   ├── classify.rs    # Command classifier
│   │   ├── models.rs      # Core data types
│   │   ├── storage.rs     # YAML persistence
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
| v0.2 | Action Safety — 4 layers · 5 categories · policy · gate · CLI | ✅ shipping |
| v0.3 | Enforcement Validation — honest enforcement · Capability Tier · Boundary Registry | 🚧 current |
| v0.4 | Browser Safety | planned |
| v0.5 | API / SaaS Safety | planned |
