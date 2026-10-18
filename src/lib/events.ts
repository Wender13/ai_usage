import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Alert, Settings, UsageSnapshot } from "./types";

export const USAGE_UPDATED = "usage://updated";
export const SETTINGS_CHANGED = "settings://changed";
export const OVERLAY_ALERTS = "overlay://alerts";

function subscribe<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (message) => handler(message.payload));
}

export const onUsageUpdated = (handler: (snapshot: UsageSnapshot) => void) =>
  subscribe(USAGE_UPDATED, handler);

export const onSettingsChanged = (handler: (settings: Settings) => void) =>
  subscribe(SETTINGS_CHANGED, handler);

export const onOverlayAlerts = (handler: (alerts: Alert[]) => void) =>
  subscribe(OVERLAY_ALERTS, handler);
