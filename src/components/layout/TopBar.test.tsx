// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { MonitorSnapshot } from "@/bindings/MonitorSnapshot";
import type { Trigger } from "@/bindings/Trigger";
import { TooltipProvider } from "@/components/ui/tooltip";
import { i18n } from "@/i18n";
import { createProfile } from "@/lib/profile-factory";
import { commands } from "@/lib/tauri";
import { useMonitorStore } from "@/stores/monitor-store";
import { TopBar } from "./TopBar";

vi.mock("@/lib/tauri", () => ({
  commands: {
    startFlight: vi.fn(() => Promise.resolve(null)),
    cancelFlightStart: vi.fn(() => Promise.resolve(null)),
    testLaunch: vi.fn(() => Promise.resolve(null)),
    listRunningProcesses: vi.fn(() => Promise.resolve([])),
  },
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

const MSFS_2020: Trigger = {
  processName: "FlightSimulator.exe",
  label: "MSFS 2020",
  launchTarget: "steam://rungameid/1250410",
};
const MSFS_2024: Trigger = {
  processName: "FlightSimulator2024.exe",
  label: "MSFS 2024",
  launchTarget: "steam://rungameid/2537590",
};
const XPLANE: Trigger = { processName: "X-Plane.exe", label: "X-Plane 12" };

const IDLE: MonitorSnapshot = {
  state: "idle",
  sessionProfileId: null,
  isTestSession: false,
  items: [],
  closesAtMs: null,
  startingSimulator: null,
};

function renderTopBar(triggers: Trigger[]) {
  const profile = { ...createProfile("MSFS"), triggers };
  render(
    <TooltipProvider>
      <TopBar profile={profile} />
    </TooltipProvider>,
  );
  return profile;
}

function startFlightButton() {
  return screen.queryByRole("button", { name: new RegExp(i18n.t("monitor.startFlight")) });
}

beforeEach(() => {
  vi.clearAllMocks();
  useMonitorStore.setState({ snapshot: IDLE });
});

afterEach(cleanup);

describe("TopBar start flight", () => {
  it("is hidden when no simulator can be started", () => {
    renderTopBar([XPLANE]);

    expect(startFlightButton()).toBeNull();
  });

  it("starts the only simulator that can be started", () => {
    const profile = renderTopBar([XPLANE, MSFS_2024]);

    const button = startFlightButton();
    if (button) fireEvent.click(button);

    expect(commands.startFlight).toHaveBeenCalledWith(profile.id, "FlightSimulator2024.exe");
  });

  it("lets the pilot pick the simulator when several can be started", async () => {
    const profile = renderTopBar([MSFS_2020, MSFS_2024]);

    const button = startFlightButton();
    if (button) fireEvent.keyDown(button, { key: "Enter" });
    fireEvent.click(await screen.findByRole("menuitem", { name: "MSFS 2020" }));

    expect(commands.startFlight).toHaveBeenCalledWith(profile.id, "FlightSimulator.exe");
  });

  it("is disabled outside idle", () => {
    useMonitorStore.setState({ snapshot: { ...IDLE, state: "paused" } });
    renderTopBar([MSFS_2024]);

    expect(startFlightButton()).toHaveProperty("disabled", true);
  });

  it("shows the simulator being started and lets the pilot cancel", () => {
    useMonitorStore.setState({
      snapshot: { ...IDLE, state: "simStarting", startingSimulator: "MSFS 2024" },
    });
    renderTopBar([MSFS_2024]);

    expect(
      screen.getByText(i18n.t("monitor.state.simStarting", { simulator: "MSFS 2024" })),
    ).toBeDefined();
    expect(screen.getByRole("button", { name: i18n.t("monitor.testLaunch") })).toHaveProperty(
      "disabled",
      true,
    );
    fireEvent.click(screen.getByRole("button", { name: i18n.t("monitor.cancelStart") }));

    expect(commands.cancelFlightStart).toHaveBeenCalled();
  });
});
