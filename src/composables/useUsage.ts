import { onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getSnapshot, refreshNow } from "../lib/api";
import { onUsageUpdated } from "../lib/events";
import type { UsageSnapshot } from "../lib/types";

/** Re-enables the refresh button if no update arrives (e.g. a hung request). */
const REFRESH_TIMEOUT_MS = 20_000;

const fetchedTime = (snapshot: UsageSnapshot | null) =>
  snapshot?.fetchedAt ? Date.parse(snapshot.fetchedAt) : -Infinity;

export function useUsage() {
  const snapshot = ref<UsageSnapshot | null>(null);
  const refreshing = ref(false);
  let unlisten: UnlistenFn | undefined;
  let refreshTimer: number | undefined;

  function stopRefreshing() {
    refreshing.value = false;
    window.clearTimeout(refreshTimer);
  }

  onMounted(async () => {
    unlisten = await onUsageUpdated((next) => {
      snapshot.value = next;
      stopRefreshing();
    });

    // Read after subscribing so no update is lost; keep whichever is newer.
    try {
      const current = await getSnapshot();
      if (fetchedTime(current) >= fetchedTime(snapshot.value)) snapshot.value = current;
    } catch (error) {
      console.error("get_snapshot failed", error);
    }
  });

  onUnmounted(() => {
    unlisten?.();
    window.clearTimeout(refreshTimer);
  });

  async function refresh() {
    if (refreshing.value) return;
    refreshing.value = true;
    refreshTimer = window.setTimeout(stopRefreshing, REFRESH_TIMEOUT_MS);
    try {
      await refreshNow();
    } catch (error) {
      console.error("refresh_now failed", error);
      stopRefreshing();
    }
  }

  return { snapshot, refreshing, refresh };
}
