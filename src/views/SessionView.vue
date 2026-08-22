<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { api } from "../api";
import EnforcementPanel from "../components/EnforcementPanel.vue";
import RiskBadge from "../components/RiskBadge.vue";
import LedgerTable from "../components/LedgerTable.vue";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import type { ActionCategory, RiskLevel } from "../types";

const { state, setView, refreshActiveStats, refreshLedger } = useStore();
const { t, tf } = useI18n();

const elapsed = ref("00:00");
let timer: ReturnType<typeof setInterval> | null = null;
let pollTimer: ReturnType<typeof setInterval> | null = null;

function tick() {
  if (!state.session) return;
  const start = new Date(state.session.started_at.replace(" ", "T")).getTime();
  const secs = Math.max(0, Math.floor((Date.now() - start) / 1000));
  const m = Math.floor(secs / 60).toString().padStart(2, "0");
  const s = (secs % 60).toString().padStart(2, "0");
  elapsed.value = `${m}:${s}`;
}

onMounted(() => {
  tick();
  timer = setInterval(tick, 1000);
  // Poll live stats + ledger every 1.5s while the session is active.
  void refreshActiveStats();
  void refreshLedger(50);
  pollTimer = setInterval(async () => {
    await Promise.all([refreshActiveStats(), refreshLedger(50)]);
  }, 1500);
});
onBeforeUnmount(() => {
  if (timer) clearInterval(timer);
  if (pollTimer) clearInterval(pollTimer);
});

const stats = computed(() => state.activeStats);
const ledger = computed(() => state.ledger);

const totalActions = computed(() => stats.value?.total_actions ?? 0);
const actionsProtected = computed(() => stats.value?.actions_protected ?? 0);
const actionsBlocked = computed(() => stats.value?.actions_blocked ?? 0);

interface RiskPill {
  level: RiskLevel;
  num: number;
  label: string;
  zero: boolean;
}

const riskPills = computed<RiskPill[]>(() => {
  const rc = stats.value?.risk_counts ?? {
    low: 0,
    medium: 0,
    high: 0,
    critical: 0,
  };
  return [
    { level: "low", num: rc.low, label: t("risk.low"), zero: rc.low === 0 },
    { level: "medium", num: rc.medium, label: t("risk.medium"), zero: rc.medium === 0 },
    { level: "high", num: rc.high, label: t("risk.high"), zero: rc.high === 0 },
    { level: "critical", num: rc.critical, label: t("risk.critical"), zero: rc.critical === 0 },
  ];
});

interface CatChip {
  category: ActionCategory;
  num: number;
  label: string;
  zero: boolean;
}

const catChips = computed<CatChip[]>(() => {
  const cc = stats.value?.category_counts ?? {
    file: 0,
    shell: 0,
    git: 0,
    package: 0,
    secret: 0,
  };
  return [
    { category: "file", num: cc.file, label: t("category.file"), zero: cc.file === 0 },
    { category: "shell", num: cc.shell, label: t("category.shell"), zero: cc.shell === 0 },
    { category: "git", num: cc.git, label: t("category.git"), zero: cc.git === 0 },
    { category: "package", num: cc.package, label: t("category.package"), zero: cc.package === 0 },
    { category: "secret", num: cc.secret, label: t("category.secret"), zero: cc.secret === 0 },
  ];
});

const stopping = ref(false);
const undoing = ref(false);
const error = ref<string | null>(null);

async function endSession() {
  stopping.value = true;
  error.value = null;
  try {
    await api.stopSession();
    setView("home");
  } catch (e) {
    error.value = String(e);
  } finally {
    stopping.value = false;
  }
}

async function undoNow() {
  if (!state.session || undoing.value) return;
  undoing.value = true;
  error.value = null;
  try {
    await api.undoActiveSession();
    setView("home");
  } catch (e) {
    error.value = String(e);
  } finally {
    undoing.value = false;
  }
}

const batchTotal = computed(() => {
  const p = state.pendingBatch;
  if (!p) return 0;
  return p.counts.create + p.counts.modify + p.counts.delete + p.counts.rename;
});

