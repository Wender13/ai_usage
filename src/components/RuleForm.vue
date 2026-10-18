<script setup lang="ts">
import { reactive } from "vue";
import { CHANNEL_LABELS, PROVIDER_LABELS, RULE_TYPE_LABELS, WINDOW_LABELS } from "../lib/format";
import type {
  Channel,
  ProviderId,
  Rule,
  RuleInput,
  RuleKind,
  RuleType,
  WindowKind,
} from "../lib/types";

const props = defineProps<{
  /** Rule being edited; absent when creating. */
  rule?: Rule;
}>();

const emit = defineEmits<{
  submit: [input: RuleInput];
  cancel: [];
}>();

const typeOptions = Object.keys(RULE_TYPE_LABELS) as RuleType[];
const providerOptions = Object.keys(PROVIDER_LABELS) as ProviderId[];
const windowOptions = Object.keys(WINDOW_LABELS) as WindowKind[];
const channelOptions = Object.keys(CHANNEL_LABELS) as Channel[];

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
      <span>Tipo</span>
      <select v-model="form.type">
        <option v-for="type in typeOptions" :key="type" :value="type">
          {{ RULE_TYPE_LABELS[type] }}
        </option>
      </select>
    </label>

    <template v-if="form.type !== 'interval'">
      <label class="field">
        <span>Provider</span>
        <select v-model="form.provider">
          <option :value="null">Qualquer</option>
          <option v-for="provider in providerOptions" :key="provider" :value="provider">
            {{ PROVIDER_LABELS[provider] }}
          </option>
        </select>
      </label>

      <label class="field">
        <span>Janela</span>
        <select v-model="form.window">
          <option :value="null">Qualquer</option>
          <option v-for="window in windowOptions" :key="window" :value="window">
            {{ WINDOW_LABELS[window] }}
          </option>
        </select>
      </label>
    </template>

    <label v-if="form.type === 'threshold'" class="field">
      <span>Limiar (%)</span>
      <input v-model.number="form.percent" type="number" min="1" max="100" step="1" required />
    </label>

    <label v-if="form.type === 'interval'" class="field">
      <span>A cada (min)</span>
      <input v-model.number="form.intervalMinutes" type="number" min="1" step="1" required />
    </label>

    <label v-if="form.type === 'beforeReset'" class="field">
      <span>Minutos antes do reset</span>
      <input v-model.number="form.beforeResetMinutes" type="number" min="1" step="1" required />
    </label>

    <label class="field">
      <span>Canal</span>
      <select v-model="form.channel">
        <option v-for="channel in channelOptions" :key="channel" :value="channel">
          {{ CHANNEL_LABELS[channel] }}
        </option>
      </select>
    </label>

    <p v-if="form.type !== 'threshold'" class="hint">
      Verificada a cada consulta de uso, então a precisão é a do intervalo de consulta.
    </p>

    <div class="actions">
      <button type="button" @click="emit('cancel')">Cancelar</button>
      <button type="submit" class="primary">{{ rule ? "Salvar" : "Criar regra" }}</button>
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
