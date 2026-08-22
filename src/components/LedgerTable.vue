<script setup lang="ts">
import { computed } from "vue";
import type { DeepReadonly } from "vue";
import type { LedgerEntry } from "../types";
import { useI18n } from "../i18n";
import RiskBadge from "./RiskBadge.vue";
import CategoryBadge from "./CategoryBadge.vue";

const props = defineProps<{
  entries: ReadonlyArray<DeepReadonly<LedgerEntry>>;
  emptyHint?: string;
}>();

const { t } = useI18n();

const hasRows = computed(() => props.entries.length > 0);
</script>

<template>
  <table v-if="hasRows" class="ledger-table">
    <thead>
      <tr>
        <th>{{ t("ledger.col.time") }}</th>
        <th>{{ t("ledger.col.source") }}</th>
        <th>{{ t("ledger.col.action") }}</th>
        <th>{{ t("ledger.col.target") }}</th>
        <th>{{ t("ledger.col.risk") }}</th>
        <th>{{ t("ledger.col.result") }}</th>
        <th>{{ t("ledger.col.reasons") }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="row in entries" :key="row.id">
        <td class="col-time">{{ row.time_hms }}</td>
        <td class="col-agent">{{ row.agent || "—" }}</td>
        <td class="col-action">
          <CategoryBadge :category="row.category" />
          <span class="kind">{{ row.kind }}</span>
        </td>
        <td class="col-target" :title="row.target">{{ row.target || "—" }}</td>
        <td class="col-risk"><RiskBadge :level="row.risk" /></td>
        <td class="col-result">{{ row.result || "—" }}</td>
        <td class="col-reasons">
          <span v-if="row.reasons.length">{{ row.reasons.join("; ") }}</span>
          <span v-else>—</span>
        </td>
      </tr>
    </tbody>
  </table>
  <div v-else class="ledger-empty">
    {{ emptyHint ?? t("ledger.empty") }}
  </div>
</template>

<style scoped>
.col-action {
  display: flex;
  align-items: center;
  gap: 6px;
}
.kind {
  font-size: 11px;
  color: var(--text-faint);
  text-transform: lowercase;
}
</style>
