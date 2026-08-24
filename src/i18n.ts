import { computed, reactive } from "vue";

export type Lang = "en" | "zh";

export const LANGS: { id: Lang; label: string; flag: string }[] = [
  { id: "en", label: "English", flag: "EN" },
  { id: "zh", label: "中文", flag: "中" },
];

export type DictKey =
  | "app.name"
  | "app.tagline"
  | "app.category"
  | "nav.home"
  | "nav.monitor"
  | "nav.review"
  | "nav.history"
  | "session.chip"
  | "home.title1"
  | "home.title2"
  | "home.subtitle"
  | "home.step1.title"
  | "home.step1.hint"
  | "home.choose"
  | "home.chooseLoading"
  | "home.noFolder"
  | "home.step2.title"
  | "home.step2.hint"
  | "home.startBtn"
  | "home.starting"
  | "home.resumeSession"
  | "home.mode.title"
  | "home.mode.hint"
  | "home.mode.observe"
  | "home.mode.observe.desc"
  | "home.mode.protected"
  | "home.mode.protected.desc"
  | "home.mode.badgeA"
  | "home.mode.badgeB"
  | "home.mode.tagObserve"
  | "home.mode.tagProtected"
  | "home.whatMonitored.k"
  | "home.whatMonitored.v"
  | "home.whatFlagged.k"
  | "home.whatFlagged.v"
  | "home.undo.k"
  | "home.undo.v"
  | "home.team.k"
  | "home.team.v"
  | "home.para.title"
  | "home.para.subtitle"
  | "home.para.desc"
  | "home.para.rateLabel"
  | "home.para.sessions"
  | "home.para.high"
  | "home.para.medium"
  | "home.para.rate"
  | "home.pill.deterministic"
  | "home.pill.neutral"
  | "home.pill.localOnly"
  | "home.protected.keyMetric"
  | "monitor.title"
  | "monitor.noActive"
  | "monitor.goStart"
  | "monitor.elapsed"
  | "monitor.riskBanner.title"
  | "monitor.riskBanner.prefix"
  | "monitor.riskBanner.suffix"
  | "monitor.riskBanner.review"
  | "monitor.riskBanner.allow"
  | "monitor.riskBanner.deny"
  | "monitor.end"
  | "monitor.ending"
  | "monitor.undo"
  | "monitor.undoing"
  | "monitor.history"
  | "monitor.disclaimer"
  | "review.title"
  | "review.empty"
  | "review.back"
  | "review.head.warn"
  | "review.head.title"
  | "review.head.subtitle1"
  | "review.head.subtitle2"
  | "review.counts.create"
  | "review.counts.modify"
  | "review.counts.delete"
  | "review.counts.rename"
  | "review.sensitive.title"
  | "review.sensitive.note"
  | "review.outside.title"
  | "review.outside.note"
  | "review.reasons.title"
  | "review.changes.title"
  | "review.allow"
  | "review.deny"
  | "review.working"
  | "review.restoring"
  | "review.disclaimer1"
  | "review.disclaimer2"
  | "history.title"
  | "history.empty"
  | "history.stat.sessions"
  | "history.stat.high"
  | "history.stat.medium"
  | "history.stat.rate"
  | "history.rateNote"
  | "history.today"
  | "history.yesterday"
  | "history.view"
  | "history.detail.actions"
  | "history.detail.sensitive"
  | "history.detail.outside"
  | "history.undo"
  | "history.undone"
  | "history.undoDone"
  | "history.tag.undone"
  | "history.tag.denied"
  | "list.group.delete"
  | "list.group.rename"
  | "list.group.modify"
  | "list.group.create"
  | "list.renameFrom"
  | "list.renameTo"
  | "list.tag.sensitive"
  | "list.tag.outside"
  | "list.more"
  | "list.none"
  | "risk.low"
  | "risk.medium"
  | "risk.high"
  | "risk.critical"
  | "category.file"
  | "category.shell"
  | "category.git"
  | "category.package"
  | "category.secret"
  | "decision.allow"
  | "decision.ask"
  | "decision.deny"
  | "ledger.title"
  | "ledger.subtitle"
  | "ledger.empty"
  | "ledger.col.time"
  | "ledger.col.source"
  | "ledger.col.action"
  | "ledger.col.target"
  | "ledger.col.risk"
  | "ledger.col.result"
  | "ledger.col.reasons"
  | "ledger.showAll"
  | "ledger.lastN"
  | "session.dashboard.title"
  | "session.dashboard.actions"
  | "session.dashboard.protected"
  | "session.dashboard.blocked"
  | "session.dashboard.riskBreakdown"
  | "session.dashboard.categoryBreakdown"
  | "home.protected.title"
  | "home.protected.subtitle"
  | "home.protected.desc"
  | "home.protected.protectedLabel"
  | "home.protected.blockedLabel"
  | "home.protected.criticalLabel"
  | "home.protected.highLabel"
  | "session.mode.observe"
  | "session.mode.protected"
  | "session.mode.badgeObserve"
  | "session.mode.badgeProtected"
  | "history.stat.critical"
  | "history.stat.blocked"
  | "history.stat.protected"
  | "enforcement.title"
  | "enforcement.subtitle"
  | "enforcement.col.path"
  | "enforcement.col.tier"
  | "enforcement.col.observe"
  | "enforcement.col.block"
  | "enforcement.col.note"
  | "enforcement.blocked"
  | "enforcement.observeOnly"
  | "enforcement.notCovered"
  | "session.dashboard.enforcement"
  | "session.dashboard.enforced"
  | "session.dashboard.observed"
  | "session.dashboard.bypassed"
  | "session.dashboard.unsupported"
  | "approval.title"
  | "approval.subtitle"
  | "approval.source"
  | "approval.wants"
  | "approval.risk"
  | "approval.reason"
  | "approval.matchedRule"
  | "approval.dueIn"
  | "approval.timeout"
  | "approval.expired"
  | "approval.allowOnce"
  | "approval.deny"
  | "approval.alwaysDeny"
  | "approval.alwaysDenyHint"
  | "approval.rulePreview"
  | "approval.resolving"
  | "lang.modal.title"
  | "lang.modal.subtitle"
  | "lang.modal.confirm"
  | "lang.switch"
  | "footer.slogan1"
  | "footer.slogan2"
  | "footer.slogan3"
  | "win.min"
  | "win.max"
  | "win.restore"
  | "win.close";

