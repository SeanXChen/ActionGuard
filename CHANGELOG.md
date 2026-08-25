# Changelog

All notable changes to ActionGuard are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

**Product positioning.** ActionGuard is explicitly a **local, vendor-neutral
safety layer for AI-powered automation** — a unified Action Policy / Evidence
layer across all automation sources, not "another exec approval" for any
specific agent. We protect *actions*, we do not attempt to identify which brand
of agent issued them.

### Changed

- **README reordered around user-facing narrative.** The first screen now
  answers the five questions a new visitor asks in five seconds: *what is it,
  why do I need it, how does it decide, is it real, can I try it now.* New
  sections: **Who is it for?** (developers → AI power users), **Why not just
  use the automation's built-in permissions?** (*Sandbox controls where it can
  go. ActionGuard controls what it can do there.*), and the **Capability ≠
  Consequence** framing. The protection matrix is condensed to a *Verified
  today* table up front (with honest Research / Observe-only labels); the full
  per-boundary matrix, dates, and platform status stay in `BOUNDARIES.md`.
  Known limitations are kept, moved below the try-it-now path. `index.html`
  GUI title fixed from "AgentGuard" to "ActionGuard".
- **GUI consumer entry point — "Protect this computer".** The Home view now
  opens with a simple, product-facing card instead of the developer workflow:
  *Protect this computer → onboarding (This computer · Recommended · the five
  protected areas) → Protection Active* with plain-language counters
  (allowed / reviewed / blocked) and an **Activity** panel that renders the
  ledger as "what AI did" (human-readable lines, "Why was this blocked?" with
  rule + decision on denied actions). The developer flow (workspace picker,
  observe/protected modes, key metrics) moved unchanged into a collapsible
  **Advanced** section — same core, one GUI, two information densities. No
  new backend capability; the consumer flow calls the existing session engine
  with the user home folder as the protected scope. CLI-start (`actionguard
  protect <ws>`) still auto-starts the developer flow.
- **Detection ≠ Protection is now visible in every output.** `actionguard
  stats` and `actionguard session show` split the old single "Actions
  Protected" number into **Detected** (recorded) / **Blocked** (deny
  decisions) / **Enforcement** (`enforced` / `observed` / `bypassed` /
  `unsupported`). The GUI dashboard gains an enforcement row, and the
  Home metric is relabeled *Actions Detected* so nobody mistakes "recorded"
  for "stopped". New `EnforcementCounts` on `SessionSummary` / active stats
  (zero-filled for old sessions; no schema break).
- **Capability Tier Model (L1 → L4).** New `CapabilityTier` in `models.rs`
  (`L1 Observe` / `L2 Pre-action` / `L3 Runtime (future)` / `L4 System
  (future)`). New CLI subcommand **`actionguard capabilities`** prints the
  tier model plus the live execution-path matrix with each path's tier
  (`not covered` when the path has no boundary). `ExecutionPathDto` and the
  GUI `EnforcementPanel` show the tier column too.
- **README: vendor-neutral positioning.** Tagline is now *"ActionGuard is a
  local, vendor-neutral safety layer for AI-powered automation."* The
  enforcement claim is sharpened to *"v0.2 can enforce actions only on
  supported execution paths; other automation sources may be observed but not
  blocked."*
- **New decision framework: Action boundary.** Replace the question *"can
  ActionGuard integrate with tool X?"* with *"does tool X have an Action
  boundary we can reliably intercept?"* — Yes → build an adapter. No → observe.
  Unstable/undocumented → observe now, enforce later via system-level
  enforcement. This keeps the roadmap independent of any vendor's API churn.
- **SECURITY_TEST_MATRIX: brand matrix → execution path matrix.** The top
  matrix is now per **execution path** (Observe / Pre-action enforcement /
  Status) instead of per tool brand, including *Automation with a supported
  hook → Adapter required* and *Automation without a hook → Observe-only*.
