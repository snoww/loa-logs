import { abbreviateNumber, timestampToMinutesAndSeconds } from "$lib/utils";

export function formatDps(value?: number | null) {
  if (value === undefined || value === null || value <= 0) return "-";
  return abbreviateNumber(value).replace(/[kmbt]$/, (suffix) => suffix.toUpperCase());
}

export function formatNumber(value?: number | null) {
  if (value === undefined || value === null || value <= 0) return "-";
  return abbreviateNumber(value).replace(/[kmbt]$/, (suffix) => suffix.toUpperCase());
}

export function formatPercent(value?: number | null) {
  if (value === undefined || value === null) return "-";
  return `${value.toFixed(1)}%`;
}

export function formatRatioPercent(value?: number | null) {
  if (value === undefined || value === null) return "-";
  return `${(value * 100).toFixed(1)}%`;
}

export function formatDecimal(value?: number | null) {
  if (value === undefined || value === null) return "-";
  return value.toFixed(1);
}

export function formatDuration(value?: number | null) {
  if (value === undefined || value === null || value <= 0) return "-";
  return timestampToMinutesAndSeconds(value);
}

export function formatTotalDuration(value?: number | null) {
  if (value === undefined || value === null || value <= 0) return "-";
  const minutes = Math.floor(value / 60000);
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return hours > 0 ? `${hours}h ${remainingMinutes}m` : `${remainingMinutes}m`;
}

export function formatShortDate(value?: number | null) {
  if (value === undefined || value === null || value <= 0) return "-";
  return new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(new Date(value));
}

export function formatLongDate(value?: number | null) {
  if (value === undefined || value === null || value <= 0) return "-";
  return new Intl.DateTimeFormat(undefined, { month: "long", day: "numeric" }).format(new Date(value));
}

export function formatDateTime(value?: number | null) {
  if (value === undefined || value === null || value <= 0) return "-";
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit"
  }).format(new Date(value));
}

export function timestampToInputDate(value?: number | null) {
  if (value === undefined || value === null || value <= 0) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return new Date(date.getTime() - date.getTimezoneOffset() * 60000).toISOString().slice(0, 10);
}

export function dateToStartTime(value: string) {
  if (!value) return undefined;
  return new Date(`${value}T00:00:00`).getTime();
}

export function dateToEndTime(value: string) {
  if (!value) return undefined;
  return new Date(`${value}T23:59:59.999`).getTime();
}