type Dict = Record<DictKey, string>;

const en: Dict = {
  "app.name": "ActionGuard",
  "app.tagline":
    "A local safety layer between AI automation and the actions it can take on your machine.",
  "app.category": "Open-source AI Automation Action Safety Layer",
  "nav.home": "Home",
  "nav.monitor": "Live",
  "nav.review": "Review",
  "nav.history": "History",
  "session.chip": "Session",
  "home.title1": "See what your AI automation is about to do",
  "home.title2": "before it actually executes.",
  "home.subtitle":
    "ActionGuard adds a local safety layer between AI automation and the real-world actions it can take on your machine. Observe, classify, policy, and human gate — keep automation within controllable boundaries.",
  "home.step1.title": "Protected Workspace",
  "home.step1.hint": "Choose the project directory that AI automation is allowed to access and operate on.",
  "home.choose": "Choose Folder",
  "home.chooseLoading": "Opening…",
  "home.noFolder": "No folder selected",
  "home.step2.title": "Start Protection",
  "home.step2.hint":
    "ActionGuard opens a protected terminal on your machine, creates a local snapshot, and monitors every action produced by the automation.",
  "home.startBtn": "Start Protection Session",
  "home.starting": "Starting…",
  "home.resumeSession": "Resume session",
  "home.mode.title": "Protection Mode",
  "home.mode.hint": "Choose how ActionGuard handles automated actions. You can switch at any time.",
  "home.mode.observe": "Observe",
  "home.mode.observe.desc":
    "Record every automated action to your local ledger — never block. Use when you want to understand what automation actually does without slowing it down.",
  "home.mode.protected": "Protected",
  "home.mode.protected.desc":
    "Block high-risk actions BEFORE they execute on supported execution paths, and require explicit human approval.",
  "home.mode.badgeA": "Mode A",
  "home.mode.badgeB": "Mode B",
  "home.mode.tagObserve": "Record only",
  "home.mode.tagProtected": "Block before execute",
  "home.whatMonitored.k": "What gets monitored",
  "home.whatMonitored.v": "CREATE · MODIFY · DELETE · RENAME inside the protected workspace, captured from automation activity.",
  "home.whatFlagged.k": "What gets flagged",
  "home.whatFlagged.v":
    "20+ files changed · 3+ deletions · sensitive files (.env, *.pem, *.key, credentials.*, id_rsa…) · paths outside the protected workspace.",
  "home.undo.k": "Undo",
  "home.undo.v":
    "Restores files to the local snapshot taken when the session started. Undo only covers file changes inside the protected workspace.",
  "home.team.k": "Team deployment",
  "home.team.v":
    "Need ActionGuard for a team? We're exploring enterprise deployment options — see the README.",
  "home.para.title": "Protected Action Rate",
  "home.para.subtitle": "The metric that matters more than stars.",
  "home.para.desc":
    "What % of sessions triggered a MEDIUM or HIGH risk flag? If most do, ActionGuard is actually catching dangerous actions.",
  "home.para.rateLabel": "Protected Action Rate",
  "home.para.sessions": "Sessions",
  "home.para.high": "HIGH risk",
  "home.para.medium": "MEDIUM risk",
  "home.para.rate": "Trigger rate",
  "home.pill.deterministic": "100% Deterministic",
  "home.pill.neutral": "Automation Neutral",
  "home.pill.localOnly": "Local Only",
  "home.protected.keyMetric": "KEY METRIC",
  "monitor.title": "No active session.",
  "monitor.noActive": "No active session.",
  "monitor.goStart": "Start a session",
  "monitor.elapsed": "Elapsed",
  "monitor.riskBanner.title": "⚠ HIGH RISK ACTION",
  "monitor.riskBanner.prefix": "Automation wants to change",
  "monitor.riskBanner.suffix": "files in the last batch.",
  "monitor.riskBanner.review": "Review Changes",
  "monitor.riskBanner.allow": "Allow",
  "monitor.riskBanner.deny": "Deny",
  "monitor.end": "End Session",
  "monitor.ending": "Ending…",
  "monitor.undo": "Undo Session",
  "monitor.undoing": "Undoing…",
  "monitor.history": "History",
  "monitor.disclaimer": "Undo only covers file changes inside the protected workspace.",
  "review.title": "HIGH RISK ACTION",
  "review.empty": "No pending review. All batches were accepted.",
  "review.back": "Back to monitor",
  "review.head.warn": "HIGH RISK ACTION",
  "review.head.title": "HIGH RISK ACTION",
  "review.head.subtitle1": "Automation wants to modify",
  "review.head.subtitle2": "files.",
  "review.counts.create": "CREATE",
  "review.counts.modify": "MODIFY",
  "review.counts.delete": "DELETE",
  "review.counts.rename": "RENAME",
  "review.sensitive.title": "⚠ Sensitive Files Detected",
  "review.sensitive.note":
    "These files may contain API keys, passwords, private keys, or credentials.",
  "review.outside.title": "⚠ Files Outside The Protected Workspace",
  "review.outside.note": "Changes outside the workspace are not covered by Undo.",
  "review.reasons.title": "Reason",
  "review.changes.title": "Review Changes",
  "review.allow": "Allow",
  "review.deny": "Deny & Restore",
  "review.working": "Working…",
  "review.restoring": "Restoring…",
  "review.disclaimer1":
    "Deny restores the workspace from the snapshot taken when the session started and ends the session.",
  "review.disclaimer2": "Undo only covers file changes inside the protected workspace.",
  "history.title": "Session History",
  "history.empty": "No sessions yet. Start a protected session from Home.",
  "history.stat.sessions": "Sessions",
  "history.stat.high": "HIGH risk",
  "history.stat.medium": "MEDIUM risk",
  "history.stat.rate": "Protected Action Rate",
  "history.rateNote":
    "Protected Action Rate = share of sessions with a MEDIUM or HIGH risk action. If most sessions trigger, ActionGuard is catching dangerous actions.",
  "history.today": "Today",
  "history.yesterday": "Yesterday",
  "history.view": "View →",
  "history.detail.actions": "file change(s)",
  "history.detail.sensitive": "sensitive",
  "history.detail.outside": "outside workspace",
  "history.undo": "Undo Session",
  "history.undone": "Already undone",
  "history.undoDone": "Undo complete.",
  "history.tag.undone": "undone",
  "history.tag.denied": "denied",
  "list.group.delete": "DELETED",
  "list.group.rename": "RENAMED",
  "list.group.modify": "MODIFIED",
  "list.group.create": "CREATED",
  "list.renameFrom": "",
  "list.renameTo": "→",
  "list.tag.sensitive": "sensitive",
  "list.tag.outside": "outside",
  "list.more": "… and more",
  "list.none": "No changes recorded.",
  "risk.low": "LOW",
  "risk.medium": "MEDIUM",
  "risk.high": "HIGH",
  "risk.critical": "CRITICAL",
  "category.file": "File",
  "category.shell": "Shell",
  "category.git": "Git",
  "category.package": "Package",
  "category.secret": "Secret",
  "decision.allow": "allow",
  "decision.ask": "confirm",
  "decision.deny": "deny",
  "ledger.title": "Action Ledger",
  "ledger.subtitle": "Every action the automation attempted, in order.",
  "ledger.empty": "No actions recorded yet.",
  "ledger.col.time": "Time",
  "ledger.col.source": "Source",
  "ledger.col.action": "Action",
  "ledger.col.target": "Target",
  "ledger.col.risk": "Risk",
  "ledger.col.result": "Result",
  "ledger.col.reasons": "Reason",
  "ledger.showAll": "Show all",
  "ledger.lastN": "Last {n}",
  "session.dashboard.title": "Session #{num}",
  "session.dashboard.actions": "Actions",
  "session.dashboard.protected": "Protected",
  "session.dashboard.blocked": "Blocked",
  "session.dashboard.riskBreakdown": "Risk breakdown",
  "session.dashboard.categoryBreakdown": "Category breakdown",
  "home.protected.title": "Actions Detected",
  "home.protected.subtitle": "Detection ≠ Protection — be honest about what we stop.",
  "home.protected.desc":
    "Every automated action that passed through ActionGuard's safety layer. Detection means it was recorded; Protection means it was actually stopped before execution. HIGH and CRITICAL actions required explicit human approval; Blocked actions were stopped at the boundary.",
  "home.protected.protectedLabel": "Actions detected",
  "home.protected.blockedLabel": "Actions blocked",
  "home.protected.criticalLabel": "CRITICAL",
  "home.protected.highLabel": "HIGH",
  "session.mode.observe": "Mode A · Observe",
  "session.mode.protected": "Mode B · Protected",
  "session.mode.badgeObserve": "A",
  "session.mode.badgeProtected": "B",
  "history.stat.critical": "CRITICAL risk",
  "history.stat.blocked": "Blocked",
  "history.stat.protected": "Actions detected",
  "enforcement.title": "Execution Path Matrix",
  "enforcement.subtitle":
    "Not every way a command can run is enforced on this platform. A row with Block = NO means a command on that path can still execute even in Protected mode.",
  "enforcement.col.path": "Path",
  "enforcement.col.tier": "Tier",
  "enforcement.col.observe": "Observe",
  "enforcement.col.block": "Block",
  "enforcement.col.note": "Note",
  "enforcement.blocked": "YES",
  "enforcement.observeOnly": "NO",
  "enforcement.notCovered": "—",
  "session.dashboard.enforcement": "Enforcement (Detection ≠ Protection)",
  "session.dashboard.enforced": "Enforced",
  "session.dashboard.observed": "Observed",
  "session.dashboard.bypassed": "Bypassed",
  "session.dashboard.unsupported": "Unsupported",
  "approval.title": "Approval required",
  "approval.subtitle":
    "An automated action crossed your safety boundary. Decide before it runs.",
  "approval.source": "Source",
  "approval.wants": "wants to",
  "approval.risk": "Risk",
  "approval.reason": "Reason",
  "approval.matchedRule": "Matched rule",
  "approval.dueIn": "Auto-deny in",
  "approval.timeout": "Timed out — denied",
  "approval.expired": "Approval expired or already resolved",
  "approval.allowOnce": "Allow once",
  "approval.deny": "Deny",
  "approval.alwaysDeny": "Always deny",
  "approval.alwaysDenyHint":
    "Adds a user rule to ~/.actionguard/policies.user.yml so this signature is blocked next time.",
  "approval.rulePreview": "Rule preview",
  "approval.resolving": "Resolving…",
  "lang.modal.title": "Choose your language",
  "lang.modal.subtitle": "Pick a language to get started. You can change it later from the top bar.",
  "lang.modal.confirm": "Continue",
  "lang.switch": "Language",
  "footer.slogan1": "ActionGuard — AI-vendor-independent Automation Safety Layer",
  "footer.slogan2": "Open-source · No account · No cloud · 100% deterministic rules",
  "footer.slogan3": "v0.2",
  "win.min": "Minimize",
  "win.max": "Maximize",
  "win.restore": "Restore",
  "win.close": "Close",
};

