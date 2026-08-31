<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useStore } from "./store";
import type { View } from "./store";
import { useI18n } from "./i18n";
import HomeView from "./views/HomeView.vue";
import OnboardingView from "./views/OnboardingView.vue";
import ReviewView from "./views/ReviewView.vue";
import HistoryView from "./views/HistoryView.vue";
import PoliciesView from "./views/PoliciesView.vue";
import BoundariesView from "./views/BoundariesView.vue";
import SettingsView from "./views/SettingsView.vue";
import ApprovalModal from "./components/ApprovalModal.vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { invoke } from "@tauri-apps/api/core";
import { api } from "./api";

const store = useStore();
const { state } = store;
const { t, lang, setLang: _setLang } = useI18n();
const locale = computed(() => lang.value);

const appWindow = WebviewWindow.getCurrent();

type DictKey = import("./i18n").DictKey;

/* ---- Navigation ---- */
type NavItem = { view: View; labelKey: DictKey; icon: string; badge?: () => number };
const NAV_ITEMS: NavItem[] = [
  {
    view: "dashboard",
    labelKey: "nav.dashboard",
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="9" rx="1.5"/><rect x="14" y="3" width="7" height="5" rx="1.5"/><rect x="14" y="12" width="7" height="9" rx="1.5"/><rect x="3" y="16" width="7" height="5" rx="1.5"/></svg>`,
  },
  {
    view: "activity",
    labelKey: "nav.activity",
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>`,
    badge: () => state.ledger.length,
  },
  {
    view: "review",
    labelKey: "nav.review",
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>`,
    badge: () => state.pendingApprovals.length,
  },
  {
    view: "policies",
    labelKey: "nav.policies",
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="M9 12l2 2 4-4"/></svg>`,
  },
  {
    view: "boundaries",
    labelKey: "nav.boundaries",
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="10" width="16" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/><circle cx="12" cy="15.5" r="1.5"/></svg>`,
  },
  {
    view: "settings",
    labelKey: "nav.settings",
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`,
  },
];

const currentView = computed(() => state.view);

function isActive(v: View) {
  return state.view === v;
}
function go(v: View) {
  store.setView(v);
}

/* ---- Session ---- */
const isRunning = computed(() => state.session !== null);
const starting = ref(false);
const stopping = ref(false);

async function startProtection() {
  starting.value = true;
  try {
    await api.startSession(".", "protected");
    await store.refreshActiveStats();
    await store.refreshLedger();
    await store.refreshPendingApprovals();
  } catch {
    /* ignore */
  } finally {
    starting.value = false;
  }
}

async function stopProtection() {
  if (!confirm(t("home.active.confirmStop"))) return;
  stopping.value = true;
  try {
    await api.stopSession();
    await store.refreshActiveStats();
    await store.refreshLedger();
  } catch {
    /* ignore */
  } finally {
    stopping.value = false;
  }
}

function fmtElapsed(started_at: string): string {
  const start = new Date(started_at).getTime();
  const now = Date.now();
  const secs = Math.max(0, Math.floor((now - start) / 1000));
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  const parts: string[] = [];
  if (h > 0) parts.push(`${h}h`);
  if (m > 0) parts.push(`${m}m`);
  parts.push(`${s}s`);
  return parts.join(" ");
}

const elapsed = ref("0s");
let elapsedTimer: ReturnType<typeof setInterval> | null = null;

function updateElapsed() {
  if (state.session?.started_at) {
    elapsed.value = fmtElapsed(state.session.started_at);
  }
}

const startedTimeStr = computed(() => {
  if (!state.session?.started_at) return "";
  const d = new Date(state.session.started_at);
  return d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
});

onMounted(() => {
  void store.init();
  void adjustWindowSize();
  updateElapsed();
  elapsedTimer = setInterval(updateElapsed, 1000);
});

watch(
  () => state.session,
  (s) => {
    if (s) {
      void adjustWindowSize();
      updateElapsed();
    }
  },
);

/* ---- Window controls ---- */
async function adjustWindowSize() {
  const w = 1200;
  const h = 820;
  try {
    await appWindow.setSize(new LogicalSize(w, h));
  } catch {
    /* ignore */
  }
}

async function startDrag() {
  try {
    await appWindow.startDragging();
  } catch (e: any) {
    console.error('[WC] startDrag error:', e);
  }
}

async function minimize() {
  try {
    await invoke("window_minimize");
  } catch (e: any) {
    console.error('[WC] minimize error:', e);
  }
}
async function toggleMaximize() {
  try {
    await invoke("window_toggle_maximize");
  } catch (e: any) {
    console.error('[WC] toggleMaximize error:', e);
  }
}
async function closeWindow() {
  try {
    await invoke("window_close");
  } catch (e: any) {
    console.error('[WC] closeWindow error:', e);
  }
}