- **First verified Action boundary: CodeBuddy PreToolUse hook (2026-08-19).**
  New adapter `scripts/hooks/ag-hook.py` forwards every real Bash action from
  CodeBuddy's `PreToolUse` hook to `actionguard policy-check` and enforces the
  verdict (`allow` / `ask` / `deny`). Measured in a real session:
  `sudo rm -rf /` → `deny` (`deny-sudo`) → **blocked before execution**,
  reason relayed to the agent. Fail-closed by default (`AG_ALLOW_ON_FAILURE=1`
  opts back into fail-open). This proves ActionGuard can stand *between* an AI
  automation and a real-world action — not merely monitor the machine.
- **Fail-closed by default at every enforcement point.** Shell hooks (bash /
  zsh / fish / PowerShell) and the CodeBuddy adapter now BLOCK the command when
  the policy engine is unreachable, the response is unparseable, or there is no
  active session — previously they allowed silently. `AG_ALLOW_ON_FAILURE=1`
  explicitly opts back into fail-open. One deliberate exception: a clean
  session stop writes a `current.closed` sentinel, so a terminal left open
  after `actionguard stop` keeps working instead of being bricked. The bridge
  already returned `deny` on timeout in v0.2; the hook/adapter layer now
  matches that posture.
- **Same vendor, two paths.** CodeBuddy's hook path is now **enforced**; its
  spawn path (`powershell -Command` subprocess) remains **observe-only** —
  exactly why the matrix is per execution path, not per tool brand. Every
  other agent stays "unverified" until measured through the same protocol.

### Added

- **Clean-machine install scripts.** `scripts/install.ps1` (Windows
  PowerShell) and `scripts/install.sh` (POSIX) check the toolchain (Node 18+,
  Rust, optional Python), run `npm install`, build the CLI, and run
  `actionguard setup --yes`. Non-interactive by default so they work on a
  fresh box / CI; `--yes` / `-SkipBuild` flags supported.
- **Data-driven Boundary Registry** — `boundaries/*.yml`. The built-in
  registry rows are now shipped as per-product YAML files
  (11 products, classes A–F) plus `boundaries/README.md` documenting the
  schema, the class legend, and the verification policy (`core` /
  `community` / unverified). Running `actionguard boundary list` from a repo
  checkout reads straight from YAML; live probes still overlay matching rows.
- **`SECURITY_MODEL.md`** — the source of truth for ActionGuard's guarantees:
  the Detection ≠ Protection model, the four capability tiers, fail-closed
  semantics, trust boundaries, the A–F class table, and the known bypass
  vectors for v0.3. Linked from the README.
- **`actionguard setup` / `uninstall` / `doctor` subcommands.** One-shot
  install path: detect OS/shell, preview rc-file edits, create
  `~/.actionguard`, install built-in rules + shell hook, run a self-check
  (`setup --yes`). `uninstall` removes only the marker-delimited hook block and
  leaves the ledger in place. `doctor` reports policy / hook / bridge / boundary
  status and supports `--test` for an end-to-end enforcement probe. Windows
  shell detection prefers `pwsh` (PowerShell 7) with fallback to
  `powershell` (5.1); rc-file paths are mapped per shell.
- **Real-world boundary probes (2026-08-19).** Four mainstream automation
  tools measured instead of assumed. **Cursor 3.11.13** (installed on this
  machine): official `hooks.json` confirmed in the binary —
  `beforeShellExecution` / `preToolUse` / `beforeMCPExecution` /
  `beforeReadFile`, `permission: deny` (exit 2, Claude Code-compatible),
  `failClosed` option, defaults fail-open. **Claude Code**: official
  `PreToolUse` / `PostToolUse` hooks documented (same protocol as the verified
  CodeBuddy adapter); not installed here. **Windsurf**: official Cascade Hooks
  (pre/post) documented; not installed here. **Codex**: reclassified from A/B
  to **Class B** — its `approval_policy` is built-in and there is **no
  third-party pre-tool hook protocol**, so ActionGuard cannot attach as a
  pre-action layer. Results recorded in `BOUNDARIES.md`, `boundaries/*.yml`,
  and the built-in registry. No adapters were written — none of these tools
  yet earns one; the boundary abstraction is now validated against the real
  world instead of against marketing claims.
