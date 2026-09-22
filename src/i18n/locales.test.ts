import { describe, expect, it } from "vitest";
import en from "./locales/en.json";
import ptBR from "./locales/pt-BR.json";

function leaves(value: unknown, prefix = ""): [string, unknown][] {
  if (typeof value !== "object" || value === null) return [[prefix, value]];
  return Object.entries(value).flatMap(([key, child]) =>
    leaves(child, prefix ? `${prefix}.${key}` : key),
  );
}

const keysOf = (locale: unknown) =>
  leaves(locale)
    .map(([key]) => key)
    .sort();

describe("locales", () => {
  it("en has exactly the same keys as pt-BR", () => {
    expect(keysOf(en)).toEqual(keysOf(ptBR));
  });

  it("has no empty translations", () => {
    const emptyKeys = [ptBR, en].flatMap((locale) =>
      leaves(locale)
        .filter(([, text]) => text === "")
        .map(([key]) => key),
    );
    expect(emptyKeys).toEqual([]);
  });
});