/* ---- Language ---- */
const showLang = ref(false);
function toggleLang() { showLang.value = !showLang.value; }
function setLang(l: "zh" | "en") {
  _setLang(l);
  showLang.value = false;
}
</script>

<template>
  <div class="app-shell">
    <!-- First-run onboarding -->
    <OnboardingView
      v-if="!state.onboardingDone"
      @done="store.completeOnboarding()"
    />

    <template v-else>
      <!-- ========== Left Sidebar ========== -->
      <aside class="sidebar">
        <!-- Brand -->
        <div class="sidebar-brand">
          <div class="logo">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
              <polyline points="9 12 11 14 15 10" />
            </svg>
          </div>
          <div class="brand-text">
            <div class="brand-name">ActionGuard</div>
            <div class="brand-tag">{{ t("brand.tag") }}</div>
          </div>
        </div>

        <!-- 4 Info Cards (matching design) -->
        <div class="sidebar-info-cards">

          <!-- Card 1: Protection Active / Start -->
          <div class="info-card" :class="{ active: isRunning }">
            <div class="info-card-head">
              <span class="info-dot" :class="{ active: isRunning }" />
              <span class="info-title">
                {{ isRunning ? t("sidebar.mode.active") : t("sidebar.protectionInactive") }}
              </span>
            </div>

            <template v-if="isRunning">
              <div class="info-card-body">
                <div class="info-sub">{{ startedTimeStr }}</div>
                <div class="info-sub">{{ elapsed }}</div>
              </div>
              <button
                class="pause-btn"
                :disabled="stopping"
                @click="stopProtection"
              >
                {{ stopping ? t("dashboard.action.pausing") : t("dashboard.action.pause") }}
              </button>
            </template>

            <template v-else>
              <div class="info-card-body">
                <div class="info-sub">{{ t("sidebar.mode.activeDesc") }}</div>
              </div>
              <button
                class="start-btn"
                :disabled="starting"
                @click="startProtection"
              >
                {{ starting ? t("dashboard.action.starting") : "▶ " + t("dashboard.action.protect") }}
              </button>
            </template>
          </div>

          <!-- Card 2: Protection Mode -->
          <div class="info-card">
            <div class="info-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
            </div>
            <div class="info-card-body">
              <div class="info-title-sm">{{ t("dashboard.meta.mode") }}</div>
              <div class="info-title-bold">{{ t("sidebar.mode.recommended") }}</div>
              <div class="info-sub-sm">{{ t("sidebar.mode.modeDesc") }}</div>
            </div>
            <button class="change-btn" @click="go('policies')">
              {{ t("sidebar.mode.change") }}
            </button>
          </div>

          <!-- Card 3: Protected Scope -->
          <div class="info-card">
            <div class="info-icon scope">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
            </div>
            <div class="info-card-body">
              <div class="info-title-sm">{{ t("sidebar.scope.label") }}</div>
              <div class="info-title-bold">{{ t("sidebar.scope.thisComputer") }}</div>
              <div class="info-sub-sm">{{ t("sidebar.scope.allBoundaries") }}</div>
            </div>
            <button class="change-btn" @click="go('boundaries')">
              {{ t("sidebar.scope.change") }}
            </button>
          </div>

          <!-- Card 4: Local First -->
          <div class="info-card local-first">
            <div class="info-card-head">
              <span class="local-dot" />
              <span class="info-title">{{ t("sidebar.local.title") }}</span>
            </div>
            <ul class="check-list">
              <li>{{ t("sidebar.local.noCloud") }}</li>
              <li>{{ t("sidebar.local.noAccount") }}</li>
              <li>{{ t("sidebar.local.noTelemetry") }}</li>
              <li>{{ t("sidebar.local.localData") }}</li>
            </ul>
          </div>

        </div>

        <!-- Navigation (moved to top header in main area) -->
        <!-- Spacer pushes footer to bottom -->
        <div class="sidebar-spacer" />

        <div class="sidebar-footer-text">
          <div>{{ t("sidebar.footer.version") }}</div>
          <div class="check-updates" @click="async () => { console.log('check-updates clicked'); try { await openUrl('https://github.com/SeanXChen/ActionGuard/releases'); } catch(e) { console.error('openUrl error:', e); } }">
            {{ t("sidebar.footer.checkUpdates") }}
          </div>
        </div>
      </aside>

      <!-- ========== Main Area ========== -->
      <div class="main-area">
        <!-- Draggable strip (full-width, outside header so it's NOT constrained by no-drag) -->
        <div class="drag-strip" @mousedown.stop.prevent="startDrag" />

        <!-- Top Header: nav + lang + win controls -->
        <header class="main-header">
          <!-- Nav items (horizontal, visible in main area) -->
          <nav class="header-nav" @mousedown.stop>
            <button
              v-for="item in NAV_ITEMS"
              :key="item.view"
              class="nav-tab"
              :class="{ active: isActive(item.view) }"
              @click="go(item.view)"
            >
              <span class="nav-tab-icon" v-html="item.icon" />
              <span class="nav-tab-label">{{ t(item.labelKey) }}</span>
              <span v-if="item.badge && item.badge() > 0" class="nav-tab-badge">{{ item.badge() }}</span>
            </button>
          </nav>

          <!-- Right controls -->
          <div class="header-right" @mousedown.stop>
            <div class="feedback-btn" @click="async () => { console.log('feedback clicked'); try { await openUrl('https://github.com/SeanXChen/ActionGuard/issues'); } catch(e) { console.error('openUrl error:', e); } }">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="width: 15px; height: 15px;"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>
              {{ t("header.feedback") }}
            </div>

            <!-- Language switcher -->
            <div class="lang-wrap">
              <button class="icon-btn" title="Language" @click="toggleLang">
                <span class="lang-label">{{ locale === "zh" ? "中" : "EN" }}</span>
              </button>
              <div v-if="showLang" class="lang-dropdown">
                <button :class="{ active: locale === 'zh' }" @click="setLang('zh')">
                  中文
                </button>
                <button :class="{ active: locale === 'en' }" @click="setLang('en')">
                  English
                </button>
              </div>
            </div>

            <!-- Win controls -->
            <div class="win-controls">
              <button class="wc" @click="minimize" :title="t('win.min')">
                <svg width="10" height="10" viewBox="0 0 10 10"><rect x="0" y="4" width="10" height="2" fill="currentColor"/></svg>
              </button>
              <button class="wc" @click="toggleMaximize" :title="t('win.max')">
                <svg width="10" height="10" viewBox="0 0 10 10"><rect x="1" y="1" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.5"/></svg>
              </button>
              <button class="wc wc-close" @click="closeWindow" :title="t('win.close')">
                <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1 L9 9 M9 1 L1 9" stroke="currentColor" stroke-width="1.5"/></svg>
              </button>
            </div>
          </div>
        </header>

        <!-- Content -->
        <main class="main-content">
          <Transition name="fade" mode="out-in">
            <HomeView v-if="state.view === 'dashboard'" />
            <HistoryView v-else-if="state.view === 'activity'" />
            <ReviewView v-else-if="state.view === 'review'" />
            <PoliciesView v-else-if="state.view === 'policies'" />
            <BoundariesView v-else-if="state.view === 'boundaries'" />
            <SettingsView v-else-if="state.view === 'settings'" />
          </Transition>
        </main>
      </div>
    </template>

    <!-- Approval modal — single instance, reads pendingApprovals[0] from the store -->
    <ApprovalModal
      v-if="state.pendingApprovals.length > 0"
    />
  </div>
</template>

<style scoped>
.app-shell {
  height: 100vh;
  display: flex;
  flex-direction: row;
  align-items: stretch;
  min-height: 0;
  background: var(--bg);
}

/* ========== Sidebar ========== */
.sidebar {
  width: 260px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg-soft);
  border-right: 1px solid var(--border-soft);
  padding: 18px 14px 14px;
  overflow-y: auto;
  min-height: 0;
}