- **Product survival-line CI.** `.github/workflows/ci.yml` (ubuntu-latest +
  windows-latest): frontend + Rust build, `cargo test`, policy lint + dry-run,
  `doctor` (a clean machine must answer NOT PROTECTED), `boundary test` on the
  core boundary, and the lifecycle `setup --yes` → `doctor --test` (asserts the
  deny simulation) → `uninstall --yes`. Third-party boundary matrix is
  deliberately **not** in CI — live third-party probes stay manual / community
  verified + the registry, so a commit never requires launching Claude / Codex
  / Cursor / Windsurf.
- **Community verification standard.** `BOUNDARIES.md` now defines the
  **boundary test standard**: a PR-based flow where the community proves a
  boundary (Automation / Version / OS / Boundary / Test / Expected / Actual /
  Evidence), the maintainer reviews and marks it **Community Verified**, and
  `actionguard boundary list` shows `✓ Community Verified` + `contributor:
  @handle`. **Core Verified** stays maintainer-only with live `boundary test` +
  ledger evidence. `boundaries/*.yml` gains a `contributor` field (required for
  `verification: community`).
- **Community rule ecosystem skeleton.** New `actionguard rule` subcommand:
  `rule search <query>` (matches id / reason / match spec across builtin + user
  rules) and `rule install <file.yml>` (lints a community rule file, then merges
  it into `~/.actionguard/policies.user.yml`, replacing same-id rules and
  tagging them `source: user`). This is the 社区贡献 → 安装 → 使用 loop; the
  rule marketplace / verified packs / author split deliberately come later.
- **Project documentation overhaul for GitHub release.** Rewrote `README.md`
  with clearer structure (tagline, core features, quick start, CLI reference,
  action categories, community rules, known limitations, contributing, license).
  Added `README.zh.md` (Chinese full translation). Added `docs/CONTRIBUTING.md`
  (contribution guide covering verification reports, rule format, code style,
  commit conventions, security reporting). Added `docs/ARCHITECTURE.md` (system
  architecture, data model, four-layer design, adapter pattern, file structure).
  `.trae/` remains in `.gitignore` and is not shipped.

### Fixed

- **zsh hook never fired.** The generated zsh script only *defined*
  `__actionguard_preexec` but never registered it via `add-zsh-hook preexec`,
  so zsh users got zero enforcement. Now `autoload -Uz add-zsh-hook` +
  `add-zsh-hook preexec __actionguard_preexec` (deliberately not a bare
  `preexec` function, so user hooks are never clobbered).
- **Rule `args_contains` was AND-only → synonym rules silently never fired.**
  New `args_any` (any-of) matcher. Rules written with synonym lists
  (`confirm-kill-9`, `confirm-npm-uninstall`, `confirm-npm-install-global`,
  `allow-npm-install`, `confirm-pnpm-remove`, `allow-pnpm-add`,
  `allow-poetry-add`, `confirm-uv-remove`, `allow-uv-add`,
  `confirm-conda-remove`, `confirm-push-force`) were effectively dead for every
  spelling except the first. They now use `args_any`.
- **`deny-rm-rf-root` was over-broad.** The old regex hard-DENIED any `rm -rf`
  whose path contained a slash, so `rm -rf ./dist` and `rm -rf build/` were
  blocked instead of asking. It now only matches root / absolute / home / glob
  wipes; relative paths fall through to `confirm-rm-rf` (ask).
- **`deny-npx-rm-rf` was dead.** `npx` is classified as Package, but the rule
  declared `category: shell`, so it could never match. Now `category: package`.
- **`git push -f` bypassed confirmation.** `confirm-push-force` only matched
  `--force`; the short flag fell through to `allow-push`. Now uses `args_any`
  and matches `-f` too. Also, `--force-with-lease` (medium) is now matched
  *before* `--force` (high), and the shared-branch deny no longer false-positives
  on `--force-with-lease`.
- **`deny-pip-install-user` renamed to `confirm-pip-install-upgrade`.** The id
  claimed "user" but the rule actually matches `-U/--upgrade`, and an `ask`
  rule shouldn't carry a `deny-` prefix.
