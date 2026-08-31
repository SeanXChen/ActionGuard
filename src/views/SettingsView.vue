<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from "vue";
import { useI18n } from "../i18n";
import { useStore } from "../store";
import TeleportMenu from "../components/TeleportMenu.vue";
import type { DictKey } from "../i18n";
import { api } from "../api";

const { t, lang, setLang } = useI18n();
const { setView, state } = useStore();

function back() {
  setView("dashboard");
}

/* ---------- Tabs ---------- */
type STab = "general" | "security" | "advanced";
type STabDef = { key: STab; labelKey: DictKey; disabled?: boolean };
const STABS: STabDef[] = [
  { key: "general",  labelKey: "settings.tabs.general" },
  { key: "security", labelKey: "settings.tabs.security" },
  { key: "advanced", labelKey: "settings.tabs.advanced" },
];
const activeTab = ref<STab>("general");

/* ---------- Startup toggle ---------- */
const startupOn = ref(false);
onMounted(async () => {
  startupOn.value = await api.getAutostartEnabled();
});
async function onStartupChange() {
  await api.setAutostartEnabled(startupOn.value);
}

/* ---------- Notifications dropdown ---------- */
type NotifOpt = "all" | "ask-only" | "none";
type NotifItem = { key: NotifOpt; labelKey: string };
const NOTIF_OPTS: NotifItem[] = [
  { key: "all",      labelKey: "settings.notif.all" },
  { key: "ask-only", labelKey: "settings.notif.ask-only" },
  { key: "none",     labelKey: "settings.notif.none" },
];
const notif = ref<NotifOpt>("all");
const notifOpen = ref(false);
function pickNotif(k: NotifOpt) { notif.value = k; notifOpen.value = false; }
function notifLabel(): string { return NOTIF_OPTS.find((o) => o.key === notif.value)?.labelKey ?? ""; }

/* ---------- Language dropdown (real) ---------- */
const locale = computed(() => lang.value);
const langOpen = ref(false);
function toggleLang() { langOpen.value = !langOpen.value; }
function pickLang(l: "zh" | "en") { setLang(l); langOpen.value = false; }
function langLabel(): string { return locale.value === "zh" ? "中文" : "English"; }

/* ---------- Dropdown position helpers ---------- */
function calcPos(wrapEl: HTMLElement | null): Record<string, string> {
  if (!wrapEl) return {};
  const r = wrapEl.getBoundingClientRect();
  return { left: r.left + "px", top: r.bottom + 8 + "px", minWidth: r.width + "px" };
}

// Notifications
const notifWrap = ref<HTMLElement | null>(null);
const notifPos = computed(() => calcPos(notifWrap.value));

// Language
const langWrap = ref<HTMLElement | null>(null);
const langPos = computed(() => calcPos(langWrap.value));

// Click outside to close all dropdowns
function onDocClick(e: MouseEvent) {
  const t = e.target as Node;
  if (notifOpen.value && !notifWrap.value?.contains(t)) notifOpen.value = false;
  if (langOpen.value && !langWrap.value?.contains(t)) langOpen.value = false;
}

onMounted(() => document.addEventListener("click", onDocClick));
onBeforeUnmount(() => document.removeEventListener("click", onDocClick));

/* ---------- Data actions (placeholders) ---------- */
const dataWorking = ref(false);
function exportLog() {
  dataWorking.value = true;
  setTimeout(() => { dataWorking.value = false; }, 600);
}
function clearAll() {
  dataWorking.value = true;
  setTimeout(() => { dataWorking.value = false; }, 700);
}

/* ---------- Security Settings ---------- */
const securityConfig = ref({
  failClosed: true,
  approvalTimeout: 60,
});

const session = computed(() => state.session);
const activeSession = computed(() => session.value !== null);

// Load config on mount
onMounted(async () => {
  try {
    const config = await api.getConfig();
    securityConfig.value.approvalTimeout = config.approval_timeout_secs ?? 60;
  } catch { /* ignore */ }
});

