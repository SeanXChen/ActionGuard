<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api";
import { useI18n } from "../i18n";
import type { CapabilityTier, ExecutionPath } from "../types";

const TIER_LABEL: Record<CapabilityTier, string> = {
  l1_observe: "L1",
  l2_pre_action: "L2",
  l3_runtime: "L3",
  l4_system: "L4",
};

const { t } = useI18n();

const paths = ref<ExecutionPath[]>([]);
const loaded = ref(false);

onMounted(async () => {
  try {
    paths.value = await api.getEnforcementPaths();
  } catch {
    paths.value = [];
  } finally {
    loaded.value = true;
  }
});
</script>

<template>
  <div class="card enforcement">
    <div class="ef-head">
      <div>
        <div class="section-label">{{ t("enforcement.title") }}</div>
        <p class="section-sub">{{ t("enforcement.subtitle") }}</p>
      </div>
    </div>
    <table v-if="loaded && paths.length" class="ef-table">
      <thead>
        <tr>
          <th>{{ t("enforcement.col.path") }}</th>
          <th>{{ t("enforcement.col.tier") }}</th>
          <th>{{ t("enforcement.col.observe") }}</th>
          <th>{{ t("enforcement.col.block") }}</th>
          <th>{{ t("enforcement.col.note") }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="p in paths" :key="p.path">
          <td class="ef-path">{{ p.path }}</td>
          <td>
            <span
              v-if="p.tier"
              class="ef-tag"
              :class="p.tier === 'l2_pre_action' ? 'ef-yes' : 'ef-obs'"
            >
              {{ TIER_LABEL[p.tier] }}
            </span>
            <span v-else class="ef-tag ef-no">{{ t("enforcement.notCovered") }}</span>
          </td>
          <td>
            <span class="ef-tag" :class="p.observe ? 'ef-yes' : 'ef-no'">
              {{ p.observe ? t("enforcement.blocked") : t("enforcement.observeOnly") }}
            </span>
          </td>
          <td>
            <span class="ef-tag" :class="p.block ? 'ef-yes' : 'ef-no'">
              {{ p.block ? t("enforcement.blocked") : t("enforcement.observeOnly") }}
            </span>
          </td>
          <td class="ef-note">{{ p.note }}</td>
        </tr>
      </tbody>
    </table>
    <p v-else-if="loaded && !paths.length" class="ef-empty">—</p>
    <p v-else class="ef-empty">…</p>
  </div>
</template>

<style scoped>
.enforcement {
  padding: 16px 18px;
}

.ef-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 10px;
}

.ef-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12.5px;
}

.ef-table th {
  text-align: left;
  font-size: 10px;
  letter-spacing: 1px;
  text-transform: uppercase;
  color: var(--text-faint);
  font-family: var(--mono);
  padding: 6px 8px;
  border-bottom: 1px solid var(--border);
}

.ef-table td {
  padding: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  color: var(--text-dim);
  vertical-align: top;
}

.ef-path {
  color: var(--text);
  font-family: var(--mono);
  white-space: nowrap;
}

.ef-note {
  font-size: 11.5px;
}

.ef-tag {
  display: inline-block;
  padding: 1px 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 800;
  font-family: var(--mono);
}

.ef-yes {
  color: var(--green);
  background: var(--green-glow);
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.ef-no {
  color: #fca5a5;
  background: var(--red-glow);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.ef-obs {
  color: var(--amber, #fbbf24);
  background: var(--amber-glow, rgba(251, 191, 36, 0.12));
  border: 1px solid rgba(251, 191, 36, 0.3);
}

.ef-empty {
  color: var(--text-faint);
  font-size: 12px;
}
</style>
