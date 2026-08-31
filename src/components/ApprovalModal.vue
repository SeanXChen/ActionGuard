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

const riskLevel = computed(() => current.value?.action.risk ?? "low");

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
        <!-- Header: badges + title -->
        <header class="modal-head">
          <div class="head-badges">
            <CategoryBadge :category="current.action.category ?? 'shell'" />
            <RiskBadge :level="riskLevel" />
          </div>
          <h2>{{ t("approval.title") }}</h2>
          <p class="m-subtitle">{{ t("approval.subtitle") }}</p>
        </header>

        <!-- Action block -->
        <section class="action-block">
          <div class="action-wants">
            <span class="wants-label">{{ t("approval.wants") }}</span>
            <code class="wants-cmd">{{ actionHeadline(current.action) }}</code>
          </div>
        </section>

        <!-- Risk info card -->
        <section class="risk-card">
          <div class="risk-row">
            <span class="r-label">{{ t("approval.risk") }}</span>
            <div class="r-badges">
              <RiskBadge :level="riskLevel" />
            </div>
          </div>
          <div v-if="current.action.reasons?.length" class="risk-row">
            <span class="r-label">{{ t("approval.reason") }}</span>
            <ul class="r-reasons">
              <li v-for="(r, i) in current.action.reasons" :key="i">{{ r }}</li>
            </ul>
          </div>
          <div class="risk-row">
            <span class="r-label">{{ t("approval.source") }}</span>
            <span class="r-value">{{ current.action.agent ?? "—" }}</span>
          </div>
          <div class="risk-row">
            <span class="r-label">{{ t("approval.technical.category") }}</span>
            <span class="r-value">{{ current.action.category ?? "—" }}</span>
          </div>
          <div v-if="current.action.matched_rule" class="risk-row">
            <span class="r-label">{{ t("approval.matchedRule") }}</span>
            <code class="r-value rule-id">{{ current.action.matched_rule }}</code>
          </div>
        </section>

        <!-- Countdown -->
        <div class="countdown-row">
          <div v-if="remaining > 0" class="countdown" :class="{ urgent: remaining <= 10 }">
            {{ t("approval.dueIn") }} {{ remaining }}s
          </div>
          <div v-else class="countdown expired">{{ t("approval.timeout") }}</div>
        </div>

        <!-- Always-deny preview -->
        <transition name="slide">
          <section v-if="showAlwaysDeny && rulePreview" class="rule-preview">
            <div class="rp-title">{{ t("approval.rulePreview") }}</div>
            <pre>{{ rulePreview }}</pre>
            <p class="hint">{{ t("approval.alwaysDenyHint") }}</p>
          </section>
        </transition>

        <p v-if="errorMsg" class="error">{{ errorMsg }}</p>

        <!-- Footer actions -->
        <footer class="actions">
          <button
            class="btn btn-deny-outline"
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
            class="btn btn-allow"
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
  background: rgba(4, 8, 16, 0.82);
  backdrop-filter: blur(8px);
  display: grid;
  place-items: center;
  z-index: 120;
  padding: 24px;
}

.modal {
  width: 100%;
  max-width: 540px;
  background: #0f1117;
  border: 1px solid #1f2937;
  border-radius: 16px;
  padding: 22px 24px 20px;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(255, 255, 255, 0.03) inset;
  display: flex;
  flex-direction: column;
  gap: 14px;
  animation: popIn 200ms ease;
}

@keyframes popIn {
  from { transform: translateY(8px) scale(0.97); opacity: 0; }
  to   { transform: translateY(0) scale(1); opacity: 1; }
}

