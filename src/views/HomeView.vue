<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { homeDir } from "@tauri-apps/api/path";
import { api } from "../api";
import { useStore } from "../store";
import { useI18n, type DictKey } from "../i18n";
import type { SessionMode, ActionCategory, Decision } from "../types";

/** Minimal shape of a ledger row as surfaced by the store (readonly). */
interface ActivityItem {
  id: string;
  time_hms: string;
  category: ActionCategory;
  kind: string;
  target: string;
  decision: Decision;
  result: string;
  reasons: readonly string[];
}

const { state, navigate, setView, refreshSessions, refreshActiveStats, refreshLedger } =
  useStore();
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

// ===========================================================================
// Consumer entry — "Protect this computer"
// ===========================================================================
const consumerPhase = ref<"landing" | "onboarding">("landing");
const consumerStarting = ref(false);
const consumerError = ref<string | null>(null);
const pausing = ref(false);
let pollTimer: ReturnType<typeof setInterval> | null = null;

const stats = computed(() => state.activeStats);
const activity = computed(() => state.ledger);

const allowedCount = computed(() => stats.value?.actions_protected ?? 0);
const blockedCount = computed(() => stats.value?.actions_blocked ?? 0);
const reviewedCount = computed(() => {
  const s = stats.value;
  if (!s) return 0;
  const r = s.total_actions - s.actions_protected - s.actions_blocked;
  return r > 0 ? r : 0;
});

const protectItems = computed(() => [
  t("home.consumer.onboarding.protect.file"),
  t("home.consumer.onboarding.protect.shell"),
  t("home.consumer.onboarding.protect.git"),
  t("home.consumer.onboarding.protect.package"),
  t("home.consumer.onboarding.protect.secret"),
]);

/** Start protection over the whole machine (user home folder) via the existing session engine. */
async function startConsumer() {
  if (consumerStarting.value) return;
  consumerStarting.value = true;
  consumerError.value = null;
  try {
    const dir = await homeDir();
    const session = await api.startSession(dir, "protected");
    navigate("home", session);
    consumerPhase.value = "landing";
    startConsumerPolling();
  } catch (e) {
    consumerError.value = String(e);
  } finally {
    consumerStarting.value = false;
  }
}

async function pauseProtection() {
  if (pausing.value) return;
  pausing.value = true;
  consumerError.value = null;
  try {
    await api.stopSession();
    setView("home");
    stopConsumerPolling();
  } catch (e) {
    consumerError.value = String(e);
  } finally {
    pausing.value = false;
  }
}

function startConsumerPolling() {
  stopConsumerPolling();
  void Promise.all([refreshActiveStats(), refreshLedger(12)]);
  pollTimer = setInterval(() => {
    void Promise.all([refreshActiveStats(), refreshLedger(12)]);
  }, 3000);
}

