import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { UsageSnapshot } from "./types";

export const USAGE_UPDATED = "usage://updated";

export function onUsageUpdated(handler: (snapshot: UsageSnapshot) => void): Promise<UnlistenFn> {
  return listen<UsageSnapshot>(USAGE_UPDATED, (event) => handler(event.payload));
}
