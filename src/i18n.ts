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
  | "home.consumer.badge"
  | "home.consumer.tagline"
  | "home.consumer.title"
  | "home.consumer.subtitle"
  | "home.consumer.cta"
  | "home.consumer.explore"
  | "home.consumer.supported"
  | "home.consumer.trust"
  | "home.consumer.onboarding.title"
  | "home.consumer.onboarding.scope.title"
  | "home.consumer.onboarding.scope.computer"
  | "home.consumer.onboarding.scope.computerDesc"
  | "home.consumer.onboarding.level.title"
  | "home.consumer.onboarding.level.recommended"
  | "home.consumer.onboarding.level.recommendedDesc"
  | "home.consumer.onboarding.protect.title"
  | "home.consumer.onboarding.protect.file"
  | "home.consumer.onboarding.protect.shell"
  | "home.consumer.onboarding.protect.git"
  | "home.consumer.onboarding.protect.package"
  | "home.consumer.onboarding.protect.secret"
  | "home.consumer.onboarding.protect.routine"
  | "home.consumer.onboarding.protect.consequential"
  | "home.consumer.onboarding.protect.critical"
  | "home.consumer.onboarding.privacy.title"
  | "home.consumer.onboarding.privacy.desc"
  | "home.consumer.onboarding.changeLater"
  | "home.consumer.onboarding.start"
  | "home.consumer.onboarding.back"
  | "home.consumer.starting"
  | "home.consumer.active.title"
  | "home.consumer.active.subtitle"
  | "home.consumer.active.allowed"
  | "home.consumer.active.reviewed"
  | "home.consumer.active.blocked"
  | "home.consumer.active.viewActivity"
  | "home.consumer.active.pause"
  | "home.consumer.active.pausing"
  | "home.consumer.active.pauseHint"
  | "home.consumer.active.lastBlocked"
  | "home.consumer.active.lastBlockedEmpty"
  | "home.consumer.active.supported"
  | "home.consumer.activity.title"
  | "home.consumer.activity.viewAll"
  | "home.consumer.activity.empty"
  | "home.consumer.activity.why"
  | "home.consumer.activity.rule"
  | "home.consumer.activity.decision"
  | "home.consumer.advanced"
  | "home.consumer.advancedHint"
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
  | "coverage.title"
  | "coverage.viewDetails"
  | "coverage.scanning"
  | "coverage.fullyProtected"
  | "coverage.partial"
  | "coverage.observeOnly"
  | "coverage.highQuality"
  | "coverage.generic"
  | "coverage.observe"
  | "coverage.inactive"
  | "coverage.genericFallback"
  | "approval.title"
  | "approval.subtitle"
  | "approval.source"
  | "approval.wants"
  | "approval.risk"
  | "approval.reason"
  | "approval.matchedRule"
  | "approval.technical"
  | "approval.technical.category"
  | "approval.technical.actionId"
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
  | "win.close"
  | "nav.dashboard"
  | "nav.activity"
  | "brand.tag"
  | "sidebar.protectionActive"
  | "sidebar.protectionInactive"
  | "onboarding.tagline"
  | "onboarding.cta"
  | "onboarding.starting"
  | "onboarding.defaults"
  | "onboarding.advanced"
  | "onboarding.workspace"
  | "onboarding.workspaceHint"
  | "onboarding.privacy"
  | "dashboard.hero.active"
  | "dashboard.hero.inactive"
  | "dashboard.hero.activeSub"
  | "dashboard.hero.inactiveSub"
  | "dashboard.meta.enforced"
  | "dashboard.meta.stopped"
  | "dashboard.meta.mode"
  | "dashboard.meta.scope"
  | "dashboard.mode.recommended"
  | "dashboard.scope.thisComputer"
  | "dashboard.action.viewActivity"
  | "dashboard.action.pause"
  | "dashboard.action.pausing"
  | "dashboard.action.protect"
  | "dashboard.action.starting"
  | "dashboard.stat.total"
  | "dashboard.stat.allowed"
  | "dashboard.stat.asked"
  | "dashboard.stat.blocked"
  | "dashboard.activity.title"
  | "dashboard.activity.viewAll"
  | "dashboard.activity.empty"
  | "dashboard.review.title"
  | "dashboard.review.requestedBy"
  | "dashboard.review.viewAll"
  | "dashboard.boundaries.title"
  | "dashboard.boundaries.empty"
  | "dashboard.advanced.title"
  | "nav.live"
  | "nav.policies"
  | "nav.boundaries"
  | "nav.settings"
  | "policies.title"
  | "policies.desc"
  | "boundaries.title"
  | "boundaries.desc"
  | "settings.title"
  | "settings.desc"
  | "settings.language"
  | "settings.languageHint"
  | "home.active.confirmStop"
  | "home.advanced.title"
  | "home.advanced.workspace.title"
  | "home.advanced.workspace.hint"
  | "home.advanced.mode.title"
  | "home.advanced.mode.interactive"
  | "home.advanced.mode.observe"
  | "home.advanced.mode.observeDesc"
  | "home.advanced.rules.title"
  | "home.advanced.rules.empty"
  | "home.advanced.enforcement.title"
  | "home.advanced.enforcement.empty"
  | "home.advanced.diagnostics.title"
  | "home.advanced.diagnostics.run"
  | "home.advanced.diagnostics.desc"
  | "home.metrics.title"
  | "home.metrics.sessions"
  | "home.metrics.flagged"
  | "home.metrics.rate"
  | "home.metrics.blockedActions"
  | "onboarding.startError"
  | "home.consumer.onboarding.changeDir"
  | "onboarding.headline"
  | "onboarding.subline"
  | "onboarding.defaults.scope"
  | "onboarding.rec"
  | "onboarding.scope.title"
  | "onboarding.scope.computer"
  | "onboarding.scope.computerDesc"
  | "onboarding.scope.customHint"
  | "onboarding.changeDir"
  | "onboarding.level.title"
  | "onboarding.level.recommended"
  | "onboarding.level.recommendedDesc"
  | "onboarding.level.observe"
  | "onboarding.level.observeDesc"
  | "onboarding.promise.routine"
  | "onboarding.promise.ask"
  | "onboarding.promise.block"
  | "onboarding.protect.title"
  | "onboarding.protect.file"
  | "onboarding.protect.shell"
  | "onboarding.protect.git"
  | "onboarding.protect.package"
  | "onboarding.protect.secret"
  | "onboarding.privacy.title"
  | "onboarding.privacy.desc"
  | "onboarding.privacy.p1"
  | "onboarding.privacy.p2"
  | "onboarding.privacy.p3"
  | "home.status.active"
  | "home.status.inactive"
  | "home.hero.title.active"
  | "home.hero.title.inactive"
  | "home.hero.desc.active"
  | "home.hero.desc.inactive"
  | "home.hero.starting"
  | "home.hero.protect"
  | "home.hero.viewActivity"
  | "home.hero.stopping"
  | "home.hero.pause"
  | "home.today.label"
  | "home.today.allowed"
  | "home.today.asked"
  | "home.today.blocked"
  | "home.today.viewReview"
  | "home.lastBlocked.k"
  | "home.lastBlocked.view"
  | "home.activity.title"
  | "home.activity.viewAll"
  | "home.activity.empty.k"
  | "home.activity.empty.v"
  | "home.protect.title"
  | "home.protect.hint"
  | "home.advanced.k"
  | "home.advanced.hint"
  | "home.advanced.rules.manage"
  | "home.advanced.rules.hint"
  | "home.advanced.enforcement.manage"
  | "home.advanced.enforcement.hint"
  | "home.advanced.diagnostics.okEngine"
  | "home.advanced.diagnostics.okPolicy"
  | "home.advanced.diagnostics.okSession"
  | "home.advanced.diagnostics.warnNoSession"
  | "home.trust.local"
  | "home.trust.noAccount"
  | "home.trust.noTelemetry"
  | "dashboard.hero.titleWhite"
  | "dashboard.hero.titleGreen"
  | "dashboard.hero.inactiveWhite"
  | "dashboard.hero.inactiveGreen"
  | "dashboard.hero.metaBoundaries"
  | "dashboard.quickActions.title"
  | "dashboard.quickActions.viewActivity"
  | "dashboard.quickActions.pendingReviews"
  | "dashboard.quickActions.openSettings"
  | "dashboard.stat.vsYesterday"
  | "dashboard.stat.ofTotal"
  | "dashboard.activity.showingLast"
  | "dashboard.donut.title"
  | "dashboard.donut.total"
  | "dashboard.donut.file"
  | "dashboard.donut.shell"
  | "dashboard.donut.git"
  | "dashboard.donut.secret"
  | "dashboard.donut.package"
  | "dashboard.health.title"
  | "dashboard.health.engine"
  | "dashboard.health.policy"
  | "dashboard.boundaries.manage"
  | "dashboard.boundaries.tagEnforced"
  | "dashboard.boundaries.tagObserve"
  | "dashboard.boundaries.tagNa"
  | "dashboard.boundaries.active"
  | "dashboard.boundaries.add"
  | "dashboard.review.requestedBy"
  | "dashboard.review.empty"
  | "dashboard.events.title"
  | "dashboard.events.viewFull"
  | "dashboard.events.fAll"
  | "dashboard.events.fAllowed"
  | "dashboard.events.fAsked"
  | "dashboard.events.fBlocked"
  | "dashboard.events.fSystem"
  | "dashboard.events.empty"
  | "dashboard.events.sourceShell"
  | "nav.group.main"
  | "nav.group.configure"
  | "page.back"
  | "crumb.dashboard"
  | "crumb.activity"
  | "crumb.review"
  | "crumb.policies"
  | "crumb.boundaries"
  | "crumb.settings"
  | "page.dashboard.desc"
  | "page.activity.desc"
  | "page.review.desc"
  | "page.policies.desc"
  | "page.boundaries.desc"
  | "page.settings.desc"
  | "policy.dashboardLink"
  | "boundaries.dashboardLink"
  /* ---------- v0.2 page-level subsections (new full page designs) ---------- */
  /* Policies */
  | "policies.tabs.rules"
  | "policies.tabs.trustZones"
  | "policies.tabs.riskLevels"
  | "policies.tabs.policySets"
  | "policies.newRule"
  | "policies.rule.criticalPaths"
  | "policies.rule.criticalPaths.desc"
  | "policies.rule.sensitiveFiles"
  | "policies.rule.sensitiveFiles.desc"
  | "policies.rule.gitSafety"
  | "policies.rule.gitSafety.desc"
  | "policies.rule.networkRestrictions"
  | "policies.rule.networkRestrictions.desc"
  | "policies.rule.packageSafety"
  | "policies.rule.packageSafety.desc"
  /* Activity */
  | "activity.tabs.all"
  | "activity.tabs.allowed"
  | "activity.tabs.asked"
  | "activity.tabs.blocked"
  | "activity.filter"
  | "activity.search"
  | "activity.col.time"
  | "activity.col.action"
  | "activity.col.source"
  | "activity.col.risk"
  | "activity.col.result"
  | "activity.col.menu"
  | "activity.showing"
  /* Review full-page UI */
  | "review.recentDecisions"
  | "review.decision"
  | "review.pending.count"
  | "review.tabs.pending"
  | "review.tabs.history"
  | "review.col.requestedBy"
  | "review.allowAlways"
  | "review.allowAlways.disabled"
  | "review.clearAll"
  /* Settings */
  | "settings.tabs.general"
  | "settings.tabs.security"
  | "settings.tabs.ui"
  | "settings.tabs.uiDesc"
  | "settings.tabs.disabledHint"
  | "settings.tabs.advanced"
  | "settings.group.startup"
  | "settings.group.startup.label"
  | "settings.group.notifications"
  | "settings.group.notifications.label"
  | "settings.notif.all"
  | "settings.notif.ask-only"
  | "settings.notif.none"
  | "settings.group.theme"
  | "settings.group.language"
  | "settings.group.data"
  | "settings.data.export"
  | "settings.data.exportDesc"
  | "settings.data.clear"
  | "settings.data.clearDesc"
  | "settings.security.failClosed"
  | "settings.security.failClosedDesc"
  | "settings.security.approvalTimeout"
  | "settings.security.approvalTimeoutDesc"
  | "settings.security.sessionMode"
  | "settings.security.sessionModeDesc"
  | "settings.security.protected"
  | "settings.security.observe"
  | "settings.security.active"
  | "settings.security.sensitiveResources"
  | "settings.security.credentialProtection"
  | "settings.security.credentialProtectionDesc"
  | "settings.security.shellProtection"
  | "settings.security.shellProtectionDesc"
  | "settings.security.gitProtection"
  | "settings.security.gitProtectionDesc"
  | "settings.advanced.diagnostics"
  | "settings.advanced.running"
  | "settings.advanced.experimental"
  | "settings.advanced.shadowMode"
  | "settings.advanced.shadowModeDesc"
  | "settings.advanced.rawFacts"
  | "settings.advanced.rawFactsDesc"
  | "settings.dropdown.system"
  /* Home additions */
  | "dashboard.types.network"
  | "dashboard.types.other"
  | "dashboard.health.hint"
  /* App sidebar */
  | "sidebar.nav.dashboard"
  | "sidebar.nav.activity"
  | "sidebar.nav.review"
  | "sidebar.nav.policies"
  | "sidebar.nav.boundaries"
  | "sidebar.nav.settings"
  | "sidebar.nav.main"
  | "sidebar.nav.configure"
  | "sidebar.mode.active"
  | "sidebar.mode.activeDesc"
  | "sidebar.mode.recommended"
  | "sidebar.mode.modeDesc"
  | "sidebar.mode.change"
  | "sidebar.scope.label"
  | "sidebar.scope.thisComputer"
  | "sidebar.scope.allBoundaries"
  | "sidebar.scope.change"
  | "sidebar.local.title"
  | "sidebar.local.noCloud"
  | "sidebar.local.noAccount"
  | "sidebar.local.noTelemetry"
  | "sidebar.local.localData"
  | "sidebar.footer.version"
  | "sidebar.footer.checkUpdates"
  | "header.feedback"
  | "header.protection.active"
  | "header.protection.inactive"
  | "header.nav.dashboard"
  /* Activity / Review / Boundaries / Policies pages */
  | "activity.title"
  | "activity.desc"
  | "review.desc"
  | "boundaries.title"
  | "boundaries.desc"
  | "boundaries.class.title"
  | "boundaries.class.desc"
  | "boundaries.col.class"
  | "boundaries.col.name"
  | "boundaries.col.description"
  | "boundaries.col.enforce"
  | "boundaries.col.obs"
  | "boundaries.col.note"
  | "boundaries.col.actions"
  | "boundaries.active"
  | "boundaries.class.A"
  | "boundaries.class.A.desc"
  | "boundaries.class.B"
  | "boundaries.class.B.desc"
  | "boundaries.class.C"
  | "boundaries.class.C.desc"
  | "boundaries.class.D"
  | "boundaries.class.D.desc"
  | "boundaries.class.E"
  | "boundaries.class.E.desc"
  | "boundaries.class.F"
  | "boundaries.class.F.desc"
  | "boundaries.active.yes"
  | "boundaries.active.no"
  | "boundaries.note.A"
  | "boundaries.note.B"
  | "boundaries.note.C"
  | "boundaries.note.D"
  | "boundaries.note.E"
  | "boundaries.note.F"
  | "boundaries.add"
  | "boundaries.current.title"
  | "boundaries.current.empty"
  | "boundaries.current.path"
  | "boundaries.current.status"
  | "boundaries.current.enforced"
  | "boundaries.current.obs"
  | "boundaries.current.unsupported"
  | "boundaries.current.active"
  | "empty.noActivity.k"
  | "empty.noActivity.v"
  | "empty.noPending.k"
  | "empty.noPending.v"
  | "empty.noBoundaries.k"
  | "empty.noBoundaries.v"
  | "policies.rule.trustZone"
  | "policies.rule.trustZone.desc"
  | "policies.rule.networkFuture"
  | "policies.rule.networkFuture.desc"
  /* Trust Zones tab */
  | "policies.trust.title"
  | "policies.trust.desc"
  | "policies.trust.addDir"
  | "policies.trust.placeholder"
  | "policies.trust.empty"
  | "policies.trust.emptyDesc"
  | "policies.trust.remove"
  | "policies.trust.systemDirs"
  /* Risk Levels tab */
  | "policies.risk.title"
  | "policies.risk.desc"
  | "policies.risk.strict"
  | "policies.risk.strictDesc"
  | "policies.risk.standard"
  | "policies.risk.standardDesc"
  | "policies.risk.relaxed"
  | "policies.risk.relaxedDesc"
  | "policies.risk.current"
  | "policies.risk.apply"
  | "policies.risk.applied"
  /* Policy Sets tab */
  | "policies.sets.title"
  | "policies.sets.desc"
  | "policies.sets.development"
  | "policies.sets.developmentDesc"
  | "policies.sets.production"
  | "policies.sets.productionDesc"
  | "policies.sets.custom"
  | "policies.sets.customDesc"
  | "policies.sets.active"
  | "policies.sets.activate"
  | "policies.sets.rulesCount";

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
  "home.consumer.badge": "ActionGuard",
  "home.consumer.tagline": "Give AI room to work. Keep control of what it can do.",
  "home.consumer.title": "Let AI work on your computer safely.",
  "home.consumer.subtitle": "Routine actions run. Dangerous ones stop or ask you first. Everything stays local.",
  "home.consumer.cta": "Protect this computer",
  "home.consumer.explore": "Explore first",
  "home.consumer.supported": "Protects AI actions that pass through supported boundaries.",
  "home.consumer.trust": "Local only · No account · Nothing uploaded",
  "home.consumer.onboarding.title": "Set up protection",
  "home.consumer.onboarding.scope.title": "What should ActionGuard protect?",
  "home.consumer.onboarding.scope.computer": "This computer",
  "home.consumer.onboarding.scope.computerDesc": "Your user files, and everything AI automation can reach on this machine.",
  "home.consumer.onboarding.level.title": "Protection level",
  "home.consumer.onboarding.level.recommended": "Recommended",
  "home.consumer.onboarding.level.recommendedDesc": "Automatically allow routine actions. Ask before high-impact actions. Deny critical actions.",
  "home.consumer.onboarding.protect.title": "ActionGuard will protect:",
  "home.consumer.onboarding.protect.file": "Important files",
  "home.consumer.onboarding.protect.shell": "Destructive actions",
  "home.consumer.onboarding.protect.git": "Git changes",
  "home.consumer.onboarding.protect.package": "Package installation",
  "home.consumer.onboarding.protect.secret": "Passwords & API keys",
  "home.consumer.onboarding.protect.routine": "Lets routine AI work continue.",
  "home.consumer.onboarding.protect.consequential": "Asks before high-impact actions.",
  "home.consumer.onboarding.protect.critical": "Blocks critical actions.",
  "home.consumer.onboarding.privacy.title": "Private by design",
  "home.consumer.onboarding.privacy.desc": "ActionGuard runs locally. No account. No cloud telemetry. Your activity stays on this computer.",
  "home.consumer.onboarding.changeLater": "You can change advanced policies later.",
  "home.consumer.onboarding.start": "Start Protection",
  "home.consumer.onboarding.back": "Back",
  "home.consumer.starting": "Starting…",
  "home.consumer.active.title": "Protection Active",
  "home.consumer.active.subtitle": "AI can keep working. ActionGuard will step in when an action crosses your safety boundary.",
  "home.consumer.active.allowed": "actions allowed",
  "home.consumer.active.reviewed": "actions reviewed",
  "home.consumer.active.blocked": "actions blocked",
  "home.consumer.active.viewActivity": "View activity",
  "home.consumer.active.pause": "Pause",
  "home.consumer.active.pausing": "Pausing…",
  "home.consumer.active.pauseHint": "Pausing stops enforcement until you turn it back on.",
  "home.consumer.active.lastBlocked": "Last blocked action",
  "home.consumer.active.lastBlockedEmpty": "No blocked actions yet.",
  "home.consumer.active.supported": "Protects AI actions that pass through supported boundaries.",
  "home.consumer.activity.title": "What AI did",
  "home.consumer.activity.viewAll": "View all",
  "home.consumer.activity.empty": "No activity yet. Actions will appear here as AI automation works.",
  "home.consumer.activity.why": "Why was this blocked?",
  "home.consumer.activity.rule": "Rule",
  "home.consumer.activity.decision": "Decision",
  "home.consumer.advanced": "Advanced",
  "home.consumer.advancedHint": "Choose a workspace and a protection mode.",
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
  "coverage.title": "Protection Coverage",
  "coverage.viewDetails": "View details",
  "coverage.scanning": "Scanning your system…",
  "coverage.fullyProtected": "Can block",
  "coverage.partial": "Monitored",
  "coverage.observeOnly": "Not covered yet",
  "coverage.highQuality": "High-quality",
  "coverage.generic": "Generic",
  "coverage.observe": "Observe",
  "coverage.inactive": "Not active",
  "coverage.genericFallback": "AI apps using shell are protected — unknown AI tools that bypass shell are not covered yet.",
  "approval.title": "Approval required",
  "approval.subtitle":
    "An automated action crossed your safety boundary. Decide before it runs.",
  "approval.source": "Source",
  "approval.wants": "wants to",
  "approval.risk": "Risk",
  "approval.reason": "Reason",
  "approval.matchedRule": "Matched rule",
  "approval.technical": "Technical details",
  "approval.technical.category": "Category",
  "approval.technical.actionId": "Action ID",
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
  "footer.slogan3": "v0.3",
  "win.min": "Minimize",
  "win.max": "Maximize",
  "win.restore": "Restore",
  "win.close": "Close",
  "nav.dashboard": "Dashboard",
  "nav.activity": "Activity",
  "brand.tag": "AI Safety Boundary",
  "sidebar.protectionActive": "Protection active",
  "sidebar.protectionInactive": "Protection inactive",
  "onboarding.tagline": "Protect your computer from risky AI actions.",
  "onboarding.cta": "Protect this computer",
  "onboarding.starting": "Starting…",
  "onboarding.defaults": "Recommended: current directory · Recommended protection",
  "onboarding.advanced": "Advanced settings",
  "onboarding.workspace": "Protected workspace",
  "onboarding.workspaceHint": "AI automation is allowed to access files inside this directory.",
  "onboarding.privacy": "Local only · No account · No telemetry",
  "onboarding.startError": "Failed to start protection. Make sure the ActionGuard CLI is installed and in your PATH.",
  "home.consumer.onboarding.changeDir": "Change",
  "dashboard.hero.active": "AI can work. You stay in control.",
  "dashboard.hero.inactive": "Protection inactive",
  "dashboard.hero.activeSub": "High-impact actions are reviewed before they run on your machine.",
  "dashboard.hero.inactiveSub": "Your AI is not currently being watched. Start protection to keep it within safe boundaries.",
  "dashboard.meta.enforced": "Enforced",
  "dashboard.meta.stopped": "Stopped",
  "dashboard.meta.mode": "Mode",
  "dashboard.meta.scope": "Scope",
  "dashboard.mode.recommended": "Recommended",
  "dashboard.scope.thisComputer": "This Computer",
  "dashboard.action.viewActivity": "View activity",
  "dashboard.action.pause": "Pause protection",
  "dashboard.action.pausing": "Pausing…",
  "dashboard.action.protect": "Protect this computer",
  "dashboard.action.starting": "Starting…",
  "dashboard.stat.total": "Total actions",
  "dashboard.stat.allowed": "Allowed",
  "dashboard.stat.asked": "Asked",
  "dashboard.stat.blocked": "Blocked",
  "dashboard.activity.title": "Recent activity",
  "dashboard.activity.viewAll": "View all",
  "dashboard.activity.empty": "No AI activity detected yet. Actions will appear here when AI automation starts working.",
  "dashboard.review.title": "Review queue",
  "dashboard.review.requestedBy": "Requested by",
  "dashboard.boundaries.title": "Protected boundaries",
  "dashboard.boundaries.empty": "No active boundaries. Start protection to see covered execution paths.",
  "dashboard.advanced.title": "Advanced settings",
  "nav.live": "Live",
  "nav.policies": "Policies",
  "nav.boundaries": "Boundaries",
  "nav.settings": "Settings",
  "policies.title": "Policy Rules",
  "policies.desc": "Rules that decide whether an action is allowed, asked, or blocked.",
  "settings.title": "Settings",
  "settings.desc": "Language and other preferences.",
  "settings.language": "Language",
  "settings.languageHint": "Choose the display language for ActionGuard.",
  "home.active.confirmStop": "Stop the current session? All actions recorded so far will remain in the ledger.",
  "home.advanced.title": "Advanced",
  "home.advanced.workspace.title": "Workspace",
  "home.advanced.workspace.hint": "Only files inside this directory are monitored.",
  "home.advanced.mode.title": "Protection Mode",
  "home.advanced.mode.interactive": "Interactive",
  "home.advanced.mode.observe": "Observe",
  "home.advanced.mode.observeDesc": "Observe mode records but never blocks.",
  "home.advanced.rules.title": "Policy Rules",
  "home.advanced.rules.empty": "No rules loaded.",
  "home.advanced.enforcement.title": "Enforcement",
  "home.advanced.enforcement.empty": "No enforcement data.",
  "home.advanced.diagnostics.title": "Diagnostics",
  "home.advanced.diagnostics.run": "Run Diagnostics",
  "home.advanced.diagnostics.desc": "Check boundary coverage and system status.",
  "home.metrics.title": "Key Metrics",
  "home.metrics.sessions": "Sessions",
  "home.metrics.flagged": "Flagged",
  "home.metrics.rate": "Rate",
  "home.metrics.blockedActions": "Blocked",
  "onboarding.headline": "Give AI room to work. Keep control of what it can do.",
  "onboarding.subline": "Routine actions continue automatically. High-impact actions ask you first. Critical dangerous actions are blocked immediately.",
  "onboarding.defaults.scope": "Default scope: This Computer · Recommended protection",
  "onboarding.rec": "RECOMMENDED",
  "onboarding.scope.title": "Protection scope",
  "onboarding.scope.computer": "This Computer",
  "onboarding.scope.computerDesc": "Your user files and everything AI automation can reach on this machine.",
  "onboarding.scope.customHint": "Or pick a specific project directory you want to protect.",
  "onboarding.changeDir": "Change",
  "onboarding.level.title": "Protection level",
  "onboarding.level.recommended": "Recommended",
  "onboarding.level.recommendedDesc": "Automatically allow routine actions. Ask before high-impact actions. Block critical actions.",
  "onboarding.level.observe": "Observe only",
  "onboarding.level.observeDesc": "Record all actions to the local ledger. Never block. Useful for shadow-mode auditing.",
  "onboarding.promise.routine": "Routine actions run automatically",
  "onboarding.promise.ask": "High-impact actions ask you first",
  "onboarding.promise.block": "Critical dangerous actions are blocked",
  "onboarding.protect.title": "ActionGuard protects these 5 categories",
  "onboarding.protect.file": "Important files",
  "onboarding.protect.shell": "Shell commands",
  "onboarding.protect.git": "Git operations",
  "onboarding.protect.package": "Package installs",
  "onboarding.protect.secret": "Secrets & API keys",
  "onboarding.privacy.title": "Private by design",
  "onboarding.privacy.desc": "ActionGuard runs entirely on your machine. Nothing ever leaves it.",
  "onboarding.privacy.p1": "No cloud. Runs 100% locally.",
  "onboarding.privacy.p2": "No account required. No sign-up.",
  "onboarding.privacy.p3": "No telemetry. Your activity stays yours.",
  "home.status.active": "Protection Active",
  "home.status.inactive": "Protection Inactive",
  "home.hero.title.active": "AI can work. You stay in control.",
  "home.hero.title.inactive": "Protection is off",
  "home.hero.desc.active": "High-impact actions are reviewed before they run on your machine. Routine work continues without friction.",
  "home.hero.desc.inactive": "Your AI is not currently being watched. Start protection to keep it within safe boundaries.",
  "home.hero.starting": "Starting protection…",
  "home.hero.protect": "🛡 Protect this computer",
  "home.hero.viewActivity": "View Activity",
  "home.hero.stopping": "Stopping…",
  "home.hero.pause": "Pause protection",
  "home.today.label": "Today",
  "home.today.allowed": "Allowed",
  "home.today.asked": "Needs your review",
  "home.today.blocked": "Blocked",
  "home.today.viewReview": "Review pending",
  "home.lastBlocked.k": "Last blocked action",
  "home.lastBlocked.view": "View details",
  "home.activity.title": "Recent AI activity",
  "home.activity.viewAll": "View all activity",
  "home.activity.empty.k": "⏳ Waiting for AI activity",
  "home.activity.empty.v": "No actions detected yet. When AI automation starts working, every action it takes will appear here in real time.",
  "home.protect.title": "What ActionGuard protects",
  "home.protect.hint": "These action categories are monitored whenever protection is active.",
  "home.advanced.k": "Advanced settings",
  "home.advanced.hint": "Workspace, mode, policies, boundaries and diagnostics.",
  "home.advanced.rules.manage": "Manage rules",
  "home.advanced.rules.hint": "Fine-tune which actions are allowed, asked, or blocked.",
  "home.advanced.enforcement.manage": "Manage boundaries",
  "home.advanced.enforcement.hint": "Execution paths where ActionGuard can observe or enforce protection.",
  "home.advanced.diagnostics.okEngine": "✓ Engine running",
  "home.advanced.diagnostics.okPolicy": "✓ Policies loaded",
  "home.advanced.diagnostics.okSession": "✓ Active protection session",
  "home.advanced.diagnostics.warnNoSession": "⚠ No active session — start protection first",
  "home.trust.local": "Local only",
  "home.trust.noAccount": "No account",
  "home.trust.noTelemetry": "No telemetry",
  "dashboard.hero.titleWhite": "AI can work.",
  "dashboard.hero.titleGreen": "You stay in control.",
  "dashboard.hero.inactiveWhite": "Protection is",
  "dashboard.hero.inactiveGreen": "currently off.",
  "dashboard.hero.metaBoundaries": "Protected Boundaries",
  "dashboard.quickActions.title": "Quick Actions",
  "dashboard.quickActions.viewActivity": "View Activity",
  "dashboard.quickActions.pendingReviews": "Pending Reviews",
  "dashboard.quickActions.openSettings": "Open Settings",
  "dashboard.stat.vsYesterday": "vs yesterday",
  "dashboard.stat.ofTotal": "of total",
  "dashboard.activity.showingLast": "Showing last {n} actions",
  "dashboard.donut.title": "Top Action Types",
  "dashboard.donut.total": "Total",
  "dashboard.donut.file": "File Operations",
  "dashboard.donut.shell": "Shell Commands",
  "dashboard.donut.git": "Git Operations",
  "dashboard.donut.secret": "Secret Access",
  "dashboard.donut.package": "Package Installs",
  "dashboard.health.title": "Protection Health",
  "dashboard.health.engine": "Enforcement engine responsive",
  "dashboard.health.policy": "Policy up to date",
  "dashboard.boundaries.manage": "Manage",
  "dashboard.boundaries.tagEnforced": "ENFORCED",
  "dashboard.boundaries.tagObserve": "OBSERVE",
  "dashboard.boundaries.tagNa": "UNSUPPORTED",
  "dashboard.boundaries.active": "Active",
  "dashboard.boundaries.add": "Add Boundary",
  "dashboard.review.viewAll": "View All Reviews",
  "dashboard.review.empty": "No pending reviews — all actions processed.",
  "dashboard.events.title": "Recent Events",
  "dashboard.events.viewFull": "View Full Log",
  "dashboard.events.fAll": "All",
  "dashboard.events.fAllowed": "Allowed",
  "dashboard.events.fAsked": "Asked",
  "dashboard.events.fBlocked": "Blocked",
  "dashboard.events.fSystem": "System",
  "dashboard.events.empty": "No events yet. Start a protection session to see AI action events here.",
  "dashboard.events.sourceShell": "Protected Shell (bash)",
  "nav.group.main": "Main",
  "nav.group.configure": "Configure",
  "page.back": "Back to Dashboard",
  "crumb.dashboard": "Dashboard",
  "crumb.activity": "Activity",
  "crumb.review": "Review",
  "crumb.policies": "Policies",
  "crumb.boundaries": "Boundaries",
  "crumb.settings": "Settings",
  "page.dashboard.desc": "Protection status, recent activity, and quick actions at a glance.",
  "page.activity.desc": "All historical AI sessions and actions recorded on this machine.",
  "page.review.desc": "Approve or deny actions that are waiting for your decision.",
  "page.policies.desc": "Define rules for which actions are allowed, asked, or blocked.",
  "page.boundaries.desc": "Execution paths where ActionGuard can observe or enforce protection.",
  "page.settings.desc": "Language, theme, and other application preferences.",
  "policy.dashboardLink": "← Back to Dashboard",
  "boundaries.dashboardLink": "← Back to Dashboard",

  /* ---------- Policies full-page UI ---------- */
  "policies.tabs.rules": "Rules",
  "policies.tabs.trustZones": "Trust Zones",
  "policies.tabs.riskLevels": "Risk Levels",
  "policies.tabs.policySets": "Policy Sets",
  "policies.newRule": "+ New Rule",
  "policies.rule.criticalPaths": "Block critical system paths",
  "policies.rule.criticalPaths.desc": "Prevents access to system critical directories",
  "policies.rule.sensitiveFiles": "Protect sensitive files",
  "policies.rule.sensitiveFiles.desc": "Blocks access to .env keys, credentials",
  "policies.rule.gitSafety": "Git safety rules",
  "policies.rule.gitSafety.desc": "Prevents destructive git operations",
  "policies.rule.networkRestrictions": "Network restrictions",
  "policies.rule.networkRestrictions.desc": "Controls outbound network access",
  "policies.rule.packageSafety": "Package safety",
  "policies.rule.packageSafety.desc": "Validates package installations",

  /* ---------- Activity full-page UI ---------- */
  "activity.tabs.all": "All",
  "activity.tabs.allowed": "Allowed",
  "activity.tabs.asked": "Asked",
  "activity.tabs.blocked": "Blocked",
  "activity.filter": "Filter",
  "activity.search": "Search actions…",
  "activity.col.time": "Time",
  "activity.col.action": "Action",
  "activity.col.source": "Source",
  "activity.col.risk": "Risk",
  "activity.col.result": "Result",
  "activity.col.menu": "",
  "activity.showing": "Showing {shown} of {total} actions",

  /* ---------- Review full-page UI ---------- */
  "review.recentDecisions": "Recent Decisions",
  "review.decision": "Decision",
  "review.pending.count": "{count} pending {count, plural, one {request} other {requests}}",
  "review.tabs.pending": "Pending",
  "review.tabs.history": "History",
  "review.col.requestedBy": "Requested by",
  "review.allowAlways": "Allow Always",
  "review.allowAlways.disabled": "Not yet supported",
  "review.clearAll": "Clear All",

  /* ---------- Settings full-page UI ---------- */
  "settings.tabs.general": "General",
  "settings.tabs.security": "Security",
  "settings.tabs.ui": "UI",
  "settings.tabs.uiDesc": "Customize the look and feel of ActionGuard.",
  "settings.tabs.disabledHint": "Coming in a future update",
  "settings.tabs.advanced": "Advanced",
  "settings.group.startup": "Startup",
  "settings.group.startup.label": "Start ActionGuard on system startup",
  "settings.group.notifications": "Notifications",
  "settings.group.notifications.label": "Show desktop notifications",
  "settings.notif.all": "All actions",
  "settings.notif.ask-only": "Ask & Blocked only",
  "settings.notif.none": "Off",
  "settings.group.theme": "Theme",
  "settings.group.language": "Language",
  "settings.group.data": "Data",
  "settings.data.export": "Export activity log",
  "settings.data.exportDesc": "Download your action history as a file.",
  "settings.data.clear": "Clear local data",
  "settings.data.clearDesc": "Remove all locally stored action history and session data.",
  "settings.dropdown.system": "System Default",

  /* Security Settings */
  "settings.security.failClosed": "Fail-closed by default",
  "settings.security.failClosedDesc": "Block actions when bridge is unreachable or policy cannot be loaded.",
  "settings.security.approvalTimeout": "Approval timeout",
  "settings.security.approvalTimeoutDesc": "Time in seconds before a pending approval is auto-denied.",
  "settings.security.sessionMode": "Current protection mode",
  "settings.security.sessionModeDesc": "Active session determines how actions are handled.",
  "settings.security.protected": "Protected",
  "settings.security.observe": "Observe",
  "settings.security.active": "Active",
  "settings.security.sensitiveResources": "Sensitive Resource Protection",
  "settings.security.credentialProtection": "Credential & Secret detection",
  "settings.security.credentialProtectionDesc": "Detects and flags attempts to access secrets.",
  "settings.security.shellProtection": "Shell command protection",
  "settings.security.shellProtectionDesc": "Intercepts shell commands for review.",
  "settings.security.gitProtection": "Git operation protection",
  "settings.security.gitProtectionDesc": "Monitors git push, commit, and config changes.",

  /* Advanced Settings */
  "settings.advanced.diagnostics": "System Diagnostics",
  "settings.advanced.running": "Running...",
  "settings.advanced.experimental": "Experimental Features",
  "settings.advanced.shadowMode": "Shadow mode",
  "settings.advanced.shadowModeDesc": "Audit all actions without any blocking.",
  "settings.advanced.rawFacts": "Raw fact viewer",
  "settings.advanced.rawFactsDesc": "Inspect the internal action facts in real-time.",

  /* ---------- Home additions ---------- */
  "dashboard.types.network": "Network Access",
  "dashboard.types.other": "Other",
  "dashboard.health.hint": "Check protection health →",
  /* App sidebar labels */
  "sidebar.nav.dashboard": "Dashboard",
  "sidebar.nav.activity": "Activity",
  "sidebar.nav.review": "Review",
  "sidebar.nav.policies": "Policies",
  "sidebar.nav.boundaries": "Boundaries",
  "sidebar.nav.settings": "Settings",
  "sidebar.nav.main": "Main",
  "sidebar.nav.configure": "Configure",
  "sidebar.mode.active": "Protection Active",
  "sidebar.mode.activeDesc": "AI can work. You stay in control.",
  "sidebar.mode.recommended": "Recommended",
  "sidebar.mode.modeDesc": "Routine allowed · High-risk ask · Critical blocked",
  "sidebar.mode.change": "Change Mode",
  "sidebar.scope.label": "Protected Scope",
  "sidebar.scope.thisComputer": "This Computer",
  "sidebar.scope.allBoundaries": "All supported boundaries",
  "sidebar.scope.change": "Change Scope",
  "sidebar.local.title": "Local First",
  "sidebar.local.noCloud": "No cloud",
  "sidebar.local.noAccount": "No account",
  "sidebar.local.noTelemetry": "No telemetry",
  "sidebar.local.localData": "All data stays on this device",
  "sidebar.footer.version": "v0.3",
  "sidebar.footer.checkUpdates": "Check for updates",
  "header.feedback": "Feedback",
  "header.protection.active": "Protection Active",
  "header.protection.inactive": "Protection Inactive",
  "header.nav.dashboard": "Dashboard",
  /* Activity page */
  "activity.title": "Activity Log",
  "activity.desc": "All recorded AI actions, sorted by time.",
  /* Review page */
  "review.title": "Review Queue",
  "review.desc": "Approve or deny pending actions.",
  /* Boundaries page */
  "boundaries.title": "Protected Boundaries",
  "boundaries.desc": "Execution paths where ActionGuard can observe or enforce protection.",
  "boundaries.class.title": "Boundary Classes",
  "boundaries.class.desc": "ActionGuard classifies execution paths into six boundary classes (A–F). The core engine is brand-agnostic.",
  "boundaries.col.class": "Class",
  "boundaries.col.name": "Name",
  "boundaries.col.description": "Description",
  "boundaries.col.enforce": "Enforced",
  "boundaries.col.obs": "Observed",
  "boundaries.col.note": "Note",
  "boundaries.col.actions": "Active Actions",
  "boundaries.active": "Active",
  "boundaries.class.A": "Tool Hook",
  "boundaries.class.A.desc": "Pre-action hook inside the automation",
  "boundaries.class.B": "Exec Approval",
  "boundaries.class.B.desc": "Automation's own execution policy",
  "boundaries.class.C": "Protected Shell",
  "boundaries.class.C.desc": "Preexec hook (PowerShell PSReadLine) — active session only",
  "boundaries.class.D": "Runtime Sandbox",
  "boundaries.class.D.desc": "Future L3 enforcement",
  "boundaries.class.E": "System Enforcement",
  "boundaries.class.E.desc": "Future L4 enforcement",
  "boundaries.class.F": "Remote",
  "boundaries.class.F.desc": "Actions never land on this machine — out of scope",
  "boundaries.active.yes": "Yes",
  "boundaries.active.no": "No",
  "boundaries.note.A": "Brand-agnostic, added via registry",
  "boundaries.note.B": "Brand-agnostic, added via registry",
  "boundaries.note.C": "Enforced only during active protected session",
  "boundaries.note.D": "",
  "boundaries.note.E": "",
  "boundaries.note.F": "Out of scope",
  "boundaries.add": "Add Boundary",
  "boundaries.current.title": "Current Execution Paths",
  "boundaries.current.empty": "No active execution paths. Start a protection session to see which paths are being monitored.",
  "boundaries.current.path": "Path",
  "boundaries.current.status": "Status",
  "boundaries.current.enforced": "Enforced",
  "boundaries.current.obs": "Observe-only",
  "boundaries.current.unsupported": "Unsupported",
  "boundaries.current.active": "Active",
  /* Empty states */
  "empty.noActivity.k": "No activity yet",
  "empty.noActivity.v": "Start a protection session to see AI actions appear here in real time.",
  "empty.noPending.k": "No pending reviews",
  "empty.noPending.v": "All actions have been processed. Great job keeping your AI in check.",
  "empty.noBoundaries.k": "No active boundaries",
  "empty.noBoundaries.v": "Start a protection session to begin monitoring execution paths.",
  /* Trust zone pill */
  "policies.rule.trustZone": "Trust Zones",
  "policies.rule.trustZone.desc": "Define trusted directories where AI can operate freely",
  "policies.rule.networkFuture": "Network restrictions",
  "policies.rule.networkFuture.desc": "Controls outbound network access — coming in v0.3",
  /* Trust Zones */
  "policies.trust.title": "Trusted Directories",
  "policies.trust.desc": "Define directories where AI can read and write without prompting. Be careful — files here are fully accessible.",
  "policies.trust.addDir": "Add Directory",
  "policies.trust.placeholder": "Enter path (e.g. C:\\Projects\\my-app)",
  "policies.trust.empty": "No trusted directories",
  "policies.trust.emptyDesc": "AI actions in untrusted directories will always be reviewed.",
  "policies.trust.remove": "Remove",
  "policies.trust.systemDirs": "System directories (C:\\, Windows, Program Files) cannot be added as trust zones.",
  /* Risk Levels */
  "policies.risk.title": "Risk Levels",
  "policies.risk.desc": "Choose how aggressively ActionGuard classifies and blocks risky actions.",
  "policies.risk.strict": "Strict",
  "policies.risk.strictDesc": "Low-risk actions also require approval. Maximum protection for sensitive environments.",
  "policies.risk.standard": "Standard",
  "policies.risk.standardDesc": "Routine actions allowed, medium and high-risk actions require approval.",
  "policies.risk.relaxed": "Relaxed",
  "policies.risk.relaxedDesc": "Only critical and high-risk actions require approval.",
  "policies.risk.current": "Current",
  "policies.risk.apply": "Apply",
  "policies.risk.applied": "Active",
  /* Policy Sets */
  "policies.sets.title": "Policy Sets",
  "policies.sets.desc": "Pre-configured policy bundles for common scenarios. Activate a set to apply all rules at once.",
  "policies.sets.development": "Development",
  "policies.sets.developmentDesc": "Git pushes, package installs, and file writes are allowed after approval. Network requests are reviewed.",
  "policies.sets.production": "Production",
  "policies.sets.productionDesc": "All write operations, git pushes, and network access require approval. Strict file path protection.",
  "policies.sets.custom": "Custom",
  "policies.sets.customDesc": "Define your own rules using the Rules tab above.",
  "policies.sets.active": "Active",
  "policies.sets.activate": "Activate",
  "policies.sets.rulesCount": "{{n}} rules",
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
  "home.consumer.badge": "ActionGuard",
  "home.consumer.tagline": "给 AI 干活的空间。对它能做什么，保持控制。",
  "home.consumer.title": "让 AI 安全地在你的电脑上干活。",
  "home.consumer.subtitle": "常规动作照常执行。危险动作先停下问你要不要。一切都在本地。",
  "home.consumer.cta": "保护这台电脑",
  "home.consumer.explore": "先看看",
  "home.consumer.supported": "保护经由受支持边界通过的 AI 动作。",
  "home.consumer.trust": "仅本地 · 无需账号 · 不上传任何数据",
  "home.consumer.onboarding.title": "设置保护",
  "home.consumer.onboarding.scope.title": "ActionGuard 保护什么？",
  "home.consumer.onboarding.scope.computer": "这台电脑",
  "home.consumer.onboarding.scope.computerDesc": "你的用户文件，以及 AI 自动化在这台机器上能碰到的所有东西。",
  "home.consumer.onboarding.level.title": "保护级别",
  "home.consumer.onboarding.level.recommended": "推荐",
  "home.consumer.onboarding.level.recommendedDesc": "常规动作自动放行。高影响动作先询问。致命动作直接拒绝。",
  "home.consumer.onboarding.protect.title": "ActionGuard 将保护：",
  "home.consumer.onboarding.protect.file": "重要文件",
  "home.consumer.onboarding.protect.shell": "破坏性动作",
  "home.consumer.onboarding.protect.git": "Git 变更",
  "home.consumer.onboarding.protect.package": "软件安装",
  "home.consumer.onboarding.protect.secret": "密码与 API 密钥",
  "home.consumer.onboarding.protect.routine": "让常规 AI 操作照常继续。",
  "home.consumer.onboarding.protect.consequential": "高风险动作先问过你。",
  "home.consumer.onboarding.protect.critical": "关键动作直接拦截。",
  "home.consumer.onboarding.privacy.title": "设计上就保护隐私",
  "home.consumer.onboarding.privacy.desc": "ActionGuard 完全本地运行。无账号。无云端遥测。你的活动只留在这台电脑上。",
  "home.consumer.onboarding.changeLater": "高级策略以后可以随时调整。",
  "home.consumer.onboarding.start": "开始保护",
  "home.consumer.onboarding.back": "返回",
  "home.consumer.starting": "启动中…",
  "home.consumer.active.title": "保护已开启",
  "home.consumer.active.subtitle": "AI 可以继续干活。当动作越过你的安全边界时，ActionGuard 会出手。",
  "home.consumer.active.allowed": "个动作已放行",
  "home.consumer.active.reviewed": "个动作已询问",
  "home.consumer.active.blocked": "个动作已被阻止",
  "home.consumer.active.viewActivity": "查看活动",
  "home.consumer.active.pause": "暂停",
  "home.consumer.active.pausing": "暂停中…",
  "home.consumer.active.pauseHint": "暂停后，在你重新开启前不会再有强制拦截。",
  "home.consumer.active.lastBlocked": "最近一次被拦截的动作",
  "home.consumer.active.lastBlockedEmpty": "还没有拦截记录。",
  "home.consumer.active.supported": "保护经由受支持边界通过的 AI 动作。",
  "home.consumer.activity.title": "AI 做了什么",
  "home.consumer.activity.viewAll": "查看全部",
  "home.consumer.activity.empty": "暂无活动。AI 自动化开始工作后，动作会显示在这里。",
  "home.consumer.activity.why": "为什么被阻止？",
  "home.consumer.activity.rule": "规则",
  "home.consumer.activity.decision": "决策",
  "home.consumer.advanced": "高级",
  "home.consumer.advancedHint": "选择工作空间与保护模式。",
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
  "coverage.title": "保护覆盖",
  "coverage.viewDetails": "查看详情",
  "coverage.scanning": "正在扫描系统…",
  "coverage.fullyProtected": "可拦截",
  "coverage.partial": "已监控",
  "coverage.observeOnly": "暂未覆盖",
  "coverage.highQuality": "高质量",
  "coverage.generic": "通用",
  "coverage.observe": "观察",
  "coverage.inactive": "未启动",
  "coverage.genericFallback": "使用 Shell 的 AI 应用受到保护——绕过 Shell 的未知 AI 工具暂未覆盖。",
  "approval.title": "需要确认",
  "approval.subtitle": "自动化的操作触发了安全边界，请决定是否允许执行。",
  "approval.source": "来源",
  "approval.wants": "想要",
  "approval.risk": "风险",
  "approval.reason": "原因",
  "approval.matchedRule": "匹配规则",
  "approval.technical": "技术细节",
  "approval.technical.category": "类别",
  "approval.technical.actionId": "动作 ID",
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
  "footer.slogan3": "v0.3",
  "win.min": "最小化",
  "win.max": "最大化",
  "win.restore": "还原",
  "win.close": "关闭",
  "nav.dashboard": "概览",
  "nav.activity": "活动",
  "brand.tag": "AI 安全边界",
  "sidebar.protectionActive": "防护已开启",
  "sidebar.protectionInactive": "防护已关闭",
  "onboarding.tagline": "保护你的电脑，免受 AI 越界操作的风险。",
  "onboarding.cta": "保护这台电脑",
  "onboarding.starting": "启动中…",
  "onboarding.defaults": "默认：当前目录 · 推荐保护模式",
  "onboarding.advanced": "高级设置",
  "onboarding.workspace": "受保护工作目录",
  "onboarding.workspaceHint": "AI 自动化只能访问该目录内的文件。",
  "onboarding.privacy": "仅本地 · 无需账号 · 无遥测",
  "onboarding.startError": "启动保护失败。请确认 ActionGuard CLI 已安装并在 PATH 中。",
  "home.consumer.onboarding.changeDir": "更改",
  "dashboard.hero.active": "AI 可以工作，控制权在你。",
  "dashboard.hero.inactive": "防护未开启",
  "dashboard.hero.activeSub": "高风险操作会在执行前经过 ActionGuard 检查。",
  "dashboard.hero.inactiveSub": "当前 AI 未被监控。开启防护，让 AI 在安全的边界内工作。",
  "dashboard.meta.enforced": "已拦截",
  "dashboard.meta.stopped": "已停止",
  "dashboard.meta.mode": "模式",
  "dashboard.meta.scope": "范围",
  "dashboard.mode.recommended": "推荐",
  "dashboard.scope.thisComputer": "本机",
  "dashboard.action.viewActivity": "查看活动",
  "dashboard.action.pause": "暂停防护",
  "dashboard.action.pausing": "暂停中…",
  "dashboard.action.protect": "保护这台电脑",
  "dashboard.action.starting": "启动中…",
  "dashboard.stat.total": "总操作",
  "dashboard.stat.allowed": "已放行",
  "dashboard.stat.asked": "已询问",
  "dashboard.stat.blocked": "已拦截",
  "dashboard.activity.title": "最近活动",
  "dashboard.activity.viewAll": "查看全部",
  "dashboard.activity.empty": "还没有检测到 AI 操作。当 AI 开始工作后，活动会出现在这里。",
  "dashboard.review.title": "待审核",
  "dashboard.review.requestedBy": "请求来源",
  "dashboard.boundaries.title": "受保护边界",
  "dashboard.boundaries.empty": "暂无活跃边界。开启防护后，这里会显示已覆盖的执行路径。",
  "dashboard.advanced.title": "高级设置",
  "nav.live": "实时监控",
  "nav.policies": "策略",
  "nav.boundaries": "边界",
  "nav.settings": "设置",
  "policies.title": "策略规则",
  "policies.desc": "决定动作是被允许、询问还是被拦截的规则。",
  "settings.title": "设置",
  "settings.desc": "语言及其他偏好设置。",
  "settings.language": "语言",
  "settings.languageHint": "选择 ActionGuard 的显示语言。",
  "home.active.confirmStop": "停止当前会话？已记录的动作会保留在账本中。",
  "home.advanced.title": "高级",
  "home.advanced.workspace.title": "工作空间",
  "home.advanced.workspace.hint": "仅监控该目录内的文件。",
  "home.advanced.mode.title": "防护模式",
  "home.advanced.mode.interactive": "交互式",
  "home.advanced.mode.observe": "观察",
  "home.advanced.mode.observeDesc": "观察模式仅记录，不拦截。",
  "home.advanced.rules.title": "策略规则",
  "home.advanced.rules.empty": "未加载规则。",
  "home.advanced.enforcement.title": "强制执行",
  "home.advanced.enforcement.empty": "无强制执行数据。",
  "home.advanced.diagnostics.title": "诊断",
  "home.advanced.diagnostics.run": "运行诊断",
  "home.advanced.diagnostics.desc": "检查边界覆盖与系统状态。",
  "home.metrics.title": "关键指标",
  "home.metrics.sessions": "会话",
  "home.metrics.flagged": "标记",
  "home.metrics.rate": "比率",
  "home.metrics.blockedActions": "已拦截",
  "onboarding.headline": "给 AI 活的空间。对它能做什么，保持控制。",
  "onboarding.subline": "常规操作照常进行。高风险操作先问你。关键危险动作直接阻止。",
  "onboarding.defaults.scope": "默认范围：这台电脑 · 推荐保护模式",
  "onboarding.rec": "推荐",
  "onboarding.scope.title": "保护范围",
  "onboarding.scope.computer": "这台电脑",
  "onboarding.scope.computerDesc": "你的用户文件，以及 AI 自动化在这台机器上能碰到的所有内容。",
  "onboarding.scope.customHint": "或者选择你想要保护的具体项目目录。",
  "onboarding.changeDir": "更改",
  "onboarding.level.title": "保护级别",
  "onboarding.level.recommended": "推荐模式",
  "onboarding.level.recommendedDesc": "常规动作自动放行。高影响动作先询问。致命动作直接拒绝。",
  "onboarding.level.observe": "仅观察",
  "onboarding.level.observeDesc": "把所有动作记录到本地账本，绝不拦截。适合影子模式下做审计。",
  "onboarding.promise.routine": "常规动作自动继续",
  "onboarding.promise.ask": "高影响动作先问过你",
  "onboarding.promise.block": "关键危险动作直接阻止",
  "onboarding.protect.title": "ActionGuard 保护这 5 个类别",
  "onboarding.protect.file": "重要文件",
  "onboarding.protect.shell": "Shell 命令",
  "onboarding.protect.git": "Git 操作",
  "onboarding.protect.package": "软件包安装",
  "onboarding.protect.secret": "密钥与 API Key",
  "onboarding.privacy.title": "设计上就保护隐私",
  "onboarding.privacy.desc": "ActionGuard 完全在你的机器上运行。任何东西都不会离开它。",
  "onboarding.privacy.p1": "无云端，100% 本地运行",
  "onboarding.privacy.p2": "无需账号，不需注册",
  "onboarding.privacy.p3": "无遥测，你的活动只属于你",
  "home.status.active": "防护已开启",
  "home.status.inactive": "防护未开启",
  "home.hero.title.active": "AI 可以继续干活，控制权在你。",
  "home.hero.title.inactive": "防护处于关闭状态",
  "home.hero.desc.active": "高风险动作会在执行前经过你的审核。常规工作照常进行，不会被打断。",
  "home.hero.desc.inactive": "当前 AI 未处于监控之下。开启防护，让它在安全的边界内工作。",
  "home.hero.starting": "正在开启防护…",
  "home.hero.protect": "🛡 保护这台电脑",
  "home.hero.viewActivity": "查看活动",
  "home.hero.stopping": "正在停止…",
  "home.hero.pause": "暂停防护",
  "home.today.label": "今日",
  "home.today.allowed": "已放行",
  "home.today.asked": "待你确认",
  "home.today.blocked": "已阻止",
  "home.today.viewReview": "查看待审核",
  "home.lastBlocked.k": "最近一次被阻止的动作",
  "home.lastBlocked.view": "查看详情",
  "home.activity.title": "最近 AI 活动",
  "home.activity.viewAll": "查看全部活动",
  "home.activity.empty.k": "⏳ 等待 AI 活动",
  "home.activity.empty.v": "还没有检测到 AI 操作。当 AI 自动化开始工作后，它执行的每一个动作都会实时出现在这里。",
  "home.protect.title": "ActionGuard 保护的内容",
  "home.protect.hint": "防护开启后，这些动作类别会被实时监控。",
  "home.advanced.k": "高级设置",
  "home.advanced.hint": "工作空间、保护模式、策略规则、边界与诊断。",
  "home.advanced.rules.manage": "管理规则",
  "home.advanced.rules.hint": "细调哪些动作被允许、询问或阻止。",
  "home.advanced.enforcement.manage": "管理边界",
  "home.advanced.enforcement.hint": "ActionGuard 可以观察或执行保护的执行路径。",
  "home.advanced.diagnostics.okEngine": "✓ 引擎运行中",
  "home.advanced.diagnostics.okPolicy": "✓ 策略已加载",
  "home.advanced.diagnostics.okSession": "✓ 防护会话已激活",
  "home.advanced.diagnostics.warnNoSession": "⚠ 无活跃会话，请先开启防护",
  "home.trust.local": "仅本地运行",
  "home.trust.noAccount": "无需账号",
  "home.trust.noTelemetry": "无遥测",
  "dashboard.hero.titleWhite": "AI 可以工作。",
  "dashboard.hero.titleGreen": "控制权在你。",
  "dashboard.hero.inactiveWhite": "防护",
  "dashboard.hero.inactiveGreen": "未开启。",
  "dashboard.hero.metaBoundaries": "受保护边界",
  "dashboard.quickActions.title": "快速操作",
  "dashboard.quickActions.viewActivity": "查看活动",
  "dashboard.quickActions.pendingReviews": "待审核",
  "dashboard.quickActions.openSettings": "打开设置",
  "dashboard.stat.vsYesterday": "较昨日",
  "dashboard.stat.ofTotal": "占总数",
  "dashboard.activity.showingLast": "显示最近 {n} 条动作",
  "dashboard.donut.title": "主要动作类型",
  "dashboard.donut.total": "总计",
  "dashboard.donut.file": "文件操作",
  "dashboard.donut.shell": "Shell 命令",
  "dashboard.donut.git": "Git 操作",
  "dashboard.donut.secret": "密钥访问",
  "dashboard.donut.package": "包安装",
  "dashboard.health.title": "防护健康状态",
  "dashboard.health.engine": "强制执行引擎响应正常",
  "dashboard.health.policy": "策略规则已是最新",
  "dashboard.boundaries.manage": "管理",
  "dashboard.boundaries.tagEnforced": "已拦截",
  "dashboard.boundaries.tagObserve": "仅观察",
  "dashboard.boundaries.tagNa": "不支持",
  "dashboard.boundaries.active": "运行中",
  "dashboard.boundaries.add": "添加边界",
  "dashboard.review.viewAll": "查看全部审核",
  "dashboard.review.empty": "暂无待审核 — 所有动作已处理。",
  "dashboard.events.title": "最近事件",
  "dashboard.events.viewFull": "查看完整日志",
  "dashboard.events.fAll": "全部",
  "dashboard.events.fAllowed": "已放行",
  "dashboard.events.fAsked": "已询问",
  "dashboard.events.fBlocked": "已阻止",
  "dashboard.events.fSystem": "系统",
  "dashboard.events.empty": "暂无事件。开启防护会话后，AI 动作事件会显示在这里。",
  "dashboard.events.sourceShell": "受保护 Shell (bash)",
  "nav.group.main": "主要",
  "nav.group.configure": "配置",
  "page.back": "返回首页",
  "crumb.dashboard": "首页",
  "crumb.activity": "活动",
  "crumb.review": "审查",
  "crumb.policies": "策略",
  "crumb.boundaries": "边界",
  "crumb.settings": "设置",
  "page.dashboard.desc": "防护状态、最近活动与快捷操作，一目了然。",
  "page.activity.desc": "这台电脑上所有 AI 会话与动作的历史记录。",
  "page.review.desc": "批准或拒绝等待你做决定的 AI 动作。",
  "page.policies.desc": "定义哪些动作被放行、询问或阻止的规则。",
  "page.boundaries.desc": "ActionGuard 可观察或强制执行保护的执行路径。",
  "page.settings.desc": "语言、主题与其他应用偏好设置。",
  "policy.dashboardLink": "← 返回首页",
  "boundaries.dashboardLink": "← 返回首页",

  /* ---------- 策略 Policies 全页 UI ---------- */
  "policies.tabs.rules": "规则",
  "policies.tabs.trustZones": "信任区",
  "policies.tabs.riskLevels": "风险级别",
  "policies.tabs.policySets": "策略集",
  "policies.newRule": "+ 新建规则",
  "policies.rule.criticalPaths": "拦截关键系统路径",
  "policies.rule.criticalPaths.desc": "阻止访问系统关键目录",
  "policies.rule.sensitiveFiles": "保护敏感文件",
  "policies.rule.sensitiveFiles.desc": "阻止访问 .env 密钥、凭据文件",
  "policies.rule.gitSafety": "Git 安全规则",
  "policies.rule.gitSafety.desc": "阻止破坏性 Git 操作",
  "policies.rule.networkRestrictions": "网络限制",
  "policies.rule.networkRestrictions.desc": "控制对外网络访问",
  "policies.rule.packageSafety": "包安全",
  "policies.rule.packageSafety.desc": "校验包安装操作",

  /* ---------- 活动 Activity 全页 UI ---------- */
  "activity.tabs.all": "全部",
  "activity.tabs.allowed": "已放行",
  "activity.tabs.asked": "待询问",
  "activity.tabs.blocked": "已阻止",
  "activity.filter": "筛选",
  "activity.search": "搜索活动…",
  "activity.col.time": "时间",
  "activity.col.action": "活动",
  "activity.col.source": "来源",
  "activity.col.risk": "风险",
  "activity.col.result": "结果",
  "activity.col.menu": "",
  "activity.showing": "共 {total} 条，当前显示 {shown} 条",

  /* ---------- 审查 Review 全页 UI ---------- */
  "review.recentDecisions": "最近决定",
  "review.decision": "决定",
  "review.pending.count": "{count} 个待处理 {count, plural, one {请求} other {请求}}",
  "review.tabs.pending": "待处理",
  "review.tabs.history": "历史",
  "review.col.requestedBy": "请求方",
  "review.allowAlways": "始终允许",
  "review.allowAlways.disabled": "尚未支持",
  "review.clearAll": "全部清空",

  /* ---------- 设置 Settings 全页 UI ---------- */
  "settings.tabs.general": "常规",
  "settings.tabs.security": "安全",
  "settings.tabs.ui": "界面",
  "settings.tabs.uiDesc": "自定义 ActionGuard 的外观与体验。",
  "settings.tabs.disabledHint": "将在后续更新中添加",
  "settings.tabs.advanced": "高级",
  "settings.group.startup": "启动",
  "settings.group.startup.label": "开机自动启动 ActionGuard",
  "settings.group.notifications": "通知",
  "settings.group.notifications.label": "显示桌面通知",
  "settings.notif.all": "所有操作",
  "settings.notif.ask-only": "仅询问和阻止的操作",
  "settings.notif.none": "关闭",
  "settings.group.theme": "主题",
  "settings.group.language": "语言",
  "settings.group.data": "数据",
  "settings.data.export": "导出活动日志",
  "settings.data.exportDesc": "将您的操作历史记录下载为文件。",
  "settings.data.clear": "清除本地数据",
  "settings.data.clearDesc": "删除所有本地存储的操作历史与会话数据。",
  "settings.dropdown.system": "跟随系统",

  /* Security Settings */
  "settings.security.failClosed": "默认失败关闭",
  "settings.security.failClosedDesc": "当桥接不可达或策略无法加载时阻止操作。",
  "settings.security.approvalTimeout": "审批超时时间",
  "settings.security.approvalTimeoutDesc": "待审批操作被自动拒绝前的秒数。",
  "settings.security.sessionMode": "当前防护模式",
  "settings.security.sessionModeDesc": "活跃会话决定操作的处理方式。",
  "settings.security.protected": "防护中",
  "settings.security.observe": "观察",
  "settings.security.active": "已启用",
  "settings.security.sensitiveResources": "敏感资源保护",
  "settings.security.credentialProtection": "凭证与密钥检测",
  "settings.security.credentialProtectionDesc": "检测并标记访问密钥的尝试。",
  "settings.security.shellProtection": "Shell 命令防护",
  "settings.security.shellProtectionDesc": "拦截 Shell 命令进行审查。",
  "settings.security.gitProtection": "Git 操作防护",
  "settings.security.gitProtectionDesc": "监控 git push、commit 和配置变更。",

  /* Advanced Settings */
  "settings.advanced.diagnostics": "系统诊断",
  "settings.advanced.running": "运行中...",
  "settings.advanced.experimental": "实验性功能",
  "settings.advanced.shadowMode": "影子模式",
  "settings.advanced.shadowModeDesc": "审计所有操作而不进行任何阻止。",
  "settings.advanced.rawFacts": "原始事实查看器",
  "settings.advanced.rawFactsDesc": "实时查看内部操作事实。",

  /* 首页补充 */
  "dashboard.types.network": "网络访问",
  "dashboard.types.other": "其他",
  "dashboard.health.hint": "查看防护健康状态 →",
  /* 侧边栏标签 */
  "sidebar.nav.dashboard": "首页",
  "sidebar.nav.activity": "活动",
  "sidebar.nav.review": "审查",
  "sidebar.nav.policies": "策略",
  "sidebar.nav.boundaries": "边界",
  "sidebar.nav.settings": "设置",
  "sidebar.nav.main": "主要",
  "sidebar.nav.configure": "配置",
  "sidebar.mode.active": "防护已开启",
  "sidebar.mode.activeDesc": "AI 可以工作，控制权在你。",
  "sidebar.mode.recommended": "推荐",
  "sidebar.mode.modeDesc": "常规自动放行 · 高风险先问 · 致命直接阻止",
  "sidebar.mode.change": "更改模式",
  "sidebar.scope.label": "保护范围",
  "sidebar.scope.thisComputer": "这台电脑",
  "sidebar.scope.allBoundaries": "所有支持的边界",
  "sidebar.scope.change": "更改范围",
  "sidebar.local.title": "本地优先",
  "sidebar.local.noCloud": "无云端",
  "sidebar.local.noAccount": "无需账号",
  "sidebar.local.noTelemetry": "无遥测",
  "sidebar.local.localData": "所有数据留在本设备",
  "sidebar.footer.version": "v0.3",
  "sidebar.footer.checkUpdates": "检查更新",
  "header.feedback": "反馈",
  "header.protection.active": "防护已开启",
  "header.protection.inactive": "防护未开启",
  "header.nav.dashboard": "首页",
  /* 活动页面 */
  "activity.title": "活动日志",
  "activity.desc": "所有已记录的 AI 操作，按时间排序。",
  /* 审查页面 */
  "review.title": "审查队列",
  "review.desc": "批准或拒绝待处理的 AI 操作。",
  /* 边界页面 */
  "boundaries.title": "受保护边界",
  "boundaries.desc": "ActionGuard 可以观察或强制执行保护的执行路径。",
  "boundaries.class.title": "边界类别",
  "boundaries.class.desc": "ActionGuard 将执行路径分为六个边界类别（A–F）。核心引擎与品牌无关。",
  "boundaries.col.class": "类别",
  "boundaries.col.name": "名称",
  "boundaries.col.description": "描述",
  "boundaries.col.enforce": "强制拦截",
  "boundaries.col.obs": "仅观察",
  "boundaries.col.note": "说明",
  "boundaries.col.actions": "活跃动作",
  "boundaries.active": "已激活",
  "boundaries.class.A": "工具钩子",
  "boundaries.class.A.desc": "自动化内部的执行前钩子",
  "boundaries.class.B": "执行审批",
  "boundaries.class.B.desc": "自动化自身的执行策略",
  "boundaries.class.C": "受保护 Shell",
  "boundaries.class.C.desc": "执行前钩子（PowerShell PSReadLine）— 仅在活跃会话期间",
  "boundaries.class.D": "运行时沙箱",
  "boundaries.class.D.desc": "未来 L3 强制执行",
  "boundaries.class.E": "系统强制执行",
  "boundaries.class.E.desc": "未来 L4 强制执行",
  "boundaries.class.F": "远程",
  "boundaries.class.F.desc": "动作从不落在这台机器上 — 超出范围",
  "boundaries.active.yes": "是",
  "boundaries.active.no": "否",
  "boundaries.note.A": "与品牌无关，通过注册表添加",
  "boundaries.note.B": "与品牌无关，通过注册表添加",
  "boundaries.note.C": "仅在活跃防护会话期间强制执行",
  "boundaries.note.D": "",
  "boundaries.note.E": "",
  "boundaries.note.F": "超出范围",
  "boundaries.add": "添加边界",
  "boundaries.current.title": "当前执行路径",
  "boundaries.current.empty": "暂无活跃执行路径。开启防护会话后，这里会显示被监控的路径。",
  "boundaries.current.path": "路径",
  "boundaries.current.status": "状态",
  "boundaries.current.enforced": "已拦截",
  "boundaries.current.obs": "仅观察",
  "boundaries.current.unsupported": "不支持",
  "boundaries.current.active": "活跃",
  /* 空状态 */
  "empty.noActivity.k": "暂无活动",
  "empty.noActivity.v": "开启防护会话后，AI 操作会实时出现在这里。",
  "empty.noPending.k": "暂无待审核",
  "empty.noPending.v": "所有操作已处理完毕。让 AI 保持安全，你做得很好。",
  "empty.noBoundaries.k": "暂无活跃边界",
  "empty.noBoundaries.v": "开启防护会话后，开始监控执行路径。",
  /* 信任区标签 */
  "policies.rule.trustZone": "信任区",
  "policies.rule.trustZone.desc": "定义 AI 可以自由操作的受信任目录",
  "policies.rule.networkFuture": "网络限制",
  "policies.rule.networkFuture.desc": "控制对外网络访问 — v0.3 推出",
  /* 信任区 */
  "policies.trust.title": "受信任目录",
  "policies.trust.desc": "定义 AI 可以直接读写而不弹出确认的目录。注意——这里的文件完全可访问。",
  "policies.trust.addDir": "添加目录",
  "policies.trust.placeholder": "输入路径（例如 C:\\Projects\\my-app）",
  "policies.trust.empty": "暂无信任目录",
  "policies.trust.emptyDesc": "不在信任目录中的 AI 操作将被审查。",
  "policies.trust.remove": "移除",
  "policies.trust.systemDirs": "系统目录（C:\\、Windows、Program Files）不能添加为信任区。",
  /* 风险级别 */
  "policies.risk.title": "风险级别",
  "policies.risk.desc": "选择 ActionGuard 对风险操作的分类和拦截力度。",
  "policies.risk.strict": "严格",
  "policies.risk.strictDesc": "低风险操作也需要审批。最高保护级别，适合敏感环境。",
  "policies.risk.standard": "标准",
  "policies.risk.standardDesc": "常规操作自动放行，中高风险操作需要审批。",
  "policies.risk.relaxed": "宽松",
  "policies.risk.relaxedDesc": "仅致命和高风险操作需要审批。",
  "policies.risk.current": "当前",
  "policies.risk.apply": "应用",
  "policies.risk.applied": "已激活",
  /* 策略集 */
  "policies.sets.title": "策略集",
  "policies.sets.desc": "针对常见场景的预配置策略包。激活一个策略集即可一次性应用所有规则。",
  "policies.sets.development": "开发环境",
  "policies.sets.developmentDesc": "Git 推送、包安装和文件写入需审批。网络请求需审查。",
  "policies.sets.production": "生产环境",
  "policies.sets.productionDesc": "所有写入操作、Git 推送和网络访问均需审批。严格的文件路径保护。",
  "policies.sets.custom": "自定义",
  "policies.sets.customDesc": "使用上方规则标签页自定义规则。",
  "policies.sets.active": "已激活",
  "policies.sets.activate": "激活",
  "policies.sets.rulesCount": "{{n}} 条规则",
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
