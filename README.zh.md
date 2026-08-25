# ActionGuard

### 给 AI 干活的空间。对它能做什么，保持控制。

**在 AI 自动化行动之前，保护你的机器。**

**本地 · 确定性 · 厂商中立。** · 无云端 · 无 SDK · 决策路径上没有模型。

**在受支持的边界上，拦截高影响的文件、Shell、Git、包与密钥动作。**

AI 自动化系统可以读你的文件、执行你的 Shell、改写你的 Git 历史、安装软件包、触碰你的密钥。ActionGuard 在自动化系统与你的机器之间，放下一道**独立的、由你掌控的安全边界**：它评估有后果的动作，回答 **放行 / 询问 / 拒绝**——并且**在受支持的边界上**，在动作**真正影响任何东西之前**执行这个决定。

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

<p align="center">
  <a href="#现在试试"><strong>试试 ActionGuard →</strong></a>
  &nbsp;·&nbsp;
  <a href="#今天已验证">查看已验证的边界</a>
  &nbsp;·&nbsp;
  <a href="https://github.com/SeanXChen/ActionGuard">GitHub</a>
</p>

> **自动化系统不应是自己行动的最终权威。**
>
> ActionGuard 提供一个**独立、确定性**的策略层，坐在动作边界上——所以「想动手」的那个系统，永远不是唯一「决定能不能动手」的那个系统。

---

## 📋 目录