const zh: Dict = {
  "app.name": "ActionGuard",
  "app.tagline": "在 AI 自动化与其可执行操作之间，增加一层本地安全边界。",
  "app.category": "开源 · AI 自动化操作安全层",
  "nav.home": "首页",
  "nav.monitor": "实时",
  "nav.review": "变更审核",
  "nav.history": "历史记录",
  "session.chip": "会话",
  "home.title1": "在 AI 自动化真正执行之前",
  "home.title2": "先看清它准备做什么。",
  "home.subtitle":
    "ActionGuard 在 AI 自动化与现实世界操作之间增加一层本地安全边界。观察、分类、策略、人工放行，让自动化在可控范围内执行。",
  "home.step1.title": "选择受保护的工作空间",
  "home.step1.hint": "选择自动化任务允许访问和操作的项目目录。",
  "home.choose": "选择文件夹",
  "home.chooseLoading": "正在打开…",
  "home.noFolder": "尚未选择文件夹",
  "home.step2.title": "启动防护",
  "home.step2.hint":
    "ActionGuard 将在本地启动受保护执行环境，创建本地快照，并实时记录自动化产生的操作。",
  "home.startBtn": "启动防护会话",
  "home.starting": "启动中…",
  "home.resumeSession": "回到会话",
  "home.mode.title": "选择防护模式",
  "home.mode.hint": "选择 ActionGuard 处理自动化动作的方式，启动后也可切换模式。",
  "home.mode.observe": "观察模式",
  "home.mode.observe.desc":
    "记录自动化产生的所有操作到本地账本，绝不阻断。适合先了解自动化实际在做什么。",
  "home.mode.protected": "防护模式",
  "home.mode.protected.desc":
    "对支持的执行路径，在高风险操作执行前阻断，并要求人工放行。",
  "home.mode.badgeA": "模式 A",
  "home.mode.badgeB": "模式 B",
  "home.mode.tagObserve": "仅记录",
  "home.mode.tagProtected": "执行前阻断",
  "home.whatMonitored.k": "监控范围",
  "home.whatMonitored.v": "受保护工作空间内的 CREATE · MODIFY · DELETE · RENAME 四种自动化操作。",
  "home.whatFlagged.k": "触发警告",
  "home.whatFlagged.v":
    "一次修改超过 20 个文件 · 超过 3 个删除 · 敏感文件（.env、*.pem、*.key、credentials.*、id_rsa …）· 受保护工作空间外的路径。",
  "home.undo.k": "撤销",
  "home.undo.v": "将会话开始时的本地快照回滚到工作空间。撤销仅覆盖受保护工作空间内的文件变更。",
  "home.team.k": "团队部署",
  "home.team.v": "团队需要 ActionGuard？我们正在探索企业部署方案——详见 README。",
  "home.para.title": "拦截率 Protected Action Rate",
  "home.para.subtitle": "比 Star 更有价值的指标。",
  "home.para.desc":
    "有多少比例的会话触发了 MEDIUM 或 HIGH 风险？如果大多数会话都会触发，说明 ActionGuard 真的拦住了危险操作。",
  "home.para.rateLabel": "拦截率",
  "home.para.sessions": "总会话",
  "home.para.high": "高风险",
  "home.para.medium": "中风险",
  "home.para.rate": "触发占比",
  "home.pill.deterministic": "100% 确定性",
  "home.pill.neutral": "自动化无关",
  "home.pill.localOnly": "仅本地",
  "home.protected.keyMetric": "关键指标",
  "monitor.title": "暂无活跃会话。",
  "monitor.noActive": "暂无活跃会话。",
  "monitor.goStart": "去启动一个会话",
  "monitor.elapsed": "时长",
  "monitor.riskBanner.title": "⚠ 高风险操作",
  "monitor.riskBanner.prefix": "自动化在上一批次中尝试变更",
  "monitor.riskBanner.suffix": "个文件。",
  "monitor.riskBanner.review": "查看变更",
  "monitor.riskBanner.allow": "允许",
  "monitor.riskBanner.deny": "拒绝",
  "monitor.end": "结束会话",
  "monitor.ending": "结束中…",
  "monitor.undo": "撤销会话",
  "monitor.undoing": "撤销中…",
  "monitor.history": "历史",
  "monitor.disclaimer": "撤销仅覆盖受保护工作空间内的文件变更。",
  "review.title": "高风险操作",
  "review.empty": "暂无待审核变更，所有批次已放行。",
  "review.back": "返回监控",
  "review.head.warn": "⚠ 高风险操作",
  "review.head.title": "⚠ 高风险操作",
  "review.head.subtitle1": "自动化将要修改",
  "review.head.subtitle2": "个文件。",
  "review.counts.create": "新增",
  "review.counts.modify": "修改",
  "review.counts.delete": "删除",
  "review.counts.rename": "重命名",
  "review.sensitive.title": "⚠ 检测到敏感文件",
  "review.sensitive.note": "这些文件可能包含 API 密钥、密码、私钥或凭证。",
  "review.outside.title": "⚠ 位于受保护工作空间外",
  "review.outside.note": "工作空间外的变更不在撤销覆盖范围内。",
  "review.reasons.title": "原因",
  "review.changes.title": "变更明细",
  "review.allow": "允许",
  "review.deny": "拒绝并还原",
  "review.working": "处理中…",
  "review.restoring": "还原中…",
  "review.disclaimer1": "选择拒绝后，ActionGuard 会按会话开始时的快照还原工作空间，并结束会话。",
  "review.disclaimer2": "撤销仅覆盖受保护工作空间内的文件变更。",
  "history.title": "会话历史",
  "history.empty": "还没有会话。先从首页启动一个受保护会话吧。",
  "history.stat.sessions": "总会话",
  "history.stat.high": "高风险",
  "history.stat.medium": "中风险",
  "history.stat.rate": "拦截率",
  "history.rateNote":
    "拦截率 = 触发 MEDIUM 或 HIGH 风险的会话占比。如果大多数会话都触发，说明 ActionGuard 真正拦住了危险操作。",
  "history.today": "今天",
  "history.yesterday": "昨天",
  "history.view": "查看 →",
  "history.detail.actions": "个文件变更",
  "history.detail.sensitive": "个敏感",
  "history.detail.outside": "个越界",
  "history.undo": "撤销本次会话",
  "history.undone": "已撤销",
  "history.undoDone": "撤销完成。",
  "history.tag.undone": "已撤销",
  "history.tag.denied": "已拒绝",
  "list.group.delete": "已删除",
  "list.group.rename": "已重命名",
  "list.group.modify": "已修改",
  "list.group.create": "已新增",
  "list.renameFrom": "",
  "list.renameTo": "→",
  "list.tag.sensitive": "敏感",
  "list.tag.outside": "越界",
  "list.more": "…还有更多",
  "list.none": "暂无变更记录。",
  "risk.low": "低",
  "risk.medium": "中",
  "risk.high": "高",
  "risk.critical": "致命",
  "category.file": "文件",
  "category.shell": "命令",
  "category.git": "Git",
  "category.package": "包",
  "category.secret": "密钥",
  "decision.allow": "放行",
  "decision.ask": "确认",
  "decision.deny": "拒绝",
  "ledger.title": "动作流水",
  "ledger.subtitle": "自动化尝试过的每一个动作，按时间顺序排列。",
  "ledger.empty": "暂无动作记录。",
  "ledger.col.time": "时间",
  "ledger.col.source": "来源",
  "ledger.col.action": "动作",
  "ledger.col.target": "目标",
  "ledger.col.risk": "风险",
  "ledger.col.result": "结果",
  "ledger.col.reasons": "原因",
  "ledger.showAll": "查看全部",
  "ledger.lastN": "最近 {n} 条",
  "session.dashboard.title": "会话 #{num}",
  "session.dashboard.actions": "动作总数",
  "session.dashboard.protected": "已保护",
  "session.dashboard.blocked": "已阻止",
  "session.dashboard.riskBreakdown": "风险分布",
  "session.dashboard.categoryBreakdown": "类别分布",
  "home.protected.title": "检测到的动作",
  "home.protected.subtitle": "检测 ≠ 保护 —— 如实展示我们真正阻止了什么。",
  "home.protected.desc":
    "所有经过 ActionGuard 安全层的自动化操作。检测到意味着已记录；保护意味着在执行前真正被拦截。HIGH 和 CRITICAL 必须经过人工放行；已阻止是在边界处被拦截的动作。",
  "home.protected.protectedLabel": "检测到的动作",
  "home.protected.blockedLabel": "已阻止动作",
  "home.protected.criticalLabel": "致命",
  "home.protected.highLabel": "高危",
  "session.mode.observe": "观察模式",
  "session.mode.protected": "防护模式",
  "session.mode.badgeObserve": "A",
  "session.mode.badgeProtected": "B",
  "history.stat.critical": "致命风险",
  "history.stat.blocked": "已阻止",
  "history.stat.protected": "动作保护数",
  "enforcement.title": "执行路径矩阵",
  "enforcement.subtitle":
    "并非所有命令执行方式都会在本平台被拦截。Block = NO 的行意味着：即使处于防护模式，该路径上的命令仍可能执行。",
  "enforcement.col.path": "执行路径",
  "enforcement.col.tier": "能力级",
  "enforcement.col.observe": "可观察",
  "enforcement.col.block": "可拦截",
  "enforcement.col.note": "说明",
  "enforcement.blocked": "是",
  "enforcement.observeOnly": "否",
  "enforcement.notCovered": "—",
  "session.dashboard.enforcement": "强制执行 (检测 ≠ 保护)",
  "session.dashboard.enforced": "已拦截",
  "session.dashboard.observed": "已观察",
  "session.dashboard.bypassed": "已绕过",
  "session.dashboard.unsupported": "不支持",
  "approval.title": "需要确认",
  "approval.subtitle": "自动化的操作触发了安全边界，请决定是否允许执行。",
  "approval.source": "来源",
  "approval.wants": "想要",
  "approval.risk": "风险",
  "approval.reason": "原因",
  "approval.matchedRule": "匹配规则",
  "approval.dueIn": "自动拒绝倒计时",
  "approval.timeout": "已超时 — 拒绝",
  "approval.expired": "审批已过期或已处理",
  "approval.allowOnce": "允许一次",
  "approval.deny": "拒绝",
  "approval.alwaysDeny": "总是拒绝",
  "approval.alwaysDenyHint":
    "将一条用户规则写入 ~/.actionguard/policies.user.yml,下次同类操作将被阻止。",
  "approval.rulePreview": "规则预览",
  "approval.resolving": "处理中…",
  "lang.modal.title": "请选择语言",
  "lang.modal.subtitle": "为 ActionGuard 选择语言，稍后也可在顶部随时切换。",
  "lang.modal.confirm": "继续",
  "lang.switch": "语言",
  "footer.slogan1": "ActionGuard —— 独立于 AI 厂商的自动化操作安全层",
  "footer.slogan2": "开源 · 无需账号 · 本地运行 · 100% 确定性规则",
  "footer.slogan3": "v0.2",
  "win.min": "最小化",
  "win.max": "最大化",
  "win.restore": "还原",
  "win.close": "关闭",
};