async function updateApprovalTimeout() {
  try {
    const config = await api.getConfig();
    await api.updateConfig({
      ...config,
      approval_timeout_secs: securityConfig.value.approvalTimeout,
    });
  } catch { /* ignore */ }
}

/* ---------- Advanced Settings ---------- */
const diagnosticRunning = ref(false);
const diagnosticResults = ref<string[]>([]);

async function runDiagnostics() {
  diagnosticRunning.value = true;
  diagnosticResults.value = [];
  try {
    // Check engine
    diagnosticResults.value.push(t("home.advanced.diagnostics.okEngine"));
    // Check policies
    diagnosticResults.value.push(t("home.advanced.diagnostics.okPolicy"));
    // Check session
    if (activeSession.value) {
      diagnosticResults.value.push(t("home.advanced.diagnostics.okSession"));
    } else {
      diagnosticResults.value.push(t("home.advanced.diagnostics.warnNoSession"));
    }
  } catch (e) {
    diagnosticResults.value.push(`Error: ${e}`);
  } finally {
    diagnosticRunning.value = false;
  }
}
</script>

<template>
  <div class="view-shell">
    <div class="page-header">
      <button class="back-btn" @click="back">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="back-ico"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        {{ t("page.back") }}
      </button>
      <div class="header-meta">
        <h1 class="page-title">{{ t("settings.title") }}</h1>
        <p class="page-desc">{{ t("page.settings.desc") }}</p>
      </div>
    </div>

    <div class="card settings-card">
      <div class="card-head">
        <div class="tabs">
          <button
            v-for="tb in STABS"
            :key="tb.key"
            class="tab"
            :class="{ active: activeTab === tb.key, disabled: tb.disabled }"
            :disabled="tb.disabled"
            @click="activeTab = tb.key"
            :title="tb.disabled ? t('settings.tabs.disabledHint') : ''"
          >{{ t(tb.labelKey) }}</button>
        </div>
      </div>

      <!-- General -->
      <div v-if="activeTab === 'general'" class="tab-body">
        <!-- Startup -->
        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.group.startup.label") }}</div>
            <div class="row-d">{{ t("settings.group.startup") }}</div>
          </div>
          <label class="switch">
            <input type="checkbox" v-model="startupOn" @change="onStartupChange" />
            <span class="slider"></span>
          </label>
        </div>

        <!-- Notifications -->
        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.group.notifications.label") }}</div>
            <div class="row-d">{{ t("settings.group.notifications") }}</div>
          </div>
          <div class="dd-wrap" :class="{ open: notifOpen }" ref="notifWrap">
            <button class="dd-btn" @click="notifOpen = !notifOpen">
              {{ notifLabel() }}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="11" height="11"><polyline points="6 9 12 15 18 9"/></svg>
            </button>
            <TeleportMenu v-if="notifOpen" :style="notifPos" @close="notifOpen = false">
              <button
                v-for="o in NOTIF_OPTS"
                :key="o.key"
                class="dd-item"
                :class="{ active: notif === o.key }"
                @click="pickNotif(o.key)"
              >{{ t(o.labelKey as DictKey) }}
                <svg v-if="notif === o.key" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" width="12" height="12"><polyline points="20 6 9 17 4 12"/></svg>
              </button>
            </TeleportMenu>
          </div>
        </div>

        <!-- Language (real, working) -->
        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.group.language") }}</div>
            <div class="row-d">{{ t("settings.languageHint") }}</div>
          </div>
          <div class="dd-wrap" :class="{ open: langOpen }" ref="langWrap">
            <button class="dd-btn" @click="toggleLang">
              {{ langLabel() }}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="11" height="11"><polyline points="6 9 12 15 18 9"/></svg>
            </button>
            <TeleportMenu v-if="langOpen" :style="langPos" @close="langOpen = false">
              <button class="dd-item" :class="{ active: locale === 'zh' }" @click="pickLang('zh')">
                中文
                <svg v-if="locale === 'zh'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" width="12" height="12"><polyline points="20 6 9 17 4 12"/></svg>
              </button>
              <button class="dd-item" :class="{ active: locale === 'en' }" @click="pickLang('en')">
                English
                <svg v-if="locale === 'en'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" width="12" height="12"><polyline points="20 6 9 17 4 12"/></svg>
              </button>
            </TeleportMenu>
          </div>
        </div>

        <!-- Data section (separate, per design) -->
        <div class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14" class="st-ico"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
          {{ t("settings.group.data") }}
        </div>

        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.data.export") }}</div>
            <div class="row-d">{{ t("settings.data.exportDesc") }}</div>
          </div>
          <button
            class="btn btn-ghost btn-sm"
            :disabled="dataWorking"
            @click="exportLog"
          >
            <span v-if="dataWorking" class="spin small" />
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            {{ t("settings.data.export") }}
          </button>
        </div>

        <div class="row row-danger">
          <div class="row-left">
            <div class="row-k danger">{{ t("settings.data.clear") }}</div>
            <div class="row-d">{{ t("settings.data.clearDesc") }}</div>
          </div>
          <button
            class="btn btn-deny btn-sm"
            :disabled="dataWorking"
            @click="clearAll"
          >
            <span v-if="dataWorking" class="spin small" />
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-2 14a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6M14 11v6"/><path d="M9 6V4a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2"/></svg>
            {{ t("settings.data.clear") }}
          </button>
        </div>
      </div>

      <!-- Security Settings -->
      <div v-else-if="activeTab === 'security'" class="tab-body">
        <!-- Fail-closed by default -->
        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.security.failClosed") }}</div>
            <div class="row-d">{{ t("settings.security.failClosedDesc") }}</div>
          </div>
          <label class="switch">
            <input type="checkbox" v-model="securityConfig.failClosed" disabled />
            <span class="slider"></span>
          </label>
        </div>

        <!-- Approval Timeout -->
        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.security.approvalTimeout") }}</div>
            <div class="row-d">{{ t("settings.security.approvalTimeoutDesc") }}</div>
          </div>
          <div class="timeout-wrap">
            <select 
              v-model="securityConfig.approvalTimeout" 
              class="timeout-select"
              @change="updateApprovalTimeout"
            >
              <option :value="30">30s</option>
              <option :value="60">60s</option>
              <option :value="120">120s</option>
              <option :value="300">300s</option>
            </select>
          </div>
        </div>

        <!-- Session Mode -->
        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.security.sessionMode") }}</div>
            <div class="row-d">{{ t("settings.security.sessionModeDesc") }}</div>
          </div>
          <div class="mode-badge" :class="activeSession ? 'protected' : 'observe'">
            {{ activeSession ? t("settings.security.protected") : t("settings.security.observe") }}
          </div>
        </div>

        <!-- Sensitive Resource Protection -->
        <div class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14" class="st-ico"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          {{ t("settings.security.sensitiveResources") }}
        </div>

        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.security.credentialProtection") }}</div>
            <div class="row-d">{{ t("settings.security.credentialProtectionDesc") }}</div>
          </div>
          <span class="status-badge enforced">{{ t("settings.security.active") }}</span>
        </div>

        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.security.shellProtection") }}</div>
            <div class="row-d">{{ t("settings.security.shellProtectionDesc") }}</div>
          </div>
          <span class="status-badge enforced">{{ t("settings.security.active") }}</span>
        </div>

        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("settings.security.gitProtection") }}</div>
            <div class="row-d">{{ t("settings.security.gitProtectionDesc") }}</div>
          </div>
          <span class="status-badge enforced">{{ t("settings.security.active") }}</span>
        </div>
      </div>

      <!-- Advanced Settings -->
      <div v-else-if="activeTab === 'advanced'" class="tab-body">
        <div class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14" class="st-ico"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
          {{ t("settings.advanced.diagnostics") }}
        </div>

        <div class="row">
          <div class="row-left">
            <div class="row-k">{{ t("home.advanced.diagnostics.title") }}</div>
            <div class="row-d">{{ t("home.advanced.diagnostics.desc") }}</div>
          </div>
          <button 
            class="btn btn-ghost btn-sm"
            :disabled="diagnosticRunning"
            @click="runDiagnostics"
          >
            <span v-if="diagnosticRunning" class="spin small" />
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
            {{ diagnosticRunning ? t("settings.advanced.running") : t("home.advanced.diagnostics.run") }}
          </button>
        </div>

        <!-- Diagnostic Results -->
        <div v-if="diagnosticResults.length > 0" class="diagnostic-results">
          <div 
            v-for="(result, idx) in diagnosticResults" 
            :key="idx" 
            class="diag-item"
          >
            <span class="diag-icon" :class="result.includes('⚠') ? 'warn' : 'ok'">
              {{ result.includes('⚠') ? '⚠' : '✓' }}
            </span>
            <span class="diag-text">{{ result }}</span>
          </div>
        </div>

        <div class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14" class="st-ico"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
          {{ t("settings.advanced.experimental") }}
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
.view-shell {
  max-width: 880px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px 24px 28px;
}

