<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch, type DeepReadonly } from "vue";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import { api } from "../api";
import type { Action, ApprovalRequest, Rule } from "../types";
import RiskBadge from "./RiskBadge.vue";
import CategoryBadge from "./CategoryBadge.vue";

const { state, resolveApproval } = useStore();
const { t } = useI18n();

// The store wraps state in `readonly()`, so the array element type is
// `DeepReadonly<ApprovalRequest>`. We deliberately keep the readonly
// modifier here — the modal never mutates the request, only reads it.
type ReadonlyApproval = DeepReadonly<ApprovalRequest>;

const current = computed<ReadonlyApproval | null>(() =>
  state.pendingApprovals.length > 0
    ? (state.pendingApprovals[0] as ReadonlyApproval)
    : null,
);

const showAlwaysDeny = ref(false);
const rulePreview = ref<Rule | null>(null);
const resolving = ref(false);
const errorMsg = ref<string | null>(null);

// Countdown to auto-deny. Re-created whenever the current approval changes.
const remaining = ref(0);
let intervalId: number | null = null;

function clearCountdown() {
  if (intervalId !== null) {
    window.clearInterval(intervalId);
    intervalId = null;
  }
}

async function loadPreviewForCurrent(req: ReadonlyApproval | null) {
  rulePreview.value = null;
  if (!req) return;
  // The backend takes a mutable Action; vue's readonly wrapper is erased at
  // runtime so the cast is sound — we're just shedding the static readonly
  // marker the store enforces.
  const action = req.action as unknown as Action;
  try {
    rulePreview.value = await api.previewLearnRule(action, "deny");
  } catch {
    // ignore — preview is best-effort
  }
}

watch(
  current,
  async (req) => {
    clearCountdown();
    showAlwaysDeny.value = false;
    errorMsg.value = null;
    if (!req) return;
    remaining.value = req.timeout_secs;
    intervalId = window.setInterval(() => {
      remaining.value = Math.max(0, remaining.value - 1);
      if (remaining.value === 0) {
        clearCountdown();
      }
    }, 1000);
    await loadPreviewForCurrent(req);
  },
  { immediate: true },
);

onBeforeUnmount(clearCountdown);

function actionHeadline(a: DeepReadonly<Action>): string {
  const verb = a.kind ?? a.action;
  const target = a.target ?? a.path ?? "";
  return `${verb} ${target}`.trim();
}

async function decide(decision: "allow" | "deny", learn: boolean) {
  if (!current.value || resolving.value) return;
  resolving.value = true;
  errorMsg.value = null;
  try {
    await resolveApproval(
      current.value.id,
      decision,
      learn,
      learn ? rulePreview.value ?? undefined : undefined,
    );
    showAlwaysDeny.value = false;
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    resolving.value = false;
  }
}
</script>

<template>
  <transition name="fade">
    <div v-if="current" class="overlay" role="dialog" aria-modal="true">
      <div class="modal">
        <header class="modal-head">
          <div class="badge-glow">
            <CategoryBadge :category="current.action.category ?? 'shell'" />
            <RiskBadge :level="current.action.risk ?? 'low'" />
          </div>
          <h2>{{ t("approval.title") }}</h2>
          <p class="m-subtitle">{{ t("approval.subtitle") }}</p>
        </header>

        <section class="action-card">
          <div class="line">
            <span class="label">{{ t("approval.source") }}</span>
            <span class="value">{{ current.action.agent ?? "—" }}</span>
          </div>
          <div class="line big">
            <span class="label">{{ t("approval.wants") }}</span>
            <code class="value cmd">{{ actionHeadline(current.action) }}</code>
          </div>
          <div class="line">
            <span class="label">{{ t("approval.risk") }}</span>
            <span class="value">
              <RiskBadge :level="current.action.risk ?? 'low'" />
            </span>
          </div>
          <div v-if="current.action.reasons?.length" class="line reasons">
            <span class="label">{{ t("approval.reason") }}</span>
            <ul class="value">
              <li v-for="(r, i) in current.action.reasons" :key="i">{{ r }}</li>
            </ul>
          </div>
          <div v-if="current.action.matched_rule" class="line">
            <span class="label">{{ t("approval.matchedRule") }}</span>
            <code class="value rule-id">{{ current.action.matched_rule }}</code>
          </div>
        </section>

        <div v-if="remaining > 0" class="countdown" :class="{ urgent: remaining <= 10 }">
          {{ t("approval.dueIn") }} {{ remaining }}s
        </div>
        <div v-else class="countdown expired">{{ t("approval.timeout") }}</div>

        <transition name="slide">
          <section v-if="showAlwaysDeny && rulePreview" class="rule-preview">
            <div class="rp-title">{{ t("approval.rulePreview") }}</div>
            <pre>{{ rulePreview }}</pre>
            <p class="hint">{{ t("approval.alwaysDenyHint") }}</p>
          </section>
        </transition>

        <p v-if="errorMsg" class="error">{{ errorMsg }}</p>

        <footer class="actions">
          <button
            class="btn btn-deny"
            :disabled="resolving"
            @click="decide('deny', false)"
          >
            {{ t("approval.deny") }}
          </button>
          <button
            class="btn btn-secondary"
            :disabled="resolving || !rulePreview"
            @click="showAlwaysDeny = !showAlwaysDeny"
          >
            {{ t("approval.alwaysDeny") }}
            <span class="chev">{{ showAlwaysDeny ? "▲" : "▼" }}</span>
          </button>
          <button
            class="btn btn-primary"
            :disabled="resolving"
            @click="decide('allow', false)"
          >
            {{ t("approval.allowOnce") }}
          </button>
        </footer>

        <div v-if="showAlwaysDeny && rulePreview" class="always-confirm">
          <button
            class="btn btn-deny-confirm"
            :disabled="resolving"
            @click="decide('deny', true)"
          >
            {{ t("approval.alwaysDeny") }} —
            <code>{{ rulePreview.match.command ?? rulePreview.match.path }}</code>
          </button>
        </div>

        <div v-if="resolving" class="resolving">{{ t("approval.resolving") }}</div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(4, 8, 16, 0.78);
  backdrop-filter: blur(10px);
  display: grid;
  place-items: center;
  z-index: 80;
  padding: 24px;
}

