// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import {
  closeDelaySchema,
  gracefulTimeoutSchema,
  MAX_CLOSE_DELAY_MS,
  MAX_GRACEFUL_TIMEOUT_MS,
  MIN_GRACEFUL_TIMEOUT_MS,
} from "@/schemas/settings";
import { SecondsField } from "./SecondsField";

afterEach(cleanup);

function renderCloseDelay(onSave = vi.fn()) {
  render(
    <SecondsField
      id="close-delay"
      valueMs={60_000}
      schemaMs={closeDelaySchema}
      minMs={0}
      maxMs={MAX_CLOSE_DELAY_MS}
      stepSeconds={10}
      onSave={onSave}
    />,
  );
  return { input: screen.getByRole("spinbutton"), onSave };
}

function renderGracefulTimeout(onSave = vi.fn()) {
  render(
    <SecondsField
      id="graceful-timeout"
      valueMs={5_000}
      schemaMs={gracefulTimeoutSchema}
      minMs={MIN_GRACEFUL_TIMEOUT_MS}
      maxMs={MAX_GRACEFUL_TIMEOUT_MS}
      stepSeconds={1}
      onSave={onSave}
    />,
  );
  return { input: screen.getByRole("spinbutton"), onSave };
}

describe("SecondsField", () => {
  it("shows the close delay in seconds and saves it in milliseconds", () => {
    const { input, onSave } = renderCloseDelay();
    expect(input).toHaveProperty("value", "60");

    fireEvent.change(input, { target: { value: "0" } });
    fireEvent.blur(input);

    expect(onSave).toHaveBeenCalledWith(0);
  });

  it("rejects a close delay over ten minutes", () => {
    const { input, onSave } = renderCloseDelay();

    fireEvent.change(input, { target: { value: "900" } });
    fireEvent.blur(input);

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toBe(i18n.t("validation.closeDelay"));
  });

  it("shows the graceful timeout in seconds and saves it in milliseconds", () => {
    const { input, onSave } = renderGracefulTimeout();
    expect(input).toHaveProperty("value", "5");

    fireEvent.change(input, { target: { value: "10" } });
    fireEvent.blur(input);

    expect(onSave).toHaveBeenCalledWith(10_000);
  });

  it("rejects a graceful timeout outside 1 to 60 seconds", () => {
    const { input, onSave } = renderGracefulTimeout();

    fireEvent.change(input, { target: { value: "0.5" } });
    fireEvent.blur(input);

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toBe(i18n.t("validation.timeout"));
  });
});
