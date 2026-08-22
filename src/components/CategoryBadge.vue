<script setup lang="ts">
import { computed } from "vue";
import type { ActionCategory } from "../types";
import { useI18n } from "../i18n";

const props = defineProps<{ category: ActionCategory }>();
const { t } = useI18n();

const cls = computed(() => ({
  "category-badge": true,
  [`category-${props.category}`]: true,
}));

const label = computed(() => {
  switch (props.category) {
    case "file":
      return t("category.file");
    case "shell":
      return t("category.shell");
    case "git":
      return t("category.git");
    case "package":
      return t("category.package");
    case "secret":
      return t("category.secret");
  }
});
</script>

<template>
  <span :class="cls">{{ label }}</span>
</template>
