export function fmtTime(sec: number): string {
  const s = Math.max(0, Math.floor(sec));
  const m = Math.floor(s / 60);
  const r = s % 60;
  return `${m}:${String(r).padStart(2, "0")}`;
}

export function coverDataUrl(base64: string | null): string | null {
  return base64 ? `data:image/png;base64,${base64}` : null;
}