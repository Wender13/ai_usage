import type { Anchor, Channel, ProviderId, RuleKind, RuleType, UsageWindow, WindowKind } from "./types";

export const PROVIDER_LABELS: Record<ProviderId, string> = {
  claude: "Claude",
};

export const WINDOW_LABELS: Record<WindowKind, string> = {
  rolling_5h: "Sessão (5h)",
  rolling_7d: "Semanal (7 dias)",
};

export const WINDOW_SHORT_LABELS: Record<WindowKind, string> = {
  rolling_5h: "5h",
  rolling_7d: "7d",
};

export const ANCHOR_LABELS: Record<Anchor, string> = {
  top_left: "Superior esquerdo",
  top_center: "Superior central",
  top_right: "Superior direito",
  bottom_left: "Inferior esquerdo",
  bottom_center: "Inferior central",
  bottom_right: "Inferior direito",
};

export const RULE_TYPE_LABELS: Record<RuleType, string> = {
  threshold: "Limiar de uso",
  interval: "Resumo periódico",
  beforeReset: "Antes do reset",
};

export const CHANNEL_LABELS: Record<Channel, string> = {
  notification: "Notificação",
  overlay: "Overlay",
  both: "Notificação e overlay",
};

const MINUTE_MS = 60_000;
const DAY_MS = 24 * 60 * MINUTE_MS;

const timeFormat = new Intl.DateTimeFormat("pt-BR", { hour: "2-digit", minute: "2-digit" });
const dayTimeFormat = new Intl.DateTimeFormat("pt-BR", {
  weekday: "short",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
});

export const providerLabel = (id: ProviderId) => PROVIDER_LABELS[id];

export const windowLabel = (kind: WindowKind) => WINDOW_LABELS[kind];

/** Same key as `UsageWindow::key` in the core, e.g. "claude:rolling_5h". */
export const windowKey = (usage: UsageWindow) => `${usage.provider}:${usage.kind}`;

export const formatPercent = (value: number) => `${Math.round(value)}%`;

/** Visual level used to color percentages. */
export function usageLevel(utilization: number): "low" | "medium" | "high" {
  if (utilization >= 90) return "high";
  if (utilization >= 70) return "medium";
  return "low";
}

export const formatTime = (iso: string) => timeFormat.format(new Date(iso));

/** Compact duration such as "3d 4h", "2h 10min" or "8min". */
export function formatDuration(ms: number): string {
  const totalMinutes = Math.floor(ms / MINUTE_MS);
  if (totalMinutes < 1) return "menos de 1 min";

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

/** Time left until the reset plus its local clock time, e.g. "reinicia em 2h 10min (22:00)". */
export function formatReset(resetsAt: string | null, now: number): string {
  if (!resetsAt) return "horário de reset desconhecido";

  const target = resetTime(resetsAt);
  const remaining = target.getTime() - now;
  if (remaining <= 0) return "reiniciando…";

  const sameDay = remaining < DAY_MS && target.getDate() === new Date(now).getDate();
  const when = sameDay ? timeFormat.format(target) : dayTimeFormat.format(target);
  return `reinicia em ${formatDuration(remaining)} (${when})`;
}

function describeTarget(provider: ProviderId | null, window: WindowKind | null): string {
  const providerText = provider ? providerLabel(provider) : "qualquer provider";
  const windowText = window ? windowLabel(window) : "qualquer janela";
  return `${providerText} · ${windowText}`;
}

/** One-line description of what a rule does. */
export function describeRule(kind: RuleKind): string {
  switch (kind.type) {
    case "threshold":
      return `Ao atingir ${kind.percent}% em ${describeTarget(kind.provider, kind.window)}`;
    case "interval":
      return `Resumo do uso a cada ${kind.minutes} min`;
    case "beforeReset":
      return `${kind.minutes} min antes do reset de ${describeTarget(kind.provider, kind.window)}`;
  }
}