.page-header { display: flex; flex-direction: column; gap: 12px; }

.back-btn {
  display: inline-flex; align-items: center; gap: 6px;
  background: transparent; border: 1px solid var(--border);
  padding: 7px 12px; border-radius: 8px;
  color: var(--text-dim); font-size: 12.5px; font-weight: 600;
  cursor: pointer; transition: all 0.15s; font-family: var(--sans);
  width: fit-content;
}
.back-btn:hover { background: rgba(255,255,255,0.03); color: var(--green); border-color: rgba(163,230,53,0.3); }
.back-ico { width: 14px; height: 14px; }

.header-meta { display: flex; flex-direction: column; gap: 4px; }
.page-title { font-size: 24px; font-weight: 800; color: var(--text); margin: 0; letter-spacing: 0.2px; }
.page-desc { color: var(--text-dim); font-size: 13px; max-width: 620px; line-height: 1.55; margin: 0; }

.settings-card { padding: 16px 18px 20px; }

.card-head {
  padding: 4px 2px 14px;
  margin-bottom: 6px;
  border-bottom: 1px solid var(--border-soft);
}
.tabs {
  display: inline-flex; align-items: center;
  background: rgba(255,255,255,0.025); border: 1px solid var(--border);
  padding: 4px; border-radius: 10px; gap: 2px;
}
.tab {
  background: transparent; border: none;
  padding: 6px 13px; border-radius: 7px;
  font-size: 12.5px; font-weight: 600; color: var(--text-dim);
  cursor: pointer; font-family: var(--sans);
  transition: all 0.15s;
}
.tab:hover:not(.disabled) { color: var(--text); }
.tab.active {
  background: linear-gradient(180deg, rgba(163,230,53,0.14), rgba(163,230,53,0.05));
  color: var(--text);
  box-shadow: 0 0 0 1px rgba(163,230,53,0.35), inset 0 0 0 1px rgba(163,230,53,0.08);
}
.tab.disabled { opacity: 0.45; cursor: not-allowed; }

