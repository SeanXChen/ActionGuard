<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import { api } from "../api";
const store = useStore();
const { state } = store;
const { t } = useI18n();

const isRunning = computed(() => state.session !== null);
const stats = computed(() => state.activeStats);
const recentLedger = computed(() => state.ledger.slice(0, 8));
const pendingCount = computed(() => state.pendingApprovals.length);

const starting = ref(false);
const stopping = ref(false);
const showAdvanced = ref(false);
const policyRules = ref<Array<{ name: string; action: string }>>([]);
const policyError = ref<string | null>(null);
const enforce = ref<{ boundaries: { name: string; status: string }[] } | null>(null);

const totalActions = computed(() => stats.value?.total_actions ?? 0);
const allowedCount = computed(() => {
  const total = stats.value?.total_actions ?? 0;
  const blocked = stats.value?.actions_blocked ?? 0;
  return Math.max(0, total - blocked);
});
const blockedCount = computed(() => stats.value?.actions_blocked ?? 0);

async function startProtection() {
  starting.value = true;
  try {
    await api.startSession(".", "protected");
    await store.refreshActiveStats();
    await store.refreshLedger();
  } catch (e) {
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
  } catch (e) {
    /* ignore */
  } finally {
    stopping.value = false;
  }
}

async function loadAdvanced() {
  policyRules.value = [];
  policyError.value = null;
  enforce.value = null;
}

function activityIcon(entry: typeof state.ledger[number]): string {
  if (entry.result === "deny") return "🛡";
  if (entry.result === "ask" || entry.risk === "high" || entry.risk === "critical") return "⚠";
  return "✓";
}

function activityClass(entry: typeof state.ledger[number]): string {
  if (entry.result === "deny") return "act-blocked";
  if (entry.result === "ask" || entry.risk === "high" || entry.risk === "critical") return "act-asked";
  return "act-allowed";
}

function fmtTime(ts: string): string {
  const d = new Date(ts);
  return d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
}

function goActivity() {
  store.setView("history");
}

function goReview() {
  store.setView("review");
}

watch(showAdvanced, (v) => {
  if (v) void loadAdvanced();
});

onMounted(() => {
  if (isRunning.value) {
    void store.refreshActiveStats();
    void store.refreshLedger();
    void store.refreshPendingApprovals();
  }
});
</script>

