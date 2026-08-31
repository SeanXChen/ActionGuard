<!--
  TeleportMenu — renders slot content via Teleport to <body>.

  This component has NO scoped styles. The menu appearance is driven by
  the CSS classes in the parent component (dd-item, etc.), which the
  parent writes inline inside its own <style> block.

  Position is passed via :style from the parent computed from the
  trigger button's getBoundingClientRect().
-->
<template>
  <Teleport to="body">
    <div class="dd-portal" :style="style" @click.stop>
      <slot />
    </div>
  </Teleport>
</template>

<script setup lang="ts">
defineProps<{ style?: Record<string, string> }>();
</script>

<style>
.dd-portal {
  position: fixed;
  z-index: 9999;
  min-width: 200px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 6px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  gap: 2px;
  animation: ddFade 120ms ease;
}
@keyframes ddFade {
  from { opacity: 0; transform: translateY(-6px); }
  to   { opacity: 1; transform: translateY(0); }
}
</style>
