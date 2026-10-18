// Mirrors the serde output of src-tauri/src/domain.rs and error.rs.

export type ProviderId = "claude";

export type WindowKind = "rolling_5h" | "rolling_7d";

export interface UsageWindow {
  provider: ProviderId;
  kind: WindowKind;
  /** Percentage of the limit used, from 0 to 100. */
  utilization: number;
  resetsAt: string | null;
}

export type ErrorCode =
  | "credentials_missing"
  | "credentials_invalid"
  | "token_expired"
  | "unauthorized"
  | "rate_limited"
  | "network"
  | "http"
  | "parse";

export interface ProviderError {
  code: ErrorCode;
  message: string;
}

export interface ProviderStatus {
  provider: ProviderId;
  /** On error, holds the windows from the last successful fetch. */
  windows: UsageWindow[];
  error: ProviderError | null;
  lastSuccessAt: string | null;
}

export interface UsageSnapshot {
  /** `null` until the first poll completes. */
  fetchedAt: string | null;
  providers: ProviderStatus[];
}

// Mirrors src-tauri/src/rules/model.rs and settings.rs.

export type Channel = "notification" | "overlay" | "both";

/** In `provider` and `window`, `null` means "any". */
export type RuleKind =
  | { type: "threshold"; provider: ProviderId | null; window: WindowKind | null; percent: number }
  | { type: "interval"; minutes: number }
  | { type: "beforeReset"; provider: ProviderId | null; window: WindowKind | null; minutes: number };

export type RuleType = RuleKind["type"];

export interface RuleInput {
  enabled: boolean;
  channel: Channel;
  kind: RuleKind;
}

export interface Rule extends RuleInput {
  id: string;
}

export interface Settings {
  pollIntervalMinutes: number;
  overlayEnabled: boolean;
}

export interface Alert {
  ruleId: string;
  channel: Channel;
  title: string;
  body: string;
}