.sidebar::-webkit-scrollbar { width: 6px; }
.sidebar::-webkit-scrollbar-thumb { background: transparent; border-radius: 3px; }
.sidebar:hover::-webkit-scrollbar-thumb { background: #253044; }

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 22px;
  padding: 0 4px;
  /* Allow dragging the window from the sidebar brand area */
  -webkit-app-region: drag;
  cursor: grab;
  width: 100%;
  box-sizing: border-box;
  flex-shrink: 0;
}

.sidebar-brand:active {
  cursor: grabbing;
}

.sidebar-brand .logo {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--green-soft), var(--green));
  display: grid;
  place-items: center;
  color: #0a0f05;
  box-shadow: 0 2px 16px var(--green-glow);
  flex-shrink: 0;
}

.sidebar-brand .logo svg { width: 20px; height: 20px; }

.brand-text { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.brand-name { font-weight: 800; font-size: 14.5px; letter-spacing: 0.2px; color: var(--text); }
.brand-tag { font-size: 10px; color: var(--text-faint); font-weight: 500; }

/* ---- 4 Info Cards ---- */
.sidebar-info-cards {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
}

.info-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 13px 13px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  position: relative;
  overflow: hidden;
  transition: border-color 0.15s;
}

.info-card::before {
  content: "";
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(163, 230, 53, 0.2), transparent);
}

