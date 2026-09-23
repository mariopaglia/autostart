// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { CloseDelayField } from "./CloseDelayField";

afterEach(cleanup);

function renderField(onSave = vi.fn()) {
  render(<CloseDelayField id="close-delay" valueMs={60_000} onSave={onSave} />);
  return { input: screen.getByRole("spinbutton"), onSave };
}

describe("CloseDelayField", () => {
  it("shows the delay in seconds and saves it in milliseconds", () => {
    const { input, onSave } = renderField();
    expect(input).toHaveProperty("value", "60");

    fireEvent.change(input, { target: { value: "0" } });
    fireEvent.blur(input);

    expect(onSave).toHaveBeenCalledWith(0);
  });

  it("rejects more than ten minutes", () => {
    const { input, onSave } = renderField();

    fireEvent.change(input, { target: { value: "900" } });
    fireEvent.blur(input);

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toBe(i18n.t("validation.closeDelay"));
  });
});