function stopConsumerPolling() {
  if (pollTimer !== null) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

function describe(e: ActivityItem): string {
  const parts: string[] = [];
  parts.push(t(`category.${e.category}` as DictKey) || e.category);
  if (e.kind) parts.push(e.kind);
  if (e.target) parts.push(e.target);
  return parts.join(" · ");
}

function actIcon(e: ActivityItem): string {
  if (e.decision === "deny" || e.result === "blocked") return "🛑";
  if (e.decision === "ask") return "⏸";
  return "✓";
}

function actClass(e: ActivityItem): string {
  if (e.decision === "deny" || e.result === "blocked") return "red";
  if (e.decision === "ask") return "amber";
  return "green";
}

// Consume CLI startup args from `actionguard protect <workspace> [--observe]`.
onMounted(async () => {
  const args = state.startupArgs;
  if (args?.workspace) {
    workspace.value = args.workspace;
    mode.value = args.mode;
    await start();
    return;
  }
  if (state.session) {
    startConsumerPolling();
  }
});

onBeforeUnmount(() => {
  stopConsumerPolling();
});
</script>

<template>
  <div class="home">
    <!-- ========= Consumer entry: Protect this computer ========= -->
    <div v-if="!isRunning" class="consumer-card">
      <!-- Landing -->
      <template v-if="consumerPhase === 'landing'">
        <div class="consumer-badge">🛡 {{ t("home.consumer.badge") }}</div>
        <h2 class="consumer-title">{{ t("home.consumer.title") }}</h2>
        <p class="consumer-sub">{{ t("home.consumer.subtitle") }}</p>
        <button class="consumer-cta" @click="consumerPhase = 'onboarding'">
          🛡 {{ t("home.consumer.cta") }}
        </button>
        <p class="consumer-trust">{{ t("home.consumer.trust") }}</p>
      </template>

      <!-- Onboarding -->
      <template v-else>
        <div class="consumer-badge">🛡 {{ t("home.consumer.badge") }}</div>
        <h2 class="consumer-title sm">{{ t("home.consumer.onboarding.title") }}</h2>

        <div class="ob-block">
          <!-- Step 1: scope -->
          <div class="ob-row">
            <div class="ob-step">1</div>
            <div class="ob-col">
              <div class="ob-label">{{ t("home.consumer.onboarding.scope.title") }}</div>
              <div class="ob-option">
                <div class="ob-option-head">
                  <span>💻</span>
                  <span>{{ t("home.consumer.onboarding.scope.computer") }}</span>
                </div>
                <span class="ob-desc">{{ t("home.consumer.onboarding.scope.computerDesc") }}</span>
              </div>
            </div>
          </div>

          <!-- Step 2: protection level -->
          <div class="ob-row">
            <div class="ob-step">2</div>
            <div class="ob-col">
              <div class="ob-label">{{ t("home.consumer.onboarding.level.title") }}</div>
              <div class="ob-option">
                <div class="ob-option-head">
                  <span>⭐</span>
                  <span>{{ t("home.consumer.onboarding.level.recommended") }}</span>
                </div>
                <span class="ob-desc">{{
                  t("home.consumer.onboarding.level.recommendedDesc")
                }}</span>
              </div>
            </div>
          </div>

          <!-- Step 3: what gets protected -->
          <div class="ob-row">
            <div class="ob-step">3</div>
            <div class="ob-col">
              <div class="ob-label">{{ t("home.consumer.onboarding.protect.title") }}</div>
              <div class="ob-checks">
                <span v-for="item in protectItems" :key="item" class="ob-check">
                  <span class="ok">✓</span>{{ item }}
                </span>
              </div>
            </div>
          </div>

          <div class="ob-actions">
            <button class="btn" :disabled="consumerStarting" @click="consumerPhase = 'landing'">
              {{ t("home.consumer.onboarding.back") }}
            </button>
            <button
              class="btn btn-primary big"
              :disabled="consumerStarting"
              @click="startConsumer"
            >
              <span v-if="consumerStarting" class="spin"></span>
              <template v-if="consumerStarting">
                {{ t("home.consumer.starting") }}
              </template>
              <template v-else>
                🛡 {{ t("home.consumer.onboarding.start") }}
              </template>
            </button>
          </div>

          <p v-if="consumerError" class="ob-error">{{ consumerError }}</p>
        </div>
      </template>
    </div>

    <!-- ========= Protection Active ========= -->
    <div v-else class="consumer-card active-card">
      <div class="active-head">
        <div>
          <div class="consumer-badge">🛡 {{ t("home.consumer.badge") }}</div>
          <h2 class="consumer-title sm">{{ t("home.consumer.active.title") }}</h2>
          <p class="consumer-sub">{{ t("home.consumer.active.subtitle") }}</p>
        </div>
        <button class="btn" :disabled="pausing" @click="pauseProtection">
          {{ pausing ? t("home.consumer.active.pausing") : "⏸ " + t("home.consumer.active.pause") }}
        </button>
      </div>

      <div class="active-stats">
        <div class="active-cell">
          <div class="av">{{ allowedCount.toLocaleString() }}</div>
          <div class="ak">{{ t("home.consumer.active.allowed") }}</div>
        </div>
        <div class="active-cell">
          <div class="av amber">{{ reviewedCount.toLocaleString() }}</div>
          <div class="ak">{{ t("home.consumer.active.reviewed") }}</div>
        </div>
        <div class="active-cell">
          <div class="av red">{{ blockedCount.toLocaleString() }}</div>
          <div class="ak">{{ t("home.consumer.active.blocked") }}</div>
        </div>
      </div>

      <button class="btn btn-ghost act-more" @click="setView('history')">
        {{ t("home.consumer.active.viewActivity") }} →
      </button>

      <p v-if="consumerError" class="ob-error">{{ consumerError }}</p>
    </div>

    <!-- ========= Activity: what AI did ========= -->
    <div v-if="isRunning" class="card activity-card">
      <div class="activity-head">
        <h2>{{ t("home.consumer.activity.title") }}</h2>
        <button class="btn btn-ghost" @click="setView('history')">
          {{ t("home.consumer.activity.viewAll") }} →
        </button>
      </div>

      <p v-if="!activity.length" class="hint activity-empty">
        {{ t("home.consumer.activity.empty") }}
      </p>

      <div v-for="e in activity.slice(0, 8)" :key="e.id" class="activity-item">
        <span class="act-icon" :class="actClass(e)">{{ actIcon(e) }}</span>
        <div class="act-body">
          <div class="act-line">
            <span class="act-time">{{ e.time_hms }}</span>
            <span class="act-text">{{ describe(e) }}</span>
          </div>
          <details v-if="e.decision === 'deny' || e.result === 'blocked'" class="act-why">
            <summary>{{ t("home.consumer.activity.why") }}</summary>
            <div class="act-why-body">
              <div class="why-row">
                <span>{{ t("home.consumer.activity.rule") }}</span>
                <span class="val">{{ (e.reasons || []).join(", ") || "—" }}</span>
              </div>
              <div class="why-row">
                <span>{{ t("home.consumer.activity.decision") }}</span>
                <span class="val">DENY</span>
              </div>
            </div>
          </details>
        </div>
      </div>
    </div>

    <!-- ========= Advanced (developer workflow) ========= -->
    <details class="advanced card">
      <summary>
        <span class="advanced-title">◈ {{ t("home.consumer.advanced") }}</span>
        <span class="advanced-hint">{{ t("home.consumer.advancedHint") }}</span>
      </summary>

      <div class="advanced-body">
        <!-- Hero -->
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

        <!-- Steps card -->
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

        <!-- Actions Detected (Detection != Protection) -->
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

        <!-- Info card -->
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
    </details>
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

/* ===== Consumer entry ===== */
.consumer-card {
  position: relative;
  border-radius: var(--radius);
  padding: 32px 30px;
  background:
    radial-gradient(1000px 380px at 10% -20%, rgba(34, 197, 94, 0.16), transparent 60%),
    radial-gradient(700px 280px at 100% 0%, rgba(56, 189, 248, 0.12), transparent 55%),
    linear-gradient(160deg, rgba(22, 33, 58, 0.95), rgba(15, 23, 42, 0.8));
  border: 1px solid var(--border);
  overflow: hidden;
}

.consumer-card::after {
  content: "🛡";
  position: absolute;
  right: -10px;
  bottom: -38px;
  font-size: 210px;
  opacity: 0.045;
  pointer-events: none;
}

.consumer-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 5px 12px;
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.35);
  border-radius: 999px;
  font-size: 11.5px;
  font-weight: 700;
  letter-spacing: 0.3px;
  margin-bottom: 16px;
}

