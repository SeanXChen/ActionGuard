<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useStore } from "./store";
import type { View } from "./store";
import { useI18n } from "./i18n";
import HomeView from "./views/HomeView.vue";
import OnboardingView from "./views/OnboardingView.vue";
import SessionView from "./views/SessionView.vue";
import ReviewView from "./views/ReviewView.vue";
import HistoryView from "./views/HistoryView.vue";
import ApprovalModal from "./components/ApprovalModal.vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize } from "@tauri-apps/api/dpi";

const store = useStore();
const { state } = store;
const { t, lang, setLang: _setLang } = useI18n();
const locale = computed(() => lang.value);

const appWindow = WebviewWindow.getCurrent();

import type { DictKey } from "./i18n";

const navItems: { view: View; labelKey: DictKey; icon: string; badge?: () => number }[] = [
  { view: "home", labelKey: "nav.dashboard", icon: "📊" },
  { view: "history", labelKey: "nav.activity", icon: "📋" },
  { view: "review", labelKey: "nav.review", icon: "🔍", badge: () => state.pendingApprovals.length },
  { view: "session", labelKey: "nav.live", icon: "📡" },
];

const pendingBadge = computed(() => state.pendingApprovals.length);

const isActive = (v: View) => state.view === v;
const go = (v: View) => {
  store.setView(v);
};

const showLang = ref(false);
function toggleLang() {
  showLang.value = !showLang.value;
}
function setLang(l: "zh" | "en") {
  _setLang(l);
  showLang.value = false;
}

async function minimize() {
  await appWindow.minimize();
}
async function toggleMaximize() {
  await appWindow.toggleMaximize();
}
async function closeWindow() {
  await appWindow.close();
}

async function adjustWindowSize() {
  const w = 1080;
  const h = 780;
  try {
    await appWindow.setSize(new LogicalSize(w, h));
  } catch {
    /* ignore */
  }
}

function isLiveDisabled() {
  return state.view !== "session" && state.session === null;
}

onMounted(() => {
  void store.init();
  void adjustWindowSize();
});

watch(
  () => state.session,
  (s) => {
    if (s) {
      void adjustWindowSize();
    }
  },
);
</script>

<template>
  <div class="app-shell">
    <!-- First-run onboarding -->
    <OnboardingView
      v-if="!state.onboardingDone"
      @done="store.completeOnboarding()"
    />

    <template v-else>
      <!-- Sidebar -->
      <aside class="sidebar">
        <div class="sidebar-brand">
          <div class="logo">AG</div>
          <div class="brand-text">
            <div class="brand-name">ActionGuard</div>
            <div class="brand-tag">{{ t("brand.tag") }}</div>
          </div>
        </div>

        <nav class="sidebar-nav">
          <button
            v-for="item in navItems"
            :key="item.view"
            class="nav-item"
            :class="{ active: isActive(item.view), disabled: item.view === 'session' && isLiveDisabled() }"
            :disabled="item.view === 'session' && isLiveDisabled()"
            @click="go(item.view)"
          >
            <span class="nav-icon">{{ item.icon }}</span>
            <span class="nav-label">{{ t(item.labelKey) }}</span>
            <span v-if="item.badge && item.badge() > 0" class="nav-badge">{{ item.badge() }}</span>
          </button>
        </nav>

        <div class="sidebar-footer">
          <div v-if="state.session" class="sb-status active">
            <span class="sb-dot" />
            <span>{{ t("sidebar.protectionActive") }}</span>
          </div>
          <div v-else class="sb-status inactive">
            <span class="sb-dot" />
            <span>{{ t("sidebar.protectionInactive") }}</span>
          </div>
        </div>
      </aside>

      <!-- Main area -->
      <div class="main-area">
        <!-- Header -->
        <header class="main-header">
          <div class="header-left">
            <h1 class="page-title">
              <template v-if="state.view === 'home'">{{ t("nav.dashboard") }}</template>
              <template v-else-if="state.view === 'history'">{{ t("nav.activity") }}</template>
              <template v-else-if="state.view === 'review'">{{ t("nav.review") }}</template>
              <template v-else-if="state.view === 'session'">{{ t("nav.live") }}</template>
            </h1>
          </div>
          <div class="header-right">
            <div class="lang-wrap">
              <button class="icon-btn" title="Language" @click="toggleLang">
                🌐
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
            <div class="win-controls">
              <button class="wc" @click="minimize">
                <svg width="10" height="10" viewBox="0 0 10 10"><rect x="0" y="4" width="10" height="2" fill="currentColor"/></svg>
              </button>
              <button class="wc" @click="toggleMaximize">
                <svg width="10" height="10" viewBox="0 0 10 10"><rect x="1" y="1" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.5"/></svg>
              </button>
              <button class="wc wc-close" @click="closeWindow">
                <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1 L9 9 M9 1 L1 9" stroke="currentColor" stroke-width="1.5"/></svg>
              </button>
            </div>
          </div>
        </header>

        <!-- Content -->
        <main class="main-content">
          <Transition name="fade" mode="out-in">
            <HomeView v-if="state.view === 'home'" />
            <SessionView v-else-if="state.view === 'session'" />
            <ReviewView v-else-if="state.view === 'review'" />
            <HistoryView v-else-if="state.view === 'history'" />
          </Transition>
        </main>
      </div>
    </template>

    <ApprovalModal
      v-for="req in state.pendingApprovals"
      :key="req.id"
      :request="req"
      @resolve="(d, learn, rule) => store.resolveApproval(req.id, d, learn, rule)"
    />
  </div>
