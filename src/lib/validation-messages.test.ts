import { describe, expect, it } from "vitest";
import { i18n } from "@/i18n";
import { translateValidationMessage } from "./validation-messages";

describe("translateValidationMessage", () => {
  it("translates known schema keys", () => {
    expect(translateValidationMessage(i18n.t, "validation.url")).toBe(
      "Informe um endereço http(s) ou um link do Steam como steam://rungameid/123",
    );
    expect(translateValidationMessage(i18n.t, "validation.launchTarget")).toMatch(/\.exe/);
  });

  it("falls back to the generic message for unknown or raw zod messages", () => {
    expect(translateValidationMessage(i18n.t, "Invalid input: expected string")).toBe(
      "Valor inválido",
    );
  });
});
