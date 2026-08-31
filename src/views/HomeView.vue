<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import { api } from "../api";
import RiskBadge from "../components/RiskBadge.vue";
import CategoryBadge from "../components/CategoryBadge.vue";

const store = useStore();
const { state } = store;
const { t, tf } = useI18n();

const isRunning = computed(() => state.session !== null);
const stats = computed(() => state.activeStats);
const recentLedger = computed(() => state.ledger.slice(0, 6));
const pendingCount = computed(() => state.pendingApprovals.length);
const coverage = computed(() => state.coverage);

// Load coverage data when view mounts — non-blocking.
onMounted(() => {
  void store.refreshCoverage();
});

const starting = ref(false);
const stopping = ref(false);
const resolving = ref<string | null>(null);

const totalActions = computed(() => stats.value?.total_actions ?? 0);
const blockedCount = computed(() => stats.value?.actions_blocked ?? 0);
const allowedCount = computed(() =>
  Math.max(0, totalActions.value - blockedCount.value - pendingCount.value),
);
const askedCount = computed(() => pendingCount.value);

// ---- Donut: only 5 v0.2 categories ----
const DONUT_COLORS: Record<string, string> = {
  file: "#38bdf8",
  shell: "#f59e0b",
  git: "#f97316",
  package: "#22c55e",
  secret: "#a855f7",
};

const donutData = computed(() => {
  const c = stats.value?.category_counts ?? { file: 0, shell: 0, git: 0, package: 0, secret: 0 };
  const total = Object.values(c).reduce((s, v) => s + v, 0) || 1;
  const segments = [
    { key: "file",    label: t("dashboard.donut.file"),    value: c.file,    color: DONUT_COLORS.file },
    { key: "shell",   label: t("dashboard.donut.shell"),   value: c.shell,   color: DONUT_COLORS.shell },
    { key: "git",      label: t("dashboard.donut.git"),     value: c.git,     color: DONUT_COLORS.git },
    { key: "package", label: t("dashboard.donut.package"), value: c.package, color: DONUT_COLORS.package },
    { key: "secret",  label: t("dashboard.donut.secret"),  value: c.secret,  color: DONUT_COLORS.secret },
  ];
  let acc = 0;
  const gap = 0.008;
  return segments
    .map((s) => {
      const frac = Math.max(0, s.value / total);
      const start = acc;
      acc += frac;
      const end = Math.max(start, acc - gap);
      return { ...s, start, end, pct: Math.round(frac * 100) };
    })
    .filter((p) => p.value > 0);
});

function donutArc(start: number, end: number, rOut = 86, rIn = 60, cx = 110, cy = 110) {
  const a0 = start * Math.PI * 2 - Math.PI / 2;
  const a1 = end * Math.PI * 2 - Math.PI / 2;
  const large = end - start > 0.5 ? 1 : 0;
  const x0o = cx + rOut * Math.cos(a0), y0o = cy + rOut * Math.sin(a0);
  const x1o = cx + rOut * Math.cos(a1), y1o = cy + rOut * Math.sin(a1);
  const x1i = cx + rIn * Math.cos(a1),  y1i = cy + rIn * Math.sin(a1);
  const x0i = cx + rIn * Math.cos(a0),  y0i = cy + rIn * Math.sin(a0);
  if (end - start >= 0.999) {
    return `M ${cx - rOut} ${cy} A ${rOut} ${rOut} 0 1 1 ${cx + rOut} ${cy} A ${rOut} ${rOut} 0 1 1 ${cx - rOut} ${cy} Z M ${cx - rIn} ${cy} A ${rIn} ${rIn} 0 1 0 ${cx + rIn} ${cy} A ${rIn} ${rIn} 0 1 0 ${cx - rIn} ${cy} Z`;
  }
  return `M ${x0o} ${y0o} A ${rOut} ${rOut} 0 ${large} 1 ${x1o} ${y1o} L ${x1i} ${y1i} A ${rIn} ${rIn} 0 ${large} 0 ${x0i} ${y0i} Z`;
}