.info-card.active::before {
  background: linear-gradient(90deg, transparent, rgba(163, 230, 53, 0.5), transparent);
}

.info-card-head {
  display: flex;
  align-items: center;
  gap: 7px;
}

.info-dot {
  width: 8px; height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--text-faint);
}

.info-dot.active {
  background: var(--green-check);
  animation: pulse-dot 1.6s infinite;
  box-shadow: 0 0 8px rgba(34, 197, 94, 0.6);
}

.local-dot {
  width: 8px; height: 8px;
  border-radius: 50%;
  background: var(--green);
  flex-shrink: 0;
}

.info-title {
  font-size: 12.5px;
  font-weight: 700;
  color: var(--text);
  letter-spacing: 0.1px;
}

.info-title-sm {
  font-size: 10px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 0.8px;
  font-weight: 600;
  margin-bottom: 2px;
}

.info-title-bold {
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
  margin-bottom: 2px;
}

.info-sub {
  font-size: 11px;
  color: var(--text-faint);
  line-height: 1.35;
}

.info-sub-sm {
  font-size: 10.5px;
  color: var(--text-faint);
  line-height: 1.4;
  margin-top: 2px;
}

.info-card-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.info-icon {
  width: 30px; height: 30px;
  border-radius: 8px;
  background: rgba(34, 197, 94, 0.12);
  display: grid;
  place-items: center;
  color: var(--green-check);
  flex-shrink: 0;
}

.info-icon svg { width: 15px; height: 15px; }

.info-icon.scope {
  background: rgba(56, 189, 248, 0.12);
  color: var(--blue);
}

/* Cards with icon: horizontal layout */
.info-card:has(.info-icon) {
  flex-direction: row;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 10px;
}

.info-card:has(.info-icon) .info-card-body {
  flex: 1;
  min-width: 0;
}

.info-card:has(.info-icon) .change-btn {
  align-self: flex-end;
  width: auto;
  margin-top: auto;
}

/* Buttons */
.start-btn {
  width: 100%;
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid rgba(163, 230, 53, 0.35);
  background: linear-gradient(135deg, var(--green-soft), var(--green));
  color: #0a0f05;
  font-size: 12px;
  font-weight: 800;
  font-family: var(--sans);
  cursor: pointer;
  transition: all 0.15s;
  box-shadow: 0 2px 10px var(--green-glow);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.start-btn:hover:not(:disabled) { filter: brightness(1.08); }
.start-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.pause-btn {
  width: 100%;
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid rgba(239, 68, 68, 0.35);
  background: linear-gradient(135deg, rgba(220, 38, 38, 0.15), rgba(239, 68, 68, 0.08));
  color: #fca5a5;
  font-size: 12px;
  font-weight: 700;
  font-family: var(--sans);
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.pause-btn:hover:not(:disabled) { background: linear-gradient(135deg, rgba(220, 38, 38, 0.25), rgba(239, 68, 68, 0.15)); }
.pause-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.change-btn {
  width: 100%;
  padding: 5px 10px;
  border-radius: 7px;
  border: 1px solid var(--border);
  background: var(--bg-soft);
  color: var(--text-dim);
  font-size: 11px;
  font-weight: 600;
  font-family: var(--sans);
  cursor: pointer;
  transition: all 0.15s;
}

.change-btn:hover { background: var(--bg-card-hover); color: var(--text); }

/* Local first card */
.local-first { }

.check-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-top: 2px;
}

.check-list li {
  font-size: 11px;
  color: var(--text-dim);
  padding-left: 16px;
  position: relative;
  line-height: 1.35;
}

.check-list li::before {
  content: "✓";
  position: absolute;
  left: 0;
  top: 0;
  color: var(--green-check);
  font-weight: 700;
  font-size: 10px;
}

/* Spacer + Footer */
.sidebar-spacer { flex: 1; min-height: 12px; }

.sidebar-footer-text {
  padding-top: 12px;
  border-top: 1px solid var(--border-soft);
  font-size: 10.5px;
  color: var(--text-faint);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.check-updates {
  text-decoration: underline;
  text-decoration-style: dotted;
  text-underline-offset: 2px;
  cursor: pointer;
  opacity: 0.8;
}

.check-updates:hover { color: var(--green); opacity: 1; }

/* ========== Main Area ========== */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  position: relative;
}

/* ---- Top Header ---- */
.main-header {
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px 0 0;
  border-bottom: 1px solid var(--border-soft);
  background: var(--bg);
  flex-shrink: 0;
  gap: 16px;
  /* The header content (nav, buttons, controls) is NOT draggable */
  -webkit-app-region: no-drag;
  /* But the left strip of the header bar IS draggable */
  position: relative;
}

/* Full-width drag strip — covers the entire header bar so any part is draggable */
.drag-strip {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 52px;
  cursor: grab;
  z-index: 1;
  pointer-events: auto;
}
.drag-strip:active {
  cursor: grabbing;
}

/* Interactive elements sit above the strip and are non-draggable;
   their children (buttons etc.) re-enable pointer-events individually */
.header-nav,
.header-right {
  position: relative;
  z-index: 2;
  pointer-events: none;
}

/* Horizontal nav tabs */
.header-nav {
  display: flex;
  align-items: center;
  gap: 2px;
  padding-left: 20px;
  height: 100%;
  overflow-x: auto;
  overflow-y: hidden;
  flex: 1 1 auto;
  min-width: 0;
  scrollbar-width: thin;
}

.header-nav::-webkit-scrollbar { height: 0; }

.nav-tab {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 6px 13px 7px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-dim);
  font-size: 12.5px;
  font-weight: 600;
  font-family: var(--sans);
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
  position: relative;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
  pointer-events: auto;
}

