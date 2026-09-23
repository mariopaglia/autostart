// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { commands } from "@/lib/tauri";
import { CloseCountdown } from "./CloseCountdown";

vi.mock("@/lib/tauri", () => ({
  commands: { keepAppsOpen: vi.fn(), closeAppsNow: vi.fn() },
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(0);
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("CloseCountdown", () => {
  it("counts down every second", () => {
    render(<CloseCountdown closesAtMs={60_000} />);
    expect(screen.getByText(i18n.t("monitor.closingIn", { seconds: 60 }))).toBeDefined();

    act(() => {
      vi.advanceTimersByTime(3_000);
    });

    expect(screen.getByText(i18n.t("monitor.closingIn", { seconds: 57 }))).toBeDefined();
  });

  it("keeps the apps open or closes them now", () => {
    render(<CloseCountdown closesAtMs={60_000} />);

    fireEvent.click(screen.getByRole("button", { name: i18n.t("monitor.keepApps") }));
    fireEvent.click(screen.getByRole("button", { name: i18n.t("monitor.closeNow") }));

    expect(vi.mocked(commands.keepAppsOpen)).toHaveBeenCalledOnce();
    expect(vi.mocked(commands.closeAppsNow)).toHaveBeenCalledOnce();
  });
});