<template>
  <div class="dashboard">
    <!-- ========== HERO ========== -->
    <div class="hero-card">
      <div class="hero-main">
        <div class="hero-shield" :class="{ inactive: !isRunning }">🛡</div>
        <div class="hero-body">
          <h2 class="hero-headline">
            {{ isRunning ? t("dashboard.hero.active") : t("dashboard.hero.inactive") }}
          </h2>
          <p class="hero-sub">
            {{ isRunning ? t("dashboard.hero.activeSub") : t("dashboard.hero.inactiveSub") }}
          </p>
          <div class="hero-meta">
            <span class="meta-chip">
              <span class="meta-dot" :class="{ active: isRunning }" />
              {{ isRunning ? t("dashboard.meta.enforced") : t("dashboard.meta.stopped") }}
            </span>
            <span class="meta-chip">{{ t("dashboard.meta.mode") }}: {{ t("dashboard.mode.recommended") }}</span>
            <span class="meta-chip">{{ t("dashboard.meta.scope") }}: {{ t("dashboard.scope.thisComputer") }}</span>
          </div>
        </div>
      </div>

      <div class="hero-actions">
        <template v-if="isRunning">
          <button class="btn" @click="goActivity">
            {{ t("dashboard.action.viewActivity") }}
          </button>
          <button class="btn btn-danger" :disabled="stopping" @click="stopProtection">
            {{ stopping ? t("dashboard.action.pausing") : t("dashboard.action.pause") }}
          </button>
        </template>
        <template v-else>
          <button class="btn btn-primary ob-cta" :disabled="starting" @click="startProtection">
            <span v-if="starting" class="ob-spinner" />
            <span v-else>🛡</span>
            {{ starting ? t("dashboard.action.starting") : t("dashboard.action.protect") }}
          </button>
        </template>
      </div>
    </div>

    <!-- ========== STATS ========== -->
    <div class="stats-row">
      <div class="stat-card">
        <div class="stat-label">{{ t("dashboard.stat.total") }}</div>
        <div class="stat-num">{{ totalActions }}</div>
      </div>
      <div class="stat-card allowed">
        <div class="stat-label">{{ t("dashboard.stat.allowed") }}</div>
        <div class="stat-num">{{ allowedCount }}</div>
        <div v-if="totalActions > 0" class="stat-pct">
          {{ totalActions ? Math.round((allowedCount / totalActions) * 100) : 0 }}%
        </div>
      </div>
      <div class="stat-card asked">
        <div class="stat-label">{{ t("dashboard.stat.asked") }}</div>
        <div class="stat-num">{{ pendingCount }}</div>
        <div v-if="totalActions > 0" class="stat-pct">
          {{ totalActions ? Math.round((pendingCount / totalActions) * 100) : 0 }}%
        </div>
      </div>
      <div class="stat-card blocked">
        <div class="stat-label">{{ t("dashboard.stat.blocked") }}</div>
        <div class="stat-num">{{ blockedCount }}</div>
        <div v-if="totalActions > 0" class="stat-pct">
          {{ totalActions ? Math.round((blockedCount / totalActions) * 100) : 0 }}%
        </div>
      </div>
    </div>

    <!-- ========== MAIN GRID ========== -->
    <div class="dash-grid">
      <!-- Recent Activity -->
      <div class="dash-card activity-card">
        <div class="dash-header">
          <h3>{{ t("dashboard.activity.title") }}</h3>
          <button v-if="recentLedger.length > 0" class="btn-ghost-link" @click="goActivity">
            {{ t("dashboard.activity.viewAll") }}
          </button>
        </div>

        <div v-if="recentLedger.length === 0" class="activity-empty">
          <div class="empty-icon">📭</div>
          <p>{{ t("dashboard.activity.empty") }}</p>
        </div>

        <div v-else class="activity-list">
          <div
            v-for="entry in recentLedger"
            :key="entry.id"
            class="activity-row"
            :class="activityClass(entry)"
          >
            <span class="act-icon">{{ activityIcon(entry) }}</span>
            <div class="act-body">
              <div class="act-primary">
                <span class="act-cat">{{ entry.category }}</span>
                <span class="act-target">{{ entry.target }}</span>
              </div>
              <div class="act-secondary">
                <span class="act-decision">{{ entry.decision }}</span>
                <span class="act-time">{{ fmtTime(entry.timestamp) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right column -->
      <div class="dash-col">
        <!-- Review Queue -->
        <div v-if="pendingCount > 0" class="dash-card review-card">
          <div class="dash-header">
            <h3>{{ t("dashboard.review.title") }}</h3>
            <span class="review-badge">{{ pendingCount }}</span>
          </div>
          <div class="review-list">
            <div
              v-for="req in state.pendingApprovals.slice(0, 3)"
              :key="req.id"
              class="review-row"
            >
              <div class="review-body">
                <div class="review-action">{{ req.action.target }}</div>
                <div class="review-meta">{{ t("dashboard.review.requestedBy") }} {{ req.action.agent ?? "AI" }}</div>
              </div>
              <div class="review-btns">
                <button class="btn btn-primary btn-sm" @click="store.resolveApproval(req.id, 'allow', false)">
                  {{ t("review.allow") }}
                </button>
                <button class="btn btn-danger btn-sm" @click="store.resolveApproval(req.id, 'deny', false)">
                  {{ t("review.deny") }}
                </button>
              </div>
            </div>
          </div>
          <button v-if="pendingCount > 3" class="btn-ghost-link" @click="goReview">
            {{ t("dashboard.review.viewAll") }}
          </button>
        </div>

        <!-- Protected Boundaries -->
        <div class="dash-card boundaries-card">
          <div class="dash-header">
            <h3>{{ t("dashboard.boundaries.title") }}</h3>
          </div>
          <div v-if="false" class="boundary-list">
            <!-- Boundaries data not yet available in ActiveStatsPayload -->
          </div>
          <div class="boundary-empty">
            {{ t("dashboard.boundaries.empty") }}
          </div>
        </div>
      </div>
    </div>

    <!-- ========== ADVANCED ========== -->
    <div class="advanced-section">
      <button class="advanced-toggle" @click="showAdvanced = !showAdvanced">
        <span>{{ showAdvanced ? "▾" : "▸" }}</span>
        {{ t("dashboard.advanced.title") }}
      </button>

      <div v-if="showAdvanced" class="advanced-content">
        <!-- Workspace & Mode -->
        <div class="adv-block">
          <h4>{{ t("home.advanced.workspace.title") }}</h4>
          <p class="adv-mono">{{ state.session?.workspace ?? "—" }}</p>
          <p class="adv-hint">
            {{ t("home.advanced.workspace.hint") }}
          </p>
          <div class="adv-mode">
            <span class="mode-tag">{{ state.session?.mode === "protected" ? t("home.advanced.mode.interactive") : t("home.advanced.mode.observe") }}</span>
          </div>
        </div>

        <!-- Rules -->
        <div class="adv-block">
          <h4>{{ t("home.advanced.rules.title") }}</h4>
          <p v-if="policyError" class="adv-error">{{ policyError }}</p>
          <ul v-else-if="policyRules.length" class="adv-rules">
            <li v-for="r in policyRules" :key="r.name">
              <span class="rule-name">{{ r.name }}</span>
              <span class="rule-action">{{ r.action }}</span>
            </li>
          </ul>
          <p v-else class="adv-empty">{{ t("home.advanced.rules.empty") }}</p>
        </div>

        <!-- Enforcement -->
        <div class="adv-block">
          <h4>{{ t("home.advanced.enforcement.title") }}</h4>
          <ul v-if="enforce?.boundaries.length" class="adv-list">
            <li v-for="b in enforce.boundaries" :key="b.name">
              {{ b.name }} — <span :class="{ ok: b.status === 'enforced', warn: b.status !== 'enforced' }">{{ b.status }}</span>
            </li>
          </ul>
          <p v-else class="adv-empty">{{ t("home.advanced.enforcement.empty") }}</p>
        </div>

        <!-- Diagnostics -->
        <div class="adv-block">
          <h4>{{ t("home.advanced.diagnostics.title") }}</h4>
          <div class="adv-diag">
            <p class="adv-hint">{{ t("home.advanced.diagnostics.desc") }}</p>
          </div>
        </div>

        <!-- Metrics -->
        <div class="adv-block">
          <h4>{{ t("home.metrics.title") }}</h4>
          <div class="metrics-row">
            <div class="risk-pill" :class="{ 'risk-zero': state.paraStats.n === 0 }">
              <span class="pill-num">{{ state.paraStats.n }}</span>
              <span class="pill-label">{{ t("home.metrics.sessions") }}</span>
            </div>
            <div class="risk-pill" :class="{ 'risk-zero': state.paraStats.flagged === 0 }">
              <span class="pill-num">{{ state.paraStats.flagged }}</span>
              <span class="pill-label">{{ t("home.metrics.flagged") }}</span>
            </div>
            <div class="risk-pill" :class="{ 'risk-zero': state.paraStats.rate === 0 }">
              <span class="pill-num">{{ state.paraStats.rate }}%</span>
              <span class="pill-label">{{ t("home.metrics.rate") }}</span>
            </div>
            <div class="risk-pill" :class="{ 'risk-zero': state.paraStats.actionsBlocked === 0 }">
              <span class="pill-num">{{ state.paraStats.actionsBlocked }}</span>
              <span class="pill-label">{{ t("home.metrics.blockedActions") }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ---------- Dashboard Layout ---------- */
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* ---------- Hero ---------- */
.hero-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 28px 28px 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
}

.hero-main {
  display: flex;
  align-items: center;
  gap: 20px;
  flex: 1;
}

.hero-shield {
  font-size: 56px;
  line-height: 1;
  filter: drop-shadow(0 4px 24px rgba(34, 197, 94, 0.3));
  flex-shrink: 0;
  transition: filter 0.3s;
}

.hero-shield.inactive {
  filter: grayscale(0.7) drop-shadow(0 4px 12px rgba(255, 255, 255, 0.05));
  opacity: 0.6;
}

.hero-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.hero-headline {
  font-size: 20px;
  font-weight: 700;
  color: var(--text);
  letter-spacing: 0.2px;
}

.hero-sub {
  font-size: 13px;
  color: var(--text-dim);
  line-height: 1.5;
  max-width: 420px;
}

.hero-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 4px;
}

.meta-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-dim);
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  padding: 4px 10px;
  border-radius: 6px;
}