.nav-tab:hover {
  background: rgba(255, 255, 255, 0.04);
  color: var(--text);
}

.nav-tab.active {
  background: linear-gradient(180deg, rgba(163, 230, 53, 0.14), rgba(163, 230, 53, 0.04));
  color: var(--green);
  border-color: rgba(163, 230, 53, 0.22);
}

.nav-tab-icon {
  width: 16px; height: 16px;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.nav-tab-icon :deep(svg) { width: 16px; height: 16px; display: block; }

.nav-tab-badge {
  min-width: 18px; height: 18px; padding: 0 5px;
  border-radius: 999px;
  background: rgba(163, 230, 53, 0.15);
  color: var(--green);
  font-size: 10px; font-weight: 700;
  display: inline-grid; place-items: center;
  border: 1px solid rgba(163, 230, 53, 0.25);
}

/* Header right */
.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  min-width: 0;
}

.feedback-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 11px;
  border-radius: 8px;
  border: 1px solid rgba(163, 230, 53, 0.22);
  background: rgba(163, 230, 53, 0.06);
  color: var(--green);
  font-size: 11.5px;
  font-weight: 600;
  font-family: var(--sans);
  cursor: pointer;
  transition: all 0.15s;
  pointer-events: auto;
}

.feedback-btn:hover { background: rgba(163, 230, 53, 0.12); border-color: rgba(163, 230, 53, 0.4); }

.icon-btn {
  width: 32px; height: 32px;
  display: grid;
  place-items: center;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-dim);
  border-radius: 8px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  transition: all 0.15s;
  pointer-events: auto;
}

.lang-label { font-family: var(--sans); font-weight: 700; font-size: 11.5px; }
.icon-btn:hover { background: var(--bg-card-hover); color: var(--text); }

.lang-wrap { position: relative; }

.lang-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 130px;
  box-shadow: 0 14px 40px rgba(0, 0, 0, 0.55);
  z-index: 55;
  animation: dropIn 140ms ease;
  transform-origin: right top;
  margin-top: -8px;
  pointer-events: auto;
}

@keyframes dropIn {
  from { opacity: 0; margin-top: -16px; }
  to   { opacity: 1; margin-top: -8px; }
}

.lang-dropdown button {
  background: transparent;
  border: none;
  color: var(--text-dim);
  padding: 7px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  text-align: left;
  font-family: var(--sans);
  transition: all 0.12s;
}

.lang-dropdown button:hover { background: rgba(255, 255, 255, 0.04); color: var(--text); }
.lang-dropdown button.active { color: var(--green); font-weight: 700; }

.win-controls {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  border-left: 1px solid var(--border-soft);
  padding-left: 10px;
  margin-left: 2px;
  position: relative;
}

.win-controls .wc {
  width: 34px; height: 32px;
  display: grid;
  place-items: center;
  background: transparent;
  border: none;
  color: var(--text-faint);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
  pointer-events: auto;
}

.win-controls .wc:hover { color: var(--text); background: var(--bg-card); }
.win-controls .wc-close:hover { background: #e81123; color: #fff; }

/* Content area */
.main-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0;
  /* Don't drag window when interacting with content */
  -webkit-app-region: no-drag;
}

/* Animations */
@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}

/* Vue transition */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.18s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
