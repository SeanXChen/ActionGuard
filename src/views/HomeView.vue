<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { api } from "../api";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import type { SessionMode } from "../types";

const { state, navigate, setView, refreshSessions } = useStore();
const { t } = useI18n();

const workspace = ref<string | null>(null);
const picking = ref(false);
const starting = ref(false);
const error = ref<string | null>(null);
const mode = ref<SessionMode>("protected");

async function chooseFolder() {
  error.value = null;
  picking.value = true;
  try {
    workspace.value = await api.chooseWorkspace();
  } catch (e) {
    error.value = String(e);
  } finally {
    picking.value = false;
  }
}

async function start() {
  if (!workspace.value || starting.value) return;
  starting.value = true;
  error.value = null;
  try {
    const session = await api.startSession(workspace.value, mode.value);
    navigate("session", session);
  } catch (e) {
    error.value = String(e);
  } finally {
    starting.value = false;
  }
}

const isRunning = computed(() => state.session !== null);

const para = computed(() => state.paraStats);
const rateColor = computed(() => {
  const r = para.value.rate;
  if (r >= 50) return "red";
  if (r >= 20) return "amber";
  return "green";
});

const startBtnLabel = computed(() =>
  mode.value === "observe"
    ? t("home.mode.observe")
    : t("home.startBtn"),
);

// Consume CLI startup args from `actionguard protect <workspace> [--observe]`.
onMounted(async () => {
  const args = state.startupArgs;
  if (!args?.workspace) return;
  workspace.value = args.workspace;
  mode.value = args.mode;
  await start();
});
</script>