const sessionTitle = computed(() => {
  const num = state.session?.num.toString().padStart(5, "0") ?? "00000";
  return tf("session.dashboard.title", { num });
});
</script>

<template>
  <div v-if="!state.session" class="empty">
    <div class="big">◈</div>
    <p>{{ t("monitor.noActive") }}</p>
    <button class="btn" @click="setView('home')">{{ t("monitor.goStart") }}</button>
  </div>

  <div v-else class="monitor">
    <div class="card head">
      <div>
        <div class="meta-line">
          <span class="chip-live">
            <span class="dot"></span> LIVE
          </span>
          <span
            class="chip-mode"
            :class="[
              state.session?.mode === 'observe'
                ? 'chip-mode-a'
                : 'chip-mode-b',
            ]"
          >
            {{
              state.session?.mode === 'observe'
                ? t('session.mode.badgeObserve')
                : t('session.mode.badgeProtected')
            }}
            <span class="chip-mode-label">
              {{
                state.session?.mode === 'observe'
                  ? t('session.mode.observe')
                  : t('session.mode.protected')
              }}
            </span>
          </span>
          <span class="meta">{{ t("monitor.elapsed") }} · {{ elapsed }}</span>
        </div>
        <h1 class="title">{{ sessionTitle }}</h1>
        <p class="subtitle mono">{{ state.session.workspace }}</p>
      </div>
      <div class="head-right">
        <RiskBadge :level="state.riskLevel" />
        <span class="snap-note">◈ snapshot {{ state.session.snapshot_files }} files</span>
      </div>
    </div>

    <!-- v0.2: Execution Path Matrix — be honest about what is enforced -->
    <EnforcementPanel />

    <!-- v0.2 dashboard: total + 4 risk pills -->
    <div class="card dashboard">
      <div class="dash-row">
        <div class="dash-total">
          <div class="dash-num">{{ totalActions }}</div>
          <div class="dash-label">{{ t("session.dashboard.actions") }}</div>
        </div>
        <div class="dash-protected">
          <div class="dash-num">{{ actionsProtected }}</div>
          <div class="dash-label">{{ t("session.dashboard.protected") }}</div>
        </div>
        <div class="dash-blocked">
          <div class="dash-num">{{ actionsBlocked }}</div>
          <div class="dash-label">{{ t("session.dashboard.blocked") }}</div>
        </div>
      </div>

      <!-- v0.3: Detection ≠ Protection — what ActionGuard actually stopped -->
      <div class="dash-enforcement" v-if="stats?.enforcement_counts">
        <div class="dash-enf-label">{{ t("session.dashboard.enforcement") }}</div>
        <div class="dash-enf-row">
          <div class="enf-cell enf-enforced">
            <span class="enf-num">{{ stats.enforcement_counts.enforced }}</span>
            <span class="enf-key">{{ t("session.dashboard.enforced") }}</span>
          </div>
          <div class="enf-cell">
            <span class="enf-num">{{ stats.enforcement_counts.observed }}</span>
            <span class="enf-key">{{ t("session.dashboard.observed") }}</span>
          </div>
          <div class="enf-cell">
            <span class="enf-num">{{ stats.enforcement_counts.bypassed }}</span>
            <span class="enf-key">{{ t("session.dashboard.bypassed") }}</span>
          </div>
          <div class="enf-cell">
            <span class="enf-num">{{ stats.enforcement_counts.unsupported }}</span>
            <span class="enf-key">{{ t("session.dashboard.unsupported") }}</span>
          </div>
        </div>
      </div>

      <div class="risk-pills">
        <div
          v-for="p in riskPills"
          :key="p.level"
          class="risk-pill"
          :class="[`risk-${p.level}`, { 'risk-zero': p.zero }]"
        >
          <div class="pill-num">{{ p.num }}</div>
          <div class="pill-label">{{ p.label }}</div>
        </div>
      </div>
    </div>

    <!-- 5 category chips -->
    <div class="card">
      <div class="section-label">{{ t("session.dashboard.categoryBreakdown") }}</div>
      <div class="category-chips">
        <div
          v-for="c in catChips"
          :key="c.category"
          class="cat-chip"
          :class="{ 'is-zero': c.zero }"
        >
          <div class="chip-top">
            <span
              class="chip-dot"
              :class="`category-${c.category}`"
            ></span>
            <span class="chip-num">{{ c.num }}</span>
          </div>
          <div class="chip-label">{{ c.label }}</div>
        </div>
      </div>
    </div>

    <div v-if="state.awaitingReview && state.pendingBatch" class="risk-banner card">
      <div class="rb-head">
        <RiskBadge :level="state.pendingBatch.risk.level" />
        <div class="rb-title">{{ t("monitor.riskBanner.title") }}</div>
      </div>
      <p class="rb-text">
        {{ t("monitor.riskBanner.prefix") }}
        <strong>{{ batchTotal }}</strong>
        {{ t("monitor.riskBanner.suffix") }}
      </p>
      <ul class="rb-reasons">
        <li v-for="(r, i) in state.pendingBatch.risk.reasons" :key="i">• {{ r }}</li>
      </ul>
      <div class="rb-actions">
        <button class="btn" @click="setView('review')">
          🔎 {{ t("monitor.riskBanner.review") }}
        </button>
        <button class="btn btn-primary" @click="api.allowBatch(); setView('session')">
          ✓ {{ t("monitor.riskBanner.allow") }}
        </button>
        <button class="btn btn-danger" @click="api.denyBatch()">
          ✕ {{ t("monitor.riskBanner.deny") }}
        </button>
      </div>
    </div>

    <!-- Action Ledger -->
    <div class="card ledger-card">
      <div class="ledger-head">
        <div>
          <div class="section-label">{{ t("ledger.title") }}</div>
          <p class="section-sub">{{ t("ledger.subtitle") }}</p>
        </div>
        <span class="ledger-meta">{{ tf("ledger.lastN", { n: 50 }) }}</span>
      </div>
      <LedgerTable :entries="ledger" />
    </div>

    <div class="card tools">
      <div class="tools-left">
        <button class="btn" :disabled="stopping" @click="endSession">
          <span v-if="stopping" class="spin small"></span>
          {{ stopping ? t("monitor.ending") : t("monitor.end") }}
        </button>
        <button class="btn" :disabled="undoing" @click="undoNow">
          <span v-if="undoing" class="spin small"></span>
          ↶ {{ undoing ? t("monitor.undoing") : t("monitor.undo") }}
        </button>
        <button class="btn btn-ghost" @click="setView('history')">
          ⏳ {{ t("monitor.history") }}
        </button>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <p class="disclaimer">
      ⚠ {{ t("monitor.disclaimer") }}
    </p>
  </div>
