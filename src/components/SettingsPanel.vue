<script setup lang="ts">
import { ref, watch } from "vue";
import { useSettings } from "../composables/useSettings";

const { settings, error, save } = useSettings();
const pollInterval = ref(3);
const saved = ref(false);

// Only follows the stored interval, so an unrelated change does not reset what is being typed.
watch(
  () => settings.value?.pollIntervalMinutes,
  (value) => {
    if (value !== undefined) pollInterval.value = value;
  },
);

async function onSubmit() {
  if (!settings.value) return;
  saved.value = await save({ ...settings.value, pollIntervalMinutes: pollInterval.value });
}
</script>

<template>
  <form class="panel" @submit.prevent="onSubmit">
    <label class="row">
      <span>Consultar o uso a cada</span>
      <input
        v-model.number="pollInterval"
        class="minutes"
        type="number"
        min="1"
        step="1"
        required
        @input="saved = false"
      />
      <span>min</span>
    </label>
    <button type="submit" :disabled="!settings">Salvar</button>
    <span v-if="saved" class="muted">Salvo.</span>
  </form>
  <p v-if="error" class="error">{{ error }}</p>
</template>

<style scoped>
.panel {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 12px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
}

.row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.minutes {
  width: 72px;
}

.muted {
  color: var(--muted);
  font-size: 0.875rem;
}
</style>
