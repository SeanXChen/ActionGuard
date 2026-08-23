# ActionGuard

### 在 AI 自动化触碰你的机器之前，保护它。

**本地 · 确定性 · 厂商中立。**

AI 自动化系统可以读取文件、执行 Shell 命令、修改 Git 仓库、安装包、访问密钥。ActionGuard 在自动化系统与你的机器之间，放下一道**独立的、由你掌控的安全边界**——它评估有后果的动作，并且——**在受支持的边界上**——在动作**真正影响任何东西之前**执行策略。

```
            AI-powered automation
                      │
                      ▼
              ┌──────────────┐
              │  ActionGuard │
              │              │
              │ Allow / Ask  │
              │    / Deny    │
              └──────┬───────┘
                     │
                     ▼
                  Machine
```

**无 SDK。无云端。决策路径上没有模型。**

**我们不试图保护 AI 本身。我们控制它对你的机器能做什么。**

```
$ actionguard protect

  ✓ 策略已加载
  ✓ 边界已检测
  ✓ 强制已生效

  AI 自动化尝试执行:  sudo rm -rf /

  ✗ 已拒绝 (DENIED)
```

> **自动化系统不应是自己行动的最终权威。**

它不监视某个品牌，它监视**边界**——自动化动作进入你系统的那个入口。编码 Agent 只是今天最容易验证的入口；桌面自动化、浏览器自动化、脚本、以及更自主的系统，属于同一类问题。**动作来源只是元数据，Boundary 才是真正的安全原语。** ActionGuard 建立在**可扩展的边界模型**之上，而非厂商专用集成：任何暴露了受支持动作边界的自动化系统都可以被强制。

> **当前范围。** ActionGuard 聚焦**本地文件、Shell、Git、Package、Secret 五类边界**，强制程度因集成与平台而异。浏览器、网络、API、远程自动化目前不在范围内。

> **按边界接入，而非按品牌接入。** [边界登记表](./BOUNDARIES.md) 记录每个边界真正能强制什么——**已验证 / 仅观察 / 不受支持**——所以「支持」永远意味着*实测过*，而不是*宣传过*。

> **诚实原则**：此 README 只宣称**今天已验证**的能力——在真实机器上测过、并记录在 [安全测试矩阵](./SECURITY_TEST_MATRIX.md) 里的能力。没有任何「计划中」被伪装成「已完成」。

> **检测永远不等于强制。**「我们记录了」不是「我们拦截了」。下方保护矩阵中每一项声明都会写明：对每一条路径，哪一个是真实的。

---

## 📋 目录

