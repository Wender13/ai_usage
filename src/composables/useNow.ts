import { onMounted, onUnmounted, ref } from "vue";

/** Current timestamp, refreshed periodically so relative times stay current. */
export function useNow(intervalMs = 30_000) {
  const now = ref(Date.now());
  let timer: number | undefined;

  onMounted(() => {
    timer = window.setInterval(() => (now.value = Date.now()), intervalMs);
  });
  onUnmounted(() => window.clearInterval(timer));

  return now;
}
