<script setup lang="ts">
import { reactive } from "vue";
import { t } from "../i18n";
import {
  CHANNELS,
  channelLabel,
  PROVIDERS,
  providerLabel,
  RULE_TYPES,
  ruleTypeLabel,
  WINDOW_KINDS,
  windowLabel,
} from "../lib/format";
import type { Rule, RuleInput, RuleKind } from "../lib/types";

const props = defineProps<{
  /** Rule being edited; absent when creating. */
  rule?: Rule;
}>();

const emit = defineEmits<{
  submit: [input: RuleInput];
  cancel: [];
}>();


const kind = props.rule?.kind;
const form = reactive({
  type: kind?.type ?? "threshold",
  provider: kind && kind.type !== "interval" ? kind.provider : null,
  window: kind && kind.type !== "interval" ? kind.window : null,
  percent: kind?.type === "threshold" ? kind.percent : 80,
  intervalMinutes: kind?.type === "interval" ? kind.minutes : 60,
  beforeResetMinutes: kind?.type === "beforeReset" ? kind.minutes : 15,
  channel: props.rule?.channel ?? "notification",
});

function buildKind(): RuleKind {
  switch (form.type) {
    case "threshold":
      return {
        type: "threshold",
        provider: form.provider,
        window: form.window,
        percent: form.percent,
      };
    case "interval":
      return { type: "interval", minutes: form.intervalMinutes };
    case "beforeReset":
      return {
        type: "beforeReset",
        provider: form.provider,
        window: form.window,
        minutes: form.beforeResetMinutes,
      };
  }
}

function onSubmit() {
  emit("submit", {
    enabled: props.rule?.enabled ?? true,
    channel: form.channel,
    kind: buildKind(),
  });
}
</script>

<template>
  <form class="rule-form" @submit.prevent="onSubmit">
    <label class="field">
      <span>{{ t("rules.form.type") }}</span>
      <select v-model="form.type">
        <option v-for="type in RULE_TYPES" :key="type" :value="type">
          {{ ruleTypeLabel(type) }}
        </option>
      </select>
    </label>

    <template v-if="form.type !== 'interval'">
      <label class="field">
        <span>{{ t("rules.form.provider") }}</span>
        <select v-model="form.provider">
          <option :value="null">{{ t("rules.form.any") }}</option>
          <option v-for="provider in PROVIDERS" :key="provider" :value="provider">
            {{ providerLabel(provider) }}
          </option>
        </select>
      </label>

      <label class="field">
        <span>{{ t("rules.form.window") }}</span>
        <select v-model="form.window">
          <option :value="null">{{ t("rules.form.any") }}</option>
          <option v-for="window in WINDOW_KINDS" :key="window" :value="window">
            {{ windowLabel(window) }}
          </option>
        </select>
      </label>
    </template>

    <label v-if="form.type === 'threshold'" class="field">
      <span>{{ t("rules.form.threshold") }}</span>
      <input v-model.number="form.percent" type="number" min="1" max="100" step="1" required />
    </label>

    <label v-if="form.type === 'interval'" class="field">
      <span>{{ t("rules.form.every") }}</span>
      <input v-model.number="form.intervalMinutes" type="number" min="1" step="1" required />
    </label>

    <label v-if="form.type === 'beforeReset'" class="field">
      <span>{{ t("rules.form.minutesBefore") }}</span>
      <input v-model.number="form.beforeResetMinutes" type="number" min="1" step="1" required />
    </label>

    <label class="field">
      <span>{{ t("rules.form.channel") }}</span>
      <select v-model="form.channel">
        <option v-for="channel in CHANNELS" :key="channel" :value="channel">
          {{ channelLabel(channel) }}
        </option>
      </select>
    </label>

    <p v-if="form.type !== 'threshold'" class="hint">
      {{ t("rules.form.hint") }}
    </p>

    <div class="actions">
      <button type="button" @click="emit('cancel')">{{ t("rules.form.cancel") }}</button>
      <button type="submit" class="primary">{{ rule ? t("rules.form.save") : t("rules.form.create") }}</button>
    </div>
  </form>
</template>

<style scoped>
.rule-form {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 12px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
}

.hint {
  grid-column: 1 / -1;
  margin: 0;
  color: var(--muted);
  font-size: 0.8125rem;
}

.actions {
  grid-column: 1 / -1;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
