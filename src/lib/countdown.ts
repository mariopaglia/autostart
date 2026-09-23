/** Whole seconds left until `targetMs`, never negative. */
export function secondsUntil(targetMs: number, nowMs: number): number {
  return Math.max(0, Math.ceil((targetMs - nowMs) / 1000));
}