- **Ledger records are now self-contained.** Each NDJSON line carries its
  `session_id` (previously implied only by the file name), so every Action is
  addressable as `(session_id, action_id)`. No schema break: old ledgers
  deserialize with `session_id` unset, and the sync-path contract is
  documented in `SECURITY_TEST_MATRIX.md`. This is the only storage change
  needed for a future cloud-sync layer to be a *new upload channel* rather
  than a storage rewrite.
- **Boundary is a first-class model.** New `BoundaryKind`
  (`protected_shell` / `tool_hook` / `runtime_hook` / `observe_only` /
  `system_level`) and `EnforcementStatus` (`enforced` / `observed` /
  `bypassed` / `unsupported`) in `models.rs`. Every `Action` now records
  `boundary` and `enforcement` alongside `decision` — **Decision ≠ Outcome**:
  a policy `Deny` plus `Bypassed` enforcement records that ActionGuard said no
  but the executor got around the boundary. Old ledgers deserialize with the
  new fields unset (forward-compatible).
- **Boundary Registry** (`src-tauri/src/boundary.rs`) — the single place that
  knows real automation sources. New CLI commands `actionguard boundary list`
  (registry + local probes) and `actionguard boundary test` (non-destructive
  ✓/✗ verification per boundary). Core code never special-cases agent brands:
  agents are *Action Sources*; integrations are *Boundary Adapters*.
- **Public `BOUNDARIES.md`** — a GitHub-facing AI Automation Boundary Registry:
  one table plus six questions per automation (local execution? where does the
  action enter? pre-action boundary? observe? enforce? last verified?).
  **Manus Desktop** (local CLI with native per-command approval) is registered
  as the next verification target — observe candidate until a reproducible
  probe exists; Manus Cloud stays remote / N/A. README tagline upgraded to
  *"a local, user-controlled safety boundary for AI-powered automation"*, and
  v0.3 is re-scoped to **Boundary Expansion** (L1 protected shell → L2 tool
  hooks → L3 runtime hooks), not more rule types.
- **Action Boundary Map — by class, not by brand.** `BOUNDARIES.md` restructured
  from a per-brand registry into a **Boundary Classes (A–F)** map: A Tool Hook
  (CodeBuddy PreToolUse), B Exec Approval (OpenClaw `exec` policy, Manus
  Desktop per-command approval), C Protected Shell, D Runtime Sandbox,
  E System Enforcement, F Remote (Manus Cloud). New products are classified in
  minutes by one question — *"which boundary class does it expose?"* — never by
  brand. `BoundaryKind` grows `exec_approval` and `remote`, and every variant's
  label is the class name, so `actionguard boundary list` prints the map plus a
  class legend. `SECURITY_TEST_MATRIX.md` rows now carry their class too.
- **OpenClaw reclassified: Class B, not "no boundary".** Its `exec` policy
  (sandbox / gateway / node), deny / allowlist / full host policy and host
  approvals bound to a canonical `systemRunPlan` (later mutation → approval
  mismatch) make it a mature execution boundary. ActionGuard's goal is an
  independent, vendor-neutral policy layer *above* `exec` approval — one
  policy engine across CodeBuddy, OpenClaw and Manus. Registry entry stays
  `TBD` until a reproducible probe exists.
- **Manus Desktop = Class B, Manus Cloud = Class F.** Local desktop automation
  (own per-command approval; ActionGuard as independent second layer) is split
  from remote execution (address-space limit, N/A locally) — different
  classes, never conflated. **Cursor / Windsurf / OpenCode** added as Class A
  (unverified, probe required); Codex stays *"must be measured, not assumed"*.

### Added

- **Facts schema 定稿（Action → Facts → Policy → Verdict）。** `ARCHITECTURE.md`
  数据模型章节重写，与 `models.rs` 逐一对应：`Action` / `ActionContext` /
  `RiskLevel` / `Decision` / `DecisionResult` / `PolicySource`，外加策略优先级
  模型（User → Project → Builtin，Project 只能收紧）与决策缓存状态机。
