<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "../i18n";
import { useStore } from "../store";
import type { DictKey } from "../i18n";

const { t } = useI18n();
const { setView } = useStore();

function back() { setView("dashboard"); }

/* ---------- Tabs ---------- */
const tabs = ref<Array<{ key: string; labelKey: DictKey }>>([
  { key: "rules",       labelKey: "policies.tabs.rules" },
  { key: "trust",       labelKey: "policies.tabs.trustZones" },
  { key: "risk",        labelKey: "policies.tabs.riskLevels" },
  { key: "sets",        labelKey: "policies.tabs.policySets" },
]);
const activeTab = ref("rules");

/* ================================================================
   TAB 1: RULES
================================================================ */
type Risk = "critical" | "high" | "medium";
const RISK_CLASS: Record<Risk, string> = {
  critical: "risk-critical",
  high: "risk-high",
  medium: "risk-medium",
};
const RISK_KEY: Record<Risk, DictKey> = {
  critical: "risk.critical",
  high: "risk.high",
  medium: "risk.medium",
};

type RuleRow = {
  key: string;
  titleKey: DictKey;
  descKey: DictKey;
  risk: Risk;
  on: boolean;
};

const rules = ref<RuleRow[]>([
  { key: "critical-paths", titleKey: "policies.rule.criticalPaths",      descKey: "policies.rule.criticalPaths.desc",      risk: "critical", on: true  },
  { key: "sensitive-files", titleKey: "policies.rule.sensitiveFiles",  descKey: "policies.rule.sensitiveFiles.desc",  risk: "high",    on: true  },
  { key: "git-safety",     titleKey: "policies.rule.gitSafety",         descKey: "policies.rule.gitSafety.desc",         risk: "high",    on: true  },
  { key: "package",        titleKey: "policies.rule.packageSafety",     descKey: "policies.rule.packageSafety.desc",     risk: "medium",  on: true  },
]);
</script>

