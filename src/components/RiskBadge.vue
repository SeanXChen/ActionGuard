<script setup lang="ts">
import { computed } from "vue";
import type { RiskLevel } from "../types";
import { useI18n } from "../i18n";

const props = defineProps<{ level: RiskLevel; size?: "sm" | "md" }>();

const fontSize = computed(() => {
  if (props.size === "sm") return "10px";
  if (props.size === "md") return "12px";
  return "11px";
});
const { t } = useI18n();

const cls = computed(() => ({
  "risk-badge": true,
  "risk-low": props.level === "low",
  "risk-medium": props.level === "medium",
  "risk-high": props.level === "high",
  "risk-critical": props.level === "critical",
}));

const label = computed(() => {
  switch (props.level) {
    case "low":
      return t("risk.low");
    case "medium":
      return t("risk.medium");
    case "high":
      return t("risk.high");
    case "critical":
      return t("risk.critical");
  }
});
</script>

<template>
  <span :class="cls" :style="{ fontSize }">{{ label }}</span>
</template>
