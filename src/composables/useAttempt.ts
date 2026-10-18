import { ref } from "vue";

/** Runs backend calls, keeping the last error message for the UI. */
export function useAttempt() {
  const error = ref<string | null>(null);

  async function attempt(action: () => Promise<void>): Promise<boolean> {
    error.value = null;
    try {
      await action();
      return true;
    } catch (cause) {
      error.value = String(cause);
      return false;
    }
  }

  return { error, attempt };
}