<template>
  <div class="view-shell">
    <div class="page-header">
      <button class="back-btn" @click="back">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="back-ico">
          <path d="M19 12H5M12 19l-7-7 7-7"/>
        </svg>
        {{ t("page.back") }}
      </button>
      <div class="header-meta">
        <h1 class="page-title">{{ t("policies.title") }}</h1>
        <p class="page-desc">{{ t("page.policies.desc") }}</p>
      </div>
    </div>

    <div class="card content-card policies-card">
      <div class="card-head">
        <div class="tabs">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            class="tab"
            :class="{ active: activeTab === tab.key }"
            @click="activeTab = tab.key"
          >{{ t(tab.labelKey) }}</button>
        </div>
      </div>

      <!-- TAB 1: RULES -->
      <div v-if="activeTab === 'rules'" class="rules">
        <div v-for="r in rules" :key="r.key" class="rule-row">
          <div class="rule-icon" :class="RISK_CLASS[r.risk]">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="11" width="18" height="11" rx="2"/>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            </svg>
          </div>
          <div class="rule-body">
            <div class="rule-head">
              <div class="rule-title">{{ t(r.titleKey) }}</div>
              <div class="rule-meta">
                <span class="pill" :class="RISK_CLASS[r.risk]">{{ t(RISK_KEY[r.risk]) }}</span>
              </div>
            </div>
            <div class="rule-desc">{{ t(r.descKey) }}</div>
          </div>
          <label class="switch">
            <input type="checkbox" v-model="r.on" />
            <span class="slider"></span>
          </label>
        </div>
      </div>

      <!-- TAB 2: TRUST ZONES -->
      <div v-else-if="activeTab === 'trust'" class="trust-tab">
        <div class="trust-intro">
          <div class="trust-intro-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            </svg>
          </div>
          <div>
            <div class="trust-intro-title">{{ t("policies.trust.title") }}</div>
            <div class="trust-intro-desc">{{ t("policies.trust.desc") }}</div>
          </div>
        </div>
        <div class="trust-empty">
          <div class="trust-empty-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            </svg>
          </div>
          <div class="trust-empty-title">{{ t("policies.trust.empty") }}</div>
          <div class="trust-empty-desc">{{ t("policies.trust.emptyDesc") }}</div>
        </div>
      </div>

      <!-- TAB 3: RISK LEVELS -->
      <div v-else-if="activeTab === 'risk'" class="risk-tab">
        <div class="tab-intro">
          <div class="tab-intro-title">{{ t("policies.risk.title") }}</div>
          <div class="tab-intro-desc">{{ t("policies.risk.desc") }}</div>
        </div>
        <div class="risk-levels">
          <div class="risk-card">
            <div class="risk-card-head">
              <span class="risk-dot" style="background: #f87171;"></span>
              <span class="risk-label">{{ t("policies.risk.strict") }}</span>
            </div>
            <div class="risk-card-desc">{{ t("policies.risk.strictDesc") }}</div>
            <button class="btn risk-apply-btn" disabled>{{ t("policies.risk.apply") }}</button>
          </div>
          <div class="risk-card applied">
            <div class="risk-card-head">
              <span class="risk-dot" style="background: #fb923c;"></span>
              <span class="risk-label">{{ t("policies.risk.standard") }}</span>
              <span class="active-badge">{{ t("policies.risk.current") }}</span>
            </div>
            <div class="risk-card-desc">{{ t("policies.risk.standardDesc") }}</div>
          </div>
          <div class="risk-card">
            <div class="risk-card-head">
              <span class="risk-dot" style="background: #4ade80;"></span>
              <span class="risk-label">{{ t("policies.risk.relaxed") }}</span>
            </div>
            <div class="risk-card-desc">{{ t("policies.risk.relaxedDesc") }}</div>
            <button class="btn risk-apply-btn" disabled>{{ t("policies.risk.apply") }}</button>
          </div>
        </div>
      </div>

      <!-- TAB 4: POLICY SETS -->
      <div v-else-if="activeTab === 'sets'" class="sets-tab">
        <div class="tab-intro">
          <div class="tab-intro-desc">{{ t("policies.sets.desc") }}</div>
        </div>
        <div class="policy-sets">
          <div class="ps-card">
            <div class="ps-card-top">
              <div class="ps-icon" style="background: rgba(168,85,247,.12); color: #c084fc;">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/>
                </svg>
              </div>
              <div class="ps-info">
                <div class="ps-name">{{ t("policies.sets.development") }}</div>
                <div class="ps-desc">{{ t("policies.sets.developmentDesc") }}</div>
              </div>
            </div>
            <button class="btn ps-activate-btn" disabled>{{ t("policies.sets.activate") }}</button>
          </div>
          <div class="ps-card">
            <div class="ps-card-top">
              <div class="ps-icon" style="background: rgba(239,68,68,.12); color: #f87171;">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                </svg>
              </div>
              <div class="ps-info">
                <div class="ps-name">{{ t("policies.sets.production") }}</div>
                <div class="ps-desc">{{ t("policies.sets.productionDesc") }}</div>
              </div>
            </div>
            <button class="btn ps-activate-btn" disabled>{{ t("policies.sets.activate") }}</button>
          </div>
          <div class="ps-card applied">
            <div class="ps-card-top">
              <div class="ps-icon" style="background: rgba(163,230,53,.12); color: #4ade80;">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.26.604.852.997 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
                </svg>
              </div>
              <div class="ps-info">
                <div class="ps-name">{{ t("policies.sets.custom") }}</div>
                <div class="ps-desc">{{ t("policies.sets.customDesc") }}</div>
              </div>
            </div>
            <span class="active-badge" style="background: rgba(34,197,94,.12); color: #4ade80; border: 1px solid rgba(34,197,94,.3); padding: 3px 10px; border-radius: 999px; font-size: 11px; font-weight: 700; display: inline-block; width: fit-content;">{{ t("policies.sets.active") }}</span>
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
  gap: 16px;
  padding: 20px 24px 28px;
}
.page-header {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 1px solid var(--border);
  padding: 7px 12px;
  border-radius: 8px;
  color: var(--text-dim);
  font-size: 12.5px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
  font-family: var(--sans);
  width: fit-content;
}
.back-btn:hover {
  background: rgba(255,255,255,0.03);
  color: var(--green);
  border-color: rgba(163,230,53,0.3);
}
.back-ico { width: 14px; height: 14px; }
.header-meta { display: flex; flex-direction: column; gap: 4px; }
.page-title { font-size: 24px; font-weight: 800; letter-spacing: 0.2px; color: var(--text); margin: 0; }
.page-desc { color: var(--text-dim); font-size: 13px; max-width: 620px; line-height: 1.55; margin: 0; }
.policies-card { padding: 16px 18px 20px; }
.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 4px 2px 14px;
  margin-bottom: 6px;
  border-bottom: 1px solid var(--border-soft);
}
.tabs {
  display: inline-flex;
  align-items: center;
  background: rgba(255,255,255,0.025);
  border: 1px solid var(--border);
  padding: 4px;
  border-radius: 10px;
  gap: 2px;
}
.tab {
  background: transparent;
  border: none;
  padding: 6px 13px;
  border-radius: 7px;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-dim);
  cursor: pointer;
  font-family: var(--sans);
  transition: all 0.15s;
}
.tab:hover { color: var(--text); }
.tab.active {
  background: linear-gradient(180deg, rgba(163,230,53,0.14), rgba(163,230,53,0.05));
  color: var(--text);
  box-shadow: 0 0 0 1px rgba(163,230,53,0.35), inset 0 0 0 1px rgba(163,230,53,0.08);
}

