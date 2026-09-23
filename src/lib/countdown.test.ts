import { describe, expect, it } from "vitest";
import { secondsUntil } from "./countdown";

describe("secondsUntil", () => {
  it("rounds partial seconds up", () => {
    expect(secondsUntil(60_000, 0)).toBe(60);
    expect(secondsUntil(1_500, 1_000)).toBe(1);
  });

  it("never goes below zero", () => {
    expect(secondsUntil(1_000, 5_000)).toBe(0);
  });
});