.consumer-title {
  font-size: 27px;
  line-height: 1.25;
  letter-spacing: -0.3px;
  font-weight: 700;
  max-width: 620px;
}

.consumer-title.sm {
  font-size: 20px;
}

.consumer-sub {
  margin-top: 10px;
  max-width: 640px;
  color: var(--text-dim);
  font-size: 14px;
  line-height: 1.6;
}

.consumer-cta {
  margin-top: 24px;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 14px 26px;
  border-radius: 12px;
  border: 1px solid rgba(34, 197, 94, 0.55);
  background: linear-gradient(135deg, #16a34a, #15803d);
  color: #fff;
  font-size: 15px;
  font-weight: 700;
  cursor: pointer;
  box-shadow: 0 6px 24px rgba(34, 197, 94, 0.25);
  transition: all 0.15s ease;
}

.consumer-cta:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 28px rgba(34, 197, 94, 0.35);
}

.consumer-trust {
  margin-top: 16px;
  color: var(--text-faint);
  font-size: 12px;
}

/* Onboarding */
.ob-block {
  display: flex;
  flex-direction: column;
  gap: 18px;
  margin-top: 20px;
}

.ob-row {
  display: flex;
  gap: 14px;
  align-items: flex-start;
}

.ob-step {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: var(--green-glow);
  border: 1px solid rgba(34, 197, 94, 0.4);
  color: var(--green);
  display: grid;
  place-items: center;
  font-size: 12.5px;
  font-weight: 800;
  flex-shrink: 0;
  margin-top: 2px;
}

