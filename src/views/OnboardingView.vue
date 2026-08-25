<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "../i18n";
import { api } from "../api";

const { t, tf } = useI18n();
const emit = defineEmits<{ done: [] }>();

const workspaceDir = ref<string>(".");
const showAdvanced = ref(false);
const starting = ref(false);
const error = ref<string | null>(null);

function selectWorkspace() {
  const input = document.createElement("input");
  input.type = "file";
  input.webkitdirectory = true;
  input.addEventListener("change", () => {
    const f = input.files?.[0];
    if (f) {
      // Use webkitRelativePath to infer directory name
      const parts = f.webkitRelativePath.split("/");
      workspaceDir.value = parts[0] || ".";
    }
  });
  input.click();
}

async function protect() {
  starting.value = true;
  error.value = null;
  try {
    await api.startSession(workspaceDir.value, "protected");
    emit("done");
  } catch (e) {
    error.value = (e as Error).message || t("onboarding.startError");
  } finally {
    starting.value = false;
  }
}
</script>

<template>
  <div class="onboarding-screen">
    <div class="onboarding-card">
      <div class="ob-hero">
        <div class="ob-shield">🛡</div>
        <h1 class="ob-title">ActionGuard</h1>
        <p class="ob-tagline">{{ t("onboarding.tagline") }}</p>
      </div>

      <button
        class="btn btn-primary ob-cta"
        :disabled="starting"
        @click="protect"
      >
        <span v-if="starting" class="ob-spinner" />
        <span v-else>🛡</span>
        {{ starting ? t("onboarding.starting") : t("onboarding.cta") }}
      </button>

      <div class="ob-defaults">
        {{ tf("onboarding.defaults", { dir: workspaceDir }) }}
      </div>

      <button class="ob-advanced-toggle" @click="showAdvanced = !showAdvanced">
        {{ showAdvanced ? "▾" : "▸" }} {{ t("onboarding.advanced") }}
      </button>

      <div v-if="showAdvanced" class="ob-advanced">
        <div class="ob-field">
          <label>{{ t("onboarding.workspace") }}</label>
          <div class="ob-input-row">
            <input
              v-model="workspaceDir"
              class="ob-input"
              type="text"
              readonly
            />
            <button class="btn" @click="selectWorkspace">
              {{ t("home.consumer.onboarding.changeDir") }}
            </button>
          </div>
          <p class="ob-hint">{{ t("onboarding.workspaceHint") }}</p>
        </div>
      </div>

      <div v-if="error" class="ob-error">{{ error }}</div>

      <div class="ob-privacy">
        <span class="ob-privacy-icon">🔒</span>
        <span>{{ t("onboarding.privacy") }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.onboarding-screen {
  height: 100vh;
  display: grid;
  place-items: center;
  padding: 24px;
  background: radial-gradient(1200px 800px at 20% -10%, #14203c 0%, var(--bg) 55%) fixed;
}

.onboarding-card {
  width: 100%;
  max-width: 440px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 40px 36px 32px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 16px;
}

.ob-hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.ob-shield {
  font-size: 48px;
  line-height: 1;
  filter: drop-shadow(0 4px 20px rgba(34, 197, 94, 0.25));
}

.ob-title {
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 0.3px;
  color: var(--text);
}

.ob-tagline {
  font-size: 14px;
  color: var(--text-dim);
  line-height: 1.5;
  max-width: 300px;
}

.ob-cta {
  width: 100%;
  padding: 14px 20px;
  font-size: 15px;
  border-radius: 10px;
  margin-top: 4px;
}

.ob-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.ob-defaults {
  font-size: 12px;
  color: var(--text-faint);
}

.ob-advanced-toggle {
  background: transparent;
  border: none;
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.15s;
  font-family: var(--sans);
}

.ob-advanced-toggle:hover {
  color: var(--text);
  background: rgba(255, 255, 255, 0.04);
}

.ob-advanced {
  width: 100%;
  text-align: left;
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ob-field label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-dim);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.6px;
}

.ob-input-row {
  display: flex;
  gap: 8px;
}

.ob-input {
  flex: 1;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 12px;
  color: var(--text);
  font-family: var(--mono);
  font-size: 12px;
  outline: none;
}

.ob-hint {
  font-size: 11px;
  color: var(--text-faint);
  margin-top: 4px;
}

.ob-error {
  width: 100%;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #fca5a5;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 12px;
}

.ob-privacy {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-faint);
  margin-top: 8px;
}

.ob-privacy-icon {
  font-size: 14px;
}
</style>