- **对抗性 bypass 测试（`tests/bypass/`，23 个用例）。** 边界被当作攻击面
  测试：路径对抗（`..` 逃逸、大小写变体、8.3 短名、尾随分隔符）、进程来源
  对抗（source spoofing 不改判、`rm -rf /` 硬拒、sudo 优先）、配置对抗
  （用户规则优先级、first-match-wins、空匹配 lint 拒绝）。**已知盲区被显式
  钉住**而不是隐藏：`rm -r -f /`（flag 重排）、`rm -rf/`（无空白）落到 Ask
  而非 Deny；规则 `path` 匹配大小写敏感；尾随分隔符令 `detect_asset` 失效。
  一句总结写进了测试 README：*"ActionGuard continuously tests its
  enforcement boundary against known bypass techniques."*
- **`PolicySource::Project` 变体（预留）。** v0.3 项目策略的枚举骨架；加载
  点与"只能收紧"约束已在 `loader.rs` 文档化。当前 CLI 显示 `project` 来源。
- **`actionguard stats --export <path>`。** 将聚合报告（detected / blocked /
  enforcement 拆分 / risk breakdown / 每个会话摘要）导出为本地 JSON，用于
  无遥测的用户验证。
- **本地用户行为埋点（User Override Rate）——验证"边界是否被信任"。**
  `Action` ledger 行新增 `user_override` / `resolved_at`：bridge 在弹窗决策后
  把"用户是否否决了 ActionGuard 的判断"落盘（`Some(true)` = 用户放行了被
  拦截的动作，`Some(false)` = 用户认同）。`SessionSummary` 新增 `popups`
  （弹窗/打断次数）与 `overrides`（否决次数）；`actionguard stats` 输出
  **Override Rate**（overrides ÷ popups，高比例 = 策略过敏感或弹窗设计失败，
  不是用户不懂安全）。`stats --export` 额外导出每行 Action 明细，可本地
  计算等待时长等——全程无遥测，数据文件在用户机器上。
- **企业部署暗示（克制版）。** README 新增 "Using ActionGuard in your
  company?" 小节（探索 team deployment / enterprise use cases），GUI 主页
  新增一行 "Team deployment" 提示。目的是验证一个假设：有没有人把
  ActionGuard 从"GitHub 上的工具"理解成"可以部署到公司的安全基础设施"。
  这个信号比几十个 Star 值钱。
- **边界发现 → 规则固化的研发基础设施（流水线）。** 不再"每天手写 YAML"：
  - `docs/FACTS_SCHEMA.md`：事实层语言定稿。Fact 字段 ↔ 实现状态 ↔ 规则
    `match` 映射（`category`/`command`/`args_*`/`regex`/`path`），原则：
    规则面向 Facts 写，不面向原始字符串。
  - `docs/BOUNDARY_BACKLOG.md`：Boundary Backlog。规则分三层（Tier 0 绝对边界 /
    Tier 1 高风险默认防护 / Tier 2 环境特定策略）；68 条内置规则全量盘点；
    候选边界漏斗（B001–B015）+ 每周节奏（2–5 个候选）。发现新边界只做一件事：
    往 Backlog 加一行。
  - **Golden 回归套件（`tests/golden/`，31 用例）。** 每条规则的正/负/边界案例，
    走真实 pipeline（`classify_shell_command` → `decide`），断言
    decision + matched_rule + risk。**首跑即抓住 2 个真实规则问题**：
    `git branch -d` 被 `-D` 规则遮蔽（`args_contains` 大小写折叠——`-d` 误配
    `["-D"]`，已改为大小写敏感的 `regex`）；`sudo rm -rf /` 实际由 `deny-sudo`
    兜底（首 token 为 sudo，`deny-rm-rf-root` 的 `^rm` 不适用——结果仍为 Deny，
    不弱化）。`cat .env.production` 命中 `confirm-cat-env`（`-prod`/`-local` 为
    死规则）与 `cat ~/.aws/credentials` 无 confirm-cat-* 覆盖（读凭证缺口）如实
    锁定为 Backlog B013 / B014，修复时同步更新 Golden。
  - **贡献机制重定义：奖励 Boundary Discovery，不奖励写 YAML。** 三层贡献
    （Tier 1 报告免 YAML / Tier 2 复现案例 / Tier 3 Policy+Test）；新增
    `boundary_report` issue 模板（"Found a dangerous agent action that
    ActionGuard doesn't handle?"）；规则贡献强制要求 Golden Test——没有测试的
    规则 PR 不合并。

