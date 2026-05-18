export function toInt(value) {
  const n = Number.parseInt(String(value ?? "0"), 10);
  if (!Number.isFinite(n) || Number.isNaN(n) || n < 0) {
    return 0;
  }
  return n;
}

export function formatInt(value) {
  return Number(value ?? 0).toLocaleString("en-US");
}

export function formatFixed(value, digits = 6) {
  return Number(value ?? 0).toFixed(digits);
}
