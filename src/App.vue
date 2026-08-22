<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useStore, type View } from "./store";
import { LANGS, useI18n, type Lang } from "./i18n";
import HomeView from "./views/HomeView.vue";
import SessionView from "./views/SessionView.vue";
import ReviewView from "./views/ReviewView.vue";
import HistoryView from "./views/HistoryView.vue";
import ApprovalModal from "./components/ApprovalModal.vue";

const { state, init, navigate } = useStore();
const { t, setLang, lang, prompted, markPrompted } = useI18n();

const appWindow = getCurrentWindow();
const isMaximized = ref(false);
let resizeUnlisten: (() => void) | null = null;

onMounted(async () => {
  await init();
  try {
    isMaximized.value = await appWindow.isMaximized();
    resizeUnlisten = await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
    });
  } catch {}
});

onBeforeUnmount(() => {
  if (resizeUnlisten) resizeUnlisten();
});

async function minimize() {
  try { await appWindow.minimize(); } catch {}
}
async function toggleMax() {
  try { await appWindow.toggleMaximize(); } catch {}
}
async function closeApp() {
  try { await appWindow.close(); } catch {}
}

const tabs = computed(() => {
  const items: { view: View; labelKey: ViewKeys; icon: string; disabled?: boolean }[] = [
    { view: "home", labelKey: "home", icon: "home" },
    { view: "session", labelKey: "monitor", icon: "activity", disabled: !state.session },
    { view: "review", labelKey: "review", icon: "alert", disabled: !state.pendingBatch },
    { view: "history", labelKey: "history", icon: "history" },
  ];
  return items.map((i) => ({
    view: i.view,
    icon: i.icon,
    label:
      i.labelKey === "home"
        ? t("nav.home")
        : i.labelKey === "monitor"
        ? t("nav.monitor")
        : i.labelKey === "review"
        ? t("nav.review")
        : t("nav.history"),
    disabled: i.disabled,
  }));
});

type ViewKeys = "home" | "monitor" | "review" | "history";