/* ---- Rules ---- */
.rules {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-top: 8px;
}
.rule-row {
  display: grid;
  grid-template-columns: 42px 1fr auto;
  gap: 14px;
  align-items: center;
  padding: 14px 15px 14px 14px;
  background: rgba(255,255,255,0.018);
  border: 1px solid var(--border);
  border-radius: 12px;
  transition: all 0.18s;
}
.rule-row:hover { border-color: rgba(163,230,53,0.25); background: rgba(163,230,53,0.025); }
.rule-icon {
  width: 42px;
  height: 42px;
  border-radius: 11px;
  display: grid;
  place-items: center;
  background: rgba(163,230,53,0.08);
  color: var(--green);
}
.rule-icon svg { width: 20px; height: 20px; }
.rule-icon.risk-critical { background: rgba(239,68,68,0.10); color: #f87171; }
.rule-icon.risk-high     { background: rgba(251,146,60,0.10); color: #fb923c; }
.rule-icon.risk-medium   { background: rgba(234,179,8,0.10);  color: #eab308; }
.rule-body { min-width: 0; }
.rule-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 4px;
  flex-wrap: wrap;
}
.rule-title { font-size: 14.5px; font-weight: 700; color: var(--text); letter-spacing: 0.1px; }
.rule-meta { display: inline-flex; align-items: center; gap: 6px; }
.rule-desc { color: var(--text-dim); font-size: 12.5px; line-height: 1.55; }
.pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2.5px 8px;
  border-radius: 999px;
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.4px;
  text-transform: uppercase;
  font-family: var(--mono);
}
.pill.risk-critical { background: rgba(239,68,68,0.13); color: #f87171; border: 1px solid rgba(239,68,68,0.30); }
.pill.risk-high     { background: rgba(251,146,60,0.13); color: #fb923c; border: 1px solid rgba(251,146,60,0.28); }
.pill.risk-medium   { background: rgba(234,179,8,0.12);  color: #eab308; border: 1px solid rgba(234,179,8,0.28); }
.switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
  flex-shrink: 0;
}
.switch input { opacity: 0; width: 0; height: 0; }
.slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background: rgba(255,255,255,0.08);
  border: 1px solid var(--border);
  transition: 0.18s;
  border-radius: 999px;
}
.slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 2.5px;
  top: 2.5px;
  background: #e5e7eb;
  transition: 0.18s;
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0,0,0,0.35);
}
.switch input:checked + .slider {
  background: linear-gradient(180deg, var(--green), var(--green-soft));
  border-color: rgba(163,230,53,0.5);
}
.switch input:checked + .slider:before { transform: translateX(18px); background: #0d1604; }

/* ---- Trust Zones ---- */
.trust-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;
}
.trust-intro {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 14px 16px;
  background: rgba(56,189,248,0.05);
  border: 1px solid rgba(56,189,248,0.2);
  border-radius: 12px;
}
.trust-intro-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  flex-shrink: 0;
  background: rgba(56,189,248,0.12);
  color: #38bdf8;
  display: grid;
  place-items: center;
}
.trust-intro-icon svg { width: 18px; height: 18px; }
.trust-intro-title { font-size: 14px; font-weight: 700; color: var(--text); margin-bottom: 3px; }
.trust-intro-desc { font-size: 12px; color: var(--text-dim); line-height: 1.55; }
.add-dir-row {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;
}
.add-dir-input-wrap {
  flex: 1;
  min-width: 280px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255,255,255,0.025);
  border: 1px solid var(--border);
  border-radius: 9px;
  padding: 8px 12px;
  transition: border-color 0.15s;
}
.add-dir-input-wrap:focus-within { border-color: rgba(163,230,53,0.4); }
.dir-icon { color: var(--text-faint); flex-shrink: 0; }
.dir-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--text);
  font-size: 13px;
  font-family: var(--mono);
}
.dir-input::placeholder { color: var(--text-faint); font-family: var(--sans); }
.dir-error {
  font-size: 12px;
  color: #f87171;
  padding: 6px 12px;
  background: rgba(239,68,68,0.07);
  border: 1px solid rgba(239,68,68,0.2);
  border-radius: 8px;
}
.dir-list { display: flex; flex-direction: column; gap: 6px; }
.dir-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: rgba(255,255,255,0.02);
  border: 1px solid var(--border);
  border-radius: 9px;
  transition: border-color 0.15s;
}
.dir-row:hover { border-color: rgba(239,68,68,0.35); background: rgba(239,68,68,0.03); }
.dir-row-icon { color: var(--text-faint); flex-shrink: 0; }
.dir-path { flex: 1; font-family: var(--mono); font-size: 12.5px; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dir-remove {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 7px;
  background: transparent;
  border: 1px solid rgba(239,68,68,0.2);
  color: #f87171;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--sans);
  transition: all 0.15s;
  flex-shrink: 0;
}
.dir-remove:hover { background: rgba(239,68,68,0.1); }
.trust-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 40px 20px;
  text-align: center;
}
.trust-empty-icon {
  width: 52px;
  height: 52px;
  border-radius: 14px;
  background: rgba(255,255,255,0.03);
  border: 1px solid var(--border-soft);
  color: var(--text-faint);
  display: grid;
  place-items: center;
  margin-bottom: 4px;
}
.trust-empty-title { font-size: 14px; font-weight: 700; color: var(--text); }
.trust-empty-desc { font-size: 12px; color: var(--text-dim); line-height: 1.5; max-width: 340px; }

