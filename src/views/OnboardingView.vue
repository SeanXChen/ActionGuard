<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed } from "vue";
import { useI18n } from "../i18n";
import { api } from "../api";
import type { SessionMode } from "../types";

const { t, tf } = useI18n();
const emit = defineEmits<{ done: [] }>();

const workspaceDir = ref<string>(".");
const protectMode = ref<SessionMode>("protected");
const showAdvanced = ref(false);
const starting = ref(false);
const error = ref<string | null>(null);



const recMode = computed(() => protectMode.value === "protected");
const modeLabel = computed(() =>
  recMode.value ? t("onboarding.level.recommended") : t("onboarding.level.observe"),
);

function selectWorkspace() {
  const input = document.createElement("input");
  input.type = "file";
  (input as any).webkitdirectory = true;
  input.addEventListener("change", () => {
    const f = input.files?.[0] as any;
    if (f && f.webkitRelativePath) {
      const parts = (f.webkitRelativePath as string).split("/");
      workspaceDir.value = parts[0] || ".";
    }
  });
  input.click();
}

async function protect() {
  starting.value = true;
  error.value = null;
  try {
    await api.startSession(workspaceDir.value, protectMode.value);
    emit("done");
  } catch (e) {
    error.value = (e as Error).message || t("onboarding.startError");
  } finally {
    starting.value = false;
  }
}

function skip() {
  emit("done");
}
</script>

