# Facts Schema — ActionGuard 的"事实层"语言

> 状态：v0.2 定稿（2026-08-24）
> 原则：**规则面向 Facts 写，不面向原始字符串。**
> 发现新边界时，先问"它对应什么 Fact"，再写规则——而不是直接抄命令字符串。

## 1. 为什么要这一层

一条规则的质量不在 YAML 本身，而在它对"Agent 行为"的抽象是否正确。

```
Agent Action
    ↓
Fact Extractor（classify + risk 引擎）
    ↓
Normalized Facts（本 schema）
    ↓
Policy Engine（rules/*.yml）
    ↓
allow / block / ask
    ↓
Decision Ledger（Action + user_override）
```

好处：以后发现"Agent 访问 `.aws/credentials` 也要拦"，不修改底层 Hook，
只增加一条 Policy——因为 `target_path` 这个 Fact 已经存在。

## 2. Fact 字段（v0.2 实现状态）

| Fact | v0.2 实现 | 说明 |
|------|-----------|------|
| `process` | ✅ 部分 | `classify_shell_command` 的首 token（小写）。如 `git`、`npm`、`rm`。见 `policy/classify.rs` `ParsedCommand.command` |
| `parent_process` | ❌ 未实现 | Agent 进程链（Hook 层可观测，v0.2 不结构化）。P0 待办 |
| `operation` | ✅ 部分 | `ActionKind` + `kind` 动词（`execute`/`install`/`uninstall`/`publish`/`git`）。见 `policy/classify.rs` `kind_for()` |
| `target` | ✅ | `Action.target`：Shell/Git/Package = 完整命令行；File = 目标路径。规则匹配的主力字段 |
| `target_path` | ✅ | `Action.path`（File 类）+ `path_str()`。secrets 规则基于它工作 |
| `workspace` | ✅ 部分 | `Action.cwd` + 会话 workspace。v0.2 未用于规则匹配（无越界判断） |
| `branch` | ❌ 未实现 | 目前只能靠正则从命令行提取（如 git.yml 的 shared-branch 规则） |
| `network_destination` | ❌ v0.2 明确不做 | AGENTS.md 范围外（Browser/Network/API 类不新增） |
| `command` | ✅ | `ParsedCommand.command`（由 `classify_shell_command` 产出） |
| `credential_detected` | ⚠️ 部分 | secrets.yml 的路径规则 + risk 引擎的 `asset=Secret`。无显式布尔字段 |
| `privilege_level` | ❌ 未实现 | sudo/su 目前由 shell.yml 规则捕获（`deny-sudo`/`deny-su`），非结构化 |
| `session_id` | ✅ | `Action.session_id`（受保护会话内非空） |
| `source_type` | ✅ | `agent`/`automation`/`workflow`/`human`（`new_shell_from_source`）。matcher 不读它（防伪造） |
| `user_override` | ✅ v0.2.1 | `Option<bool>`：用户是否否决了引擎判断。埋点核心字段 |
| `resolved_at` | ✅ v0.2.1 | 弹窗决策时间戳 |

## 3. Operation 枚举（目标语言）

不同 Action 最终应能标准化成统一操作。v0.2 的实际枚举见
`models.rs::ActionKind`；以下是**目标粒度**，逐步对齐：

| Operation | v0.2 现状 | 覆盖类别 |
|-----------|-----------|----------|
| `FILE_READ` | ⚠️ 部分（`ActionKind::Read` + secrets 规则） | File |
| `FILE_WRITE` | ✅（`ActionKind::Write/Create/Modify`） | File |
| `FILE_DELETE` | ✅（`ActionKind::Delete` + `deny-delete-*`） | File |
| `PROCESS_EXEC` | ⚠️ 部分（Shell 类全部视为 execute，无细粒度 exec） | Shell |
| `GIT_OPERATION` | ⚠️ 部分（Git 类规则用 `command=git` + 第二个 token） | Git |
| `PACKAGE_OPERATION` | ✅（`kind_for` 产出 install/uninstall/publish） | Package |
| `CREDENTIAL_ACCESS` | ⚠️ 部分（secrets.yml 路径规则，非显式枚举） | Secret |
| `NETWORK_CONNECT` | ❌ v0.2 不做 | — |

## 4. Fact → 规则匹配映射

规则作者写 `match:` 时，字段与 Fact 的对应关系（见 `policy/matcher.rs`）：

| 规则字段 | 对应 Fact | 语义 |
|----------|-----------|------|
| `category` | 分类结果 | `Action.category` 精确匹配（Git/Package/Shell/File/Secret） |
| `command` | `process` | Shell/Package：首 token；Git：第二个 token（push/reset/clean…） |
| `args_contains` | `target` | **全部**子串必须在命令行中出现（AND） |
| `args_any` | `target` | **任一**子串出现即命中（OR，同义词场景） |
| `path` | `target_path` | 通配符路径（File 类） |
| `regex` | `target` | 作用于完整命令行（Shell/Git/Package）或路径（File） |

## 5. 给规则作者的一句话

> 写规则前先填空：`这个危险行为对应的 Facts 是什么？`
> `git push --force origin main` →
> process=`git`, operation=`push`, target 含 `--force`+`main`（branch 未结构化，用 regex）→ `deny-push-force-shared-branch`。

## 6. 演进节奏

- 每发现一个新边界，先检查：它需要的 Fact 是否已在上面？
- 缺 → 在 [BOUNDARY_BACKLOG.md](./BOUNDARY_BACKLOG.md) 记一条"Facts 缺口"，
  同时标记 P0（Hook 可靠性 / Facts Schema 是最高优先级）。
- 有了 Fact，才写规则 + Golden Test。