## [0.2.1] — 2026-08-19

**Enforcement Validation.** This release is scoped to one question: *"can we
prove that a real command on a supported execution path is actually blocked?"*
It does not add new features for the sake of breadth — it makes v0.2's
enforcement claims verifiable and honest.

### Changed

- **PowerShell Phase B → Phase C.** The PowerShell hook no longer merely
  logs denied commands. The PSReadLine `Enter` key handler now **reverts the
  input line on `deny`** (and swallows the keypress), so the command never
  reaches the execution pipeline. Scope: interactive PowerShell only.
- **Execution Path Matrix.** ActionGuard now surfaces *per-execution-path*
  capability (observe vs. pre-action block) instead of claiming blanket
  "Protected" support:
  - CLI: `actionguard status` and `actionguard protect` print the matrix.
  - GUI: `Live` view shows the matrix at the top of the session page.
  - Source of truth: `src-tauri/src/platform.rs`, documented in
    `SECURITY_TEST_MATRIX.md`.

### Added

- **`e2e-windows.ps1`** — end-to-end enforcement validation for Windows /
  PowerShell. Runs Allow / Ask / Deny / Bypass / hook-content cases and records
  `ActionGuard received` + `Command actually executed` for each. Exit code 0
  means every enforced case passed.
- **Version roadmap update.** v0.2.x is now explicitly "Enforcement
  Validation"; stronger platform interception (PATH shim, LD_PRELOAD/fanotify,
  Endpoint Security) moves to v0.3 "Enforcement Coverage", to be driven by the
  Execution Path Matrix gaps that real users actually hit.

### Fixed (found by the new test)

- **User-rule YAML shape.** A malformed top-level-array `policies.user.yml`
  was silently parsed as "no rules" (`unwrap_or_default`), so a deny rule could
  fail to load while the app reported it was protecting. The e2e script now
  lints and asserts the injected rule is actually in the loaded policy set.
- **UTF-8 BOM kills user rules.** Windows editors (notepad, PowerShell 5.1
  `Set-Content -Encoding UTF8`) write a BOM by default; serde_yaml rejected the
  stream with `missing field scope at line 1 column 2`, so a BOM-prefixed
  `policies.user.yml` failed to load. Found during the real-agent test
  (2026-08-19). Fixed via `policy::loader::strip_bom()` applied in `parse()`,
  `lint_file()`, and `storage::load_policies_user()`.

## [0.2.0] — 2026-08-18

The headline change: **ActionGuard is no longer a file monitor — it is an
Action Safety Layer that sits between AI-powered automation and the actions it can take
on your machine.** Four layers (Observe → Classify → Policy → Gate), five
action categories, a real pre-execution approval gate, a developer-first CLI,
and a new headline metric (Agent Actions Protected).

### Added

- **Action abstraction** — `FileChange` is now `Action` (with a backward-compat
  `pub type FileChange = Action;` alias). Every action carries an
  `ActionCategory` (File / Shell / Git / Package / Secret), a free-form `kind`
  verb, an optional `Asset`, and optional `Evidence`.
- **`RiskLevel::Critical`** — new tier above HIGH, used for irreversible
  actions: reading/writing `.env`/`*.pem`/`id_rsa*`/`.aws/credentials*`,
  `rm -rf /`, `git push --force origin main`, `chmod -R` on system dirs.
- **Sensitive asset detection** — extended from v0.1's `.env`/`*.pem`/`id_rsa`
  to also cover `.aws/credentials*`, `.gnupg/**`, `.git/config`, `.ssh/id_*`,
  `.npmrc`, `.pypirc`, `.netrc`, `credentials.*`. The `Asset` struct records
  `kind`, `matched_pattern`, and (for `.env` reads) redacted `contains` key
  names — never values.
- **Shell bridge** (`bridge.rs`) — TCP listener on `127.0.0.1:<random>`,
  per-session 32-char hex secret, HTTP endpoints:
  - `POST /preexec` — shell hook asks before running a command
  - `POST /resolve` — headless CLI allow/deny
  - `GET /pending` — list pending approvals for the CLI's interactive picker
  - `GET /status` — liveness probe