/* Header */
.modal-head {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.head-badges {
  display: inline-flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 2px;
}

.modal-head h2 {
  font-size: 17px;
  font-weight: 700;
  color: #e5e7eb;
  letter-spacing: 0.2px;
}

.m-subtitle {
  color: #64748b;
  font-size: 12px;
  max-width: 420px;
  line-height: 1.5;
}

/* Action block — "wants to run" */
.action-block {
  background: rgba(239, 68, 68, 0.06);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 10px;
  padding: 12px 14px;
}

.action-wants {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.wants-label {
  font-size: 10.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: rgba(239, 68, 68, 0.7);
}

.wants-cmd {
  font-family: var(--mono);
  font-size: 13.5px;
  color: #fca5a5;
  background: transparent;
  border: none;
  padding: 0;
  word-break: break-all;
  line-height: 1.5;
}

/* Risk info card */
.risk-card {
  background: #111827;
  border: 1px solid #1f2937;
  border-radius: 10px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.risk-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  font-size: 12.5px;
}

.r-label {
  flex: 0 0 80px;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-size: 10px;
  font-weight: 600;
  padding-top: 1px;
}

.r-value {
  flex: 1;
  color: #94a3b8;
  word-break: break-all;
}

.r-badges {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.r-reasons {
  flex: 1;
  margin: 0;
  padding-left: 14px;
  color: #94a3b8;
  font-size: 12px;
  line-height: 1.55;
}

.r-reasons li + li {
  margin-top: 3px;
}

.rule-id {
  font-family: var(--mono);
  font-size: 11px;
  color: #64748b;
  background: rgba(255, 255, 255, 0.04);
  padding: 2px 6px;
  border-radius: 4px;
  word-break: break-all;
}

/* Countdown */
.countdown-row {
  text-align: center;
}

.countdown {
  text-align: center;
  font-size: 11.5px;
  color: #64748b;
  font-family: var(--mono);
}

.countdown.urgent {
  color: #ef4444;
  font-weight: 700;
}

.countdown.expired {
  color: #ef4444;
}

/* Always-deny preview */
.rule-preview {
  background: #0c1017;
  border: 1px solid #1f2937;
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.rp-title {
  font-size: 10.5px;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}

.rule-preview pre {
  margin: 0;
  font-family: var(--mono);
  font-size: 11px;
  color: #64748b;
  white-space: pre-wrap;
  word-break: break-all;
}

.hint {
  font-size: 10.5px;
  color: #64748b;
  margin: 0;
  line-height: 1.5;
}

/* Action buttons */
.actions {
  display: flex;
  gap: 8px;
  align-items: stretch;
}

.btn {
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid;
  background: transparent;
  transition: all 0.15s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  font-family: var(--sans);
  padding: 10px 14px;
  flex-shrink: 0;
}

.btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

/* Deny — small outline (least prominent) */
.btn-deny-outline {
  border-color: rgba(239, 68, 68, 0.3);
  color: #fca5a5;
  background: rgba(239, 68, 68, 0.06);
  flex: 0 0 auto;
  min-width: 80px;
}
.btn-deny-outline:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.5);
}

/* Always Deny — secondary (middle prominence) */
.btn-secondary {
  border-color: rgba(239, 68, 68, 0.25);
  color: #94a3b8;
  background: rgba(255, 255, 255, 0.04);
  flex: 1;
  font-size: 12.5px;
}
.btn-secondary:hover:not(:disabled) {
  color: #e5e7eb;
  background: rgba(255, 255, 255, 0.07);
  border-color: rgba(239, 68, 68, 0.4);
}

.chev {
  font-size: 9px;
  color: #64748b;
}

/* Allow — largest and most prominent (green) */
.btn-allow {
  border-color: rgba(34, 197, 94, 0.4);
  color: #fff;
  background: linear-gradient(135deg, #15803d, #22c55e);
  box-shadow: 0 2px 12px rgba(34, 197, 94, 0.2);
  flex: 1.4;
  font-size: 14px;
  font-weight: 700;
  padding: 12px 20px;
}
.btn-allow:hover:not(:disabled) {
  filter: brightness(1.1);
  box-shadow: 0 4px 16px rgba(34, 197, 94, 0.3);
}
.btn-allow:active:not(:disabled) {
  transform: scale(0.98);
}

.btn-deny-confirm {
  width: 100%;
  background: linear-gradient(135deg, #b91c1c, #dc2626);
  color: #fff;
  border-color: transparent;
  font-size: 12.5px;
  box-shadow: 0 2px 12px rgba(220, 38, 38, 0.2);
  padding: 11px 16px;
}

.btn-deny-confirm code {
  font-family: var(--mono);
  font-size: 11px;
  background: rgba(0, 0, 0, 0.2);
  padding: 2px 6px;
  border-radius: 4px;
}

.always-confirm {
  display: flex;
}

.error {
  margin: 0;
  text-align: center;
  color: #ef4444;
  font-size: 12px;
}

.resolving {
  text-align: center;
  color: #64748b;
  font-size: 11.5px;
}

/* Transitions */
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
