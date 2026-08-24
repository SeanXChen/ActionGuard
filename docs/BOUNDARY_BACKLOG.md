# Boundary Backlog — 边界发现漏斗 + 规则治理

> 状态：v0.2 起步（2026-08-24）
> 角色：**我是 Maintainer + Boundary Curator**，不是"每天手写 YAML 的规则苦力"。
> 每天发现问题，只做一件事：**往 Backlog 加一行**。攒够一批，再批量提炼成
> Facts → Golden Test → Policy → Regression。

## 1. 规则分三层（Tier 模型）

| Tier | 定义 | 谁可关闭 | 示例 |
|------|------|----------|------|
| **Tier 0 绝对边界** | 默认永远开启，用户不应轻易关闭。破坏或泄露即不可逆 | 仅显式安全审查 | SSH 私钥、云凭证、`deny-rm-rf-root`、`deny-write-aws-creds`、sudo/su |
| **Tier 1 高风险默认防护** | 默认开启，用户可调整 | 用户可改 | force push、`reset --hard`、npm publish、全局 git config、curl/wget 远程脚本 |
| **Tier 2 环境特定策略** | 用户/企业自配，不进内置基线 | 用户自管 | production 目录、特定 repo、特定 registry/域名 |

> 内置基线的目标不是 2000 条规则全部默认开，而是
> **少量、高质量、强解释性的 Tier 0 + Tier 1 基线 + 可扩展 Tier 2 Policy**。

## 2. 流水线（边界生产）

```
┌─ 自己的真实 Agent 使用（主实验室，Shadow 捕获）
├─ 公开安全研究 / CVE / 攻击案例
├─ 传统安全工具（EDR/DLP/secrets scanner → 转译为 Facts）
├─ 开源规则生态（Semgrep/Sigma/YARA → 只看风险模型，不抄规则）
└─ 用户贡献（Boundary Report，见 CONTRIBUTING.md）
        ↓
  Boundary Candidates（本文档 Backlog 表）
        ↓
  我人工审核（判断 + 抽象）
        ↓
  Facts 定义（docs/FACTS_SCHEMA.md）
        ↓
  Golden Test（tests/golden/，先定预期）
        ↓
  YAML Policy（src-tauri/rules/*.yml）
        ↓
  Regression（cargo test --test golden_runner）
        ↓
  Built-in / Community
```

**每一条规则必须同时拥有测试**。规则 + 测试成对提交，否则不算完成
（Golden Corpus 是回归保障：加第 100 条规则不能搞坏前 99 条）。

## 3. 现有内置规则盘点（68 条，v0.2）

> 测试列：G = Golden Corpus 已覆盖，B = bypass 测试已覆盖，— = 待覆盖。

### secrets.yml（19）— Tier 0
| 规则 | 处理 | 风险 | 测试 |
|------|------|------|------|
| deny-write-env / deny-write-env-variant | Block | critical | G |
| deny-delete-pem / deny-delete-key | Block | critical | G |
| deny-write-ssh-id / deny-write-ssh-id-ed25519 | Block | critical | G |
| deny-write-aws-creds | Block | critical | G |
| deny-write-gnupg / deny-write-credentials | Block | critical | — |
| confirm-cat-env / -prod / -local | Ask | high | — |
| confirm-cat-pem / -key / -id-rsa / -id-ed25519 | Ask | high | — |
| confirm-cat-credentials-json / -yml | Ask | high | — |

### shell.yml（15）— 混合
| 规则 | 处理 | 风险 | 测试 |
|------|------|------|------|
| deny-rm-rf-root | Block | critical | G+B |
| deny-sudo / deny-su | Block | critical | G+B |
| deny-shutdown / deny-reboot | Block | high | G |
| confirm-rm-rf / -fr / -r / -f / -no-flags | Ask | high | G+B |
| confirm-curl / confirm-wget | Ask | high | G+B |
| confirm-chmod / confirm-chown | Ask | high | G |
| confirm-kill-9 | Ask | high | G+B |

### git.yml（14）— Tier 1（deny 的为 Tier 0）
| 规则 | 处理 | 风险 | 测试 |
|------|------|------|------|
| deny-push-force-shared-branch / deny-push-f-shared-branch | Block | critical | G |
| confirm-push-force-with-lease / confirm-push-force | Ask | high | G |
| confirm-reset-hard | Ask | high | G |
| confirm-clean-fd / -df / -f / -d / -ndx | Ask | medium | G |
| confirm-branch-delete-D / -d | Ask | medium | — |
| allow-push / allow-pull / allow-fetch | Allow | low | G |

