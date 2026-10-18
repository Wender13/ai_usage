import { onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getSettings, setOverlayVisible, updateSettings } from "../lib/api";
import { onSettingsChanged } from "../lib/events";
import type { Settings } from "../lib/types";
import { useAttempt } from "./useAttempt";

export function useSettings() {
  const settings = ref<Settings | null>(null);
  const { error, attempt } = useAttempt();
  let unlisten: UnlistenFn | undefined;

  onMounted(async () => {
    // Settings also change outside this window, e.g. the overlay toggle in the tray.
    unlisten = await onSettingsChanged((next) => (settings.value = next));
    await attempt(async () => {
      settings.value = await getSettings();
    });
  });
  onUnmounted(() => unlisten?.());

  const save = (next: Settings) =>
    attempt(async () => {
      settings.value = await updateSettings(next);
    });

  const setOverlay = (visible: boolean) =>
    attempt(async () => {
      settings.value = await setOverlayVisible(visible);
    });

  return { settings, error, save, setOverlay };
}
