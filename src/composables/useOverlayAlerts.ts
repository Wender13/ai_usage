import { onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { dismissOverlayAlert, getOverlayAlerts } from "../lib/api";
import { onOverlayAlerts } from "../lib/events";
import type { Alert } from "../lib/types";

/** Alert queue kept by the core; the first item is the one on screen. */
export function useOverlayAlerts() {
  const alerts = ref<Alert[]>([]);
  let received = false;
  let unlisten: UnlistenFn | undefined;

  onMounted(async () => {
    unlisten = await onOverlayAlerts((next) => {
      received = true;
      alerts.value = next;
    });
    // Picks up alerts raised before this page loaded, unless an update already arrived.
    try {
      const current = await getOverlayAlerts();
      if (!received) alerts.value = current;
    } catch (error) {
      console.error("get_overlay_alerts failed", error);
    }
  });
  onUnmounted(() => unlisten?.());

  async function dismiss() {
    try {
      await dismissOverlayAlert();
    } catch (error) {
      console.error("dismiss_overlay_alert failed", error);
    }
  }

  return { alerts, dismiss };
}
