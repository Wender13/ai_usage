<script setup lang="ts">
import UsageBar from "./UsageBar.vue";
import { t } from "../i18n";
import { formatTime, providerLabel } from "../lib/format";
import type { ProviderStatus } from "../lib/types";

defineProps<{
  status: ProviderStatus;
  now: number;
}>();
</script>

<template>
  <section class="card">
    <h3>{{ providerLabel(status.provider) }}</h3>

    <p v-if="status.error" class="error" role="alert">{{ status.error.message }}</p>

    <p v-if="!status.windows.length && !status.error" class="muted">{{ t("status.loading") }}</p>

    <UsageBar
      v-for="usage in status.windows"
      :key="usage.kind"
      :usage="usage"
      :now="now"
      :stale="status.error !== null"
    />

    <p v-if="status.error && status.lastSuccessAt" class="muted note">
      {{ t("status.stale", { time: formatTime(status.lastSuccessAt) }) }}
    </p>
  </section>
</template>

<style scoped>
.card {
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
}

.card + .card {
  margin-top: 12px;
}

h3 {
  margin: 0 0 12px;
  font-size: 1rem;
}

.muted {
  margin: 0;
  color: var(--muted);
  font-size: 0.875rem;
}

.note {
  margin-top: 12px;
}
</style>
