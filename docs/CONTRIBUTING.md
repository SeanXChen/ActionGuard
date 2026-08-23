# Contributing to ActionGuard

> English version: [CONTRIBUTING.md](../CONTRIBUTING.md) (repo root)

感谢你对 ActionGuard 的兴趣！本指南涵盖所有类型的贡献：验证报告、规则、代码、文档。

---

## 📋 目录

- [贡献类型](#贡献类型)
- [Boundary 验证报告](#boundary-验证报告)
- [规则贡献](#规则贡献)
- [代码贡献](#代码贡献)
- [安全漏洞报告](#安全漏洞报告)
- [开发环境](#开发环境)

---

## 贡献类型

| 类型 | 说明 | 起始文件 |
|------|------|----------|
| **验证报告** | 对某个 AI 工具执行 ActionGuard Boundary Test | 见下方 PR 模板 |
| **规则** | 新的 YAML 安全规则或规则包 | `src-tauri/rules/` 或社区仓库 |
| **代码** | Rust 后端 / Vue 前端 / CLI 功能 | 先开 Issue 讨论 |
| **文档** | README、架构文档、翻译 | 直接提 PR |

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

### 规则格式

所有规则使用 YAML，Schema 如下：

```yaml
id: unique-rule-id          # 必填，全局唯一
category: file|command|network|secret|irreversible  # 必填
severity: low|medium|high|critical  # 必填
description: "简短描述"      # 必填
match:
  patterns:                  # 至少一个
    - "pattern1"
    - "pattern2"
  regex: "optional-regex"    # 可选
action:
  decision: allow|warn|ask|deny  # 必填
  reason: "解释为什么"        # 可选
```

### 规则存储位置

| 来源 | 路径 | 说明 |
|------|------|------|
| 内置 | `src-tauri/rules/*.yml` | 随引擎发布 |
| 用户 | `~/.actionguard/policy/` | 用户自定义 |
| 社区 | 独立仓库（计划中） | `actionguard rule install` 安装 |

### 提交规则

1. 在 `src-tauri/rules/` 新增 `.yml` 文件，或准备独立规则包。
2. 运行 `actionguard policy-check <测试命令>` 验证规则生效。
3. 提交 PR，包含规则的测试用例和预期决策。

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
