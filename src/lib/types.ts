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
