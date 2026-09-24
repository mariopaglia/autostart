// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { SessionLog } from "@/bindings/SessionLog";
import { i18n } from "@/i18n";
import { useLogsStore } from "@/stores/logs-store";
import { useMonitorStore } from "@/stores/monitor-store";
import { LogsScreen } from "./LogsScreen";

vi.mock("@/lib/tauri", () => ({
  commands: {
    openLogDir: vi.fn(() => Promise.resolve()),
    getSessionHistory: vi.fn(() => Promise.resolve([])),
    clearSessionHistory: vi.fn(() => Promise.resolve(null)),
  },
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

function session(profileName: string, startedAtMs: number, overrides: Partial<SessionLog> = {}) {
  return {
    profileId: profileName,
    profileName,
    isTest: false,
    startedAtMs,
    endedAtMs: startedAtMs + 1,
    entries: [{ timestampMs: startedAtMs, kind: "sessionStarted" }],
    ...overrides,
  } satisfies SessionLog;
}

const yesterday = session("Yesterday", 1_000, {
  entries: [
    { timestampMs: 1_000, kind: "sessionStarted" },
    {
      timestampMs: 1_001,
      kind: "error",
      itemName: "Volanta",
      error: { kind: "launchFailed", message: "x" },
    },
    {
      timestampMs: 1_002,
      kind: "error",
      itemName: "vPilot",
      error: { kind: "launchFailed", message: "y" },
    },
  ],
});
const today = session("Today", 2_000);

function openSelector() {
  const trigger = screen.getByRole("combobox", { name: i18n.t("logs.session") });
  fireEvent.keyDown(trigger, { key: "ArrowDown" });
  return screen.getByRole("listbox");
}

beforeEach(() => {
  useMonitorStore.setState({ sessionLog: null });
  useLogsStore.setState({ history: [today, yesterday] });
});

afterEach(cleanup);

describe("LogsScreen", () => {
  it("shows the newest session and the error count of each session", () => {
    render(<LogsScreen />);

    const selector = screen.getByRole("combobox", { name: i18n.t("logs.session") });
    expect(within(selector).getByText("Today")).toBeDefined();

    const options = openSelector();
    expect(within(options).getByText(i18n.t("logs.errors", { count: 2 }))).toBeDefined();
  });

  it("shows an older session once it is picked", () => {
    render(<LogsScreen />);

    fireEvent.click(within(openSelector()).getByText("Yesterday"));

    expect(screen.getByText(/Volanta/)).toBeDefined();
  });

  it("keeps the picked session when a new one starts", () => {
    render(<LogsScreen />);
    fireEvent.click(within(openSelector()).getByText("Yesterday"));

    act(() => {
      useMonitorStore.setState({ sessionLog: session("Live", 3_000, { endedAtMs: null }) });
    });

    const selector = screen.getByRole("combobox", { name: i18n.t("logs.session") });
    expect(within(selector).getByText("Yesterday")).toBeDefined();
    expect(within(openSelector()).getByText("Live")).toBeDefined();
  });

  it("follows a new session when nothing was picked", () => {
    render(<LogsScreen />);

    act(() => {
      useMonitorStore.setState({ sessionLog: session("Live", 3_000, { endedAtMs: null }) });
    });

    const selector = screen.getByRole("combobox", { name: i18n.t("logs.session") });
    expect(within(selector).getByText("Live")).toBeDefined();
  });

  it("clears the history after confirmation", async () => {
    render(<LogsScreen />);

    fireEvent.click(screen.getByRole("button", { name: i18n.t("logs.clearHistory") }));
    const dialog = screen.getByRole("alertdialog");
    await act(async () => {
      fireEvent.click(within(dialog).getByRole("button", { name: i18n.t("logs.clearHistory") }));
      await Promise.resolve();
    });

    expect(screen.getByText(i18n.t("logs.emptyTitle"))).toBeDefined();
  });
});
