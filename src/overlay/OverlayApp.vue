<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useOverlayAlerts } from "../composables/useOverlayAlerts";
import { useUsage } from "../composables/useUsage";
import { formatPercent, providerLabel, usageLevel, WINDOW_SHORT_LABELS } from "../lib/format";

const { snapshot } = useUsage();
const { alerts, dismiss } = useOverlayAlerts();

// The window follows the content size, so no empty transparent area blocks clicks.
const root = ref<HTMLElement | null>(null);
let observer: ResizeObserver | undefined;

onMounted(() => {
  observer = new ResizeObserver(([entry]) => {
    const { width, height } = entry.target.getBoundingClientRect();
    getCurrentWindow()
      .setSize(new LogicalSize(Math.ceil(width), Math.ceil(height)))
      .catch((error) => console.error("setSize failed", error));
  });
  if (root.value) observer.observe(root.value);
});
onUnmounted(() => observer?.disconnect());
</script>

<template>
  <div ref="root" class="overlay" data-tauri-drag-region="deep">
    <button
      v-if="alerts.length"
      type="button"
      class="alert"
      title="Clique para dispensar"
      @click="dismiss"
    >
      <span class="alert-title">{{ alerts[0].title }}</span>
      <span class="alert-body">{{ alerts[0].body }}</span>
      <span v-if="alerts.length > 1" class="alert-more">+{{ alerts.length - 1 }} alerta(s)</span>
    </button>

    <div v-if="!snapshot?.fetchedAt" class="provider muted">Carregando…</div>

    <template v-else>
      <div
        v-for="status in snapshot.providers"
        :key="status.provider"
        class="provider"
        :class="{ stale: status.error }"
        :title="status.error?.message"
      >
        <span class="name">{{ providerLabel(status.provider) }}</span>
        <span v-for="usage in status.windows" :key="usage.kind" class="usage">
          <span class="kind">{{ WINDOW_SHORT_LABELS[usage.kind] }}</span>
          <span class="percent" :class="usageLevel(usage.utilization)">
            {{ formatPercent(usage.utilization) }}
          </span>
        </span>
        <span v-if="status.error" class="warning" aria-label="Erro na última consulta">!</span>
      </div>
    </template>
  </div>
</template>

<style>
html,
body {
  margin: 0;
  background: transparent;
}

body {
  font-family: system-ui, sans-serif;
  font-size: 13px;
  user-select: none;
  cursor: default;
}
</style>

<style scoped>
.overlay {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: max-content;
  max-width: 320px;
  padding: 8px 12px;
  border-radius: 10px;
  background: rgba(13, 17, 23, 0.86);
  color: #e6edf3;
}

.provider {
  display: flex;
  align-items: baseline;
  gap: 10px;
  white-space: nowrap;
}

.name {
  font-weight: 600;
}

.usage {
  display: inline-flex;
  gap: 4px;
}

.kind {
  color: #9198a1;
}

.percent {
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.percent.medium {
  color: #d29922;
}

.percent.high {
  color: #f85149;
}

.stale .usage {
  opacity: 0.5;
}

.warning {
  color: #f85149;
  font-weight: 700;
}

.muted {
  color: #9198a1;
}

.alert {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 8px;
  border: 1px solid #d29922;
  border-radius: 6px;
  background: rgba(210, 153, 34, 0.15);
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.alert-title {
  font-weight: 600;
}

.alert-body {
  white-space: pre-line;
}

.alert-more {
  color: #9198a1;
  font-size: 12px;
}
</style>
