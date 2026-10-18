import { invoke } from "@tauri-apps/api/core";
import type { UsageSnapshot } from "./types";

export const getSnapshot = () => invoke<UsageSnapshot>("get_snapshot");

export const refreshNow = () => invoke<void>("refresh_now");
