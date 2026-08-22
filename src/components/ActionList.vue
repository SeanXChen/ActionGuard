<script setup lang="ts">
import { computed } from "vue";
import type { ActionKind, FileChange } from "../types";
import { useI18n } from "../i18n";

const props = defineProps<{ actions: FileChange[]; limit?: number }>();
const { t } = useI18n();

const groups = computed(() => {
  const order: ActionKind[] = ["delete", "rename", "modify", "create"];
  const map = new Map<ActionKind, FileChange[]>();
  for (const k of order) map.set(k, []);
  for (const a of props.actions) map.get(a.action)?.push(a);
  return order
    .map((kind) => ({ kind, items: map.get(kind) ?? [] }))
    .filter((g) => g.items.length > 0);
});

const groupTitle = computed(() => (k: ActionKind) => {
  switch (k) {
    case "delete":
      return t("list.group.delete");
    case "rename":
      return t("list.group.rename");
    case "modify":
      return t("list.group.modify");
    case "create":
      return t("list.group.create");
  }
});

const tag = (kind: ActionKind) => `tag-${kind}`;

function shown(items: FileChange[]): FileChange[] {
  const lim = props.limit ?? 50;
  return items.slice(0, lim);
}
</script>

<template>
  <div class="action-groups">
    <section v-for="g in groups" :key="g.kind" class="agroup">
      <header>
        <span class="gk">{{ groupTitle(g.kind) }}</span>
        <span class="gn">{{ g.items.length }}</span>
      </header>
      <div class="row-list">
        <div v-for="(a, i) in shown(g.items)" :key="i" class="row">
          <span class="tag" :class="tag(a.action)">
            {{ a.action.toUpperCase() }}
          </span>
          <span class="p">
            <template v-if="a.action === 'rename' && a.from">
              <span class="dimmed">{{ a.from }}</span>
              <span class="arrow"> {{ t("list.renameTo") }} </span>
            </template>
            {{ a.path }}
          </span>
          <span v-if="a.sensitive" class="tag tag-flag">
            {{ t("list.tag.sensitive") }}
          </span>
          <span v-if="a.outside" class="tag tag-flag">
            {{ t("list.tag.outside") }}
          </span>
        </div>
        <div v-if="g.items.length > (limit ?? 50)" class="more">
          {{ t("list.more") }} ({{ g.items.length - (limit ?? 50) }})
        </div>
      </div>
    </section>
    <div v-if="groups.length === 0" class="empty-small">
      {{ t("list.none") }}
    </div>
  </div>
</template>

<style scoped>
.action-groups {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.agroup header {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 8px;
}

.gk {
  font-family: var(--mono);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 1.5px;
  text-transform: uppercase;
}

.gn {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text-faint);
}

.dimmed {
  color: var(--text-faint);
  text-decoration: line-through;
}

.arrow {
  color: var(--text-faint);
}

.more {
  font-size: 12px;
  color: var(--text-faint);
  padding: 4px 2px;
  font-family: var(--mono);
}

.empty-small {
  color: var(--text-faint);
  font-size: 13px;
  padding: 12px 0;
}
</style>