- **Shell hooks** (`shell_hooks.rs`) — generators for bash, zsh, fish, and
  PowerShell. Each script reads `~/.actionguard/sessions/current.hook`,
  POSTs the command to the bridge before execution, and aborts on `deny`.
  Documented limitation: preexec hooks are bypassable by calling the binary
  directly (`/usr/bin/rm` — rules match by program name); v0.3 will add a
  `PATH`-prepend shim directory.
- **Policy engine** (`policy/{mod,loader,matcher,classify}.rs`) — YAML rule
  system with first-match-wins semantics. Built-in rules live in
  `src-tauri/rules/{secrets,shell,git,node,python}.yml` (compiled in via
  `include_str!`). User rules in `~/.actionguard/policies.user.yml` always
  override built-ins.
- **Policy hot-reload** — the bridge re-stats `policies.user.yml` on every
  `/preexec` and swaps the in-memory `PolicySet` under the write lock if the
  mtime changed. No restart needed.
- **Approval gate** (`approval.rs` + `ApprovalModal.vue`) — when the policy
  returns `Ask` or the risk is HIGH/CRITICAL, the bridge blocks the shell
  command and emits `actionguard://approval/request`. The modal offers
  **Allow once · Deny · Always deny** — the last one learns a user rule,
  persists it to `policies.user.yml`, and hot-reloads the policy set.
- **Action Ledger UI** — `SessionView` redesigned with: total count + 4 risk
  pills (LOW/MEDIUM/HIGH/CRITICAL) + 5 clickable category chips
  (Files/Shell/Git/Packages/Secrets) + reusable `LedgerTable` (time / agent /
  action / target / risk / result / reasons). `LedgerView` deferred — the
  SessionView already embeds a refreshable `LedgerTable`.
- **`actionguard` binary** (`src/bin/cli.rs`) — 15 clap subcommands:
  `status`, `policy-check`, `policy-list`, `policy-lint`, `policy-path`,
  `policy-edit`, `session list`, `session show`, `actions show`, `allow`,
  `deny --always`, `init-bash`, `init-zsh`, `init-fish`, `init-powershell`,
  `run`, `stats`. The bridge runs in its own thread and stays alive even when
  the GUI is closed — `actionguard allow` works from a TTY.
- **Headless approvals** — `POST /resolve` mirrors the `resolve_approval`
  Tauri command. The CLI can resolve a pending GUI approval without the GUI
  being open.
- **Extended `SessionSummary`** — `category_counts`, `risk_counts`,
  `actions_protected`, `actions_blocked` (all `#[serde(default)]` so v0.1
  sessions load with zeros).
- **Extended `AppConfig`** — `shell_blocking` (default true),
  `approval_timeout_secs` (default 60), `default_agent` (default `shell-user`).
- **New metric: Agent Actions Protected** — `actionguard stats` prints the
  aggregate across all sessions. Home tab shows it live. PAR (Protected Action
  Rate) is now a sub-metric, still tracked in History.
- **NDJSON append-only ledger** (`<id>.ledger.json`) — one line per finalized
  Action. Loaded mid-session without re-serializing the whole action list.
  Filtered by category + risk.

### Changed

- **Positioning** — README rewritten. Tagline: "ActionGuard adds a local safety
  layer between AI-powered automation and the actions it can take on your machine."
  Category is now "Open-source AI Action Safety Layer".
- **SessionView redesign** — replaces the v0.1 4-cell count-grid with the
  Action Ledger layout described above.
- **`commands.rs`** — `ActiveSession` now holds `bridge: Option<Bridge>`,
  `approvals: Arc<ApprovalStore>` (via `AppState`), and the policy set is
  behind `Arc<RwLock<PolicySet>>` for hot-reload + the "Always deny" flow.
  `classify_action` is now a pure helper that stamps risk + decision without
  touching the ledger; `bump_counters` and `push_action` are separate so the
  bridge can wait for approval before recording.
