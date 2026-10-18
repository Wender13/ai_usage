<script setup lang="ts">
import ProviderCard from "../components/ProviderCard.vue";
import { useNow } from "../composables/useNow";
import { useUsage } from "../composables/useUsage";
import { formatTime } from "../lib/format";

const { snapshot, refreshing, refresh } = useUsage();
const now = useNow();
</script>

<template>
  <main class="app">
    <header class="top">
      <div>
        <h1>AI Usage</h1>
        <p class="meta">
          <template v-if="snapshot?.fetchedAt">
            Última consulta às {{ formatTime(snapshot.fetchedAt) }}
          </template>
          <template v-else>Aguardando a primeira consulta…</template>
        </p>
      </div>
      <button type="button" :disabled="refreshing" @click="refresh">
        {{ refreshing ? "Atualizando…" : "Atualizar" }}
      </button>
    </header>

    <ProviderCard
      v-for="status in snapshot?.providers ?? []"
      :key="status.provider"
      :status="status"
      :now="now"
    />

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
</style>
