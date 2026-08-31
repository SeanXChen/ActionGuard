<script setup lang="ts">
import { useStore } from "../store";
import { useI18n } from "../i18n";

const { state } = useStore();
const { t, tf } = useI18n();

function back() { void 0; }
</script>

<template>
  <div class="view-shell">
    <div class="page-header">
      <button class="back-btn" @click="back">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="back-ico"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        {{ t("page.back") }}
      </button>
      <div class="header-meta">
        <h1 class="page-title">{{ t("review.title") }}</h1>
        <p class="page-desc">{{ t("review.desc") }}</p>
      </div>
    </div>

    <div class="card">
      <!-- Has pending approvals -->
      <div v-if="state.pendingApprovals && state.pendingApprovals.length > 0" class="pending-list">
        <div class="pending-hint">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="15" height="15"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
          {{ tf("review.pending.count", { count: String(state.pendingApprovals.length) }) }}
        </div>
        <!-- Approval cards handled by ApprovalModal.vue in App.vue -->
      </div>

      <!-- Empty state: no pending -->
      <div v-else class="empty-state">
        <div class="e-ico-wrap">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" class="e-ico"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
        </div>
        <div class="e-k">{{ t("empty.noPending.k") }}</div>
        <p class="e-v">{{ t("empty.noPending.v") }}</p>
      </div>
    </div>

    <!-- Recent decisions -->
    <div class="card">
      <div class="section-header">
        <h2 class="section-title">{{ t("review.recentDecisions") }}</h2>
      </div>
      <div v-if="state.ledger.length === 0" class="empty-state small">
        <div class="e-k">{{ t("empty.noActivity.k") }}</div>
        <p class="e-v">{{ t("empty.noActivity.v") }}</p>
      </div>
      <div v-else class="recent-decisions">
        <div class="rd-tbl-header">
          <div class="rh-col rh-time">{{ t("activity.col.time") }}</div>
          <div class="rh-col rh-act">{{ t("activity.col.action") }}</div>
          <div class="rh-col rh-decision">{{ t("review.decision") }}</div>
        </div>
        <div v-for="(entry, i) in state.ledger.slice(0, 10)" :key="i" class="rd-row">
          <div class="rh-col rh-time">
            <span class="t-time mono">{{ entry.timestamp }}</span>
          </div>
          <div class="rh-col rh-act">
            <div class="act-target mono">{{ entry.target || "—" }}</div>
            <div class="act-cat">{{ entry.category }}</div>
          </div>
          <div class="rh-col rh-decision">
            <span class="res" :class="entry.decision === 'allow' ? 'res-allow' : entry.decision === 'deny' ? 'res-block' : 'res-ask'">
              {{ entry.decision === "allow" ? t("decision.allow") : entry.decision === "deny" ? t("decision.deny") : t("decision.ask") }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.view-shell {
  max-width: 1040px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 20px 24px 28px;
}
.page-header { display: flex; flex-direction: column; gap: 10px; }
.back-btn {
  display: inline-flex; align-items: center; gap: 6px;
  background: transparent; border: 1px solid var(--border); padding: 7px 12px;
  border-radius: 8px; color: var(--text-dim); font-size: 12px; font-weight: 600;
  cursor: pointer; transition: all 0.15s; font-family: var(--sans); width: fit-content;
}
.back-btn:hover { background: rgba(255,255,255,0.03); color: var(--green); border-color: rgba(163,230,53,0.3); }
.back-ico { width: 13px; height: 13px; }
.header-meta { display: flex; flex-direction: column; gap: 4px; }
.page-title { font-size: 22px; font-weight: 800; letter-spacing: 0.2px; color: var(--text); margin: 0; }
.page-desc { color: var(--text-dim); font-size: 12.5px; line-height: 1.55; margin: 0; }

.card {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 20px 22px;
}

.pending-hint {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 10px 14px; border-radius: 10px;
  background: rgba(234,179,8,.10); border: 1px solid rgba(234,179,8,.28);
  color: #eab308; font-size: 13px; font-weight: 700;
  margin-bottom: 4px;
}

.empty-state {
  padding: 44px 16px; display: flex; flex-direction: column;
  align-items: center; gap: 10px; text-align: center;
}
.empty-state.small { padding: 28px 16px; }
.e-ico-wrap {
  width: 56px; height: 56px; border-radius: 14px;
  background: rgba(163,230,53,.07); color: var(--green);
  display: grid; place-items: center; margin-bottom: 4px;
}
.e-ico { width: 26px; height: 26px; }
.e-k { font-size: 14px; font-weight: 700; color: var(--text); }
.e-v { font-size: 12px; color: var(--text-dim); line-height: 1.5; margin: 0; max-width: 400px; }

.section-header { margin-bottom: 16px; }
.section-title { font-size: 15px; font-weight: 700; color: var(--text); margin: 0; }

/* Recent decisions */
.recent-decisions { display: flex; flex-direction: column; gap: 0; }
.rd-tbl-header {
  display: grid; grid-template-columns: 180px 1fr 130px;
  gap: 12px; padding: 8px 12px;
  border-bottom: 1px solid var(--border-soft); margin-bottom: 4px;
}
.rh-col {
  font-size: 10.5px; font-weight: 700;
  text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-faint);
}
.rd-row {
  display: grid; grid-template-columns: 180px 1fr 130px;
  gap: 12px; align-items: center; padding: 10px 12px;
  border-radius: 9px; transition: background 0.12s;
}
.rd-row:hover { background: rgba(255,255,255,0.02); }
.rh-time { white-space: nowrap; }
.rh-act { min-width: 0; }
.rh-decision { }
.t-time { color: var(--text-dim); font-size: 12px; }
.act-target { color: var(--text); font-size: 12.5px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.act-cat { margin-top: 3px; color: var(--text-faint); font-size: 10.5px; text-transform: uppercase; letter-spacing: 0.4px; font-family: var(--mono); }

.res {
  display: inline-flex; align-items: center; gap: 6px; padding: 3px 9px 3px 7px;
  border-radius: 7px; font-size: 11.5px; font-weight: 700;
  letter-spacing: 0.2px; font-family: var(--mono);
}
.res.res-allow { background: rgba(163,230,53,.10); color: var(--green); border: 1px solid rgba(163,230,53,.28); }
.res.res-block { background: rgba(239,68,68,.10);  color: #f87171; border: 1px solid rgba(239,68,68,.28); }
.res.res-ask   { background: rgba(234,179,8,.10);  color: #eab308; border: 1px solid rgba(234,179,8,.28); }
.mono { font-family: var(--mono); }
</style>
