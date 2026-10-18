<script setup lang="ts">
import { computed } from "vue";
import { formatPercent, formatReset, usageLevel, windowLabel } from "../lib/format";
import type { UsageWindow } from "../lib/types";

const props = defineProps<{
  usage: UsageWindow;
  now: number;
  stale: boolean;
}>();

const label = computed(() => windowLabel(props.usage.kind));
const level = computed(() => usageLevel(props.usage.utilization));
</script>

<template>
  <div class="usage" :class="{ stale }">
    <div class="row">
      <span class="label">{{ label }}</span>
      <span class="percent">{{ formatPercent(usage.utilization) }}</span>
    </div>
    <div
      class="track"
      role="progressbar"
      :aria-label="label"
      aria-valuemin="0"
      aria-valuemax="100"
      :aria-valuenow="Math.round(usage.utilization)"
    >
      <div class="fill" :class="level" :style="{ width: `${usage.utilization}%` }"></div>
    </div>
    <div class="reset">{{ formatReset(usage.resetsAt, now) }}</div>
  </div>
</template>

<style scoped>
.usage + .usage {
  margin-top: 14px;
}

.row {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 6px;
}

.label {
  font-weight: 500;
}

.percent {
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.track {
  height: 8px;
  border-radius: 4px;
  background: var(--track);
  overflow: hidden;
}

.fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.fill.low {
  background: var(--accent);
}

.fill.medium {
  background: var(--warn);
}

.fill.high {
  background: var(--danger);
}

.reset {
  margin-top: 4px;
  font-size: 0.8125rem;
  color: var(--muted);
}

.stale .fill {
  opacity: 0.45;
}
</style>
