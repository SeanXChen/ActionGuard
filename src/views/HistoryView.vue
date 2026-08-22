<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api } from "../api";
import ActionList from "../components/ActionList.vue";
import RiskBadge from "../components/RiskBadge.vue";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import type { SessionDetails, SessionSummary } from "../types";

const { state, refreshSessions } = useStore();
const { t, lang } = useI18n();

const detail = ref<SessionDetails | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

async function load() {
  loading.value = true;
  error.value = null;
  await refreshSessions();
  loading.value = false;
}

onMounted(load);

function groupLabel(dateStr: string): string {
  const d = new Date(dateStr.replace(" ", "T"));
  const today = new Date();
  const yest = new Date();
  yest.setDate(today.getDate() - 1);
  const same = (a: Date, b: Date) =>
    a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
  if (same(d, today)) return t("history.today");
  if (same(d, yest)) return t("history.yesterday");
  return d.toLocaleDateString(lang.value === "zh" ? "zh-CN" : undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

const grouped = computed(() => {
  const map = new Map<string, SessionSummary[]>();
  for (const s of state.sessions) {
    const g = groupLabel(s.started_at);
    if (!map.has(g)) map.set(g, []);
    map.get(g)!.push(s);
  }
  return [...map.entries()];
});

const stats = computed(() => state.paraStats);

const rateColor = computed(() => {
  const r = stats.value.rate;
  if (r >= 50) return "red";
  if (r >= 20) return "amber";
  return "green";
});

function fmtDuration(secs: number) {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}m ${s.toString().padStart(2, "0")}s`;
}

function fmtTime(str: string) {
  const d = new Date(str.replace(" ", "T"));
  return d.toLocaleTimeString(lang.value === "zh" ? "zh-CN" : undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

async function openDetail(id: string) {
  detail.value = null;
  try {
    detail.value = await api.getSession(id);
  } catch (e) {
    error.value = String(e);
  }
}

const undoing = ref(false);

async function undo(sessionId: string) {
  if (undoing.value) return;
  undoing.value = true;
  error.value = null;
  try {
    const res = await api.undoSession(sessionId);
    await load();
    if (detail.value?.summary.id === sessionId) {
      await openDetail(sessionId);
    }
    const lines = [
      `${t("history.undoDone")}`,
      "",
      `${t("history.detail.actions").replace(/\).*/, "")}: ${res.restored_files})`,
    ];
    const extra = [
      res.deleted_files ? `- Deleted ${res.deleted_files} file(s) created after snapshot` : "",
      res.removed_dirs ? `- Removed ${res.removed_dirs} dir(s)` : "",
      res.skipped ? `- Skipped ${res.skipped} unchanged file(s)` : "",
    ].filter(Boolean);
    alert([...lines, ...extra].join("\n"));
  } catch (e) {
    error.value = String(e);
  } finally {
    undoing.value = false;
  }
}
</script>

<template>
  <div class="history">
    <div class="page-head">
      <div>
        <h1 class="title">{{ t("history.title") }}</h1>
        <p class="subtitle">
          {{ t("history.rateNote") }}
        </p>
      </div>
      <button class="btn btn-ghost" @click="load">
        ↻
      </button>
    </div>

    <div class="stats">
      <div class="stat card">
        <div class="sv">{{ stats.n }}</div>
        <div class="sk">{{ t("history.stat.sessions") }}</div>
      </div>
      <div class="stat card">
        <div class="sv red">{{ stats.high }}</div>
        <div class="sk">{{ t("history.stat.high") }}</div>
      </div>
      <div class="stat card">
        <div class="sv purple">{{ stats.critical }}</div>
        <div class="sk">{{ t("history.stat.critical") }}</div>
      </div>
      <div class="stat card">
        <div class="sv amber">{{ stats.actionsBlocked.toLocaleString() }}</div>
        <div class="sk">{{ t("history.stat.blocked") }}</div>
      </div>
      <div class="stat card rate" :class="rateColor">
        <div class="sv">{{ stats.rate }}<small>%</small></div>
        <div class="sk">{{ t("history.stat.rate") }}</div>
      </div>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="!loading && state.sessions.length === 0" class="empty card">
      <div class="big">◈</div>
      <p>{{ t("history.empty") }}</p>
    </div>

    <section v-for="[label, items] in grouped" :key="label" class="day">
      <h3>{{ label }}</h3>
      <div class="row-list">
        <div
          v-for="s in items"
          :key="s.id"
          class="row session-row card-sm"
          @click="openDetail(s.id)"
        >
          <span class="sid mono">#{{ s.num.toString().padStart(5, "0") }}</span>
          <span class="time mono">{{ fmtTime(s.started_at) }}</span>
          <RiskBadge :level="s.risk" />
          <span
            v-if="s.mode"
            class="row-mode"
            :class="s.mode === 'observe' ? 'row-mode-a' : 'row-mode-b'"
          >
            {{ s.mode === 'observe' ? t('session.mode.badgeObserve') : t('session.mode.badgeProtected') }}
          </span>
          <span class="counts mono mini">
            <span class="c create">{{ s.counts.create }}</span>
            <span class="c modify">{{ s.counts.modify }}</span>
            <span class="c delete">{{ s.counts.delete }}</span>
            <span class="c rename">{{ s.counts.rename }}</span>
          </span>
          <span class="sensitive-count" v-if="s.sensitive_count" title="sensitive">
            ⚠ {{ s.sensitive_count }}
          </span>
          <span class="dur mono">{{ fmtDuration(s.duration_secs) }}</span>
          <span v-if="s.undone" class="undone tag tag-flag">{{ t("history.tag.undone") }}</span>
          <span v-else-if="s.status === 'denied'" class="tag tag-flag">{{
            t("history.tag.denied")
          }}</span>
          <span class="view">{{ t("history.view") }}</span>
        </div>
      </div>
    </section>

    <div v-if="detail" class="card detail">
      <div class="detail-head">
        <div>
          <div class="detail-kicker">
            <span class="sid-mini mono">#{{ detail.summary.num.toString().padStart(5, "0") }}</span>
            <span class="dur-mini mono">{{ fmtDuration(detail.summary.duration_secs) }}</span>
          </div>
          <h2>
            {{ t("session.chip") }} #{{ detail.summary.num.toString().padStart(5, "0") }}
          </h2>
          <p class="sub-line mono">{{ detail.summary.workspace }}</p>
          <p class="sub-line-2">
            {{ detail.actions.length }} {{ t("history.detail.actions") }}
            <template v-if="detail.summary.sensitive_count">
              · {{ detail.summary.sensitive_count }} {{ t("history.detail.sensitive") }}
            </template>
            <template v-if="detail.summary.outside_count">
              · {{ detail.summary.outside_count }} {{ t("history.detail.outside") }}
            </template>
          </p>
        </div>
        <div class="detail-meta">
          <RiskBadge :level="detail.summary.risk" />
          <span
            v-if="detail.summary.mode"
            class="row-mode"
            :class="detail.summary.mode === 'observe' ? 'row-mode-a' : 'row-mode-b'"
          >
            {{ detail.summary.mode === 'observe' ? t('session.mode.observe') : t('session.mode.protected') }}
          </span>
          <span v-if="detail.summary.undone" class="tag tag-flag">{{ t("history.tag.undone") }}</span>
          <span v-else-if="detail.summary.status === 'denied'" class="tag tag-flag">
            {{ t("history.tag.denied") }}
          </span>
        </div>
      </div>
      <ActionList :actions="detail.actions" :limit="100" />
      <div class="detail-actions">
        <button
          class="btn btn-danger"
          :disabled="undoing || detail.summary.undone"
          @click="undo(detail.summary.id)"
        >
          <span v-if="undoing" class="spin small"></span>
          {{ detail.summary.undone ? t("history.undone") : `↶ ${t("history.undo")}` }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history {
  max-width: 960px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.page-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.page-head .title {
  font-size: 22px;
  font-weight: 700;
}

.page-head .subtitle {
  color: var(--text-dim);
  font-size: 12.5px;
  max-width: 620px;
  line-height: 1.5;
  margin-top: 4px;
}

.stats {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
}

@media (max-width: 760px) {
  .stats {
    grid-template-columns: repeat(2, 1fr);
  }
}

.stat {
  padding: 14px 18px;
  position: relative;
  overflow: hidden;
}

.stat::after {
  content: "";
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 3px;
  opacity: 0.6;
}

.stat:nth-child(1)::after { background: var(--green); }
.stat:nth-child(2)::after { background: var(--red); }
.stat:nth-child(3)::after { background: var(--amber); }
.stat.rate::after { background: var(--green); }
.stat.rate.amber::after { background: var(--amber); }
.stat.rate.red::after { background: var(--red); }

.sv {
  font-family: var(--mono);
  font-size: 24px;
  font-weight: 700;
  line-height: 1;
}

.sv small {
  font-size: 14px;
  opacity: 0.7;
  margin-left: 2px;
}

.sv.red { color: #fca5a5; }
.sv.amber { color: var(--amber); }
.sv.green { color: var(--green); }
.sv.purple { color: var(--purple); }

.sk {
  font-size: 11px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 0.8px;
  margin-top: 4px;
}

.error {
  color: #fca5a5;
  font-size: 12.5px;
  font-family: var(--mono);
}

.empty {
  text-align: center;
  padding: 44px 24px;
  color: var(--text-dim);
}

.empty .big {
  font-size: 34px;
  margin-bottom: 10px;
  color: var(--text-faint);
}

.day h3 {
  font-size: 11.5px;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: var(--text-faint);
  margin: 18px 6px 8px;
}

.card-sm {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 10px 14px;
}

.session-row {
  cursor: pointer;
  transition: all 0.12s ease;
}

.session-row:hover {
  background: var(--bg-card-hover);
  border-color: var(--green);
  transform: translateY(-1px);
}

.sid {
  color: var(--text);
  font-weight: 700;
  width: 84px;
}

.time {
  color: var(--text-dim);
  width: 52px;
  font-size: 12px;
}

.counts.mini {
  flex: 1;
  font-size: 11.5px;
  color: var(--text-faint);
  display: inline-flex;
  gap: 8px;
  font-family: var(--mono);
}

.counts.mini .create::before { content: "C "; color: var(--blue); }
.counts.mini .modify::before { content: "M "; color: var(--green); }
.counts.mini .delete::before { content: "D "; color: #fca5a5; }
.counts.mini .rename::before { content: "R "; color: var(--amber); }

.sensitive-count {
  font-size: 11px;
  color: #fcd34d;
  font-weight: 700;
  font-family: var(--mono);
}

.dur {
  color: var(--text-faint);
  width: 64px;
  font-size: 12px;
}

.view {
  color: var(--green);
  font-size: 12px;
  opacity: 0.6;
}
.session-row:hover .view { opacity: 1; }

.undone {
  margin-left: 4px;
}

.detail {
  display: flex;
  flex-direction: column;
  gap: 16px;
  position: relative;
}

.detail-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.detail-kicker {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.sid-mini {
  color: var(--green);
  font-size: 11.5px;
  font-weight: 700;
}

.dur-mini {
  color: var(--text-faint);
  font-size: 11.5px;
}

.detail h2 {
  font-size: 18px;
}

.sub-line {
  color: var(--text-faint);
  font-size: 12px;
  margin-top: 2px;
}

.sub-line-2 {
  color: var(--text-dim);
  font-size: 13px;
  margin-top: 4px;
}

.detail-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.detail-actions {
  display: flex;
  gap: 10px;
  padding-top: 4px;
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

.row-mode {
  display: inline-flex;
  align-items: center;
  padding: 1px 7px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.6px;
  font-family: var(--mono);
}

.row-mode-a {
  background: rgba(56, 189, 248, 0.15);
  color: var(--blue);
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.row-mode-b {
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.35);
}
</style>