.meta-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--text-faint);
}

.meta-dot.active {
  background: var(--green);
  animation: pulse 1.6s infinite;
}

.hero-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-shrink: 0;
}

.ob-cta {
  padding: 12px 24px;
  font-size: 14px;
  border-radius: 10px;
}

.ob-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ---------- Stats ---------- */
.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
}

.stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-faint);
  font-weight: 600;
}

.stat-num {
  font-family: var(--mono);
  font-size: 30px;
  font-weight: 700;
  line-height: 1;
  color: var(--text);
}

.stat-card.allowed .stat-num { color: var(--green); }
.stat-card.asked .stat-num { color: var(--amber); }
.stat-card.blocked .stat-num { color: var(--red); }

.stat-pct {
  font-size: 11px;
  color: var(--text-faint);
  margin-top: 2px;
}

/* ---------- Grid ---------- */
.dash-grid {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 16px;
}

.dash-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px 20px;
}

.dash-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.dash-header h3 {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  text-transform: uppercase;
  letter-spacing: 0.6px;
}

.btn-ghost-link {
  background: transparent;
  border: none;
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  font-family: var(--sans);
  padding: 2px 6px;
  border-radius: 4px;
  transition: all 0.12s;
}

.btn-ghost-link:hover {
  color: var(--text);
  background: rgba(255, 255, 255, 0.04);
}