.ob-col {
  flex: 1;
  min-width: 0;
}

.ob-label {
  font-size: 13.5px;
  font-weight: 700;
  margin-bottom: 8px;
}

.ob-option {
  display: block;
  background: var(--bg-soft);
  border: 1.5px solid rgba(34, 197, 94, 0.45);
  border-radius: 12px;
  padding: 12px 16px;
}

.ob-option-head {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13.5px;
  font-weight: 700;
}

.ob-desc {
  display: block;
  margin-top: 5px;
  font-size: 12px;
  color: var(--text-dim);
  line-height: 1.5;
}

.ob-checks {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 18px;
}

.ob-check {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
  color: var(--text-dim);
}

.ob-check .ok {
  color: var(--green);
  font-weight: 800;
}

.ob-actions {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-top: 6px;
}

.ob-error {
  margin-top: 12px;
  color: #fca5a5;
  font-size: 12.5px;
  font-family: var(--mono);
}

/* Protection active */
.active-card {
  padding-bottom: 26px;
}

.active-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.active-stats {
  margin-top: 22px;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  max-width: 640px;
}

.active-cell {
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  border-radius: 12px;
  padding: 16px 14px;
  text-align: center;
}

.active-cell .av {
  font-family: var(--mono);
  font-size: 26px;
  font-weight: 800;
  color: var(--green);
}

.active-cell .av.amber {
  color: var(--amber);
}

.active-cell .av.red {
  color: #fca5a5;
}

.active-cell .ak {
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 0.8px;
}

.act-more {
  margin-top: 18px;
}

/* Activity */
.activity-card {
  padding-bottom: 8px;
}

.activity-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.activity-head h2 {
  font-size: 16px;
}

.activity-empty {
  margin: 4px 0 10px;
}

.activity-item {
  display: flex;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid var(--border-soft);
  align-items: flex-start;
}

.activity-item:last-child {
  border-bottom: none;
}

.act-icon {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 12px;
  font-weight: 900;
  flex-shrink: 0;
  margin-top: 1px;
}

.act-icon.green {
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.35);
}

.act-icon.amber {
  background: var(--amber-glow);
  color: var(--amber);
  border: 1px solid rgba(245, 158, 11, 0.35);
}

.act-icon.red {
  background: var(--red-glow);
  color: #fca5a5;
  border: 1px solid rgba(239, 68, 68, 0.4);
}

.act-body {
  flex: 1;
  min-width: 0;
}

.act-line {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.act-time {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-faint);
  flex-shrink: 0;
}

.act-text {
  font-size: 13px;
  color: var(--text);
  line-height: 1.5;
  word-break: break-word;
}

.act-why {
  margin-top: 6px;
}

.act-why summary {
  cursor: pointer;
  font-size: 11.5px;
  color: #fca5a5;
  list-style: none;
  user-select: none;
}

.act-why summary::-webkit-details-marker {
  display: none;
}

.act-why-body {
  margin-top: 8px;
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  padding: 10px 12px;
  font-size: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.why-row {
  display: flex;
  gap: 10px;
}

.why-row span {
  color: var(--text-faint);
  flex-shrink: 0;
  width: 74px;
}

.why-row .val {
  color: var(--text);
  font-family: var(--mono);
  word-break: break-word;
}

/* Advanced */
.advanced summary {
  list-style: none;
  cursor: pointer;
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 16px 18px;
}

.advanced summary::-webkit-details-marker {
  display: none;
}

.advanced-title {
  font-size: 15px;
  font-weight: 700;
}

.advanced-hint {
  font-size: 12px;
  color: var(--text-faint);
}

.advanced-body {
  padding: 4px 18px 18px;
  display: flex;
  flex-direction: column;
  gap: 24px;
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
