<script setup lang="ts">
import { ref, watch } from "vue";
import { useSettings } from "../composables/useSettings";
import { t } from "../i18n";
import type { LanguagePref } from "../lib/types";

const { settings, error, save, changeLanguage } = useSettings();
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

function onLanguage(event: Event) {
  void changeLanguage((event.target as HTMLSelectElement).value as LanguagePref);
}
</script>

<template>
  <div class="panel">
    <form class="row" @submit.prevent="onSubmit">
      <label class="row">
        <span>{{ t("settings.checkEvery") }}</span>
        <input
          v-model.number="pollInterval"
          class="minutes"
          type="number"
          min="1"
          step="1"
          required
          @input="saved = false"
        />
        <span>{{ t("settings.minutes") }}</span>
      </label>
      <button type="submit" :disabled="!settings">{{ t("settings.save") }}</button>
      <span v-if="saved" class="muted">{{ t("settings.saved") }}</span>
    </form>

    <label class="row">
      <span>{{ t("settings.language") }}</span>
      <select :value="settings?.language ?? 'auto'" :disabled="!settings" @change="onLanguage">
        <option value="auto">{{ t("settings.languageAuto") }}</option>
        <!-- Language names stay in their own language so anyone can find theirs. -->
        <option value="en">English</option>
        <option value="pt">Português</option>
      </select>
    </label>
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
</style>
