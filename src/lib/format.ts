import { locale, t } from "../i18n";
import type {
  Anchor,
  Channel,
  ProviderId,
  RuleKind,
  RuleType,
  UsageWindow,
  WindowKind,
} from "./types";

export const ANCHORS: Anchor[] = [
  "top_left",
  "top_center",
  "top_right",
  "bottom_left",
  "bottom_center",
  "bottom_right",
];
export const PROVIDERS: ProviderId[] = ["claude"];
export const WINDOW_KINDS: WindowKind[] = ["rolling_5h", "rolling_7d"];
export const RULE_TYPES: RuleType[] = ["threshold", "interval", "beforeReset"];
export const CHANNELS: Channel[] = ["notification", "overlay", "both"];

const MINUTE_MS = 60_000;
const DAY_MS = 24 * 60 * MINUTE_MS;

export const providerLabel = (id: ProviderId) => t(`providers.${id}`);
export const windowLabel = (kind: WindowKind) => t(`windows.${kind}`);
export const windowShortLabel = (kind: WindowKind) => t(`windowsShort.${kind}`);
export const anchorLabel = (anchor: Anchor) => t(`anchors.${anchor}`);
export const ruleTypeLabel = (type: RuleType) => t(`rules.types.${type}`);
export const channelLabel = (channel: Channel) => t(`rules.channels.${channel}`);

/** Same key as `UsageWindow::key` in the core, e.g. "claude:rolling_5h". */
export const windowKey = (usage: UsageWindow) => `${usage.provider}:${usage.kind}`;

export const formatPercent = (value: number) => `${Math.round(value)}%`;

/** Visual level used to color percentages. */
export function usageLevel(utilization: number): "low" | "medium" | "high" {
  if (utilization >= 90) return "high";
  if (utilization >= 70) return "medium";
  return "low";
}

// Formatters are costly to build, so one is kept per locale and style.
const formatters = new Map<string, Intl.DateTimeFormat>();

function dateFormat(withDay: boolean): Intl.DateTimeFormat {
  const tag = locale();
  const key = `${tag}|${withDay}`;
  let format = formatters.get(key);
  if (!format) {
    const options: Intl.DateTimeFormatOptions = withDay
      ? { weekday: "short", day: "2-digit", hour: "2-digit", minute: "2-digit" }
      : { hour: "2-digit", minute: "2-digit" };
    try {
      format = new Intl.DateTimeFormat(tag, options);
    } catch {
      // An unusual system locale must not break the screen.
      format = new Intl.DateTimeFormat(undefined, options);
    }
    formatters.set(key, format);
  }
  return format;
}

export const formatTime = (iso: string) => dateFormat(false).format(new Date(iso));

/** Compact duration such as "3d 4h", "2h 10min" or "8min" (units read the same in both languages). */
export function formatDuration(ms: number): string {
  const totalMinutes = Math.floor(ms / MINUTE_MS);
  if (totalMinutes < 1) return t("time.lessThanMinute");

  const days = Math.floor(totalMinutes / (24 * 60));
  const hours = Math.floor((totalMinutes % (24 * 60)) / 60);
  const minutes = totalMinutes % 60;

  if (days > 0) return hours > 0 ? `${days}d ${hours}h` : `${days}d`;
  if (hours > 0) return minutes > 0 ? `${hours}h ${minutes}min` : `${hours}h`;
  return `${minutes}min`;
}

/** The API adds up to about a second of jitter around the reset time; use the nearest minute. */
const resetTime = (resetsAt: string) =>
  new Date(Math.round(Date.parse(resetsAt) / MINUTE_MS) * MINUTE_MS);

/** Compact time left until the reset, e.g. "2h 10min"; `null` when unknown or already past. */
export function formatRemaining(resetsAt: string | null, now: number): string | null {
  if (!resetsAt) return null;
  const remaining = resetTime(resetsAt).getTime() - now;
  return remaining > 0 ? formatDuration(remaining) : null;
}

/** Time left until the reset plus its local clock time, e.g. "resets in 2h 10min (10:00 PM)". */
export function formatReset(resetsAt: string | null, now: number): string {
  if (!resetsAt) return t("time.resetUnknown");

  const target = resetTime(resetsAt);
  const remaining = target.getTime() - now;
  if (remaining <= 0) return t("time.resetting");

  const sameDay = remaining < DAY_MS && target.getDate() === new Date(now).getDate();
  const when = dateFormat(!sameDay).format(target);
  return t("time.resetsIn", { duration: formatDuration(remaining), when });
}

function describeTarget(provider: ProviderId | null, window: WindowKind | null): string {
  const providerText = provider ? providerLabel(provider) : t("rules.anyProvider");
  const windowText = window ? windowLabel(window) : t("rules.anyWindow");
  return `${providerText} · ${windowText}`;
}

/** One-line description of what a rule does. */
export function describeRule(kind: RuleKind): string {
  switch (kind.type) {
    case "threshold":
      return t("rules.describe.threshold", {
        target: describeTarget(kind.provider, kind.window),
        percent: kind.percent,
      });
    case "interval":
      return t("rules.describe.interval", { minutes: kind.minutes });
    case "beforeReset":
      return t("rules.describe.beforeReset", {
        target: describeTarget(kind.provider, kind.window),
        minutes: kind.minutes,
      });
  }
}
