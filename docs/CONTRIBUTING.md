# Contributing to ActionGuard

> English version: [CONTRIBUTING.md](../CONTRIBUTING.md) (repo root)

感谢你对 ActionGuard 的兴趣！本指南涵盖所有类型的贡献：验证报告、规则、代码、文档。

---

## 📋 目录

- [贡献类型](#贡献类型)
- [边界发现贡献（最高价值）](#边界发现贡献最高价值)
- [Boundary 验证报告](#boundary-验证报告)
- [规则贡献](#规则贡献)
- [代码贡献](#代码贡献)
- [安全漏洞报告](#安全漏洞报告)
- [开发环境](#开发环境)

---

## 贡献类型

| 类型 | 说明 | 起始文件 |
|------|------|----------|
| **边界发现** | **发现 ActionGuard 未覆盖的危险行为。我们奖励"发现"，不奖励"写规则"。** | 无需 YAML，见 [BOUNDARY_BACKLOG](../docs/BOUNDARY_BACKLOG.md) |
| **验证报告** | 对某个 AI 工具执行 ActionGuard Boundary Test | 见下方 PR 模板 |
| **规则** | 新的 YAML 安全规则（必须带 Golden Test） | `src-tauri/rules/` + `src-tauri/tests/golden/` |
| **代码** | Rust 后端 / Vue 前端 / CLI 功能 | 先开 Issue 讨论 |
| **文档** | README、架构文档、翻译 | 直接提 PR |

---

## 边界发现贡献（最高价值）

ActionGuard 的内置基线覆盖的是**我们已经知道的**。更值钱的是我们还不知道的：
一个 AI 自动化做出的、ActionGuard 尚未建模的危险行为。

**发现了一个 ActionGuard 拦不住的危险动作？直接报告即可——不需要写 YAML，不需要读代码。**

我们奖励的是**边界发现**，不是规则写作。贡献分三层：

| 层级 | 做什么 | 需要的能力 | 回报 |
|------|--------|------------|------|
| **Tier 1 · 报告** | 「ActionGuard 没拦住 X」+ 为什么危险，无需 YAML | 无 | 在边界 Backlog 中署名 |
| **Tier 2 · 复现** | 动作 / 环境 / 预期 / 实际 / 复现步骤 | 基础 | 署名 + 关联到修复 |
| **Tier 3 · Policy + Test** | 策略规则 + Golden Test（`src-tauri/tests/golden/`） | YAML + Rust 测试 | 署名 + 维护者审核 |

每个被接受的发现都会成为 [`docs/BOUNDARY_BACKLOG.md`](../docs/BOUNDARY_BACKLOG.md)
中的一行。**每条新规则必须带 Golden Test——没有测试的规则不合并。**
队列由维护者策展：我们审核的是*你发现了什么*，而不是你 YAML 写得多好。

使用 [Boundary Report 模板](../.github/ISSUE_TEMPLATE/boundary_report.yml) 提交。

---

## Boundary 验证报告

ActionGuard 的核心资产是**诚实的验证数据**。我们不接受「声称」，只接受「证据」。

### 提交步骤

1. 在目标 AI 工具上安装 ActionGuard 适配器（参考 README 中的 CodeBuddy 示例）。
2. 执行具体的边界测试用例（如 `sudo rm -rf /`、`git push --force`、`curl ...` 等）。
3. 记录实际行为（被拦 / 执行 / 部分拦 / 绕过）。
4. 提交 PR 到本仓库，更新 `boundaries/<tool>.yml` 和 `SECURITY_TEST_MATRIX.md`。

### PR 模板（必须填写）

```markdown
## Boundary Verification Report

- **Automation**: [工具名称]
- **Version**: [精确版本号]
- **OS**: [操作系统 + shell]
- **Execution path**: [动作触发方式：agent prompt / CLI / script]
- **Boundary**: [边界名称，如 "Protected Shell (bash/zsh/fish)"]
- **Test**: [具体命令或场景]
- **Expected**: [期望结果]
- **Actual**: [实际结果]
- **ActionGuard version**: [引擎版本]
- **Evidence**: [截图 / 录屏 / 日志]

> ⚠️ Claims without evidence are closed without merge.
```

### 验证标准

1. **实测，非假设**：必须是真实执行的结果，不是代码审查后的推测。
2. **一行一边界**：每个验证条目只测一个具体边界（一个命令、一个场景）。
3. **版本 + OS 必填**：不同版本行为可能完全不同。
4. **维护者审核合并**：只有维护者能合并验证报告。
5. **Core Verified 仅维护者可标**：社区验证通过标记为 **Community Verified**，维护者实机复现后才可升级为 **Core Verified**。

---

## 规则贡献

### 规则格式（实际 Schema）

规则是 YAML，**必须面向 Facts 编写**（见 [FACTS_SCHEMA.md](./FACTS_SCHEMA.md)）。
实际格式（`src-tauri/rules/*.yml`，匹配引擎见 `src-tauri/src/policy/matcher.rs`）：

```yaml
- id: unique-rule-id          # 必填，全局唯一
  match:
    category: git|package|shell|file|secret   # 分类（由 classify 产出）
    command: git               # 精确命令（Shell/Git/Package）
    args_contains: ["-rf"]     # 全部子串必须出现（大小写折叠）
    args_any: ["install","i"]  # 任一出现即命中
    regex: "-D(\\s|$)"         # 作用于完整命令行（大小写敏感）
    path: "*.env"              # 通配符路径（File 类）
  action: allow|ask|deny       # 决策
  risk: low|medium|high|critical
  reason: "解释为什么"          # 必填，会展示给用户
```

> 注意：`args_contains` 会做大小写折叠（`-d` 会被 `["-D"]` 误配）。需要区分
> 大小写的参数请用 `regex`（如 `git branch -d` vs `-D`，见 `tests/golden/git.rs`）。

### 规则必须带 Golden Test

**一条规则 + 一个 Golden 用例（`src-tauri/tests/golden/<域>.rs`）成对提交。**
Golden 断言 decision + matched_rule + risk，锁定规则行为；将来规则改动导致
行为漂移会立刻被回归套件抓住。参考现有 `tests/golden/git.rs` / `shell.rs` /
`secrets.rs` / `package.rs`。

### 规则存储位置

| 来源 | 路径 | 说明 |
|------|------|------|
| 内置 | `src-tauri/rules/*.yml` | 随引擎发布，分域组织（git/shell/secrets/node/python） |
| 用户 | `~/.actionguard/policy/` | 用户自定义（Tier 2 环境特定策略） |
| 社区 | 独立仓库（计划中） | `actionguard rule install` 安装 |

### 提交规则

1. 在 `src-tauri/rules/` 的对应域文件新增规则（新域才新建文件）。
2. 在 `src-tauri/tests/golden/` 增加对应正/负/边界用例。
3. 运行 `cargo test --test golden` 验证全部绿。
4. 提交 PR；没有 Golden Test 的规则 PR 不合并。

---

## 代码贡献

### 技术栈

- **后端**: Rust + Tauri
- **前端**: Vue 3 + TypeScript + Vite
- **CLI**: Rust (`src-tauri/src/bin/cli.rs`)
- **测试**: `cargo test` + `e2e-windows.ps1`

### 开发流程

1. **Fork & Branch**: `git checkout -b feature/your-feature`
2. **编码**: 遵循现有代码风格（`cargo fmt` + `cargo clippy`）
3. **测试**: `cargo test` 必须全绿
4. **提交**: 使用清晰的 commit message
5. **PR**: 描述变更动机和影响范围

### Commit Message 规范

```
type(scope): subject

body (optional)

footer (optional)
```

| Type | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档变更 |
| `test` | 测试相关 |
| `refactor` | 重构 |
| `chore` | 构建/工具链 |

示例：`feat(policy): add category.secret detection for AWS credentials`

---

## 安全漏洞报告

**请不要通过公开 Issue 报告安全漏洞。** 请通过 GitHub 私密漏洞报告私下提交 — 参见 [SECURITY.md](../SECURITY.md)。

---

## 开发环境

### 前置依赖

- Rust >= 1.75
- Node.js >= 18
- npm

### 构建

```bash
# 安装前端依赖
npm install

# 开发模式（GUI）
npm run tauri dev

# 构建 CLI
cargo build --release --bin actionguard

# 运行测试
cargo test

# 运行 E2E 测试（Windows）
.\scripts\e2e-windows.ps1
```

---

## 许可证

通过提交 PR，你同意你的贡献将在 [Apache-2.0](../LICENSE) 下发布。