<template>
  <div class="onboarding-screen">
    <div class="ob-radial ob-radial-a" />
    <div class="ob-radial ob-radial-b" />

    <div class="onboarding-card">
      <div class="ob-hero">
        <div class="ob-shield-wrap">
          <svg class="ob-shield-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            <polyline points="9 12 11 14 15 10" />
          </svg>
        </div>
        <h1 class="ob-title">ActionGuard</h1>
        <p class="ob-tagline">{{ t("onboarding.headline") }}</p>
        <p class="ob-subline">{{ t("onboarding.subline") }}</p>
      </div>

      <button
        class="btn btn-primary ob-cta"
        :disabled="starting"
        @click="protect"
      >
        <span v-if="starting" class="ob-spinner" />
        <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
          <polyline points="9 12 11 14 15 10" />
        </svg>
        {{ starting ? t("onboarding.starting") : t("onboarding.cta") }}
      </button>

      <div class="ob-defaults">
        <span class="ob-default-pill">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
          {{ t("onboarding.defaults.scope") }}
        </span>
        <span class="ob-dot">·</span>
        <span class="ob-default-pill">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
          {{ modeLabel }}
        </span>
      </div>

      <!-- What you'll see next — so users know this button leads somewhere -->
      <div class="ob-next">
        <div class="ob-next-k">点击后进入防护大盘，左侧可查看：</div>
        <div class="ob-next-row">
          <span class="ob-next-chip">📊 活动日志</span>
          <span class="ob-next-chip">⏳ 审查队列</span>
          <span class="ob-next-chip">🛡️ 策略规则</span>
          <span class="ob-next-chip">🔌 防护边界</span>
          <span class="ob-next-chip">⚙️ 设置</span>
        </div>
      </div>

      <button class="ob-advanced-toggle" @click="showAdvanced = !showAdvanced">
        <span class="ob-arrow">{{ showAdvanced ? "▾" : "▸" }}</span>
        <span>{{ t("onboarding.advanced") }}</span>
      </button>

      <div v-if="showAdvanced" class="ob-advanced">
        <div class="ob-field">
          <label class="ob-label">
            <span>{{ t("onboarding.scope.title") }}</span>
            <span class="ob-label-dot" />
          </label>

          <div class="ob-scope-card is-rec" @click="workspaceDir = '.'">
            <div class="ob-scope-check" :class="{ on: workspaceDir === '.' }">
              <svg v-if="workspaceDir === '.'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
            </div>
            <div class="ob-scope-body">
              <div class="ob-scope-title">{{ t("onboarding.scope.computer") }}</div>
              <div class="ob-scope-desc">{{ t("onboarding.scope.computerDesc") }}</div>
            </div>
            <span class="ob-rec-badge">{{ t("onboarding.rec") }}</span>
          </div>

          <div class="ob-field-inner">
            <div class="ob-input-row">
              <input
                v-model="workspaceDir"
                class="ob-input"
                type="text"
                readonly
              />
              <button class="btn ob-btn" @click="selectWorkspace">
                {{ t("onboarding.changeDir") }}
              </button>
            </div>
            <p class="ob-hint">{{ t("onboarding.scope.customHint") }}</p>
          </div>
        </div>

        <div class="ob-field">
          <label class="ob-label">
            <span>{{ t("onboarding.level.title") }}</span>
          </label>

          <div class="ob-mode-card is-rec" @click="protectMode = 'protected'">
            <div class="ob-scope-check" :class="{ on: protectMode === 'protected' }">
              <svg v-if="protectMode === 'protected'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
            </div>
            <div class="ob-scope-body">
              <div class="ob-scope-title">
                {{ t("onboarding.level.recommended") }}
                <span class="ob-rec-badge-inline">{{ t("onboarding.rec") }}</span>
              </div>
              <div class="ob-scope-desc">{{ t("onboarding.level.recommendedDesc") }}</div>
              <ul class="ob-bullets">
                <li><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>{{ t("onboarding.promise.routine") }}</li>
                <li><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9" /><line x1="12" y1="8" x2="12" y2="13" /><line x1="12" y1="16" x2="12.01" y2="16" /></svg>{{ t("onboarding.promise.ask") }}</li>
                <li><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>{{ t("onboarding.promise.block") }}</li>
              </ul>
            </div>
          </div>

          <div class="ob-mode-card" @click="protectMode = 'observe'">
            <div class="ob-scope-check" :class="{ on: protectMode === 'observe' }">
              <svg v-if="protectMode === 'observe'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
            </div>
            <div class="ob-scope-body">
              <div class="ob-scope-title">{{ t("onboarding.level.observe") }}</div>
              <div class="ob-scope-desc">{{ t("onboarding.level.observeDesc") }}</div>
            </div>
          </div>
        </div>

        <div class="ob-field">
          <label class="ob-label"><span>{{ t("onboarding.protect.title") }}</span></label>
          <div class="ob-protect-grid">
            <div class="ob-protect-pill cat-file">📄 {{ t("onboarding.protect.file") }}</div>
            <div class="ob-protect-pill cat-shell">💻 {{ t("onboarding.protect.shell") }}</div>
            <div class="ob-protect-pill cat-git">🌿 {{ t("onboarding.protect.git") }}</div>
            <div class="ob-protect-pill cat-package">📦 {{ t("onboarding.protect.package") }}</div>
            <div class="ob-protect-pill cat-secret">🔐 {{ t("onboarding.protect.secret") }}</div>
          </div>
        </div>
      </div>

      <div v-if="error" class="ob-error">{{ error }}</div>

      <div class="ob-divider" />

      <div class="ob-privacy">
        <div class="ob-privacy-row">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
          <span class="ob-privacy-title">{{ t("onboarding.privacy.title") }}</span>
        </div>
        <p class="ob-privacy-desc">{{ t("onboarding.privacy.desc") }}</p>
        <ul class="ob-privacy-list">
          <li><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>{{ t("onboarding.privacy.p1") }}</li>
          <li><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>{{ t("onboarding.privacy.p2") }}</li>
          <li><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>{{ t("onboarding.privacy.p3") }}</li>
        </ul>
      </div>

      <!-- Escape hatch: if user has already configured or is returning, skip to dashboard directly -->
      <button class="ob-skip" @click="skip">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
        我已经配置过，直接进入防护大盘 →
      </button>
    </div>
  </div>
</template>

<style scoped>
.onboarding-screen {
  position: relative;
  width: 100%;
  min-width: 100%;
  height: 100vh;
  display: grid;
  place-items: center;
  padding: 24px;
  overflow: hidden;
  background: var(--bg);
  flex: 1 1 auto;
}

.ob-radial {
  position: absolute;
  pointer-events: none;
  filter: blur(60px);
  opacity: 0.9;
  z-index: 0;
}
.ob-radial-a {
  width: 680px;
  height: 680px;
  left: -160px;
  top: -260px;
  background: radial-gradient(circle, rgba(163, 230, 53, 0.22) 0%, transparent 60%);
}
.ob-radial-b {
  width: 640px;
  height: 640px;
  right: -180px;
  bottom: -220px;
  background: radial-gradient(circle, rgba(56, 189, 248, 0.12) 0%, transparent 60%);
}