<template>
  <div class="home">
    <!-- ========= Hero ========= -->
    <div class="hero">
      <div class="hero-badge">◈ {{ t("app.tagline") }}</div>
      <h1 class="title">
        {{ t("home.title1") }}<br />
        <span class="accent">{{ t("home.title2") }}</span>
      </h1>
      <p class="subtitle">
        {{ t("home.subtitle") }}
      </p>
      <div class="hero-pills">
        <span class="pill pill-green">{{ t("home.pill.deterministic") }}</span>
        <span class="pill pill-blue">{{ t("home.pill.neutral") }}</span>
        <span class="pill pill-amber">{{ t("home.pill.localOnly") }}</span>
      </div>
    </div>

    <!-- ========= Steps card ========= -->
    <div class="card workspace-card">
      <div class="step">
        <h2 class="step-title">
          <span class="num">1</span>
          <span>{{ t("home.step1.title") }}</span>
        </h2>
        <p class="hint">{{ t("home.step1.hint") }}</p>
        <div class="picker">
          <button class="btn btn-ghost" :disabled="picking" @click="chooseFolder">
            <span class="pick-icon">📁</span>
            {{ picking ? t("home.chooseLoading") : t("home.choose") }}
          </button>
          <div class="path" :class="{ empty: !workspace }">
            {{ workspace ?? t("home.noFolder") }}
          </div>
        </div>
      </div>

      <div class="divider"></div>

      <!-- Mode A / Mode B selector -->
      <div class="step">
        <h2 class="step-title">
          <span class="num">2</span>
          <span>{{ t("home.mode.title") }}</span>
        </h2>
        <p class="hint">{{ t("home.mode.hint") }}</p>
        <div class="mode-picker">
          <button
            class="mode-card"
            :class="{ active: mode === 'observe' }"
            @click="mode = 'observe'"
          >
            <div class="mode-head">
              <div class="mode-name">{{ t("home.mode.observe") }}</div>
              <span class="mode-tag mode-tag-ghost">{{ t("home.mode.tagObserve") }}</span>
            </div>
            <p class="mode-desc">{{ t("home.mode.observe.desc") }}</p>
          </button>
          <button
            class="mode-card"
            :class="{ active: mode === 'protected' }"
            @click="mode = 'protected'"
          >
            <div class="mode-head">
              <div class="mode-name">{{ t("home.mode.protected") }}</div>
              <span class="mode-tag mode-tag-protect">{{ t("home.mode.tagProtected") }}</span>
            </div>
            <p class="mode-desc">{{ t("home.mode.protected.desc") }}</p>
          </button>
        </div>
      </div>

      <div class="divider"></div>

      <div class="step">
        <h2 class="step-title">
          <span class="num">3</span>
          <span>{{ t("home.step2.title") }}</span>
        </h2>
        <p class="hint">{{ t("home.step2.hint") }}</p>
        <div class="actions">
          <button class="btn btn-primary big" :disabled="!workspace || starting" @click="start">
            <span v-if="starting" class="spin"></span>
            <span v-if="starting">{{ t("home.starting") }}</span>
            <template v-else>
              <span class="shield">◈</span>
              {{ startBtnLabel }}
            </template>
          </button>
          <button v-if="isRunning" class="btn" @click="setView('session')">
            {{ t("home.resumeSession") }} #{{ state.session!.num.toString().padStart(5, "0") }}
          </button>
        </div>
      </div>

      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <!-- ========= Actions Detected (Detection != Protection) ========= -->
    <div class="card para-card" v-if="state.sessionsLoaded">
      <div class="para-head">
        <div>
          <div class="para-kicker">✦ {{ t("home.protected.keyMetric") }}</div>
          <h2>{{ t("home.protected.title") }}</h2>
          <p class="hint">{{ t("home.protected.desc") }}</p>
        </div>
        <div class="rate-big" :class="rateColor">
          <span class="rate-v">{{ para.actionsProtected.toLocaleString() }}</span>
          <span class="rate-k">{{ t("home.protected.protectedLabel") }}</span>
        </div>
      </div>
      <div class="para-grid">
        <div class="para-cell">
          <div class="pv red">{{ para.actionsBlocked.toLocaleString() }}</div>
          <div class="pk">{{ t("home.protected.blockedLabel") }}</div>
        </div>
        <div class="para-cell">
          <div class="pv purple">{{ para.critical }}</div>
          <div class="pk">{{ t("home.protected.criticalLabel") }}</div>
        </div>
        <div class="para-cell">
          <div class="pv red">{{ para.high }}</div>
          <div class="pk">{{ t("home.protected.highLabel") }}</div>
        </div>
        <div class="para-cell">
          <div class="pv amber">{{ para.rate }}%</div>
          <div class="pk">{{ t("home.para.rate") }}</div>
        </div>
      </div>
      <button v-if="!isRunning" class="btn btn-ghost right-link" @click="refreshSessions">
        ↻
      </button>
    </div>

    <!-- ========= Info card ========= -->
    <div class="note card">
      <div class="nrow">
        <strong>{{ t("home.whatMonitored.k") }}</strong>
        <span>{{ t("home.whatMonitored.v") }}</span>
      </div>
      <div class="nrow">
        <strong>{{ t("home.whatFlagged.k") }}</strong>
        <span>{{ t("home.whatFlagged.v") }}</span>
      </div>
      <div class="nrow">
        <strong>{{ t("home.undo.k") }}</strong>
        <span>{{ t("home.undo.v") }}</span>
      </div>
      <div class="nrow team-row">
        <strong>{{ t("home.team.k") }}</strong>
        <span>{{ t("home.team.v") }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home {
  max-width: 900px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding-bottom: 10px;
}

/* Hero */
.hero {
  position: relative;
  border-radius: var(--radius);
  padding: 28px 26px;
  background:
    radial-gradient(900px 320px at 0% -20%, rgba(34, 197, 94, 0.15), transparent 60%),
    radial-gradient(700px 300px at 100% -10%, rgba(56, 189, 248, 0.12), transparent 60%),
    linear-gradient(160deg, rgba(22, 33, 58, 0.9), rgba(15, 23, 42, 0.7));
  border: 1px solid var(--border);
  overflow: hidden;
}

.hero::after {
  content: "◈";
  position: absolute;
  right: -12px;
  bottom: -40px;
  font-size: 230px;
  color: rgba(34, 197, 94, 0.04);
  font-weight: 900;
  pointer-events: none;
}

.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 5px 12px;
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.35);
  border-radius: 999px;
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.3px;
  margin-bottom: 16px;
}

.hero .title {
  font-size: 30px;
  line-height: 1.22;
  letter-spacing: -0.4px;
  font-weight: 700;
}

