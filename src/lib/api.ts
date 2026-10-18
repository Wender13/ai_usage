import { invoke } from "@tauri-apps/api/core";
import type { Alert, Rule, RuleInput, Settings, UsageSnapshot } from "./types";

export const getSnapshot = () => invoke<UsageSnapshot>("get_snapshot");

export const refreshNow = () => invoke<void>("refresh_now");

export const getSettings = () => invoke<Settings>("get_settings");

export const updateSettings = (settings: Settings) =>
  invoke<Settings>("update_settings", { settings });

export const listRules = () => invoke<Rule[]>("list_rules");

export const createRule = (input: RuleInput) => invoke<Rule>("create_rule", { input });

export const updateRule = (rule: Rule) => invoke<Rule>("update_rule", { rule });

export const deleteRule = (id: string) => invoke<void>("delete_rule", { id });

export const setOverlayVisible = (visible: boolean) =>
  invoke<Settings>("set_overlay_visible", { visible });

export const getOverlayAlerts = () => invoke<Alert[]>("get_overlay_alerts");

export const dismissOverlayAlert = () => invoke<void>("dismiss_overlay_alert");