.onboarding-card {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 520px;
  background: linear-gradient(180deg, rgba(17, 24, 39, 0.9), rgba(12, 16, 23, 0.92));
  border: 1px solid var(--border);
  border-radius: 20px;
  padding: 44px 40px 32px;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 18px;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.55), 0 0 0 1px rgba(255, 255, 255, 0.02) inset;
  backdrop-filter: blur(8px);
  max-height: calc(100vh - 48px);
  overflow-y: auto;
}

.ob-hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 10px;
  margin-bottom: 4px;
}

.ob-shield-wrap {
  width: 64px;
  height: 64px;
  border-radius: 18px;
  background: linear-gradient(135deg, var(--green-soft), var(--green));
  display: grid;
  place-items: center;
  color: #0a0f05;
  box-shadow: 0 8px 32px var(--green-glow);
  margin-bottom: 6px;
}
.ob-shield-svg { width: 34px; height: 34px; }

.ob-title {
  font-size: 30px;
  font-weight: 700;
  letter-spacing: 0.2px;
  color: #fff;
  margin: 0;
}

.ob-tagline {
  font-size: 17px;
  font-weight: 600;
  color: #fff;
  line-height: 1.4;
  margin: 0;
  max-width: 380px;
}
.ob-tagline :deep(br) { display: none; }

.ob-subline {
  font-size: 13px;
  color: var(--text-dim);
  line-height: 1.6;
  margin: 0;
  max-width: 400px;
}

.ob-cta {
  width: 100%;
  padding: 14px 20px;
  font-size: 15.5px;
  border-radius: 12px;
  margin-top: 2px;
}

.ob-spinner {
  width: 16px;
  height: 16px;
  border: 2.5px solid rgba(10, 15, 5, 0.3);
  border-top-color: #0a0f05;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  display: inline-block;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

.ob-defaults {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  font-size: 12px;
  color: var(--text-dim);
  flex-wrap: wrap;
}
.ob-default-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: rgba(163, 230, 53, 0.08);
  border: 1px solid rgba(163, 230, 53, 0.2);
  border-radius: 999px;
  color: var(--green);
  font-weight: 600;
  font-size: 11.5px;
}
.ob-dot { color: var(--text-faint); }

.ob-advanced-toggle {
  align-self: center;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: none;
  color: var(--text-dim);
  font-size: 12.5px;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 8px;
  transition: all 0.15s;
  font-family: var(--sans);
  font-weight: 500;
}
.ob-advanced-toggle:hover {
  color: var(--text);
  background: rgba(255, 255, 255, 0.04);
}
.ob-arrow {
  display: inline-block;
  transition: transform 0.15s;
  color: var(--green);
  font-weight: 700;
}

.ob-advanced {
  margin-top: 2px;
  text-align: left;
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  border-radius: 14px;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.ob-field { display: flex; flex-direction: column; gap: 10px; }
.ob-field-inner { display: flex; flex-direction: column; gap: 6px; margin-top: 2px; }

.ob-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11.5px;
  font-weight: 700;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.8px;
}
.ob-label-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--green);
}

.ob-scope-card,
.ob-mode-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
  position: relative;
}
.ob-scope-card:hover,
.ob-mode-card:hover {
  border-color: rgba(163, 230, 53, 0.4);
  transform: translateY(-1px);
}
.ob-scope-card.is-rec,
.ob-mode-card.is-rec {
  background: linear-gradient(180deg, rgba(163, 230, 53, 0.06), var(--bg-card));
  border-color: rgba(163, 230, 53, 0.25);
}

.ob-scope-check {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  border-radius: 50%;
  border: 1.5px solid var(--border);
  display: grid;
  place-items: center;
  background: var(--bg);
  color: var(--green);
  transition: all 0.15s;
  margin-top: 1px;
}
.ob-scope-check.on {
  background: var(--green);
  color: #0a0f05;
  border-color: var(--green);
  box-shadow: 0 0 0 3px var(--green-glow);
}

.ob-scope-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
.ob-scope-title {
  font-size: 14px;
  font-weight: 700;
  color: #fff;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.ob-scope-desc {
  font-size: 12px;
  color: var(--text-dim);
  line-height: 1.5;
}