</template>

<style scoped>
.empty {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-dim);
}

.empty .big {
  font-size: 38px;
  margin-bottom: 12px;
}

.monitor {
  max-width: 980px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.meta-line {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.chip-live {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 9px;
  border-radius: 999px;
  background: var(--red-glow);
  border: 1px solid rgba(239, 68, 68, 0.35);
  color: #fca5a5;
  font-size: 10.5px;
  font-weight: 800;
  letter-spacing: 1.2px;
}

.chip-live .dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #ef4444;
  animation: pulse 1.1s infinite;
}

.chip-mode {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 10.5px;
  font-weight: 800;
  letter-spacing: 0.8px;
  font-family: var(--mono);
}

.chip-mode-a {
  background: rgba(56, 189, 248, 0.15);
  color: var(--blue);
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.chip-mode-b {
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.35);
}

.chip-mode-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.3px;
  opacity: 0.85;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.meta {
  font-size: 12px;
  color: var(--text-faint);
  font-family: var(--mono);
}

.title {
  font-size: 22px;
  font-weight: 700;
}

.subtitle {
  margin-top: 4px;
}

.mono {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text-faint);
}

.head-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
}

.snap-note {
  font-size: 11px;
  color: var(--text-faint);
  font-family: var(--mono);
}

/* ---------- dashboard ---------- */
.dashboard {
  padding: 18px 20px;
}
.dash-row {
  display: flex;
  gap: 32px;
  align-items: baseline;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 14px;
  flex-wrap: wrap;
}
.dash-total .dash-num,
.dash-protected .dash-num,
.dash-blocked .dash-num {
  font-family: var(--mono);
  font-size: 30px;
  font-weight: 800;
  line-height: 1;
}
.dash-total .dash-num { color: var(--text); }
.dash-protected .dash-num { color: var(--blue); }
.dash-blocked .dash-num { color: var(--red); }
.dash-enforcement {
  margin-top: 6px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}