.hero .title .accent {
  background: linear-gradient(135deg, #22c55e, #38bdf8);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.hero .subtitle {
  margin-top: 12px;
  max-width: 680px;
  color: var(--text-dim);
  font-size: 14px;
  line-height: 1.55;
}

.hero-pills {
  margin-top: 18px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.pill {
  font-size: 11.5px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid;
  letter-spacing: 0.3px;
}

.pill-green {
  color: var(--green);
  border-color: rgba(34, 197, 94, 0.35);
  background: var(--green-glow);
}

.pill-blue {
  color: var(--blue);
  border-color: rgba(56, 189, 248, 0.3);
  background: rgba(56, 189, 248, 0.08);
}

.pill-amber {
  color: var(--amber);
  border-color: rgba(245, 158, 11, 0.35);
  background: var(--amber-glow);
}

/* Steps */
.step-title {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 16px;
  margin-bottom: 4px;
  font-weight: 700;
}

.step-title .num {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--green-glow);
  border: 1px solid rgba(34, 197, 94, 0.4);
  color: var(--green);
  display: grid;
  place-items: center;
  font-size: 13px;
  font-weight: 800;
  flex-shrink: 0;
}

.hint {
  color: var(--text-dim);
  font-size: 13px;
  margin-bottom: 14px;
  line-height: 1.5;
}

.picker {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-ghost {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.pick-icon {
  font-size: 14px;
}

.path {
  flex: 1;
  font-family: var(--mono);
  font-size: 12.5px;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 10px 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-height: 38px;
}

.path.empty {
  color: var(--text-faint);
}

.divider {
  height: 1px;
  background: var(--border-soft);
  margin: 22px 0 24px;
}

/* Mode picker */
.mode-picker {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.mode-card {
  position: relative;
  background: var(--bg-soft);
  border: 1.5px solid var(--border);
  border-radius: 14px;
  padding: 18px 16px 16px;
  cursor: pointer;
  text-align: left;
  font-family: var(--sans);
  transition: all 0.15s ease;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.mode-card:hover {
  border-color: var(--green);
  background: rgba(34, 197, 94, 0.06);
  transform: translateY(-1px);
}

.mode-card.active {
  border-color: var(--green);
  background: linear-gradient(180deg, rgba(34, 197, 94, 0.14), rgba(22, 163, 74, 0.04));
  box-shadow: 0 0 0 1px var(--green-soft) inset;
}

.mode-badge {
  position: absolute;
  top: 10px;
  right: 12px;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 1px;
  padding: 2px 8px;
  border-radius: 999px;
}

.mode-badge-a {
  background: rgba(56, 189, 248, 0.15);
  color: var(--blue);
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.mode-badge-b {
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.35);
}

.mode-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.mode-name {
  font-size: 14px;
  font-weight: 700;
}

.mode-tag {
  display: inline-block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.3px;
  padding: 2px 9px;
  border-radius: 999px;
  white-space: nowrap;
}

.mode-tag-ghost {
  background: rgba(56, 189, 248, 0.1);
  color: var(--blue);
  border: 1px solid rgba(56, 189, 248, 0.25);
}

.mode-tag-protect {
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.35);
}

.mode-desc {
  font-size: 12px;
  color: var(--text-dim);
  line-height: 1.5;
}

.actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.big {
  padding: 12px 22px;
  font-size: 14px;
}

.shield {
  font-size: 15px;
  font-weight: 900;
}

.spin {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(4, 20, 10, 0.35);
  border-top-color: #04140a;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error {
  margin-top: 12px;
  color: #fca5a5;
  font-size: 12.5px;
  font-family: var(--mono);
}

/* Para stats */
.para-card {
  position: relative;
  overflow: hidden;
}

.para-card::after {
  content: "";
  position: absolute;
  inset: 0;
  background: radial-gradient(420px 200px at 100% 0%, rgba(245, 158, 11, 0.1), transparent 60%);
  pointer-events: none;
}

.para-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 24px;
}

.para-kicker {
  font-size: 10.5px;
  font-weight: 800;
  letter-spacing: 2px;
  color: var(--amber);
  margin-bottom: 4px;
}

.para-head h2 {
  font-size: 18px;
}

.rate-big {
  border-radius: 14px;
  padding: 14px 20px;
  border: 1px solid var(--border);
  background: var(--bg-soft);
  min-width: 160px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.rate-big.red {
  border-color: rgba(239, 68, 68, 0.4);
  background: var(--red-glow);
}
.rate-big.amber {
  border-color: rgba(245, 158, 11, 0.4);
  background: var(--amber-glow);
}
.rate-big.green {
  border-color: rgba(34, 197, 94, 0.4);
  background: var(--green-glow);
}

.rate-big .rate-v {
  font-family: var(--mono);
  font-size: 32px;
  font-weight: 800;
  line-height: 1;
}

.rate-big.red .rate-v { color: #fecaca; }
.rate-big.amber .rate-v { color: #fde68a; }
.rate-big.green .rate-v { color: var(--green); }

.rate-big .rate-v small {
  font-size: 18px;
  margin-left: 2px;
  opacity: 0.7;
}

.rate-big .rate-k {
  font-size: 11px;
  color: var(--text-dim);
  letter-spacing: 0.6px;
  margin-top: 4px;
}

.para-grid {
  margin-top: 22px;
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.para-cell {
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  padding: 14px 14px 12px;
}

.para-cell .pv {
  font-family: var(--mono);
  font-size: 22px;
  font-weight: 800;
}
.para-cell .pv.red { color: #fca5a5; }
.para-cell .pv.amber { color: var(--amber); }
.para-cell .pv.green { color: var(--green); }
.para-cell .pv.purple { color: var(--purple); }

.para-cell .pk {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.right-link {
  position: absolute;
  top: 10px;
  right: 10px;
  opacity: 0.6;
}
.right-link:hover {
  opacity: 1;
}

/* Info card */
.note {
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 12.5px;
}

.nrow {
  display: grid;
  grid-template-columns: 130px 1fr;
  gap: 6px 16px;
  align-items: start;
}

.nrow strong {
  color: var(--text-dim);
  font-weight: 600;
  text-align: right;
  padding-top: 1px;
}

.nrow span {
  color: var(--text);
  line-height: 1.5;
}
</style>