- [看它工作——真实机器上](#看它工作真实机器上)
- [它给谁用？](#它给谁用)
- [为什么你需要它](#为什么你需要它)
- [为什么不用自动化自带的权限就够了？](#为什么不用自动化自带的权限就够了)
- [ActionGuard 保护什么](#actionguard-保护什么)
- [现在试试](#现在试试)
- [今天已验证](#今天已验证)
- [工作原理](#工作原理)
- [ActionGuard 今天能强制什么——不能强制什么](#actionguard-今天能强制什么不能强制什么)
- [反馈](#反馈)
- [公司使用 ActionGuard？](#公司使用-actionguard)
- [贡献](#贡献)
- [许可证](#许可证)

---

## 看它工作——真实机器上

> **AI 尝试一个有破坏性的动作。ActionGuard 在执行前评估它。**

```
   CodeBuddy / 受保护 Shell
                │
                ▼
      sudo rm -rf /
                │
                ▼
        ActionGuard
                │
                ▼
    需要批准 (APPROVAL REQUIRED)  →  已拒绝 (DENY)
```

*这就是已验证边界在你机器上走的真实路径——不是模拟。下面的 CodeBuddy
PreToolUse 钩子、受保护 Shell（bash/zsh/fish）、交互式 PowerShell 都在
[今天已验证](#今天已验证)中实测过，并可用 `actionguard doctor --test` 复现。*

## 它给谁用？

任何让 AI 自动化在自己的电脑上动手的人：

- **开发者**——使用 Codex、Claude Code、Cursor、CodeBuddy 等编码 Agent。
- **AI 重度用户**——让自动化处理文件、脚本、应用或本地工作流，不需要会编程。
- **任何运行自主自动化的人**——想在自动化与自己的机器之间加一道安全边界。

ActionGuard **面向使用 AI 自动化系统的人**——如 Codex、Claude Code、Cursor、OpenClaw、Manus。它**当前已验证的强制能力**是：CodeBuddy PreToolUse 钩子、受保护 Shell（bash/zsh/fish）、以及 Windows 交互式 PowerShell。其余均已记录或处于调研中——见[今天已验证](#今天已验证)。没有实测过的东西，绝不暗示。

---

## 为什么你需要它

AI 已经可以修改文件、执行命令、无人值守地连续工作数小时。问题不在于给不给 AI 访问权。**问题在于给了它不受约束的处置权。**

- **动作边界就是攻击面。** AI 自动化默认拥有 `execute` 权限。内置权限与沙箱控制的是*访问权*——但它们未必能为每一个有后果的动作提供独立的策略。
- **检测 ≠ 防护。**「我们记录了」不等于「我们拦截了」。ActionGuard 在它支持的每条路径上都区分这两者。
- **审批疲劳是真实存在的。** 每一步都要批准时，人会开始习惯性点「允许」。ActionGuard 只在动作真正越界时才介入。
- **安全碎片化本身就是安全缺口。** 分散在 N 个厂商各自审批系统里的控制彼此不相通。ActionGuard 提供**一个**横跨各边界、彼此独立的策略层。
- **确定性，不是「AI 味道」。** 什么危险不危险，不由黑箱模型决定。风险分类跑在你**能读、能改**的显式规则上。
- **诚实本身就是功能。** 一个过度承诺的安全产品会让用户更不安全。本文档里的每一项强制声明都有测试记录背书。

> **让它跑。** ActionGuard 只在动作越过你的安全边界时才介入——所以你可以放心让 AI 干活，而不是每一步都盯着。

---

## 为什么不用自动化自带的权限就够了？

问得好。内置控制回答的是「自动化**能访问**什么？」——ActionGuard 回答的是另一个问题：「自动化**能用这些访问权做什么**？」

> **沙箱控制 AI「能去哪里」。**
> **ActionGuard 控制它「能在那里做什么」。**
>
> *访问权是能力。后果是策略。* 拥有某资源的*访问权*，不代表自动化有权对该资源执行有后果的*动作*——这个决定属于策略，不属于自动化。

下面三个例子——三种情况下，自动化都拥有完全访问权：

```
Agent 可以访问你的项目        →   git reset --hard
Agent 可以写入你的工作区      →   rm .env
Agent 可以安装软件包          →   npm publish
```

内置权限由厂商设定，只在那一款产品内生效，而且厂商自己的自动化是自己行为的最终裁判。ActionGuard 是**独立**的——它不由同一厂商编写、不运行在同一进程内、并且用**同一套策略**横跨你使用的所有自动化。这份独立性正是关键所在：自动化系统不应是自己行动的最终权威。

---

## ActionGuard 保护什么

| | 拦截什么 | 风险引擎关注什么 |
|---|---|---|
| **文件 Files** | 创建 / 修改 / 删除 / 重命名 | 敏感路径、工作区外写入 |
| **Shell** | `rm`, `chmod`, `sudo`, `curl` | 危险模式、不可逆操作 |
| **Git** | `reset --hard`, `clean -f`, 强推 | 破坏性引用操作 |
| **包 Packages** | `npm`, `pnpm`, `pip`, `cargo` | 不可信安装 |
| **密钥 Secrets** | `.env`、SSH 密钥、凭据 | 访问或外泄 |

这些是**当前 v0.2 的动作类别**，在本地强制执行并记入追加式账本。浏览器、网络、API、远程自动化**刻意不在** v0.2 内。

---

## 现在试试

> **本地优先。无账号。无云端遥测。** 一切都在你自己的机器上——不向任何地方回传数据。

**30 秒，无需 Rust 工具链**——从 **GitHub Releases** 下载二进制（校验和见 `SHA256SUMS`）。

**给开发者——CLI：**

```bash
actionguard setup           # 一键安装：Shell 钩子 + 规则包 + 自检
actionguard doctor --test   # 非破坏性端到端测试——逐边界打印 ✓/✗
actionguard protect ./my-project   # 启动受保护会话
```

**给 AI 用户——GUI：**

想要点点鼠标就能开始？打开 ActionGuard → **保护这台电脑（Protect this computer）**。一个按钮、大白话计数、无需命令。

### 看你的第一个决策

想先看一个「只决策、不执行」的例子？

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

`policy-check` 从不执行任何东西。强制只发生在活跃的受保护会话内——在那里，同样的命令在运行前就会被拒绝：

```text
→ 拒绝 (DENY)
→ 已强制 (ENFORCED)
→ 命令未执行
```

这就是完整闭环：动作到达 → ActionGuard 决策 → 账本记录 → 没有有害的东西运行。默认本地安装无需 root/admin 权限。

---

## 今天已验证

> **真实机器实测，不是营销。** 状态于 **2026-08-21 在 ActionGuard v0.2 上实测**；每个边界的完整细节与日期在 [BOUNDARIES.md](./BOUNDARIES.md)。每条声明都可用 `actionguard boundary test` 复现，并记录在 [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md)。

| 边界 | 状态 |
|---|---|
| CodeBuddy PreToolUse 钩子 | ✅ 已强制 |
| 受保护 Shell — bash / zsh / fish | ✅ 已强制 |
| PowerShell — 交互式（PSReadLine，Windows） | ✅ 已强制 |
| PowerShell — 脚本 / `-Command` / 管道输入 | ⚠️ 仅观察 |
| 直接子进程（`os.system`、绝对路径） | ⚠️ 仅观察 |
| Claude Code | 🔬 已记录 — 未验证 |
| Cursor | 🔬 已记录 — 未验证 |
| Codex | 🔬 已记录 — 未验证 |
| OpenClaw | 🔬 调研中 |
| Manus Desktop (My Computer) | 🔬 调研中 |
| Manus Cloud | 不适用 — 远程，范围外 |

`已强制` = 动作在**执行前**被拦截。`仅观察` = 被记录但不阻止——并且会被明确标注为「仅观察」，绝不暗示为防护。平台状态（Windows / Linux / macOS）见 [BOUNDARIES.md](./BOUNDARIES.md)。

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
- **按边界接入，而非按品牌接入。** 核心引擎从不特判某个自动化品牌。接入新的自动化来源 = 把它映射到一个边界类型，而不是做集成。

### 动作边界——问任何自动化工具的问题

把「ActionGuard 能集成工具 X 吗？」换成「工具 X 有我们可以可靠拦截的动作边界吗？」——有 → 写适配器。没有 → 仅观察。不稳定/未文档化 → 先观察，以后通过系统级强制实现。这就是为什么边界登记表按**执行路径**而非按工具品牌列出。

- **确定性策略。** 决策值只有 `Allow` / `Ask` / `Deny`。`Warn` 不是决策——它是策略结果上的一个注解，展示在 UI 和账本中。
- **默认关闭式（fail-closed）。** 引擎不可达时，每个强制点**拦截**而非放行。

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

## ActionGuard 今天能强制什么——不能强制什么

- **本地优先** — 无遥测、无需账号。一切都在你自己的机器上。
- **默认关闭式（fail-closed）** — 引擎不可达、响应无法解析、或没有活跃会话时，每个强制点都**拒绝**。（唯一的有意例外：干净关闭的会话——终端永远不能被砖；`AG_ALLOW_ON_FAILURE=1` 显式选择才回到 fail-open。）
- **开放验证** — 强制声明由 [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md) 中的可复现测试背书，而不是营销文案。

这些限制是真实的、实测的——与 `actionguard doctor` 打印的内容一致：

1. **Windows 上 PowerShell 仅交互式生效。** 交互式行已强制（Phase C——2026-08-21 实测）；脚本、`-Command`、管道输入仅观察。
2. **直接子进程可绕过 Shell 钩子。** `os.system`、`/usr/bin/rm` 等绝对路径或非 Shell 派生子进程——只观察，不阻止。
3. **远程自动化不在范围内。** 在另一台机器上执行的动作（如 Manus Cloud）无法被本地工具强制。
4. **已验证的边界很少。** CodeBuddy 与受保护 Shell 为核心已验证。Cursor、Claude Code、Codex、OpenClaw、Manus 是「已记录」或「调查中」——尚未强制。
5. **尚无沙箱。** 在隔离环境（容器 / 网络隔离）中运行动作属于未来工作。
6. **`undo` 在 v0.2 的 CLI 中未暴露。** v0.1 的快照/恢复机制仍存在于 GUI 流程背后；CLI `undo` 命令计划于 v0.3。

> **关于绕过的诚实声明。** ActionGuard 只强制**经过受支持边界**的动作。绕过边界的动作会被观察、记录，并标记为 **Bypassed / Unsupported（已绕过 / 不受支持）**——绝不会被默认为已拦截。「我们看到了但没拦住」对一条不受支持的路径而言是正确的、诚实的结果，而不是产品失效。

---

## 反馈

ActionGuard 是本地优先的：**产品内没有任何遥测**。作为替代，我们主动倾听——我们最需要的是你的真实使用体验。

- 🛡 **它拦住了什么吗？** 告诉我们你的第一次拦截——拦住了什么、你当时怎么想：[打开反馈表单](https://github.com/SeanXChen/ActionGuard/issues/new?template=feedback.yml)（只需 1 分钟）。
- 🐞 **安装、启动或保护失败了吗？** [提交 bug 报告](https://github.com/SeanXChen/ActionGuard/issues/new?template=bug_report.yml)——这能帮我们找到用户在哪个环节流失。
- 💬 更喜欢自由讨论？[发起讨论](https://github.com/SeanXChen/ActionGuard/discussions)。

我们会在 [docs/USER_VALIDATION.md](./docs/USER_VALIDATION.md) 中追踪这些信号。

---

## 公司使用 ActionGuard？

ActionGuard 是本地优先的，但我们正在积极探索**团队部署与企业场景**——共享策略、审计友好导出、托管规则包。

如果你在问「这能用在一个团队里吗？」，我们很想听：[发起讨论](https://github.com/SeanXChen/ActionGuard/discussions)或[提交企业咨询 issue](https://github.com/SeanXChen/ActionGuard/issues/new)。这一个信号比十几个 star 更有价值。

---

## 贡献

欢迎贡献——尤其是**边界发现**（一个 ActionGuard 还没处理、但确实危险的自动化行为——不需要写 YAML）和**边界验证报告**（本项目的核心资产）。见 [CONTRIBUTING.md](./docs/CONTRIBUTING.md)（报告模板、规则格式、代码规范、安全漏洞报告）。

另见：[SECURITY_MODEL.md](./SECURITY_MODEL.md) · [SECURITY_TEST_MATRIX.md](./SECURITY_TEST_MATRIX.md) · [BOUNDARIES.md](./BOUNDARIES.md) · [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) · [docs/FACTS_SCHEMA.md](./docs/FACTS_SCHEMA.md) · [docs/BOUNDARY_BACKLOG.md](./docs/BOUNDARY_BACKLOG.md)

---

## 许可证

[Apache-2.0](./LICENSE)

---

## 语言

- [English](./README.md)
- [简体中文](./README.zh.md)
