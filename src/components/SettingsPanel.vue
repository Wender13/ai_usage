<script setup lang="ts">
import { ref, watch } from "vue";
import { useSettings } from "../composables/useSettings";

const { settings, error, save, setOverlay } = useSettings();
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

function onOverlayChange(event: Event) {
  void setOverlay((event.target as HTMLInputElement).checked);
}
</script>

<template>
  <div class="panel">
    <form class="row" @submit.prevent="onSubmit">
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

    <div>
      <label class="row">
        <input
          type="checkbox"
          :checked="settings?.overlayEnabled ?? false"
          :disabled="!settings"
          @change="onOverlayChange"
        />
        <span>Mostrar overlay</span>
      </label>
      <p class="muted hint">
        No GNOME em Wayland o overlay não fica sempre no topo; o README explica como rodar via
        XWayland.
      </p>
    </div>
  </div>
  <p v-if="error" class="error">{{ error }}</p>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
}

.row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 12px;
}

.minutes {
  width: 72px;
}

.muted {
  color: var(--muted);
  font-size: 0.875rem;
}

.hint {
  margin: 4px 0 0 28px;
  font-size: 0.8125rem;
}
</style>
