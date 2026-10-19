import { onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  getSettings,
  setLanguage,
  setOverlayAnchor,
  setOverlayVisible,
  setUseXwayland,
  updateOverlayPrefs,
  updateSettings,
} from "../lib/api";
import { onSettingsChanged } from "../lib/events";
import type { Anchor, LanguagePref, OverlayPrefs, Settings } from "../lib/types";
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

  /** Applies a partial change on top of the current overlay preferences. */
  const changeOverlay = (change: Partial<OverlayPrefs>) =>
    attempt(async () => {
      if (!settings.value) return;
      settings.value = await updateOverlayPrefs({ ...settings.value.overlay, ...change });
    });

  const setAnchor = (anchor: Anchor) =>
    attempt(async () => {
      settings.value = await setOverlayAnchor(anchor);
    });

  const setXwayland = (enabled: boolean) =>
    attempt(async () => {
      settings.value = await setUseXwayland(enabled);
    });

  /** The new settings and texts arrive through the change events. */
  const changeLanguage = (language: LanguagePref) =>
    attempt(async () => {
      await setLanguage(language);
    });

  return {
    settings,
    error,
    save,
    setOverlay,
    changeOverlay,
    setAnchor,
    setXwayland,
    changeLanguage,
  };
}
