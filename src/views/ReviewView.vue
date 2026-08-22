<script setup lang="ts">
import { computed, ref } from "vue";
import { api } from "../api";
import ActionList from "../components/ActionList.vue";
import RiskBadge from "../components/RiskBadge.vue";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import type { BatchData } from "../types";

const { state, setView } = useStore();
const { t } = useI18n();

const pending = computed<BatchData | null>(() => state.pendingBatch as BatchData | null);

const batchTotal = computed(() => {
  const c = pending.value?.counts;
  if (!c) return 0;
  return c.create + c.modify + c.delete + c.rename;
});

const busy = ref(false);
const error = ref<string | null>(null);

async function allow() {
  if (!pending.value || busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    await api.allowBatch();
    setView("session");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function deny() {
  if (!pending.value || busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    await api.denyBatch();
    setView("home");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="!pending || !state.session" class="empty card">
    <div class="big-check">✓</div>
    <p class="empty-title">{{ t("review.empty") }}</p>
    <button class="btn" @click="setView('session')">→ {{ t("review.back") }}</button>
  </div>

  <div v-else class="review">
    <div class="warn-head card">
      <div class="warn-row">
        <RiskBadge level="high" />
        <div class="sigil">⚠</div>
      </div>
      <h1 class="title">{{ t("review.head.title") }}</h1>
      <p class="subtitle">
        {{ t("review.head.subtitle1") }} <strong>{{ batchTotal }}</strong>
        {{ t("review.head.subtitle2") }}
      </p>
    </div>

    <div class="count-grid">
      <div class="count-cell count-create">
        <div class="k">{{ t("review.counts.create") }}</div>
        <div class="v">{{ pending.counts.create }}</div>
      </div>
      <div class="count-cell count-modify">
        <div class="k">{{ t("review.counts.modify") }}</div>
        <div class="v">{{ pending.counts.modify }}</div>
      </div>
      <div class="count-cell count-delete">
        <div class="k">{{ t("review.counts.delete") }}</div>
        <div class="v">{{ pending.counts.delete }}</div>
      </div>
      <div class="count-cell count-rename">
        <div class="k">{{ t("review.counts.rename") }}</div>
        <div class="v">{{ pending.counts.rename }}</div>
      </div>
    </div>

    <div v-if="pending.risk.sensitive.length" class="warn-box sensitive card">
      <div class="wb-title">⚠ {{ t("review.sensitive.title") }}
        <span class="count-pill">{{ pending.risk.sensitive.length }}</span>
      </div>
      <div class="wb-paths">
        <span v-for="p in pending.risk.sensitive" :key="p" class="path-chip">{{ p }}</span>
      </div>
      <p class="wb-note">
        ⚡ {{ t("review.sensitive.note") }}
      </p>
    </div>

    <div v-if="pending.risk.outside.length" class="warn-box outside card">
      <div class="wb-title">
        ⚠ {{ t("review.outside.title") }}
        <span class="count-pill danger">{{ pending.risk.outside.length }}</span>
      </div>
      <div class="wb-paths">
        <span v-for="p in pending.risk.outside" :key="p" class="path-chip danger">{{ p }}</span>
      </div>
      <p class="wb-note">
        ⚡ {{ t("review.outside.note") }}
      </p>
    </div>

    <div class="card reasons">
      <h3>{{ t("review.reasons.title") }}</h3>
      <ul>
        <li v-for="(r, i) in pending.risk.reasons" :key="i">
          <span class="bull">•</span>
          {{ r }}
        </li>
      </ul>
    </div>

    <div class="card">
      <h3>{{ t("review.changes.title") }}</h3>
      <ActionList :actions="pending.actions" :limit="100" />
    </div>

    <div class="buttons card">
      <div class="brow">
        <button class="btn btn-primary big" :disabled="busy" @click="allow">
          <span v-if="busy" class="spin small"></span>
          {{ busy ? t("review.working") : `✓ ${t("review.allow")}` }}
        </button>
        <button class="btn btn-danger big" :disabled="busy" @click="deny">
          <span v-if="busy" class="spin small"></span>
          {{ busy ? t("review.restoring") : `↶ ${t("review.deny")}` }}
        </button>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <p class="disclaimer">
      • {{ t("review.disclaimer1") }}<br />
      • {{ t("review.disclaimer2") }}
    </p>
  </div>
</template>

<style scoped>
.review {
  max-width: 900px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.empty {
  max-width: 520px;
  margin: 60px auto;
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
  padding: 42px 30px;
}

.big-check {
  width: 62px;
  height: 62px;
  border-radius: 50%;
  background: var(--green-glow);
  border: 1px solid rgba(34, 197, 94, 0.4);
  color: var(--green);
  display: grid;
  place-items: center;
  font-size: 32px;
  font-weight: 900;
}

.empty-title {
  color: var(--text);
  font-size: 15px;
}

.warn-head {
  position: relative;
  overflow: hidden;
}

.warn-head::after {
  content: "";
  position: absolute;
  inset: 0;
  background:
    radial-gradient(500px 200px at 100% 0%, rgba(239, 68, 68, 0.18), transparent 55%),
    radial-gradient(500px 200px at 0% 100%, rgba(245, 158, 11, 0.12), transparent 55%);
  pointer-events: none;
}

.warn-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.sigil {
  font-size: 18px;
  color: #fca5a5;
  font-weight: 900;
}

.warn-head .title {
  font-size: 26px;
  letter-spacing: 0.5px;
  color: #fecaca;
}

.subtitle {
  color: var(--text-dim);
  font-size: 14px;
  margin-top: 4px;
  position: relative;
  z-index: 1;
}

.subtitle strong {
  font-family: var(--mono);
  color: var(--text);
  font-size: 20px;
  margin: 0 4px;
}

.count-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.count-cell {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px 18px;
  position: relative;
  overflow: hidden;
}
.count-cell::after {
  content: "";
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 3px;
  opacity: 0.7;
}
.count-create::after { background: var(--blue); }
.count-modify::after { background: var(--green); }
.count-delete::after { background: var(--red); }
.count-rename::after { background: var(--amber); }

.count-cell .k {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: var(--text-faint);
  margin-bottom: 8px;
  font-family: var(--mono);
}

.count-cell .v {
  font-family: var(--mono);
  font-size: 30px;
  font-weight: 700;
  line-height: 1;
}
.count-create .v { color: var(--blue); }
.count-modify .v { color: var(--green); }
.count-delete .v { color: var(--red); }
.count-rename .v { color: var(--amber); }

.warn-box {
  border-radius: var(--radius);
  padding: 18px 20px;
  border: 1px solid;
}

.warn-box.sensitive {
  border-color: rgba(245, 158, 11, 0.45);
  background: var(--amber-glow);
}

.warn-box.outside {
  border-color: rgba(239, 68, 68, 0.45);
  background: var(--red-glow);
}

.wb-title {
  font-weight: 700;
  font-size: 14px;
  margin-bottom: 10px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.count-pill {
  display: inline-grid;
  place-items: center;
  min-width: 24px;
  height: 20px;
  padding: 0 7px;
  border-radius: 999px;
  background: rgba(245, 158, 11, 0.3);
  border: 1px solid rgba(245, 158, 11, 0.4);
  color: #fcd34d;
  font-family: var(--mono);
  font-size: 11px;
  font-weight: 800;
}

.count-pill.danger {
  background: rgba(239, 68, 68, 0.3);
  border-color: rgba(239, 68, 68, 0.4);
  color: #fca5a5;
}

.wb-paths {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.path-chip {
  font-family: var(--mono);
  font-size: 12px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--border);
  padding: 3px 9px;
  border-radius: 5px;
  color: #fcd34d;
}

.path-chip.danger {
  color: #fca5a5;
}

.wb-note {
  margin-top: 10px;
  font-size: 12.5px;
  color: var(--text-dim);
}

.reasons h3,
.card h3 {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 1.2px;
  color: var(--text-dim);
  margin-bottom: 10px;
}

.reasons ul {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13.5px;
}

.reasons li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  line-height: 1.45;
}

.reasons .bull {
  color: var(--red);
  font-weight: 900;
}

.buttons {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.brow {
  display: flex;
  gap: 14px;
  flex-wrap: wrap;
}

.big {
  padding: 13px 26px;
  font-size: 14.5px;
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
}

.disclaimer {
  color: var(--text-faint);
  font-size: 12px;
  line-height: 1.6;
  padding: 0 2px;
}
</style>