const navIcons: Record<string, string> = {
  home: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12L12 3l9 9"/><path d="M5 10v10a1 1 0 001 1h4v-6h4v6h4a1 1 0 001-1V10"/></svg>`,
  activity: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>`,
  alert: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>`,
  history: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>`,
};

function go(view: View) {
  if (view === "home") navigate("home", null);
  else navigate(view);
}

// --- Language prompt modal ---
const showLangModal = computed(() => !prompted.value);
const tempLang = ref<Lang>(lang.value);

function previewLang(l: Lang) {
  tempLang.value = l;
}

function confirmLang() {
  setLang(tempLang.value);
  markPrompted();
}

function switchLangQuick(l: Lang) {
  setLang(l);
}
</script>

<template>
  <div class="app-shell">
    <!-- ================= Top bar ================= -->
    <header class="topbar" data-tauri-drag-region>
      <div class="brand">
        <span class="logo">◈</span>
        <span>{{ t("app.name") }}<small>v0.2</small></span>
        <span class="brand-dot"></span>
        <span class="brand-sub">{{ t("app.category") }}</span>
      </div>
      <nav>
        <button
          v-for="(tab, i) in tabs"
          :key="tab.view + i"
          :class="{ active: state.view === tab.view }"
          :disabled="tab.disabled"
          data-tauri-drag-region="false"
          @click="go(tab.view)"
        >
          <span class="nav-icon" v-html="navIcons[tab.icon]"></span>
          <span class="nav-label">{{ tab.label }}</span>
        </button>
      </nav>
      <div class="top-right">
        <div v-if="state.session" class="session-chip">
          <span class="dot"></span>
          {{ t("session.chip") }} #{{ state.session.num.toString().padStart(5, "0") }}
        </div>
        <div class="lang-switch" :title="t('lang.switch')">
          <button
            v-for="l in LANGS"
            :key="l.id"
            :class="{ active: lang === l.id }"
            data-tauri-drag-region="false"
            @click="switchLangQuick(l.id)"
          >
            <span class="flag">{{ l.flag }}</span>
            <span class="lname">{{ l.label }}</span>
          </button>
        </div>
        <div class="win-controls">
          <button
            class="wc"
            :title="t('win.min')"
            data-tauri-drag-region="false"
            @click="minimize"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
              <line x1="2.5" y1="6" x2="9.5" y2="6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            </svg>
          </button>
          <button
            class="wc"
            :title="isMaximized ? t('win.restore') : t('win.max')"
            data-tauri-drag-region="false"
            @click="toggleMax"
          >
            <svg v-if="isMaximized" width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
              <rect x="3" y="1" width="7" height="7" fill="none" stroke="currentColor" stroke-width="1.2" rx="0.5"/>
              <rect x="1" y="3" width="7" height="7" fill="var(--bg)" stroke="currentColor" stroke-width="1.2" rx="0.5"/>
            </svg>
            <svg v-else width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
              <rect x="2" y="2" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.3" rx="1"/>
            </svg>
          </button>
          <button
            class="wc wc-close"
            :title="t('win.close')"
            data-tauri-drag-region="false"
            @click="closeApp"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
              <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
              <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
      </div>
    </header>

    <!-- ================= Main area ================= -->
    <main class="main">
      <HomeView v-if="state.view === 'home'" />
      <SessionView v-else-if="state.view === 'session'" />
      <ReviewView v-else-if="state.view === 'review'" />
      <HistoryView v-else />
    </main>

    <!-- ================= Footer ================= -->
    <footer class="footer">
      <div class="f-slogan">{{ t("footer.slogan1") }}</div>
      <div class="f-meta">
        <span>{{ t("footer.slogan2") }}</span>
        <span class="sep">·</span>
        <span>{{ t("footer.slogan3") }}</span>
      </div>
    </footer>

    <!-- ================= Language prompt ================= -->
    <transition name="fade">
      <div v-if="showLangModal" class="overlay">
        <div class="modal">
          <div class="modal-head">
            <div class="big-logo">◈</div>
            <h2>{{ t("lang.modal.title") }}</h2>
            <p class="m-subtitle">{{ t("lang.modal.subtitle") }}</p>
          </div>
          <div class="lang-picks">
            <button
              v-for="l in LANGS"
              :key="l.id"
              :class="{ active: tempLang === l.id }"
              @mouseenter="previewLang(l.id)"
              @click="previewLang(l.id)"
            >
              <span class="big-flag">{{ l.flag }}</span>
              <span class="l-label">{{ l.label }}</span>
              <span class="check" v-if="tempLang === l.id">✓</span>
            </button>
          </div>
          <button class="btn btn-primary big confirm" @click="confirmLang">
            {{ t("lang.modal.confirm") }}
          </button>
        </div>
      </div>
    </transition>

    <!-- ================= Approval gate (always-on overlay) ================= -->
    <ApprovalModal />
  </div>
</template>

<style scoped>
.top-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-sub {
  font-size: 11px;
  color: var(--text-faint);
  letter-spacing: 0.3px;
  font-weight: 500;
}

.brand-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--border);
}

.lang-switch {
  display: inline-flex;
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--bg-soft);
}

.lang-switch button {
  background: transparent;
  border: 0;
  color: var(--text-faint);
  cursor: pointer;
  padding: 5px 10px;
  font-size: 12px;
  font-family: var(--sans);
  display: inline-flex;
  align-items: center;
  gap: 5px;
  transition: all 0.15s;
}

.lang-switch button:hover {
  color: var(--text);
  background: rgba(255, 255, 255, 0.04);
}

.lang-switch button.active {
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.2), rgba(22, 163, 74, 0.15));
  color: var(--green);
  font-weight: 700;
}

.lang-switch .flag {
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.5px;
}

/* ================= Footer ================= */
.footer {
  border-top: 1px solid var(--border-soft);
  background: rgba(11, 18, 32, 0.55);
  padding: 12px 24px 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
  flex-shrink: 0;
}

.f-slogan {
  font-size: 12.5px;
  color: var(--text-dim);
  letter-spacing: 0.2px;
}

.f-meta {
  font-size: 11.5px;
  color: var(--text-faint);
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.f-meta .sep {
  opacity: 0.6;
}

/* ================= Language prompt ================= */
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(4, 8, 16, 0.72);
  backdrop-filter: blur(10px);
  display: grid;
  place-items: center;
  z-index: 50;
  padding: 24px;
}

.modal {
  width: 100%;
  max-width: 480px;
  background: linear-gradient(160deg, var(--bg-card) 0%, var(--bg-soft) 100%);
  border: 1px solid var(--border);
  border-radius: 20px;
  padding: 28px 30px 26px;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  gap: 20px;
  animation: popIn 220ms ease;
}

@keyframes popIn {
  from {
    transform: translateY(6px) scale(0.97);
    opacity: 0;
  }
  to {
    transform: translateY(0) scale(1);
    opacity: 1;
  }
}

.modal-head {
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
}

.modal-head h2 {
  font-size: 20px;
}

.modal-head .m-subtitle {
  color: var(--text-dim);
  font-size: 13px;
  max-width: 380px;
  line-height: 1.5;
}

.big-logo {
  width: 58px;
  height: 58px;
  border-radius: 16px;
  display: grid;
  place-items: center;
  color: #fff;
  font-size: 26px;
  font-weight: 900;
  margin-bottom: 6px;
  background: linear-gradient(135deg, #16a34a, #22c55e);
  box-shadow: 0 12px 32px var(--green-glow);
}

.lang-picks {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.lang-picks button {
  position: relative;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 14px;
  padding: 22px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-family: var(--sans);
  transition: all 0.15s ease;
}

.lang-picks button:hover {
  border-color: var(--green);
  background: rgba(34, 197, 94, 0.08);
  transform: translateY(-1px);
}

.lang-picks button.active {
  border-color: var(--green);
  background: linear-gradient(180deg, rgba(34, 197, 94, 0.14), rgba(22, 163, 74, 0.06));
  box-shadow: 0 0 0 1px var(--green-soft) inset;
}

.lang-picks .big-flag {
  font-size: 30px;
  font-weight: 900;
  letter-spacing: 1px;
  line-height: 1;
  color: var(--green);
}

.lang-picks .l-label {
  font-size: 14px;
  font-weight: 600;
}

.lang-picks .check {
  position: absolute;
  top: 10px;
  right: 12px;
  color: var(--green);
  font-weight: 800;
  font-size: 13px;
}

.confirm {
  align-self: center;
  min-width: 180px;
}

.big {
  padding: 12px 22px;
  font-size: 14px;
}
</style>
