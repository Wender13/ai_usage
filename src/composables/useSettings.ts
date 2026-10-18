import { onMounted, ref } from "vue";
import { getSettings, updateSettings } from "../lib/api";
import type { Settings } from "../lib/types";
import { useAttempt } from "./useAttempt";

export function useSettings() {
  const settings = ref<Settings | null>(null);
  const { error, attempt } = useAttempt();

  onMounted(() =>
    attempt(async () => {
      settings.value = await getSettings();
    }),
  );

  const save = (next: Settings) =>
    attempt(async () => {
      settings.value = await updateSettings(next);
    });

  return { settings, error, save };
}