/* ---- Risk Levels ---- */
.risk-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;
}
.tab-intro { display: flex; flex-direction: column; gap: 3px; }
.tab-intro-title { font-size: 14px; font-weight: 700; color: var(--text); }
.tab-intro-desc { font-size: 12.5px; color: var(--text-dim); line-height: 1.55; }
.risk-levels { display: flex; flex-direction: column; gap: 10px; }
.risk-card {
  padding: 16px 18px;
  background: rgba(255,255,255,0.02);
  border: 1px solid var(--border);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: all 0.18s;
}
.risk-card:hover { border-color: rgba(163,230,53,0.2); background: rgba(163,230,53,0.02); }
.risk-card.applied { border-color: rgba(163,230,53,0.3); }
.risk-card-head { display: flex; align-items: center; gap: 10px; }
.risk-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
.risk-label { font-size: 15px; font-weight: 700; color: var(--text); flex: 1; }
.active-badge {
  padding: 2px 9px;
  border-radius: 999px;
  font-size: 10.5px;
  font-weight: 700;
  border: 1px solid;
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.current-badge {
  padding: 2px 9px;
  border-radius: 999px;
  font-size: 10.5px;
  font-weight: 600;
  background: rgba(148,163,184,0.1);
  color: #94a3b8;
  border: 1px solid rgba(148,163,184,0.25);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.risk-card-desc { font-size: 12.5px; color: var(--text-dim); line-height: 1.55; }
.risk-apply-btn {
  align-self: flex-start;
  background: rgba(255,255,255,0.03);
  border: 1px solid var(--border);
  color: var(--text);
  transition: all 0.15s;
}
.risk-apply-btn:hover:not(:disabled) { background: rgba(255,255,255,0.06); }
.risk-apply-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.spinner-sm {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid rgba(255,255,255,0.2);
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* ---- Policy Sets ---- */
.sets-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;
}
.policy-sets { display: flex; flex-direction: column; gap: 10px; }
.ps-card {
  padding: 16px 18px;
  background: rgba(255,255,255,0.02);
  border: 1px solid var(--border);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: all 0.18s;
}
.ps-card:hover { background: rgba(255,255,255,0.03); }
.ps-card-top { display: flex; align-items: flex-start; gap: 12px; }
.ps-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}
.ps-icon :deep(svg) { width: 18px; height: 18px; }
.ps-info { flex: 1; min-width: 0; }
.ps-name { font-size: 14.5px; font-weight: 700; margin-bottom: 2px; transition: color 0.15s; }
.ps-rules { font-size: 11px; color: var(--text-faint); font-family: var(--mono); }
.ps-desc { font-size: 12.5px; color: var(--text-dim); line-height: 1.55; }
.ps-activate-btn {
  align-self: flex-start;
  background: rgba(255,255,255,0.03);
  border: 1px solid var(--border);
  color: var(--text);
  display: inline-flex;
  align-items: center;
  gap: 5px;
  transition: all 0.15s;
}
.ps-activate-btn:hover { background: rgba(255,255,255,0.06); }

/* Shared */
.btn {
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid;
  font-size: 12.5px;
  font-weight: 700;
  cursor: pointer;
  font-family: var(--sans);
  transition: all 0.15s;
}
.btn-primary {
  background: linear-gradient(135deg, var(--green-soft), var(--green));
  color: #0a0f05;
  border-color: rgba(163,230,53,0.4);
}
.btn-primary:hover:not(:disabled) { filter: brightness(1.08); }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-sm { padding: 7px 13px; font-size: 12px; display: inline-flex; align-items: center; gap: 6px; }
</style>
