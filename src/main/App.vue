<script setup lang="ts">
import { computed, ref } from "vue";
import OverlaySettings from "../components/OverlaySettings.vue";
import ProviderCard from "../components/ProviderCard.vue";
import RuleForm from "../components/RuleForm.vue";
import RuleList from "../components/RuleList.vue";
import SettingsPanel from "../components/SettingsPanel.vue";
import { useNow } from "../composables/useNow";
import { useRules } from "../composables/useRules";
import { useUsage } from "../composables/useUsage";
import { t } from "../i18n";
import { formatTime } from "../lib/format";
import type { Rule, RuleInput } from "../lib/types";

const { snapshot, refreshing, refresh } = useUsage();
const { rules, error: rulesError, create, update, remove } = useRules();
const now = useNow();
const windows = computed(() => snapshot.value?.providers.flatMap((status) => status.windows) ?? []);

const formOpen = ref(false);
/** Rule being edited; `null` while creating a new one. */
const editingRule = ref<Rule | null>(null);

function openForm(rule: Rule | null) {
  editingRule.value = rule;
  formOpen.value = true;
}

function closeForm() {
  formOpen.value = false;
  editingRule.value = null;
}

async function onSubmit(input: RuleInput) {
  const saved = editingRule.value
    ? await update({ ...input, id: editingRule.value.id })
    : await create(input);
  if (saved) closeForm();
}

const onToggle = (rule: Rule) => update({ ...rule, enabled: !rule.enabled });

async function onRemove(rule: Rule) {
  if ((await remove(rule.id)) && editingRule.value?.id === rule.id) closeForm();
}
</script>

<template>
  <main class="app">
    <header class="top">
      <div>
        <h1>AI Usage</h1>
        <p class="meta">
          <template v-if="snapshot?.fetchedAt">
            {{ t("app.lastCheck", { time: formatTime(snapshot.fetchedAt) }) }}
          </template>
          <template v-else>{{ t("app.waiting") }}</template>
        </p>
      </div>
      <button type="button" :disabled="refreshing" @click="refresh">
        {{ refreshing ? t("app.refreshing") : t("app.refresh") }}
      </button>
    </header>

    <ProviderCard
      v-for="status in snapshot?.providers ?? []"
      :key="status.provider"
      :status="status"
      :now="now"
    />

    <section class="section">
      <div class="section-header">
        <h2>{{ t("sections.rules") }}</h2>
        <button v-if="!formOpen" type="button" @click="openForm(null)">
          {{ t("rules.new") }}
        </button>
      </div>
      <p v-if="rulesError" class="error">{{ rulesError }}</p>
      <RuleForm
        v-if="formOpen"
        :key="editingRule?.id ?? 'new'"
        :rule="editingRule ?? undefined"
        @submit="onSubmit"
        @cancel="closeForm"
      />
      <RuleList
        :rules="rules"
        :editing-id="editingRule?.id ?? null"
        @edit="openForm"
        @toggle="onToggle"
        @remove="onRemove"
      />
    </section>

    <section class="section">
      <div class="section-header">
        <h2>{{ t("sections.settings") }}</h2>
      </div>
      <SettingsPanel />
    </section>

    <section class="section">
      <div class="section-header">
        <h2>{{ t("sections.overlay") }}</h2>
      </div>
      <OverlaySettings :windows="windows" />
    </section>
  </main>
</template>

<style>
:root {
  color-scheme: light dark;
  --bg: #f6f8fa;
  --surface: #ffffff;
  --text: #1f2328;
  --muted: #59636e;
  --border: #d1d9e0;
  --track: #e6eaef;
  --accent: #0969da;
  --accent-text: #ffffff;
  --warn: #bf8700;
  --danger: #cf222e;
  --danger-bg: #ffebe9;

  font-family: system-ui, sans-serif;
  font-size: 15px;
  color: var(--text);
  background: var(--bg);
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg: #0d1117;
    --surface: #151b23;
    --text: #e6edf3;
    --muted: #9198a1;
    --border: #3d444d;
    --track: #262c36;
    --accent: #4493f8;
    --accent-text: #0d1117;
    --warn: #d29922;
    --danger: #f85149;
    --danger-bg: #3c1618;
  }
}

body {
  margin: 0;
  background: var(--bg);
}

button,
input,
select {
  font: inherit;
  color: var(--text);
}

button {
  padding: 6px 14px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface);
  cursor: pointer;
}

button:disabled {
  cursor: default;
  opacity: 0.6;
}

button.primary {
  border-color: var(--accent);
  background: var(--accent);
  color: var(--accent-text);
}

button.danger {
  color: var(--danger);
}

input,
select {
  box-sizing: border-box;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
}

input[type="checkbox"] {
  width: 16px;
  height: 16px;
  margin: 0;
  accent-color: var(--accent);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 0.875rem;
}

.field > span {
  color: var(--muted);
}

.error {
  margin: 0 0 12px;
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--danger-bg);
  color: var(--danger);
  font-size: 0.875rem;
}
</style>

<style scoped>
.app {
  padding: 16px;
}

.top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 16px;
}

h1 {
  margin: 0;
  font-size: 1.25rem;
}

.meta {
  margin: 2px 0 0;
  color: var(--muted);
  font-size: 0.8125rem;
}

.section {
  margin-top: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

h2 {
  margin: 0;
  font-size: 1rem;
}
</style>