.modal {
  width: 100%;
  max-width: 560px;
  background: linear-gradient(160deg, var(--bg-card) 0%, var(--bg-soft) 100%);
  border: 1px solid var(--border);
  border-radius: 18px;
  padding: 24px 26px 22px;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.55);
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: popIn 220ms ease;
}

@keyframes popIn {
  from { transform: translateY(6px) scale(0.97); opacity: 0; }
  to   { transform: translateY(0) scale(1); opacity: 1; }
}

.modal-head {
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
}

.modal-head h2 {
  font-size: 18px;
  letter-spacing: 0.2px;
}

.m-subtitle {
  color: var(--text-dim);
  font-size: 12.5px;
  max-width: 460px;
  line-height: 1.5;
}

.badge-glow {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  padding: 4px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-soft);
  margin-bottom: 4px;
}

.action-card {
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--border-soft);
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.line {
  display: flex;
  align-items: baseline;
  gap: 12px;
  font-size: 13px;
}

.line .label {
  flex: 0 0 90px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-size: 10.5px;
  font-weight: 600;
}

.line .value {
  flex: 1;
  color: var(--text);
  word-break: break-all;
}

.line.big .value.cmd {
  font-family: var(--mono);
  font-size: 13.5px;
  color: var(--red);
  background: rgba(239, 68, 68, 0.08);
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.line.reasons ul {
  margin: 0;
  padding-left: 16px;
  color: var(--text-dim);
  font-size: 12.5px;
  line-height: 1.55;
}

.rule-id {
  font-family: var(--mono);
  font-size: 11.5px;
  color: var(--text-dim);
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
}

.countdown {
  text-align: center;
  font-size: 12px;
  color: var(--text-dim);
  font-family: var(--mono);
}

.countdown.urgent {
  color: var(--red);
  font-weight: 700;
}

.countdown.expired {
  color: var(--red);
}

.rule-preview {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.rp-title {
  font-size: 11px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}

.rule-preview pre {
  margin: 0;
  font-family: var(--mono);
  font-size: 11.5px;
  color: var(--text-dim);
  white-space: pre-wrap;
  word-break: break-all;
}

.hint {
  font-size: 11px;
  color: var(--text-faint);
  margin: 0;
  line-height: 1.5;
}

.actions {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 8px;
}

.btn {
  padding: 10px 14px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid var(--border);
  background: var(--bg-soft);
  color: var(--text);
  transition: all 0.15s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-family: var(--sans);
}

.btn:hover:not(:disabled) {
  transform: translateY(-1px);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(135deg, #16a34a, #22c55e);
  color: #fff;
  border-color: transparent;
}

.btn-secondary {
  background: rgba(239, 68, 68, 0.08);
  color: var(--red);
  border-color: rgba(239, 68, 68, 0.25);
}

.btn-deny {
  background: rgba(239, 68, 68, 0.15);
  color: var(--red);
  border-color: rgba(239, 68, 68, 0.3);
}

.btn-deny:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.25);
}

.btn-deny-confirm {
  width: 100%;
  background: linear-gradient(135deg, #b91c1c, #dc2626);
  color: #fff;
  border-color: transparent;
  font-size: 12.5px;
}

.btn-deny-confirm code {
  font-family: var(--mono);
  font-size: 11.5px;
  background: rgba(0, 0, 0, 0.25);
  padding: 2px 6px;
  border-radius: 4px;
}

.chev {
  font-size: 9px;
}

.always-confirm {
  display: flex;
  justify-content: center;
}

.error {
  margin: 0;
  text-align: center;
  color: var(--red);
  font-size: 12px;
}

.resolving {
  text-align: center;
  color: var(--text-faint);
  font-size: 11.5px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.18s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  max-height: 0;
  transform: translateY(-4px);
}

.slide-enter-to,
.slide-leave-from {
  opacity: 1;
  max-height: 400px;
}
</style>