- **`watcher.rs`** — subscribes to `EventKind::Access` *only* for paths
  matching `is_sensitive_path` (so Secret READs surface without noise).
- **`storage.rs`** — new helpers: `hook_file`, `current_hook_symlink`,
  `closed_sentinel`, `user_policy_path`, `ledger_path`, `append_ledger`,
  `load_ledger`, `load_policies_user`, `save_policies_user`,
  `write_hook_file`, `point_current_hook`, `teardown_current_hook`.
- **`Cargo.toml`** — `version = "0.2.0"`. New deps: `serde_yaml = "0.9"`,
  `uuid = { version = "1", features = ["v4"] }`, `regex = "1"`,
  `clap = { version = "4", features = ["derive"] }`. New `[[bin]]` for
  `actionguard`. `[lib] name = "actionguard_lib"`.

### Backward compatibility

- v0.1 session JSON files load read-only in v0.2 — new fields default to zero.
- v0.1 snapshots still restore via the existing `undo` flow.
- `FileChange` is now `pub type FileChange = Action;` — existing imports keep
  compiling.
- The 14 v0.1 Tauri commands keep their signatures; 7 new commands are
  additive.
- v0.1 sessions are never re-saved on read — migration is in-memory only.

### Tests

- `cargo test` now reports **37 passing** (was 9 in v0.1). New coverage:
  - `risk::critical_rm_rf_root`, `risk::critical_force_push_main`,
    `risk::critical_env_read`, `risk::critical_chmod_recursive`
  - `policy::deny_rule_overrides_builtin_allow`,
    `policy::always_deny_learn_rule`, `policy::matcher_glob_doublestar`,
    `policy::matcher_args_contains_all`
  - `approval::state_machine`, `approval::expired_entries_are_pruned`,
    `approval::clear_drops_all_waiters`
  - `policy::loader::builtin_rules_load_without_error`,
    `policy::loader::user_rules_not_present_yet_all_builtin`
  - `policy::classify::{classify_git, classify_npm, classify_pip3_uppercase,
    classify_rm_shell, classify_brew, kind_for_install_vs_uninstall,
    empty_command}`

### Known limitations

- Bash/zsh/PowerShell preexec hooks are bypassable by direct binary call
  (`/usr/bin/rm` — rules match by program name). v0.3 adds a `PATH`-prepend
  shim directory.
- `notify` `Access` events are noisy; we subscribe to them *only* for paths
  matching `is_sensitive_path`. Windows may not fire reliably for reads; the
  shell hook covers `cat .env` / `Get-Content .env` as a second path.
- One active session per `current.hook` symlink — multi-session
  disambiguation via `AG_SESSION_ID` env is v0.3.
- `tauri` is a hard dep of the CLI binary in v0.2 (~20MB). v0.3 should refactor
  engine modules to compile without `tauri`.
- macOS Keychain / Windows Credential Manager are out of scope for v0.2
  (file-based secrets only).
- "Always deny" user regex rules are budgeted (10ms per match via
  `regex::RegexBuilder::size_limit`); catastrophic backtracking rules are
  rejected + logged.

## [0.1.0] — 2026-08-17

Initial release. File Safety.

### Added

- Protected Workspace — pick a folder, ActionGuard takes a snapshot before
  anything runs.
- File Change Monitoring — every CREATE / MODIFY / DELETE / RENAME inside the
  workspace is captured live.
- Risk Detection — 100% deterministic rules flag LOW / MEDIUM / HIGH batches.
- Sensitive File Warnings — `.env`, `*.pem`, `*.key`, `credentials.*`, `id_rsa`,
  `id_ed25519`, … are flagged automatically.
- Change Review — when the agent goes HIGH risk, you get a real review screen:
  Review · Allow · Deny.
- Session History — every session is recorded with counts, risk level,
  duration, and the full action list.
- Undo — restore the workspace to the snapshot taken when the session started.
- Chinese / English language selection with automatic detection and
  localStorage memory.
- 9 unit tests: risk rules + snapshot roundtrip.

[0.2.0]: https://github.com/your-org/actionguard/releases/tag/v0.2.0
[0.1.0]: https://github.com/your-org/actionguard/releases/tag/v0.1.0