</template>

<style scoped>
.app-shell {
  height: 100vh;
  display: flex;
  min-height: 0;
  background: var(--bg);
}

/* ---------- Sidebar ---------- */
.sidebar {
  width: 210px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg-soft);
  border-right: 1px solid var(--border-soft);
  padding: 20px 14px 16px;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 28px;
  padding: 0 4px;
  -webkit-app-region: drag;
}

.sidebar-brand .logo {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  background: linear-gradient(135deg, var(--green-soft), var(--green));
  display: grid;
  place-items: center;
  color: #fff;
  font-size: 13px;
  font-weight: 800;
  box-shadow: 0 2px 12px var(--green-glow);
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.brand-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.brand-name {
  font-weight: 700;
  font-size: 14px;
  letter-spacing: 0.2px;
  color: var(--text);
}

.brand-tag {
  font-size: 10px;
  color: var(--text-faint);
  font-weight: 500;
}

/* ---------- Sidebar Nav ---------- */
.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-dim);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: var(--sans);
  text-align: left;
  position: relative;
}

.nav-item:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.04);
  color: var(--text);
}

.nav-item.active {
  background: var(--bg-card);
  color: var(--text);
  box-shadow: inset 2px 0 0 var(--green);
}

.nav-item.disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.nav-icon {
  width: 20px;
  text-align: center;
  font-size: 14px;
  opacity: 0.85;
}

.nav-item.active .nav-icon {
  opacity: 1;
}

.nav-badge {
  margin-left: auto;
  background: var(--red-soft);
  color: #fff;
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 999px;
  min-width: 18px;
  text-align: center;
}

/* ---------- Sidebar Footer ---------- */
.sidebar-footer {
  margin-top: auto;
  padding-top: 12px;
  border-top: 1px solid var(--border-soft);
}

.sb-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 500;
  padding: 8px 12px;
  border-radius: 8px;
}

.sb-status.active {
  color: var(--green);
  background: rgba(34, 197, 94, 0.08);
}

.sb-status.inactive {
  color: var(--text-faint);
  background: var(--bg-card);
}

.sb-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.sb-status.active .sb-dot {
  background: var(--green);
  animation: pulse 1.6s infinite;
}

.sb-status.inactive .sb-dot {
  background: var(--text-faint);
}

/* ---------- Main Area ---------- */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.main-header {
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px 0 24px;
  border-bottom: 1px solid var(--border-soft);
  background: rgba(11, 18, 32, 0.85);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  flex-shrink: 0;
  -webkit-app-region: drag;
}

.header-left,
.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  -webkit-app-region: no-drag;
}

.page-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
  letter-spacing: 0.2px;
}

.icon-btn {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  background: transparent;
  border: none;
  color: var(--text-dim);
  border-radius: 8px;
  cursor: pointer;
  font-size: 15px;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--text);
}

.lang-wrap {
  position: relative;
}

.lang-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 110px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  z-index: 100;
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

.lang-dropdown button:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text);
}

.lang-dropdown button.active {
  color: var(--green);
  font-weight: 600;
}

.win-controls {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  margin-left: 4px;
  border-left: 1px solid var(--border-soft);
  padding-left: 8px;
}

.win-controls .wc {
  width: 34px;
  height: 32px;
  display: grid;
  place-items: center;
  background: transparent;
  border: none;
  color: var(--text-faint);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.win-controls .wc:hover {
  color: var(--text);
  background: var(--bg-card);
}

.win-controls .wc-close:hover {
  background: #e81123;
  color: #fff;
}

.main-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 24px 28px 32px;
}
</style>