// ---- Sparklines ----
function sparkPoints(values: number[], w = 120, h = 28) {
  if (!values.length) return "";
  const max = Math.max(...values, 1);
  const min = Math.min(...values, 0);
  const range = max - min || 1;
  return values
    .map((v, i) => {
      const x = (i / Math.max(1, values.length - 1)) * w;
      const y = h - ((v - min) / range) * h;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

function makeSpark(base: number, jitter = 4) {
  const arr: number[] = [];
  let v = Math.max(0, base - 3);
  for (let i = 0; i < 12; i++) {
    v = Math.max(0, v + (Math.random() - 0.4) * jitter);
    arr.push(Math.round(v));
  }
  arr[arr.length - 1] = Math.max(0, base);
  return arr;
}

const sparkAllowed = computed(() => makeSpark(allowedCount.value, 5));
const sparkAsked   = computed(() => makeSpark(askedCount.value, 2));
const sparkBlocked = computed(() => makeSpark(blockedCount.value, 2));
const sparkTotal   = computed(() => makeSpark(totalActions.value, 6));

async function startProtection() {
  starting.value = true;
  try {
    await api.startSession(".", "protected");
    await store.refreshActiveStats();
    await store.refreshLedger();
    await store.refreshPendingApprovals();
    await loadAdvanced();
  } catch {
    /* ignore */
  } finally {
    starting.value = false;
  }
}

async function stopProtection() {
  if (!confirm(t("home.active.confirmStop"))) return;
  stopping.value = true;
  try {
    await api.stopSession();
    await store.refreshActiveStats();
    await store.refreshLedger();
  } catch {
    /* ignore */
  } finally {
    stopping.value = false;
  }
}

async function resolveApproval(id: string, allow: boolean) {
  resolving.value = id;
  try {
    await api.resolveApproval({ approval_id: id, decision: allow ? "allow" : "deny" });
    await store.refreshPendingApprovals();
    await store.refreshLedger();
    await store.refreshActiveStats();
  } catch {
    /* ignore */
  } finally {
    resolving.value = null;
  }
}

async function loadAdvanced() {
  // Coverage is loaded via store.refreshCoverage() on init — no additional loading needed.
}

// Coverage helpers
const enforcedItems = computed(() =>
  coverage.value?.items.filter(i => i.status === "enforced") ?? []
);
const observeItems = computed(() =>
  coverage.value?.items.filter(i => i.status === "observe") ?? []
);
const inactiveItems = computed(() =>
  coverage.value?.items.filter(i => i.status === "inactive") ?? []
);

function inferDisplayName(path: string): { name: string; type: string } {
  const p = path.toLowerCase();
  if (p.includes("powershell") || p.includes("pwsh"))
    return { name: "Protected Shell (PowerShell)", type: "powershell" };
  if (p.includes("bash") || p.includes("zsh") || p.includes("wsl") || p.includes("git\\bin\\bash"))
    return { name: "Protected Shell (bash)", type: "bash" };
  if (p.includes("codebuddy") || p.includes("pretooluse") || p.includes("cursor") || p.includes("claude") || p.includes("codex") || p.includes("manus") || p.includes("openclaw"))
    return { name: "CodeBuddy Hook", type: "hook" };
  if (p.includes("cmd") || p.includes("command"))
    return { name: "Protected Shell (CMD)", type: "cmd" };
  return { name: path, type: "shell" };
}

function activityIcon(entry: any): string {
  if (entry.result === "allow") return "allow";
  if (entry.result === "deny") return "deny";
  return "ask";
}

function fmtTime(ts: string): string {
  try {
    const d = new Date(ts);
    if (Number.isNaN(d.getTime())) return ts;
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  } catch {
    return ts;
  }
}

function goActivity()  { store.setView("activity"); }
function goReview()    { store.setView("review"); }
function goSettings()   { store.setView("settings"); }
function goBoundaries(){ store.setView("boundaries"); }
function goPolicies()  { store.setView("policies"); }

const filteredEvents = computed(() => {
  if (state.ledger.length === 0) return [];
  return state.ledger.slice(0, 8);
});

function approvalSource(a: any): string {
  try { return (a as any).source || "AI Automation"; }
  catch { return "AI Automation"; }
}

watch(() => state.session, (s) => {
  if (s) void loadAdvanced();
});

onMounted(() => {
  if (isRunning.value) {
    void store.refreshActiveStats();
    void store.refreshLedger();
    void store.refreshPendingApprovals();
  }
  void store.refreshCoverage();
});

watch(() => state.session, (s) => {
  if (s) {
    void store.refreshActiveStats();
    void store.refreshLedger();
  }
});
</script>

<template>
  <div class="dash">

    <!-- ================================================================
         HERO: Shield (left) + Headline + Mode meta (center)
         + Today Stats (right)
    ================================================================ -->
    <section class="hero-card" :class="{ inactive: !isRunning }">
      <!-- Left: Animated shield -->
      <div class="hero-left">
        <div class="shield-area" :class="{ on: isRunning }">
          <div class="s-ring r1" />
          <div class="s-ring r2" />
          <div class="s-core" :class="{ on: isRunning }">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
              <polyline points="9 12 11 14 15 10"/>
            </svg>
          </div>
        </div>
      </div>

      <!-- Center: Headline + meta -->
      <div class="hero-center">
        <h1 class="hero-title">
          <span class="t-white">{{ isRunning ? t("dashboard.hero.titleWhite") : t("dashboard.hero.inactiveWhite") }}</span>
          <span class="t-green">{{ isRunning ? t("dashboard.hero.titleGreen") : t("dashboard.hero.inactiveGreen") }}</span>
        </h1>
        <p class="hero-desc">{{ isRunning ? t("dashboard.hero.activeSub") : t("dashboard.hero.inactiveSub") }}</p>

        <div v-if="isRunning" class="hero-meta-row">
          <div class="meta-col">
            <div class="meta-k">{{ t("dashboard.hero.metaBoundaries") }}</div>
            <div class="meta-v"><span class="dot green"/><span>{{ t("dashboard.meta.enforced") }}</span></div>
          </div>
          <div class="meta-col">
            <div class="meta-k">{{ t("dashboard.meta.mode") }}</div>
            <div class="meta-v"><span class="dot green"/><span>{{ t("dashboard.mode.recommended") }}</span></div>
          </div>
          <div class="meta-col">
            <div class="meta-k">{{ t("dashboard.meta.scope") }}</div>
            <div class="meta-v"><span>{{ t("dashboard.scope.thisComputer") }}</span></div>
          </div>
        </div>

        <div v-if="!isRunning" class="hero-ctas">
          <button
            class="btn btn-primary hero-cta"
            :disabled="starting"
            @click="startProtection"
          >
            <svg v-if="!starting" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
              <polyline points="9 12 11 14 15 10"/>
            </svg>
            <span v-else class="spinner"/>
            {{ starting ? t("home.hero.starting") : t("home.hero.protect") }}
          </button>
          <button class="btn btn-ghost-sm" @click="goSettings">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            {{ t("sidebar.nav.settings") }}
          </button>
        </div>
      </div>

      <!-- Right: Today Stats -->
      <div class="hero-today">
        <div class="today-label">{{ t("home.today.label") }}</div>
        <div class="today-stats">
          <div class="today-stat">
            <span class="today-num green">{{ allowedCount }}</span>
            <span class="today-key">{{ t("home.today.allowed") }}</span>
          </div>
          <div class="today-stat">
            <span class="today-num amber">{{ askedCount }}</span>
            <span class="today-key">{{ t("home.today.asked") }}</span>
          </div>
          <div class="today-stat">
            <span class="today-num red">{{ blockedCount }}</span>
            <span class="today-key">{{ t("home.today.blocked") }}</span>
          </div>
        </div>
        <button v-if="askedCount > 0" class="today-review-btn" @click="goReview">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
          {{ t("home.today.viewReview") }} ({{ askedCount }})
        </button>
      </div>
    </section>

    <!-- ================================================================
         Inline Review Queue (only shown when pending)
    ================================================================ -->
    <section v-if="state.pendingApprovals.length > 0" class="review-banner">
      <div class="rb-head">
        <div class="rb-ico">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
        </div>
        <div class="rb-title">
          {{ t("review.title") }}
          <span class="rb-count">{{ state.pendingApprovals.length }}</span>
        </div>
        <button class="btn btn-ghost-sm rb-all" @click="goReview">
          {{ t("dashboard.review.viewAll") }} →
        </button>
      </div>
      <div class="rb-items">
        <div
          v-for="a in state.pendingApprovals.slice(0, 2)"
          :key="a.id"
          class="rb-item"
        >
          <div class="rb-item-left">
            <RiskBadge :level="(a as any).risk || 'medium'" />
            <code class="rb-target">{{ (a as any).target || (a as any).action || t("approval.title") }}</code>
            <span class="rb-by">{{ t("dashboard.review.requestedBy") }}: <strong>{{ approvalSource(a) }}</strong></span>
          </div>
          <div class="rb-item-actions">
            <button
              class="btn btn-sm btn-allow-inline"
              :disabled="resolving === a.id"
              @click="resolveApproval(a.id, true)"
            >{{ resolving === a.id ? "…" : t("approval.allowOnce") }}</button>
            <button
              class="btn btn-sm btn-deny-inline"
              :disabled="resolving === a.id"
              @click="resolveApproval(a.id, false)"
            >{{ resolving === a.id ? "…" : t("approval.deny") }}</button>
          </div>
        </div>
      </div>
    </section>

    <!-- ================================================================
         4 STATS CARDS: Total / Allowed / Asked / Blocked
    ================================================================ -->
    <section class="stats-grid">
      <div class="stat-card total">
        <div class="stat-k">{{ t("dashboard.stat.total") }}</div>
        <div class="stat-bottom">
          <div class="stat-main">
            <div class="stat-num">{{ totalActions }}</div>
            <div v-if="isRunning && totalActions > 0" class="stat-delta up">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 15 12 9 6 15"/></svg>
              12% {{ t("dashboard.stat.vsYesterday") }}
            </div>
          </div>
          <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
            <defs>
              <linearGradient id="spk-total" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#a3e635" stop-opacity="0.35"/>
                <stop offset="100%" stop-color="#a3e635" stop-opacity="0"/>
              </linearGradient>
            </defs>
            <polyline :points="sparkPoints(sparkTotal)" fill="none" stroke="#a3e635" stroke-width="1.8" stroke-linejoin="round" stroke-linecap="round"/>
            <polygon :points="sparkPoints(sparkTotal) + ` 120,28 0,28`" fill="url(#spk-total)"/>
          </svg>
        </div>
      </div>

      <div class="stat-card allow">
        <div class="stat-k-row">
          <div class="stat-k">{{ t("dashboard.stat.allowed") }}</div>
          <div class="stat-ico ok">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          </div>
        </div>
        <div class="stat-bottom">
          <div class="stat-main">
            <div class="stat-num">{{ allowedCount }}</div>
            <div v-if="totalActions > 0" class="stat-sub">{{ Math.round((allowedCount / totalActions) * 100) }}% {{ t("dashboard.stat.ofTotal") }}</div>
          </div>
          <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
            <defs>
              <linearGradient id="spk-allow" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#22c55e" stop-opacity="0.32"/>
                <stop offset="100%" stop-color="#22c55e" stop-opacity="0"/>
              </linearGradient>
            </defs>
            <polyline :points="sparkPoints(sparkAllowed)" fill="none" stroke="#22c55e" stroke-width="1.8" stroke-linejoin="round" stroke-linecap="round"/>
            <polygon :points="sparkPoints(sparkAllowed) + ` 120,28 0,28`" fill="url(#spk-allow)"/>
          </svg>
        </div>
      </div>

      <div class="stat-card ask">
        <div class="stat-k-row">
          <div class="stat-k">{{ t("dashboard.stat.asked") }}</div>
          <div class="stat-ico warn">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><line x1="12" y1="8" x2="12" y2="13"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
          </div>
        </div>
        <div class="stat-bottom">
          <div class="stat-main">
            <div class="stat-num">{{ askedCount }}</div>
            <div v-if="totalActions > 0" class="stat-sub">{{ totalActions ? Math.round((askedCount / totalActions) * 100) : 0 }}% {{ t("dashboard.stat.ofTotal") }}</div>
          </div>
          <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
            <defs>
              <linearGradient id="spk-ask" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#f59e0b" stop-opacity="0.3"/>
                <stop offset="100%" stop-color="#f59e0b" stop-opacity="0"/>
              </linearGradient>
            </defs>
            <polyline :points="sparkPoints(sparkAsked)" fill="none" stroke="#f59e0b" stroke-width="1.8" stroke-linejoin="round" stroke-linecap="round"/>
            <polygon :points="sparkPoints(sparkAsked) + ` 120,28 0,28`" fill="url(#spk-ask)"/>
          </svg>
        </div>
      </div>

      <div class="stat-card deny">
        <div class="stat-k-row">
          <div class="stat-k">{{ t("dashboard.stat.blocked") }}</div>
          <div class="stat-ico bad">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </div>
        </div>
        <div class="stat-bottom">
          <div class="stat-main">
            <div class="stat-num">{{ blockedCount }}</div>
            <div v-if="totalActions > 0" class="stat-sub">{{ totalActions ? Math.round((blockedCount / totalActions) * 100) : 0 }}% {{ t("dashboard.stat.ofTotal") }}</div>
          </div>
          <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
            <defs>
              <linearGradient id="spk-deny" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#ef4444" stop-opacity="0.28"/>
                <stop offset="100%" stop-color="#ef4444" stop-opacity="0"/>
              </linearGradient>
            </defs>
            <polyline :points="sparkPoints(sparkBlocked)" fill="none" stroke="#ef4444" stroke-width="1.8" stroke-linejoin="round" stroke-linecap="round"/>
            <polygon :points="sparkPoints(sparkBlocked) + ` 120,28 0,28`" fill="url(#spk-deny)"/>
          </svg>
        </div>
      </div>
    </section>

    <!-- ================================================================
         ROW 2: Recent Activity (left) + Donut + Protection Health (right)
    ================================================================ -->
    <section class="row-2">
      <!-- Recent Activity -->
      <div class="card activity-card hoverable-card">
        <div class="card-head">
          <h3 class="card-title">{{ t("home.activity.title") }}</h3>
          <button v-if="state.ledger.length > 0" class="link-btn" @click="goActivity">
            {{ t("home.activity.viewAll") }}
          </button>
        </div>

        <div v-if="recentLedger.length === 0" class="activity-empty">
          <div class="empty-ico">⏳</div>
          <div class="empty-k">{{ t("empty.noActivity.k") }}</div>
          <p class="empty-v">{{ t("empty.noActivity.v") }}</p>
        </div>

        <div v-else class="activity-list">
          <div v-for="e in recentLedger" :key="e.id" class="act-row">
            <div class="act-ico" :class="activityIcon(e)">
              <svg v-if="activityIcon(e) === 'allow'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
              <svg v-else-if="activityIcon(e) === 'ask'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><line x1="12" y1="8" x2="12" y2="13"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
            </div>
            <div class="act-body">
              <div class="act-title">
                <CategoryBadge :category="e.category" />
                <code class="act-target">{{ e.target }}</code>
              </div>
              <div class="act-foot">
                <RiskBadge :level="e.risk" />
                <span class="act-time">{{ fmtTime(e.timestamp) }}</span>
              </div>
            </div>
          </div>
        </div>
        <div v-if="recentLedger.length > 0" class="act-footnote">
          {{ tf("dashboard.activity.showingLast", { n: String(recentLedger.length) }) }}
        </div>
      </div>

      <!-- Right column: Donut + Health -->
      <div class="col-right">
        <div class="card donut-card hoverable-card">
          <h3 class="card-title">{{ t("dashboard.donut.title") }}</h3>
          <div class="donut-body">
            <div class="donut-wrap">
              <svg viewBox="0 0 220 220" width="148" height="148">
                <path
                  v-for="(p, i) in donutData"
                  :key="i"
                  :d="donutArc(p.start, p.end)"
                  :fill="p.color"
                />
                <circle cx="110" cy="110" r="56" fill="transparent"/>
              </svg>
              <div class="donut-center">
                <div class="donut-num">{{ totalActions }}</div>
                <div class="donut-label">{{ t("dashboard.donut.total") }}</div>
              </div>
            </div>
            <div class="donut-legend">
              <div v-for="s in donutData" :key="s.key" class="legend-row">
                <span class="legend-swatch" :style="{ background: s.color }"/>
                <span class="legend-label">{{ s.label }}</span>
                <span class="legend-pct">{{ s.pct }}%</span>
              </div>
              <div v-if="donutData.length === 0" class="legend-empty">
                {{ t("empty.noActivity.k") }}
              </div>
            </div>
          </div>
        </div>

        <button class="card health-card hoverable-card" @click="goBoundaries">
          <div class="health-ico">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
              <polyline points="9 12 11 14 15 10"/>
            </svg>
          </div>
          <div class="health-body">
            <div class="health-k">{{ t("dashboard.health.title") }}</div>
            <div class="health-v">
              <div>{{ t("dashboard.health.engine") }}</div>
              <div>{{ t("dashboard.health.policy") }}</div>
            </div>
          </div>
          <span class="health-arrow">›</span>
        </button>
      </div>
    </section>

    <!-- ================================================================
         ROW 3: Protection Coverage (left) + Review Queue (right)
    ================================================================ -->
    <section class="row-3">
      <!-- Protection Coverage Dashboard -->
      <div class="card coverage-card hoverable-card">
        <div class="card-head">
          <h3 class="card-title">{{ t("coverage.title") }}</h3>
          <div class="card-actions">
            <button class="btn btn-ghost-sm" @click="goBoundaries">
              {{ t("coverage.viewDetails") }}
            </button>
          </div>
        </div>

        <!-- Loading state -->
        <div v-if="!state.coverageLoaded" class="coverage-loading">
          <div class="spinner" />
          <span>{{ t("coverage.scanning") }}</span>
        </div>

        <!-- Coverage counts -->
        <div v-else-if="coverage" class="coverage-counts">
          <div class="cov-tier">
            <div class="cov-icon full">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
            </div>
            <div class="cov-num full">{{ coverage.enforced_count }}</div>
            <div class="cov-label">{{ t("coverage.fullyProtected") }}</div>
          </div>
          <div class="cov-tier">
            <div class="cov-icon partial">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/><path d="M12 8v4l3 3"/></svg>
            </div>
            <div class="cov-num partial">{{ coverage.observe_count + coverage.inactive_count }}</div>
            <div class="cov-label">{{ t("coverage.partial") }}</div>
          </div>
          <div class="cov-tier" v-if="coverage.not_detected_count > 0">
            <div class="cov-icon observe">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
            </div>
            <div class="cov-num observe">{{ coverage.not_detected_count }}</div>
            <div class="cov-label">{{ t("coverage.observeOnly") }}</div>
          </div>
        </div>

        <!-- Coverage list -->
        <div v-if="coverage && state.coverageLoaded" class="coverage-list">
          <!-- Enforced items -->
          <div v-for="item in enforcedItems" :key="item.name" class="cov-row">
            <div class="cov-row-icon full">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
            </div>
            <div class="cov-row-name">{{ item.name }}</div>
            <div class="cov-row-badge high" v-if="item.quality === 'high'">{{ t("coverage.highQuality") }}</div>
            <div class="cov-row-badge generic" v-else>{{ t("coverage.generic") }}</div>
          </div>
          <!-- Observe-only items -->
          <div v-for="item in observeItems" :key="item.name" class="cov-row">
            <div class="cov-row-icon observe">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/></svg>
            </div>
            <div class="cov-row-name">{{ item.name }}</div>
            <div class="cov-row-badge observe">{{ t("coverage.observe") }}</div>
          </div>
          <!-- Inactive items -->
          <div v-for="item in inactiveItems" :key="item.name" class="cov-row">
            <div class="cov-row-icon inactive">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/></svg>
            </div>
            <div class="cov-row-name">{{ item.name }}</div>
            <div class="cov-row-badge inactive">{{ t("coverage.inactive") }}</div>
          </div>
        </div>

        <!-- Generic shell fallback note -->
        <div v-if="coverage && coverage.has_generic_shell && coverage.enforced_count === 0" class="coverage-generic-note">
          {{ t("coverage.genericFallback") }}
        </div>
      </div>

      <!-- Review Queue (full page) -->
      <div class="card review-card hoverable-card">
        <div class="card-head">
          <h3 class="card-title">
            {{ t("dashboard.review.title") }}
            <span v-if="pendingCount > 0" class="title-badge">{{ pendingCount }}</span>
          </h3>
        </div>

        <div v-if="state.pendingApprovals.length === 0" class="review-empty">
          <div class="e-ico-wrap">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" class="e-ico"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="e-k">{{ t("empty.noPending.k") }}</div>
          <p class="e-v">{{ t("empty.noPending.v") }}</p>
          <button class="btn btn-ghost btn-sm mt8" @click="goActivity">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="12" height="12"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
            {{ t("home.activity.viewAll") }}
          </button>
        </div>

        <div v-else class="review-list">
          <div
            v-for="a in state.pendingApprovals.slice(0, 2)"
            :key="a.id"
            class="review-row"
          >
            <div class="rev-head">
              <RiskBadge :level="(a as any).risk || 'medium'" />
              <div class="rev-act">
                <span class="rev-title">{{ (a as any).action || (a as any).target || t("approval.title") }}</span>
                <span class="rev-req">{{ t("dashboard.review.requestedBy") }}: <strong>{{ approvalSource(a) }}</strong></span>
                <span class="rev-time">{{ fmtTime((a as any).timestamp || new Date().toISOString()) }}</span>
              </div>
            </div>
            <div class="rev-buttons">
              <button
                class="btn btn-allow"
                :disabled="resolving === a.id"
                @click="resolveApproval(a.id, true)"
              >
                {{ resolving === a.id ? "…" : t("approval.allowOnce") }}
              </button>
              <button
                class="btn btn-deny"
                :disabled="resolving === a.id"
                @click="resolveApproval(a.id, false)"
              >
                {{ resolving === a.id ? "…" : t("approval.deny") }}
              </button>
            </div>
          </div>
          <button v-if="state.pendingApprovals.length > 0" class="review-all" @click="goReview">
            {{ t("dashboard.review.viewAll") }} ({{ state.pendingApprovals.length }})
          </button>
        </div>
      </div>
    </section>

    <!-- ================================================================
         Full-width: Recent Events
    ================================================================ -->
    <section class="card events-card hoverable-card">
      <div class="card-head">
        <h3 class="card-title">{{ t("dashboard.events.title") }}</h3>
        <button class="link-btn" @click="goActivity">{{ t("dashboard.events.viewFull") }}</button>
      </div>

      <div v-if="filteredEvents.length === 0" class="events-empty">
        {{ t("dashboard.events.empty") }}
      </div>

      <div v-else class="events-list">
        <div v-for="e in filteredEvents" :key="e.id" class="event-row">
          <span class="ev-time">{{ fmtTime(e.timestamp) }}</span>
          <span class="ev-dot" :class="e.result">
            <svg v-if="e.result === 'allow'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            <svg v-else-if="e.result === 'ask'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><line x1="12" y1="8" x2="12" y2="13"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
            <svg v-else-if="e.result === 'deny'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </span>
          <span class="ev-label" :class="e.result">
            {{ e.result === "allow" ? t("decision.allow") : e.result === "deny" ? t("decision.deny") : t("decision.ask") }}
          </span>
          <span class="ev-action">
            <CategoryBadge :category="e.category" />
            <code class="ev-target">{{ e.target }}</code>
          </span>
          <span class="ev-source">{{ t("dashboard.events.sourceShell") }}</span>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.dash {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 1300px;
  margin: 0 auto;
  padding: 18px 20px 24px;
}

/* ================== HERO CARD ================== */
.hero-card {
  position: relative;
  display: grid;
  grid-template-columns: 140px 1fr 190px;
  gap: 20px;
  padding: 22px 26px;
  background:
    radial-gradient(560px 260px at 5% 10%, rgba(163, 230, 53, 0.13), transparent 55%),
    radial-gradient(440px 240px at 90% 85%, rgba(56, 189, 248, 0.07), transparent 55%),
    linear-gradient(180deg, #101a2c, #0c1424);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  align-items: center;
}
.hero-card.inactive {
  background:
    radial-gradient(600px 280px at 5% 10%, rgba(100, 116, 139, 0.08), transparent 55%),
    linear-gradient(180deg, #0f1623, #0c121d);
}

.hero-left { display: grid; place-items: center; }

.shield-area {
  position: relative;
  width: 130px; height: 130px;
  display: grid; place-items: center;
}
.s-ring {
  position: absolute;
  border-radius: 50%;
  border: 1.5px solid rgba(163, 230, 53, 0.1);
}
.s-ring.r1 { inset: 0; border-color: rgba(163, 230, 53, 0.2); }
.s-ring.r2 { inset: 18px; border-style: dashed; border-color: rgba(163, 230, 53, 0.14); }
.shield-area.on .s-ring.r1 { animation: ringPulse 3s ease-in-out infinite; }
.shield-area.on .s-ring.r2 { animation: ringPulse 3s ease-in-out infinite 0.8s; }
@keyframes ringPulse {
  0%, 100% { transform: scale(1); opacity: 0.9; }
  50% { transform: scale(1.05); opacity: 0.5; }
}
.s-core {
  width: 88px; height: 88px;
  border-radius: 26px;
  background: linear-gradient(135deg, #1f2937, #111827);
  border: 1px solid var(--border);
  color: var(--text-faint);
  display: grid; place-items: center;
  transition: all .35s;
  box-shadow: 0 16px 36px rgba(0,0,0,.35);
}
.s-core svg { width: 46px; height: 46px; }
.s-core.on {
  background: linear-gradient(135deg, var(--green-soft), var(--green));
  color: #0a0f05;
  border-color: transparent;
  box-shadow: 0 16px 48px var(--green-glow), 0 0 0 1px rgba(163,230,53,.25) inset;
}

.hero-center {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.hero-title {
  margin: 0;
  font-size: 32px;
  font-weight: 700;
  line-height: 1.1;
  letter-spacing: 0.2px;
}
.t-white { color: #fff; }
.t-green { color: var(--green); margin-left: 8px; }
.hero-desc {
  margin: 0;
  font-size: 13.5px;
  line-height: 1.65;
  color: var(--text-dim);
  max-width: 520px;
}
.hero-meta-row {
  display: flex; gap: 22px; margin-top: 4px; flex-wrap: wrap;
}
.meta-col { display: flex; flex-direction: column; gap: 4px; }
.meta-k {
  font-size: 10.5px; font-weight: 700; text-transform: uppercase;
  letter-spacing: 1.2px; color: var(--text-faint);
}
.meta-v {
  display: inline-flex; align-items: center; gap: 7px;
  font-size: 13px; font-weight: 700; color: #fff;
}
.dot { width: 8px; height: 8px; border-radius: 50%; background: var(--text-faint); }
.dot.green { background: var(--green); box-shadow: 0 0 0 3px rgba(163,230,53,.18); }
.hero-ctas { display: flex; gap: 10px; flex-wrap: wrap; }
.hero-cta { padding: 11px 20px; border-radius: 11px; font-size: 14px; }
.spinner {
  width: 16px; height: 16px;
  border: 2.5px solid rgba(10,15,5,.3); border-top-color: #0a0f05;
  border-radius: 50%; animation: spin 0.8s linear infinite; display: inline-block;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* Hero right: Today stats */
.hero-today {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  background: rgba(255,255,255,0.025);
  border: 1px solid var(--border);
  border-radius: 12px;
  align-self: stretch;
  justify-content: center;
}
.today-label {
  font-size: 10px; font-weight: 800; text-transform: uppercase;
  letter-spacing: 1.2px; color: var(--text-faint); margin-bottom: 2px;
}
.today-stats { display: flex; flex-direction: column; gap: 8px; }
.today-stat { display: flex; align-items: baseline; gap: 8px; }
.today-num {
  font-family: var(--mono); font-size: 22px; font-weight: 800;
  line-height: 1; min-width: 40px;
}
.today-num.green { color: var(--green-check); }
.today-num.amber { color: var(--amber); }
.today-num.red { color: var(--red); }
.today-key { font-size: 11.5px; color: var(--text-dim); font-weight: 600; }
.today-review-btn {
  display: inline-flex; align-items: center; gap: 6px;
  background: rgba(163,230,53,0.08); border: 1px solid rgba(163,230,53,0.2);
  border-radius: 8px; padding: 6px 11px; font-size: 12px; font-weight: 700;
  color: var(--green); cursor: pointer; font-family: var(--sans);
  transition: all .15s;
}
.today-review-btn:hover { background: rgba(163,230,53,0.15); }

/* ================== REVIEW BANNER ================== */
.review-banner {
  background: rgba(234,179,8,0.05);
  border: 1px solid rgba(234,179,8,0.25);
  border-radius: var(--radius);
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.rb-head {
  display: flex; align-items: center; gap: 10px;
}
.rb-ico {
  width: 32px; height: 32px; border-radius: 8px;
  background: rgba(234,179,8,0.12); color: var(--amber);
  display: grid; place-items: center; flex-shrink: 0;
}
.rb-title {
  font-size: 14px; font-weight: 700; color: var(--text);
  display: flex; align-items: center; gap: 8px; flex: 1;
}
.rb-count {
  min-width: 22px; height: 22px; padding: 0 6px;
  background: var(--amber); color: #1a1200; border-radius: 999px;
  font-size: 11px; font-weight: 800; display: grid; place-items: center;
}
.rb-all { font-size: 11.5px; padding: 5px 10px; }
.rb-items { display: flex; flex-direction: column; gap: 8px; }
.rb-item {
  display: flex; align-items: center; justify-content: space-between;
  gap: 12px; padding: 10px 12px;
  background: rgba(0,0,0,0.2); border: 1px solid var(--border); border-radius: 10px;
  flex-wrap: wrap;
}
.rb-item-left { display: flex; align-items: center; gap: 10px; flex: 1; min-width: 0; flex-wrap: wrap; }
.rb-target { font-family: var(--mono); font-size: 12px; color: #fff; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 200px; }
.rb-by { font-size: 11px; color: var(--text-dim); }
.rb-by strong { color: var(--text); }
.rb-item-actions { display: flex; gap: 6px; flex-shrink: 0; }
.btn-sm { padding: 6px 12px; font-size: 12px; }
.btn-allow-inline { background: rgba(34,197,94,.1); border-color: rgba(34,197,94,.25); color: var(--green-check); }
.btn-deny-inline { background: rgba(239,68,68,.1); border-color: rgba(239,68,68,.25); color: #fca5a5; }
.btn-allow-inline:hover:not(:disabled), .btn-deny-inline:hover:not(:disabled) { opacity: 0.85; }
.btn-allow-inline:disabled, .btn-deny-inline:disabled { opacity: 0.5; cursor: not-allowed; }

/* ================== 4 STATS CARDS ================== */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}
.stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 13px 15px 15px;
  position: relative;
  overflow: hidden;
}
.stat-card::before {
  content: ""; position: absolute; inset: 0 0 auto 0; height: 3px;
}
.stat-card.total::before { background: linear-gradient(90deg, var(--green), var(--green-check)); }
.stat-card.allow::before { background: linear-gradient(90deg, #22c55e, #4ade80); }
.stat-card.ask::before   { background: linear-gradient(90deg, #f59e0b, #fbbf24); }
.stat-card.deny::before  { background: linear-gradient(90deg, #ef4444, #f87171); }
.stat-k {
  font-size: 11.5px; font-weight: 600; color: var(--text-dim);
  letter-spacing: 0.3px; margin-bottom: 8px;
}
.stat-k-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; }
.stat-k-row .stat-k { margin-bottom: 0; }
.stat-ico {
  width: 22px; height: 22px; border-radius: 7px; display: grid; place-items: center;
}
.stat-ico.ok   { background: rgba(34,197,94,.14);  color: var(--green-check); }
.stat-ico.warn { background: rgba(245,158,11,.14);  color: var(--amber); }
.stat-ico.bad  { background: rgba(239,68,68,.14);   color: var(--red); }

.stat-bottom { display: flex; align-items: flex-end; justify-content: space-between; gap: 10px; }
.stat-main { display: flex; flex-direction: column; gap: 3px; }
.stat-num {
  font-family: var(--mono); font-size: 30px; font-weight: 700;
  line-height: 1; color: #fff;
}
.stat-card.total .stat-num { color: var(--green); }
.stat-card.allow .stat-num { color: var(--green-check); }
.stat-card.ask .stat-num   { color: var(--amber); }
.stat-card.deny .stat-num  { color: var(--red); }

.stat-delta {
  display: inline-flex; align-items: center; gap: 3px;
  font-size: 11px; font-weight: 700; font-family: var(--mono);
  padding: 2px 6px; border-radius: 6px; width: fit-content;
}
.stat-delta.up { color: var(--green-check); background: rgba(34,197,94,.08); }
.stat-sub { font-size: 11px; color: var(--text-faint); font-family: var(--mono); font-weight: 600; }
.spark { width: 110px; height: 26px; flex-shrink: 0; }

/* ================== CARD BASE ================== */
.card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px 18px;
}
.card-head {
  display: flex; justify-content: space-between; align-items: center;
  margin-bottom: 14px;
}
.card-title {
  font-size: 14.5px; font-weight: 700; color: #fff;
  margin: 0; display: inline-flex; align-items: center; gap: 8px;
}
.title-badge {
  min-width: 20px; height: 20px; padding: 0 6px; border-radius: 999px;
  background: var(--amber); color: #1a1200; font-size: 11px; font-weight: 800;
  display: inline-grid; place-items: center;
}
.link-btn {
  background: none; border: none; color: var(--green); font-size: 12px; font-weight: 700;
  cursor: pointer; padding: 4px 8px; border-radius: 6px; font-family: var(--sans);
}
.link-btn:hover { background: rgba(163,230,53,.08); }
.card-actions { display: inline-flex; align-items: center; gap: 8px; }

/* ================== ROW 2 ================== */
.row-2 { display: grid; grid-template-columns: 1.5fr 1fr; gap: 14px; }
.activity-card { display: flex; flex-direction: column; }

.activity-empty {
  display: flex; flex-direction: column; align-items: center;
  padding: 28px 16px; text-align: center; gap: 6px;
}
.empty-ico {
  width: 50px; height: 50px; border-radius: 14px;
  background: var(--bg-soft); border: 1px solid var(--border-soft);
  display: grid; place-items: center; font-size: 20px; margin-bottom: 4px;
}
.empty-k { font-size: 13.5px; font-weight: 700; color: var(--text); }
.empty-v { font-size: 12px; color: var(--text-dim); line-height: 1.5; margin: 0; max-width: 380px; }

.activity-list { display: flex; flex-direction: column; gap: 6px; }
.act-row {
  display: flex; align-items: flex-start; gap: 12px;
  padding: 11px 13px; background: var(--bg-soft);
  border: 1px solid var(--border-soft); border-radius: 11px; transition: all .15s;
}
.act-row:hover { background: #0f1729; border-color: rgba(163,230,53,.15); }
.act-ico {
  width: 28px; height: 28px; border-radius: 8px; display: grid; place-items: center;
  flex-shrink: 0; margin-top: 1px;
}
.act-ico.allow { background: rgba(34,197,94,.14);  color: var(--green-check); }
.act-ico.ask   { background: rgba(245,158,11,.14);  color: var(--amber); }
.act-ico.deny  { background: rgba(239,68,68,.14);   color: var(--red); }
.act-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
.act-title { display: flex; align-items: center; gap: 9px; flex-wrap: wrap; }
.act-target {
  font-family: var(--mono); font-size: 12px; color: #fff;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100%;
}
.act-foot { display: flex; align-items: center; gap: 9px; flex-wrap: wrap; }
.act-time { margin-left: auto; font-size: 11px; color: var(--text-faint); font-family: var(--mono); }
.act-footnote { margin-top: 8px; font-size: 11px; color: var(--text-faint); }

/* Donut */
.col-right { display: flex; flex-direction: column; gap: 14px; }
.donut-body { display: flex; gap: 12px; align-items: center; }
.donut-wrap {
  position: relative; width: 148px; height: 148px; flex-shrink: 0;
}
.donut-center {
  position: absolute; inset: 0; display: grid; place-items: center; text-align: center;
}
.donut-num { font-family: var(--mono); font-size: 28px; font-weight: 700; color: #fff; line-height: 1; }
.donut-label { font-size: 10.5px; color: var(--text-dim); font-weight: 600; letter-spacing: 0.5px; margin-top: 3px; }
.donut-legend { flex: 1; display: flex; flex-direction: column; gap: 8px; min-width: 0; }
.legend-row { display: flex; align-items: center; gap: 8px; font-size: 12px; }
.legend-swatch { width: 8px; height: 8px; border-radius: 3px; flex-shrink: 0; }
.legend-label { flex: 1; color: var(--text-dim); font-weight: 600; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.legend-pct { font-family: var(--mono); color: #fff; font-weight: 700; font-size: 12px; flex-shrink: 0; }
.legend-empty { font-size: 12px; color: var(--text-faint); }

/* Health */
.health-card {
  display: flex; align-items: center; gap: 14px;
  padding: 14px 16px;
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius); cursor: pointer; text-align: left;
  color: inherit; font-family: var(--sans); transition: all .15s;
}
.health-card:hover { border-color: rgba(163,230,53,.28); background: rgba(163,230,53,.03); }
.health-ico {
  width: 42px; height: 42px; border-radius: 11px;
  background: rgba(34,197,94,.12); border: 1px solid rgba(34,197,94,.25);
  color: var(--green-check); display: grid; place-items: center; flex-shrink: 0;
}
.health-ico svg { width: 20px; height: 20px; }
.health-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.health-k { font-size: 13.5px; font-weight: 700; color: #fff; }
.health-v { font-size: 11.5px; color: var(--text-dim); line-height: 1.5; }
.health-arrow { color: var(--text-faint); font-size: 22px; font-weight: 300; }
.health-card:hover .health-arrow { color: var(--green); }

/* ================== ROW 3 ================== */
.row-3 { display: grid; grid-template-columns: 1.4fr 1fr; gap: 14px; }
.coverage-card, .review-card { display: flex; flex-direction: column; }

/* Coverage counts row */
.coverage-loading {
  display: flex; align-items: center; gap: 10px;
  padding: 14px 4px; color: var(--text-dim); font-size: 12.5px;
}
.spinner {
  width: 16px; height: 16px; border: 2px solid var(--border);
  border-top-color: var(--green); border-radius: 50%;
  animation: spin .7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.coverage-counts {
  display: flex; gap: 0; margin-bottom: 14px;
  background: var(--bg-soft); border: 1px solid var(--border-soft);
  border-radius: 12px; overflow: hidden;
}
.cov-tier {
  flex: 1; display: flex; flex-direction: column; align-items: center;
  padding: 14px 8px; gap: 3px; border-right: 1px solid var(--border-soft);
}
.cov-tier:last-child { border-right: none; }
.cov-icon {
  width: 28px; height: 28px; border-radius: 8px;
  display: grid; place-items: center; margin-bottom: 2px;
}
.cov-icon.full  { background: rgba(34,197,94,.12);  color: #22c55e; }
.cov-icon.partial { background: rgba(245,158,11,.1); color: #f59e0b; }
.cov-icon.observe { background: rgba(239,68,68,.1); color: #ef4444; }
.cov-num { font-size: 22px; font-weight: 800; line-height: 1; }
.cov-num.full  { color: #4ade80; }
.cov-num.partial { color: #fbbf24; }
.cov-num.observe { color: #f87171; }
.cov-label { font-size: 10px; font-weight: 600; color: var(--text-dim); text-align: center; }

/* Coverage list */
.coverage-list { display: flex; flex-direction: column; gap: 5px; }
.cov-row {
  display: flex; align-items: center; gap: 10px;
  padding: 9px 12px; background: var(--bg-soft);
  border: 1px solid var(--border-soft); border-radius: 9px;
}
.cov-row-icon {
  width: 22px; height: 22px; border-radius: 6px;
  display: grid; place-items: center; flex-shrink: 0;
}
.cov-row-icon.full   { background: rgba(34,197,94,.12);  color: #22c55e; }
.cov-row-icon.observe { background: rgba(239,68,68,.1);  color: #ef4444; }
.cov-row-icon.inactive { background: rgba(255,255,255,.05); color: var(--text-faint); }
.cov-row-name { flex: 1; font-size: 12.5px; font-weight: 600; color: var(--text); }
.cov-row-badge {
  font-size: 9px; font-weight: 800; letter-spacing: .6px;
  padding: 2px 7px; border-radius: 999px; text-transform: uppercase; flex-shrink: 0;
}
.cov-row-badge.high   { background: rgba(34,197,94,.12);  color: #22c55e; border: 1px solid rgba(34,197,94,.2); }
.cov-row-badge.generic { background: rgba(59,130,246,.1); color: #60a5fa; border: 1px solid rgba(59,130,246,.2); }
.cov-row-badge.observe { background: rgba(239,68,68,.1); color: #ef4444; border: 1px solid rgba(239,68,68,.2); }
.cov-row-badge.inactive { background: rgba(255,255,255,.05); color: var(--text-faint); border: 1px solid var(--border); }

.coverage-generic-note {
  margin-top: 10px; padding: 10px 12px;
  background: rgba(59,130,246,.07); border: 1px solid rgba(59,130,246,.18);
  border-radius: 9px; font-size: 11.5px; color: var(--text-dim); line-height: 1.5;
}

/* Review */
.review-empty {
  display: flex; flex-direction: column; align-items: center; gap: 8px;
  padding: 28px 16px; text-align: center;
}
.e-ico-wrap {
  width: 50px; height: 50px; border-radius: 14px;
  background: rgba(163,230,53,.07); color: var(--green);
  display: grid; place-items: center; margin-bottom: 4px;
}
.e-ico { width: 24px; height: 24px; }
.e-k { font-size: 13.5px; font-weight: 700; color: var(--text); }
.e-v { font-size: 12px; color: var(--text-dim); line-height: 1.5; margin: 0; max-width: 300px; }
.mt8 { margin-top: 8px; }

.review-list { display: flex; flex-direction: column; gap: 9px; }
.review-row {
  display: flex; flex-direction: column; gap: 9px;
  padding: 13px 14px; background: var(--bg-soft);
  border: 1px solid var(--border-soft); border-radius: 11px;
  transition: all .18s;
}
.review-row:hover { border-color: rgba(163,230,53,.22); background: rgba(163,230,53,.02); }
.rev-head { display: flex; align-items: flex-start; gap: 10px; }
.rev-act { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.rev-title { font-size: 13px; font-weight: 700; color: #fff; word-break: break-all; }
.rev-req { font-size: 11px; color: var(--text-dim); }
.rev-req strong { color: var(--text); }
.rev-time { font-size: 10.5px; color: var(--text-faint); font-family: var(--mono); }

.rev-buttons { display: flex; gap: 7px; }
.btn-allow, .btn-deny {
  flex: 1; padding: 7px 12px; border-radius: 8px;
  font-size: 12px; font-weight: 700; cursor: pointer;
  font-family: var(--sans); border: 1px solid transparent; transition: all .15s;
}
.btn-allow {
  background: rgba(34,197,94,.1); border-color: rgba(34,197,94,.25); color: var(--green-check);
}
.btn-allow:hover:not(:disabled) { background: rgba(34,197,94,.18); }
.btn-deny {
  background: rgba(239,68,68,.1); border-color: rgba(239,68,68,.25); color: #fca5a5;
}
.btn-deny:hover:not(:disabled) { background: rgba(239,68,68,.18); }
.btn-allow:disabled, .btn-deny:disabled { opacity: 0.6; cursor: not-allowed; }

.review-all {
  align-self: center; background: none; border: none;
  color: var(--green); font-size: 12px; font-weight: 700;
  cursor: pointer; padding: 5px 8px; border-radius: 6px;
  font-family: var(--sans); margin-top: 2px;
}
.review-all:hover { background: rgba(163,230,53,.08); }

/* ================== EVENTS ================== */
.events-card { padding-bottom: 16px; }
.events-empty { padding: 24px 10px; color: var(--text-dim); font-size: 12.5px; text-align: center; }
.events-list { display: flex; flex-direction: column; }
.event-row {
  display: grid;
  grid-template-columns: 76px 30px 72px 1fr auto;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 9px;
  font-size: 12px;
  transition: background .12s;
}
.event-row + .event-row { border-top: 1px solid var(--border-soft); }
.event-row:hover { background: var(--bg-soft); }
.ev-time { font-family: var(--mono); font-size: 11px; color: var(--text-faint); }
.ev-dot {
  width: 26px; height: 26px; border-radius: 7px; display: grid; place-items: center;
}
.ev-dot.allow { background: rgba(34,197,94,.12);  color: var(--green-check); }
.ev-dot.ask   { background: rgba(245,158,11,.12);  color: var(--amber); }
.ev-dot.deny  { background: rgba(239,68,68,.12);   color: var(--red); }
.ev-label {
  font-size: 10px; font-weight: 800; letter-spacing: 0.7px;
  padding: 3px 8px; border-radius: 999px; text-transform: uppercase;
  text-align: center; width: fit-content;
}
.ev-label.allow { color: var(--green-check); background: rgba(34,197,94,.1); border: 1px solid rgba(34,197,94,.22); }
.ev-label.ask   { color: var(--amber);       background: rgba(245,158,11,.1); border: 1px solid rgba(245,158,11,.22); }
.ev-label.deny  { color: #fca5a5;           background: rgba(239,68,68,.1);  border: 1px solid rgba(239,68,68,.22); }
.ev-action { display: inline-flex; align-items: center; gap: 9px; min-width: 0; }
.ev-target { font-family: var(--mono); font-size: 12px; color: #fff; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ev-source { font-size: 11px; color: var(--text-faint); text-align: right; font-family: var(--mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* ================== HOVER AFFORDANCE ================== */
.hoverable-card {
  transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
  cursor: default;
}
.hoverable-card:hover {
  transform: translateY(-1px);
  border-color: rgba(163, 230, 53, 0.35) !important;
  box-shadow: 0 4px 18px rgba(163, 230, 53, 0.08);
}

/* ================== RESPONSIVE ================== */
@media (max-width: 1160px) {
  .row-2, .row-3 { grid-template-columns: 1fr; }
  .stats-grid { grid-template-columns: repeat(2, 1fr); }
  .hero-card { grid-template-columns: 180px 1fr; grid-template-rows: auto auto; }
  .hero-today { grid-column: 1 / -1; }
  .event-row { grid-template-columns: 70px 30px 70px 1fr; }
  .ev-source { display: none; }
}
@media (max-width: 720px) {
  .stats-grid { grid-template-columns: 1fr; }
  .hero-card { grid-template-columns: 1fr; }
  .hero-left { order: -1; }
  .shield-area { width: 150px; height: 150px; }
  .s-core { width: 90px; height: 90px; }
  .s-core svg { width: 46px; height: 46px; }
  .hero-title { font-size: 24px; }
  .donut-body { flex-direction: column; }
  .event-row { grid-template-columns: 60px 26px 1fr; }
  .ev-label { display: none; }
}
</style>