### node.yml（9）+ python.yml（11）— Package，Tier 1（deny 的为 Tier 0）
| 规则 | 处理 | 风险 | 测试 |
|------|------|------|------|
| deny-npm-publish / deny-npx-rm-rf | Block | critical | G |
| confirm-npm-install-global / confirm-pip-install-upgrade | Ask | high | G |
| confirm-*-uninstall / -remove（npm/yarn/pnpm/pip/pip3/poetry/uv/conda） | Ask | medium | G |
| allow-*（install/add/pull） | Allow | low | G |

## 4. 候选边界 Backlog（未固化 → 待审核 → 进入 Golden）

> 来源：真实 Agent 行为 / 攻击案例 / 传统安全模型转译 / 社区报告。
> 每行一条；审核通过 → 提炼 Facts → 先写测试 → 再写规则。

| ID | 边界 | 风险 | 处理（预期） | Facts 缺口 | 测试 | 来源 |
|----|------|------|--------------|-----------|------|------|
| B001 | `git push --force-with-lease origin main` | high | Ask | branch 未结构化 | — | git 规则矩阵 |
| B002 | `rm -r -f /`（flag 重排绕过 `args_contains`） | critical | Block | —（已知 gap） | B（断言当前 Ask） | bypass |
| B003 | `rm -rf/`（无空白，regex 不匹配） | critical | Block | —（已知 gap） | B | bypass |
| B004 | 读取 `~/.aws/credentials` 内容 | critical | Block | credential_detected 未显式化 | — | 传统安全模型 |
| B005 | 写入 `.git/config`（remote/url/hooks 篡改） | high | Ask | — | — | 攻击案例 |
| B006 | 下载远程脚本后立即执行（curl \| sh 管道） | critical | Block | parent_process/管道 | — | 供应链攻击 |
| B007 | `npm publish` 到特定 registry | high | Ask | network_destination 未实现 | — | 社区 |
| B008 | 修改 PATH（export PATH=）持久化 | high | Ask | privilege_level/环境变量 | — | AI agent 安全研究 |
| B009 | 创建 scheduled task / 自启动项 | critical | Block | — | — | persistence 模型 |
| B010 | PowerShell encoded command（`-EncodedCommand`） | critical | Block | — | — | 传统 EDR 规则转译 |
| B011 | chmod +x 后执行 `./<下载的脚本>` | medium | Ask | — | — | 供应链 |
| B012 | `git reset --hard` 之外的破坏性 revert（checkout .） | medium | Ask | — | — | 真实使用 |
| B013 | `confirm-cat-env-prod/local` 是死规则：`cat .env.production` 先命中 `confirm-cat-env`（first-match） | low | 修复 | — | G（锁定当前行为） | Golden 套件 |
| B014 | `cat ~/.aws/credentials` 无 confirm-cat-* 覆盖 → 回退放行（写被 deny-write-aws-creds 拦，读没拦） | critical | Block/Ask | credential_detected 未显式化 | G（锁定缺口） | Golden 套件 |
| B015 | matcher 的 `args_contains` 大小写折叠，`-d`（安全删除）被 `-D` 规则遮蔽 | medium | ✅ 已修复（git.yml 改用大小写敏感 regex） | — | G | Golden 套件 |

## 5. 每周节奏

- 正常节奏：**每周新增 2–5 个候选边界**（进上面的表即可，不需要当周转成规则）。
- 每两周：把积压候选批量走一遍流水线（审核 → Facts → Golden → Policy → Regression）。
- 追求的是 **Boundary Coverage**，不是 YAML 数量。50 条高质量 > 1000 条垃圾。

## 6. 已知盲区（先承认，再测试）

- 管道 / 重定向 / shell 操作符：Hook 只看到第一条命令（见 `classify.rs` 注释）。
- `parent_process`、`branch`、`privilege_level`、`network_destination` 未结构化。
- v0.2 范围外：Browser/Network/API/SaaS/Finance 动作类（AGENTS.md）。

## 7. 维护入口

- 发现新边界：加一行到 §4 表。
- 发现规则误报：先看 `stats --export` 的 Override Rate，再动规则
  （高 Override 的规则大概率是策略问题，不是用户问题）。
- 贡献者提交边界报告：见 `CONTRIBUTING.md` 三层机制。