- [ActionGuard 是什么](#actionguard-是什么)
- [为什么存在](#为什么存在)
- [安全姿态](#安全姿态)
- [10 秒演示](#10-秒演示)
- [当前保护矩阵](#当前保护矩阵)
- [快速开始](#快速开始)
- [工作原理](#工作原理)
- [已知限制](#已知限制)
- [反馈](#反馈)
- [贡献](#贡献)

---

## ActionGuard 是什么

AI 自动化现在被授权修改你的文件、执行你的 Shell、安装包、触碰凭据。工具信任模型。ActionGuard 是**不信任**的那一层——一个坐在动作边界上的**确定性**策略引擎，在任何有后果的动作运行前说 *放行*、*拒绝* 或 *问一下人*。

```
AI 自动化想要执行：

    sudo rm -rf /

ActionGuard

    严重
    拒绝

  ✓ 命中策略
  ✓ 动作被拦截
  ✓ 证据已记录
```

ActionGuard 建立在**可扩展的边界模型**之上，而非厂商专用集成。不同的自动化系统可能暴露不同的强制点——工具钩子、执行审批层、受保护运行时、以及系统级边界。今天它适用于 **AI 编程工具以及任何暴露了受支持动作边界的自动化系统**——它不限于某个品牌、也不限于某一类 Agent。

当前范围（**v0.2**）：本地机器上的 **File（文件）/ Shell / Git / Package（包）/ Secret（密钥）** 五类动作。仅此而已。浏览器、网络、API/SaaS、远程自动化**明确不在** v0.2 内。

---

## 为什么存在

- **动作边界就是攻击面。** AI 自动化默认拥有 `execute` 权限，但没有任何东西核实它实际做了什么。终端执行是 v0.2 中 ActionGuard 保护的第一个边界。
- **检测 ≠ 防护。**「我们记录了」不等于「我们拦截了」。ActionGuard 在它支持的每条路径上都区分这两者——见[保护矩阵](#当前保护矩阵)。
- **安全碎片化本身就是安全缺口。** 随着用户采用越来越多的自动化工具，其安全控制会分散在各厂商各自的策略与审批系统之间，彼此互不相通。ActionGuard 提供**一个**横跨各 Boundary 的独立策略层，而不是让用户去信任 N 个厂商各自的机制。
- **厂商各自的控制会产生不一致的安全边界。** 只保护一个 Agent 而漏掉另一个，就留下了缺口。因为 ActionGuard 挂在**边界**上而非品牌上，任何暴露可挂钩预执行边界的工具都能被强制——而无法做到的，会被**明确标注**，绝不暗示。
- **确定性，不是「AI 味道」。** 什么危险不危险，不由黑箱模型决定。风险分类跑在你**能读、能改**的显式规则上。
- **诚实本身就是功能。** 一个过度承诺的安全产品会让用户更不安全。本文档里的每一项强制声明都有测试记录背书。

---

## 安全姿态

- **本地优先** — 无遥测、无需账号。一切都在你自己的机器上。
- **默认关闭式（fail-closed）** — 引擎不可达、响应无法解析、或没有活跃会话时，每个强制点都**拒绝**。（唯一的有意例外：干净关闭的会话——终端永远不能被砖；`AG_ALLOW_ON_FAILURE=1` 显式选择才回到 fail-open。）
- **开放验证** — 强制声明由 [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md) 中的可复现测试背书，而不是营销文案。

---

## 10 秒演示

先干跑——只做决策，不执行：

```bash
actionguard policy-check "git reset --hard HEAD~1" --explain
```

```text
决策:       询问 (ASK)
风险:       高 (HIGH)
规则:       git-reset-hard
原因:       破坏性仓库重写
边界:       ProtectedShell
模式:       DRY RUN
```

再看真实强制——在受保护会话内，同样的命令根本不会执行：

```bash
actionguard protect ./my-project
# 自动化系统运行破坏性命令，例如  sudo rm -rf /
```

```text
→ 拒绝 (DENY)
→ 已强制 (ENFORCED)
→ 命令未执行
```

`policy-check` 从不执行任何东西。强制只发生在活跃的受保护会话内。

---

## 当前保护矩阵

> **实测的边界，而不是营销的勾选。** 状态于 **2026-08-21 在 ActionGuard v0.2 上实测**；每行保留各自的「最后验证」。 「Enforced」= 动作在**执行前**被拦截。「Observe-only」= 被记录，但不阻止。你也可以在自己机器上实时查看：运行 `actionguard boundary list` 或 `actionguard capabilities`。完整登记表见 [BOUNDARIES.md](./BOUNDARIES.md)。

| 边界 | 观察 | 强制 | 验证 | 最后验证 |
|---|---|---|---|---|
| CodeBuddy PreToolUse hook | ✅ | ✅ | **核心已验证** — 真实 `sudo rm -rf /` 在执行前被拒绝 | 2026-08-19 |
| 受保护 Shell — bash | ✅ | ✅ | 核心已验证 | 2026-08-19 |
| 受保护 Shell — zsh | ✅ | ✅ | 核心已验证 | 2026-08-19 |
| 受保护 Shell — fish | ✅ | ✅ | 核心已验证 | 2026-08-19 |
| PowerShell — 交互式 (PSReadLine) | ✅ | ✅ | **Phase C** — block + exit 126，marker 保留；需要活跃受保护会话 | 2026-08-21 |
| PowerShell — 脚本 / `-Command` / 管道输入 | ✅ | ❌ | **仅观察** — 已实测绕过（marker 被删） | 2026-08-21 |
| 直接子进程（`os.system`、绝对路径） | ✅ | ❌ | **仅观察** — 已知绕过路径 | 2026-08-19 |
| Claude Code | ⏳ 已记录 | ❓ | 已记录 — 未验证强制 | — |
| Cursor | ⏳ 已安装 | ❓ | 已记录 — 已安装，未接入 | — |
| Codex | ❓ | ❓ | 已记录 — ExecApproval，不可扩展，未验证强制 | — |
| OpenClaw | ❓ | ❓ | 调查中 — 候选独立策略层（ExecApproval） | — |
| Manus Desktop (My Computer) | ❓ | ❓ | 调查中 — 候选第二策略层（ExecApproval） | — |
| Manus Cloud | 不适用 | 不适用 | 远程 — 本地工具无法强制 | — |

> 两层真相：**Boundary 类型**（动作从哪里进入——工具钩子 / 受保护 Shell / 执行审批…）与**强制状态**（当前到底能不能拦）。能力层级细节（L1–L4）在 [SECURITY_MODEL.md](./SECURITY_MODEL.md)——README 刻意不使用它们。

> **关于绕过的诚实声明。** ActionGuard 只强制**经过受支持边界**的动作。绕过边界的动作会被观察、记录，并标记为 **Bypassed / Unsupported（已绕过 / 不受支持）**——绝不会被默认为已拦截。「我们看到了但没拦住」对一条不受支持的路径而言是正确的、诚实的结果，而不是产品失效。

### 平台状态

> **构建 ≠ 已验证。** CI 在三个平台上都执行编译、测试与安装生命周期——但**只有存在真实机器测试记录时，才会做出「强制」声明**（见 [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md)）。macOS 或 Linux 上的构建通过，不代表这些平台已被完整验证为「强制生效」。

| 平台 | CI 构建 | 生命周期（setup → doctor → uninstall） | 真实机器强制验证 |
|---|---|---|---|
| Windows | ✅ | ✅ | ✅ PowerShell 交互式（Phase C）、受保护 Shell |
| Linux | ✅ | ✅ | ✅ 受保护 Shell（bash/zsh/fish） |
| macOS | ✅ | ✅ | ⏳ **构建可用——强制能力需平台特定验证**（Gatekeeper、权限、shell hook 行为） |

macOS 与 Linux 二进制已发布，供早期用户帮助验证真实强制效果。如果你使用这些平台，提交一份[边界验证报告](./docs/CONTRIBUTING.md)就是你能做出的最有价值的贡献。

---

## 快速开始

> **面向非 Rust 开发者**：从 **GitHub Releases** 下载二进制（无需 Rust 工具链）。Release 附带 `SHA256SUMS` 文件——运行前请校验，例如 `sha256sum actionguard` 或 Windows 下 `Get-FileHash actionguard.exe -Algorithm SHA256`。源码构建为次要路径。

### 1. 安装

**从 GitHub Releases 安装** — 下载对应平台的 `actionguard` 二进制，校验 checksum，加入 `PATH`。

**从源码构建**：

```bash
npm install
cargo build --release --bin actionguard
```

### 2. 一键设置

```bash
actionguard setup
```

自动检测你的 OS 与 Shell，预览每一项改动，创建 `~/.actionguard`，安装内置规则包，安装 Shell 钩子，并执行自检。**默认本地安装无需 root/admin 权限。**

### 3. 验证

```bash
actionguard doctor --test
```

运行一次非破坏性的端到端边界测试，对已检测/可强制的边界逐条打印 ✓/✗ —— 是证据，不是承诺。

### 4. 保护

```bash
actionguard protect ./my-project
```

启动受保护会话：高风险动作在运行前必须经过策略引擎和审批闸（`allow` / `deny`）。

### 5. 不执行，先检查

```bash
actionguard policy-check "git reset --hard HEAD~1" --explain
```

---

## 工作原理

```
  动作来源（CodeBuddy、Shell、脚本、Agent…）
        │
        ▼
   边界（Boundary）  ← 动作跨入系统的入口（hook / shell / 审批）
        │
        ▼
   ActionGuard 核心
        ├── 分类（Classify）   → 类别 + 风险（File / Shell / Git / Package / Secret）
        ├── 策略（Policy）     → YAML 规则 → 放行 / 询问 / 拒绝
        ├── 审批（Approval）   → 问人（CLI 或 GUI）；超时默认拒绝
        └── 证据（Evidence）   → 追加式账本，逐动作记录，统计
```

- **观察与强制是「边界」的属性，不是核心的属性。** 一个边界要么是 `Enforced`（执行前拦截）、要么是 `Observe-only`（事后记录）、要么是 `Not detected`（未检测到）——登记表里的每个边界都精确标注其一。
- **按边界接入，而非按品牌接入。** 核心引擎从不特判某个 Agent 品牌。接入新的自动化来源 = 把它映射到一个边界类型，而不是做集成。
- **确定性策略。** 决策值只有 `Allow` / `Ask` / `Deny`。`Warn` 不是决策——它是策略结果上的一个注解，展示在 UI 和账本中。
- **默认关闭式（fail-closed）。** 引擎不可达时，每个强制点**拦截**而非放行。

### 动作分类（v0.2）

| 类别 | 示例 | 风险引擎关注 |
|---|---|---|
| **File 文件** | 创建 / 修改 / 删除 / 重命名 | 敏感路径、工作区外写入 |
| **Shell** | `rm`, `chmod`, `sudo`, `curl` | 危险模式、不可逆操作 |
| **Git** | `reset --hard`, `clean -f`, 强推 | 破坏性引用操作 |
| **Package 包** | `npm`, `pnpm`, `pip`, `cargo` | 不可信安装 |
| **Secret 密钥** | `.env`、SSH 密钥、凭据 | 访问或外泄 |

这些是**当前 v0.2 的动作类别**。新类别（浏览器、网络、API、金融）在边界模型验证完成前**刻意冻结**——不是因为没想到，而是故意不扩。

敏感资产会被风险引擎**识别**（家目录、`.git`、`~/.ssh`、`.env`）；对它们的改动是否真的被**拦截**，取决于当前生效的边界——见[保护矩阵](#当前保护矩阵)。**检测永远不等于强制。**

### 规则

内置规则包（`shell.yml`、`git.yml`、`node.yml`、`python.yml`、`secrets.yml`）随二进制发布。用户可添加 YAML 规则：

```yaml
id: risky-shell-operations
category: shell
severity: high
match:
  patterns:
    - "rm -rf /*"
    - ":(){ :|: & };:"
action:
  decision: ask
```

管理命令：`actionguard policy-list` · `policy-lint <file>` · `policy-edit` · `policy-path` · `rule search` · `rule install <file.yml>`。

### CLI 一览

```bash
actionguard setup            # 一键安装
actionguard doctor --test    # 验证已检测/可强制的边界
actionguard status           # 会话 / hook 状态
actionguard protect <dir>    # 受保护会话
actionguard policy-check <cmd> --explain
actionguard allow | deny [id]
actionguard stats | capabilities | boundary list
actionguard rule search <q> | rule install <file.yml>
```

完整命令参考随二进制提供——运行 `actionguard --help` 即可查看。本清单已对照 `--help` 逐一核对（v0.2）。

---

## 已知限制

这些是真实的、实测的限制——与 `actionguard doctor` 打印的内容一致：

1. **Windows 上 PowerShell 仅交互式生效。** 交互式行已强制（Phase C——2026-08-21 由 `scripts/tests/verify-powershell-phase-c.ps1` 实测验证）；脚本、`-Command`、管道输入仅观察。登记表将其建模为**两个独立条目**（`PowerShell (PSReadLine interactive)` = 已强制，`PowerShell (script/-Command/piped)` = 仅观察），`boundary list`、`capabilities`、`doctor` 全部反映同一拆分。
2. **直接子进程可绕过 Shell 钩子。** `os.system`、`/usr/bin/rm` 等绝对路径或非 Shell 派生子进程——只观察，不阻止。
3. **远程自动化不在范围内。** 在另一台机器上执行的动作（如 Manus Cloud）无法被本地工具强制。
4. **已验证的边界很少。** CodeBuddy 与受保护 Shell 为核心已验证。Cursor、Windsurf、OpenClaw、Codex、Manus 是「已记录」或「调查中」——尚未强制。
5. **尚无沙箱。** 在隔离环境（容器 / 网络隔离）中运行动作属于未来工作。
6. **`undo` 在 v0.2 的 CLI 中未暴露。** v0.1 的快照/恢复机制仍存在于 GUI 流程背后；CLI `undo` 命令计划于 v0.3。

---

## 反馈

ActionGuard 是本地优先的：**产品内没有任何遥测**。作为替代，我们主动倾听——我们最需要的是你的真实使用体验。

- 🛡 **它拦住了什么吗？** 告诉我们你的第一次拦截——拦住了什么、你当时怎么想：[打开反馈表单](https://github.com/SeanXChen/ActionGuard/issues/new?template=feedback.yml)（只需 1 分钟）。
- 🐞 **安装、启动或保护失败了吗？** [提交 bug 报告](https://github.com/SeanXChen/ActionGuard/issues/new?template=bug_report.yml)——这能帮我们找到用户在哪个环节流失。
- 💬 更喜欢自由讨论？[发起讨论](https://github.com/SeanXChen/ActionGuard/discussions)。

我们会在 [docs/USER_VALIDATION.md](./docs/USER_VALIDATION.md) 中追踪这些信号。

---

## 贡献

欢迎贡献——尤其是**边界验证报告**（本项目的核心资产）。见 [CONTRIBUTING.md](./docs/CONTRIBUTING.md)（报告模板、规则格式、代码规范、安全漏洞报告）。

另见：[SECURITY_MODEL.md](./SECURITY_MODEL.md) · [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md) · [BOUNDARIES.md](./BOUNDARIES.md) · [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)

---

## 许可证

[Apache-2.0](./LICENSE)

---

## 语言

- [English](./README.md)
- [简体中文](./README.zh.md)