.ob-rec-badge {
  position: absolute;
  top: 10px;
  right: 10px;
  padding: 2px 8px;
  background: rgba(163, 230, 53, 0.12);
  border: 1px solid rgba(163, 230, 53, 0.3);
  color: var(--green);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.4px;
  border-radius: 999px;
  text-transform: uppercase;
}
.ob-rec-badge-inline {
  padding: 1px 7px;
  background: rgba(163, 230, 53, 0.12);
  border: 1px solid rgba(163, 230, 53, 0.3);
  color: var(--green);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.4px;
  border-radius: 999px;
  text-transform: uppercase;
}

.ob-input-row {
  display: flex;
  gap: 8px;
}
.ob-input {
  flex: 1;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 9px 12px;
  color: var(--text);
  font-family: var(--mono);
  font-size: 12.5px;
  outline: none;
}
.ob-input:focus { border-color: rgba(163, 230, 53, 0.5); }

.ob-btn {
  padding: 9px 14px;
  font-size: 12.5px;
  border-radius: 10px;
}
.ob-hint {
  font-size: 11.5px;
  color: var(--text-faint);
  margin: 0;
  line-height: 1.5;
}

.ob-bullets {
  list-style: none;
  padding: 0;
  margin: 2px 0 0 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.ob-bullets li {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  color: var(--text-dim);
  line-height: 1.4;
}
.ob-bullets li svg { color: var(--green); flex-shrink: 0; }
.ob-bullets li:nth-child(2) svg { color: var(--amber); }
.ob-bullets li:nth-child(3) svg { color: var(--red); }

.ob-protect-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
  gap: 8px;
}
.ob-protect-pill {
  padding: 8px 10px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
  display: flex;
  align-items: center;
  gap: 6px;
}
.cat-file { border-left: 3px solid var(--blue); }
.cat-shell { border-left: 3px solid var(--amber); }
.cat-git { border-left: 3px solid var(--orange); }
.cat-package { border-left: 3px solid var(--green-check); }
.cat-secret { border-left: 3px solid var(--purple); }

.ob-error {
  width: 100%;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #fca5a5;
  padding: 10px 14px;
  border-radius: 10px;
  font-size: 12.5px;
  line-height: 1.5;
}

.ob-divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--border), transparent);
  margin: 2px 0 0 0;
}

.ob-privacy {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.ob-privacy-row {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-dim);
  font-weight: 700;
  font-size: 13px;
}
.ob-privacy-row svg { color: var(--green); flex-shrink: 0; }
.ob-privacy-title { color: #fff; }
.ob-privacy-desc {
  font-size: 12px;
  color: var(--text-dim);
  line-height: 1.6;
  margin: 0;
}
.ob-privacy-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.ob-privacy-list li {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 11.5px;
  color: var(--text-dim);
  line-height: 1.4;
}
.ob-privacy-list li svg { color: var(--green); flex-shrink: 0; }

/* ---------- Next-step preview + Skip link (so users know WHERE the green button goes) ---------- */
.ob-next {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px 16px;
  background: rgba(163, 230, 53, 0.04);
  border: 1px solid rgba(163, 230, 53, 0.18);
  border-radius: 12px;
}
.ob-next-k {
  font-size: 12px;
  font-weight: 600;
  color: var(--green);
  letter-spacing: 0.3px;
}
.ob-next-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.ob-next-chip {
  display: inline-flex;
  align-items: center;
  padding: 5px 10px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-dim);
  font-size: 11.5px;
  font-weight: 600;
  border-radius: 999px;
}

.ob-skip {
  margin-top: 14px;
  width: 100%;
  background: transparent;
  border: 1px dashed var(--border);
  padding: 10px 12px;
  border-radius: 10px;
  color: var(--text-faint);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  transition: all 0.15s;
  font-family: var(--sans);
}
.ob-skip:hover {
  color: var(--green);
  border-color: rgba(163, 230, 53, 0.35);
  background: rgba(163, 230, 53, 0.04);
}

/* Scrollbar */
.onboarding-card::-webkit-scrollbar { width: 8px; }
.onboarding-card::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 4px;
}

@media (max-width: 520px) {
  .onboarding-card { padding: 32px 22px; }
  .ob-title { font-size: 26px; }
  .ob-tagline { font-size: 15.5px; }
}
</style>