.dash-enf-label {
  font-size: 10px;
  letter-spacing: 0.8px;
  text-transform: uppercase;
  color: var(--text-faint);
  font-family: var(--mono);
  margin-bottom: 8px;
}
.dash-enf-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}
.enf-cell {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.enf-num {
  font-family: var(--mono);
  font-size: 18px;
  font-weight: 800;
  line-height: 1;
  color: var(--text);
}
.enf-enforced .enf-num {
  color: var(--green);
}
.enf-key {
  font-size: 9.5px;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--text-faint);
  font-family: var(--mono);
}
.dash-label {
  font-size: 10px;
  letter-spacing: 0.8px;
  text-transform: uppercase;
  color: var(--text-faint);
  margin-top: 4px;
}

.risk-pills {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

/* ---------- category chips ---------- */
.cat-chip .chip-dot {
  width: 8px;
  height: 8px;
  border-radius: 2px;
  display: inline-block;
}
.cat-chip .chip-dot.category-file { background: var(--blue); }
.cat-chip .chip-dot.category-shell { background: var(--amber); }
.cat-chip .chip-dot.category-git { background: #f97316; }
.cat-chip .chip-dot.category-package { background: var(--green); }
.cat-chip .chip-dot.category-secret { background: var(--purple); }

/* ---------- section labels ---------- */
.section-label {
  font-size: 11px;
  letter-spacing: 1px;
  text-transform: uppercase;
  color: var(--text-faint);
  font-family: var(--mono);
  margin-bottom: 4px;
}
.section-sub {
  font-size: 12px;
  color: var(--text-faint);
  margin-bottom: 12px;
}

/* ---------- ledger card ---------- */
.ledger-card {
  padding: 16px 18px;
}
.ledger-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 10px;
}
.ledger-meta {
  font-size: 11px;
  color: var(--text-faint);
  font-family: var(--mono);
}

.risk-banner {
  border: 1px solid rgba(239, 68, 68, 0.5);
  background:
    radial-gradient(420px 180px at 0% 0%, rgba(239, 68, 68, 0.14), transparent 60%),
    linear-gradient(135deg, rgba(220, 38, 38, 0.05), rgba(239, 68, 68, 0.02));
}

.rb-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.rb-title {
  font-family: var(--mono);
  font-weight: 800;
  color: #fca5a5;
  letter-spacing: 1px;
  font-size: 15px;
}

.rb-text {
  color: var(--text);
  font-size: 14px;
}
.rb-text strong {
  color: #fca5a5;
  font-size: 18px;
  font-family: var(--mono);
  margin: 0 2px;
}

.rb-reasons {
  margin: 10px 0 0 18px;
  color: var(--text-dim);
  font-size: 13px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.rb-actions {
  margin-top: 16px;
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.tools {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  padding: 14px 18px;
}

.tools-left {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
}

.spin {
  width: 12px;
  height: 12px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  display: inline-block;
}
.spin.small {
  width: 10px;
  height: 10px;
  border-width: 1.7px;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

.error {
  color: #fca5a5;
  font-size: 12px;
  font-family: var(--mono);
  margin-left: 8px;
}

.disclaimer {
  color: var(--text-faint);
  font-size: 12px;
}
</style>