/* ---------- Activity ---------- */
.activity-empty {
  text-align: center;
  padding: 36px 20px;
  color: var(--text-faint);
}

.empty-icon {
  font-size: 32px;
  margin-bottom: 10px;
  opacity: 0.5;
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.activity-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  transition: background 0.12s;
}

.activity-row:hover {
  background: var(--bg-card-hover);
}

.act-icon {
  font-size: 14px;
  width: 22px;
  text-align: center;
  flex-shrink: 0;
}

.act-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.act-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
  overflow: hidden;
}

.act-cat {
  font-family: var(--mono);
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 1px 6px;
  border-radius: 4px;
  flex-shrink: 0;
}

.act-row.act-allowed .act-cat { background: rgba(34, 197, 94, 0.12); color: var(--green); }
.act-row.act-asked .act-cat { background: var(--amber-glow); color: var(--amber); }
.act-row.act-blocked .act-cat { background: rgba(239, 68, 68, 0.12); color: #fca5a5; }

.act-target {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text);
}

.act-secondary {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 11px;
  color: var(--text-faint);
}

.act-decision {
  text-transform: capitalize;
}

.act-time {
  font-family: var(--mono);
}

/* ---------- Review ---------- */
.review-card {
  margin-bottom: 16px;
}

.review-badge {
  background: var(--red-soft);
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 999px;
  min-width: 22px;
  text-align: center;
}

.review-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 10px;
}

.review-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
}

.review-body {
  flex: 1;
  min-width: 0;
}

.review-action {
  font-size: 12.5px;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.review-meta {
  font-size: 11px;
  color: var(--text-faint);
  margin-top: 2px;
}

.review-btns {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}

/* ---------- Boundaries ---------- */
.boundary-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.boundary-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--bg-soft);
  font-size: 12px;
}

.b-icon {
  font-size: 12px;
  opacity: 0.7;
}

.b-name {
  flex: 1;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.b-status {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 2px 8px;
  border-radius: 4px;
}

.b-status.ok {
  background: rgba(34, 197, 94, 0.12);
  color: var(--green);
}

.b-status.warn {
  background: var(--amber-glow);
  color: var(--amber);
}

.boundary-empty {
  font-size: 12px;
  color: var(--text-faint);
  padding: 16px 8px;
  text-align: center;
}

/* ---------- Advanced ---------- */
.advanced-section {
  margin-top: 4px;
}

.advanced-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 14px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-dim);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--sans);
  transition: all 0.15s;
}

.advanced-toggle:hover {
  background: var(--bg-card-hover);
  color: var(--text);
}

.advanced-content {
  margin-top: 12px;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 14px;
}

.adv-block {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px 20px;
}

.adv-block h4 {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.6px;
  color: var(--text-dim);
  margin-bottom: 10px;
}

.adv-mono {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text);
  background: var(--bg-soft);
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-soft);
  word-break: break-all;
}

.adv-hint {
  font-size: 11px;
  color: var(--text-faint);
  margin-top: 6px;
}

.adv-mode {
  margin-top: 10px;
}

.mode-tag {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 6px;
  background: rgba(34, 197, 94, 0.1);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.25);
}

.adv-error {
  font-size: 12px;
  color: #fca5a5;
  background: rgba(239, 68, 68, 0.08);
  padding: 8px 10px;
  border-radius: 6px;
}

.adv-rules {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.adv-rules li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  background: var(--bg-soft);
  border-radius: 6px;
  font-size: 12px;
}

.rule-name {
  color: var(--text);
}

.rule-action {
  font-family: var(--mono);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--green);
}

.adv-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--text-dim);
}

.adv-list .ok { color: var(--green); font-weight: 600; }
.adv-list .warn { color: var(--amber); font-weight: 600; }

.adv-empty {
  font-size: 12px;
  color: var(--text-faint);
  padding: 8px 0;
}

.adv-diag {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.metrics-row {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
</style>
