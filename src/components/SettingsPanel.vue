<script setup lang="ts">
import { ref, watch } from "vue";
import { useSettings } from "../composables/useSettings";

const { settings, error, save } = useSettings();
const pollInterval = ref(3);
const saved = ref(false);

watch(settings, (value) => {
  if (value) pollInterval.value = value.pollIntervalMinutes;
});

async function onSubmit() {
  if (!settings.value) return;
  saved.value = await save({ ...settings.value, pollIntervalMinutes: pollInterval.value });
}
</script>

<template>
  <form class="settings" @submit.prevent="onSubmit">
    <label class="row">
      <span>Consultar o uso a cada</span>
      <input
        v-model.number="pollInterval"
        type="number"
        min="1"
        step="1"
        required
        @input="saved = false"
      />
      <span>min</span>
    </label>
    <button type="submit" :disabled="!settings">Salvar</button>
    <span v-if="saved" class="ok">Salvo.</span>
  </form>
  <p v-if="error" class="error">{{ error }}</p>
</template>

<style scoped>
.settings {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
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

.row input {
  width: 72px;
}

.ok {
  color: var(--muted);
  font-size: 0.875rem;
}
</style>