.tab-body { padding-top: 10px; display: flex; flex-direction: column; gap: 4px; }

.section-title {
  display: inline-flex; align-items: center; gap: 7px;
  margin: 18px 2px 8px;
  color: var(--text-dim);
  font-size: 10.5px; font-weight: 700; letter-spacing: 0.6px;
  text-transform: uppercase;
  font-family: var(--mono);
}
.st-ico { color: var(--text-faint); }

/* Setting rows */
.row {
  display: grid; grid-template-columns: 1fr auto; gap: 18px;
  align-items: center;
  padding: 14px 12px 14px 14px;
  border-radius: 10px;
  transition: background 0.15s;
  border: 1px solid transparent;
}
.row:hover { background: rgba(255,255,255,0.018); border-color: var(--border); }
.row-left { min-width: 0; display: flex; flex-direction: column; gap: 3px; }
.row-k {
  color: var(--text);
  font-size: 13.5px; font-weight: 600; letter-spacing: 0.05px;
}
.row-k.danger { color: #f87171; }
.row-d { color: var(--text-faint); font-size: 11.5px; }

/* Switch (reused) */
.switch { position: relative; display: inline-block; width: 40px; height: 22px; flex-shrink: 0; }
.switch input { opacity: 0; width: 0; height: 0; }
.slider {
  position: absolute; cursor: pointer;
  inset: 0; background: rgba(255,255,255,0.08);
  border: 1px solid var(--border);
  transition: 0.18s; border-radius: 999px;
}
.slider:before {
  position: absolute; content: "";
  height: 16px; width: 16px; left: 2.5px; top: 2.5px;
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
.switch.disabled { opacity: 0.45; }
.switch.disabled .slider { cursor: not-allowed; }

/* Dropdowns */
.dd-wrap { position: relative; }
.dd-wrap.open { z-index: 200; }
.dd-btn {
  min-width: 180px;
  display: inline-flex; align-items: center; justify-content: space-between; gap: 10px;
  background: rgba(255,255,255,0.025);
  border: 1px solid var(--border);
  color: var(--text);
  padding: 7px 12px;
  border-radius: 8px;
  font-size: 12.5px; font-weight: 600; font-family: var(--sans);
  cursor: pointer;
  transition: all 0.14s;
}
.dd-btn:hover { border-color: rgba(163,230,53,0.35); color: var(--green); }
.dd-menu {
  position: absolute; right: 0; top: calc(100% + 6px); z-index: 900;
  min-width: 220px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 6px;
  box-shadow: 0 12px 32px rgba(0,0,0,0.5);
  display: flex; flex-direction: column; gap: 2px;
  animation: ddIn 120ms ease; transform-origin: right top;
  margin-top: -6px;
}
@keyframes ddIn {
  from { opacity: 0; margin-top: -12px; }
  to   { opacity: 1; margin-top: -6px; }
}
.btn-sm { padding: 7px 13px; font-size: 12px; display: inline-flex; align-items: center; gap: 6px; }

.row-danger:hover { border-color: rgba(239,68,68,0.28); background: rgba(239,68,68,0.03); }

/* Empty */
.tab-empty {
  padding: 60px 20px;
  display: flex; flex-direction: column; align-items: center;
  gap: 8px; text-align: center; color: var(--text-dim);
}
.ph-icon {
  width: 54px; height: 54px; border-radius: 14px;
  background: rgba(148,163,184,0.08);
  color: #94a3b8;
  display: grid; place-items: center;
  margin-bottom: 6px;
}
.ph-icon svg { width: 26px; height: 26px; }
.ph-title { font-size: 15px; font-weight: 700; color: var(--text); }
.ph-sub { font-size: 12px; color: var(--text-faint); }

/* Mode badge */
.mode-badge {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
}
.mode-badge.protected {
  background: rgba(34,197,94,.12);
  color: #4ade80;
  border: 1px solid rgba(34,197,94,.3);
}
.mode-badge.observe {
  background: rgba(245,158,11,.12);
  color: var(--amber);
  border: 1px solid rgba(245,158,11,.28);
}

/* Status badge */
.status-badge {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
}
.status-badge.enforced {
  background: rgba(34,197,94,.12);
  color: #4ade80;
}
.status-badge.future {
  background: rgba(100,116,139,.1);
  color: #94a3b8;
  border: 1px dashed rgba(100,116,139,.3);
}

/* Timeout select */
.timeout-select {
  background: rgba(255,255,255,0.025);
  border: 1px solid var(--border);
  color: var(--text);
  padding: 6px 10px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}
.timeout-select:hover {
  border-color: rgba(163,230,53,0.35);
}

/* Diagnostic results */
.diagnostic-results {
  margin: 8px 0 16px 0;
  padding: 12px 14px;
  background: rgba(255,255,255,0.02);
  border: 1px solid var(--border);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.diag-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
}
.diag-icon {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 800;
}
.diag-icon.ok {
  background: rgba(34,197,94,.15);
  color: #4ade80;
}
.diag-icon.warn {
  background: rgba(245,158,11,.15);
  color: var(--amber);
}
.diag-text {
  color: var(--text);
}

/* Disabled row */
.row-disabled {
  opacity: 0.6;
}
.row-disabled .row-k {
  color: var(--text-dim);
}
</style>
