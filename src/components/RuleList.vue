<script setup lang="ts">
import { CHANNEL_LABELS, describeRule } from "../lib/format";
import type { Rule } from "../lib/types";

defineProps<{
  rules: Rule[];
  editingId: string | null;
}>();

const emit = defineEmits<{
  edit: [rule: Rule];
  toggle: [rule: Rule];
  remove: [rule: Rule];
}>();
</script>

<template>
  <p v-if="!rules.length" class="empty">Nenhuma regra cadastrada.</p>

  <ul v-else class="rules">
    <li
      v-for="rule in rules"
      :key="rule.id"
      :class="{ disabled: !rule.enabled, editing: rule.id === editingId }"
    >
      <input
        type="checkbox"
        :checked="rule.enabled"
        :aria-label="rule.enabled ? 'Desativar regra' : 'Ativar regra'"
        @change="emit('toggle', rule)"
      />
      <div class="text">
        <span>{{ describeRule(rule.kind) }}</span>
        <span class="channel">{{ CHANNEL_LABELS[rule.channel] }}</span>
      </div>
      <div class="actions">
        <button type="button" @click="emit('edit', rule)">Editar</button>
        <button type="button" class="danger" @click="emit('remove', rule)">Excluir</button>
      </div>
    </li>
  </ul>
</template>

<style scoped>
.empty {
  margin: 0;
  color: var(--muted);
  font-size: 0.875rem;
}

.rules {
  margin: 0;
  padding: 0;
  list-style: none;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
}

li {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
}

li + li {
  border-top: 1px solid var(--border);
}

li.editing {
  box-shadow: inset 3px 0 0 var(--accent);
}

.text {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.channel {
  color: var(--muted);
  font-size: 0.8125rem;
}

.disabled .text {
  opacity: 0.55;
}

.actions {
  display: flex;
  gap: 6px;
}

.actions button {
  padding: 4px 10px;
  font-size: 0.8125rem;
}
</style>