const DICTS: Record<Lang, Dict> = { en, zh };

const STORAGE_KEY = "actionguard.lang";

function detectInitial(): Lang {
  if (typeof localStorage !== "undefined") {
    const saved = localStorage.getItem(STORAGE_KEY) as Lang | null;
    if (saved === "en" || saved === "zh") return saved;
  }
  try {
    const nav = (typeof navigator !== "undefined" && navigator.language) || "en";
    return nav.toLowerCase().startsWith("zh") ? "zh" : "en";
  } catch {
    return "en";
  }
}

const state = reactive<{
  lang: Lang;
  prompted: boolean;
}>({
  lang: (typeof localStorage !== "undefined" && (localStorage.getItem(STORAGE_KEY) as Lang)) === "en" || (localStorage.getItem(STORAGE_KEY) as Lang) === "zh"
    ? (localStorage.getItem(STORAGE_KEY) as Lang)
    : detectInitial(),
  prompted: (typeof localStorage !== "undefined" && !!localStorage.getItem(STORAGE_KEY)),
});

export function t(key: DictKey): string {
  return DICTS[state.lang][key] ?? key;
}

/** Templated translate: replaces `{name}` placeholders with values. */
export function tf(
  key: DictKey,
  vars: Record<string, string | number> = {},
): string {
  let s = DICTS[state.lang][key] ?? key;
  for (const [k, v] of Object.entries(vars)) {
    s = s.replace(new RegExp(`\\{${k}\\}`, "g"), String(v));
  }
  return s;
}

export function setLang(l: Lang) {
  state.lang = l;
  if (typeof localStorage !== "undefined") localStorage.setItem(STORAGE_KEY, l);
  state.prompted = true;
  try {
    if (typeof document !== "undefined") document.documentElement.setAttribute("lang", l);
  } catch {}
}

export function markPrompted() {
  state.prompted = true;
  if (typeof localStorage !== "undefined" && !localStorage.getItem(STORAGE_KEY)) {
    localStorage.setItem(STORAGE_KEY, state.lang);
  }
}

export function useI18n() {
  return {
    t,
    tf,
    setLang,
    markPrompted,
    lang: computed(() => state.lang as Lang),
    prompted: computed(() => state.prompted),
  };
}
