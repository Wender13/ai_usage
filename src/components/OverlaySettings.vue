<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useSettings } from "../composables/useSettings";
import { t } from "../i18n";
import { ANCHORS, anchorLabel, providerLabel, windowKey, windowLabel } from "../lib/format";
import type { Anchor, OverlayPrefs, UsageWindow } from "../lib/types";

defineProps<{
  /** Windows currently reported by the providers, offered for hiding. */
  windows: UsageWindow[];
}>();

const { settings, error, setOverlay, changeOverlay, setAnchor, setXwayland } = useSettings();

const CUSTOM = "custom";

type FieldKey = keyof OverlayPrefs & `show${string}`;
const FIELDS: FieldKey[] = ["showProvider", "showLabel", "showPercent", "showReset"];

// Positioning only works on X11, so the XWayland option matters only on Linux.
const isLinux = navigator.userAgent.includes("Linux");

const prefs = computed(() => settings.value?.overlay);
const position = computed(() =>
  settings.value?.overlayPosition ? CUSTOM : (prefs.value?.anchor ?? "top_right"),
);

// Local copy so the label follows the slider while it moves; saved on release.
const opacity = ref(86);
watch(
  () => prefs.value?.opacity,
  (value) => {
    if (value !== undefined) opacity.value = value;
  },
);

const checked = (event: Event) => (event.target as HTMLInputElement).checked;

function onPosition(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  if (value !== CUSTOM) void setAnchor(value as Anchor);
}

function onWindowToggle(usage: UsageWindow, visible: boolean) {
  if (!prefs.value) return;
  const key = windowKey(usage);
  const hidden = prefs.value.hiddenWindows.filter((existing) => existing !== key);
  void changeOverlay({ hiddenWindows: visible ? hidden : [...hidden, key] });
}
</script>

<template>
  <div class="panel">
    <label class="row">
      <input
        type="checkbox"
        :checked="settings?.overlayEnabled ?? false"
        :disabled="!settings"
        @change="setOverlay(checked($event))"
      />
      <span>{{ t("overlay.show") }}</span>
    </label>

    <template v-if="settings && prefs">
      <div class="grid">
        <label class="field">
          <span>{{ t("overlay.position") }}</span>
          <select :value="position" @change="onPosition">
            <option v-for="anchor in ANCHORS" :key="anchor" :value="anchor">
              {{ anchorLabel(anchor) }}
            </option>
            <option v-if="settings.overlayPosition" :value="CUSTOM" disabled>
              {{ t("overlay.custom") }}
            </option>
          </select>
        </label>

        <label class="field">
          <span>{{ t("overlay.opacity", { value: opacity }) }}</span>
          <input
            v-model.number="opacity"
            type="range"
            min="0"
            max="100"
            step="5"
            @change="changeOverlay({ opacity })"
          />
        </label>
      </div>

      <label class="row">
        <input
          type="checkbox"
          :checked="prefs.locked"
          @change="changeOverlay({ locked: checked($event) })"
        />
        <span>{{ t("overlay.lock") }}</span>
      </label>

      <fieldset>
        <legend>{{ t("overlay.fieldsLegend") }}</legend>
        <label v-for="field in FIELDS" :key="field" class="row">
          <input
            type="checkbox"
            :checked="prefs[field]"
            @change="changeOverlay({ [field]: checked($event) })"
          />
          <span>{{ t(`overlay.fields.${field}`) }}</span>
        </label>
      </fieldset>

      <fieldset v-if="windows.length">
        <legend>{{ t("overlay.windowsLegend") }}</legend>
        <label v-for="usage in windows" :key="windowKey(usage)" class="row">
          <input
            type="checkbox"
            :checked="!prefs.hiddenWindows.includes(windowKey(usage))"
            @change="onWindowToggle(usage, checked($event))"
          />
          <span>{{ providerLabel(usage.provider) }} · {{ windowLabel(usage.kind) }}</span>
        </label>
      </fieldset>

      <div v-if="isLinux">
        <label class="row">
          <input
            type="checkbox"
            :checked="settings.useXwayland"
            @change="setXwayland(checked($event))"
          />
          <span>{{ t("overlay.xwayland") }}</span>
        </label>
        <p class="hint">
          {{ t("overlay.xwaylandHint") }}
        </p>
      </div>
    </template>
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

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.row {
  display: flex;
  align-items: center;
  gap: 8px;
}

fieldset {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0;
  padding: 10px 12px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
}

legend {
  padding: 0 4px;
  color: var(--muted);
  font-size: 0.875rem;
}

.hint {
  margin: 4px 0 0 24px;
  color: var(--muted);
  font-size: 0.8125rem;
}
</style>
