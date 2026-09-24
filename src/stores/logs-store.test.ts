import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SessionLog } from "@/bindings/SessionLog";
import { commands } from "@/lib/tauri";
import { useLogsStore } from "@/stores/logs-store";

vi.mock("@/lib/tauri", () => ({
  commands: { getSessionHistory: vi.fn(), clearSessionHistory: vi.fn() },
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

const toast = vi.hoisted(() => ({ error: vi.fn() }));
vi.mock("sonner", () => ({ toast }));

const mocked = vi.mocked(commands);

const stored: SessionLog = {
  profileId: "profile",
  profileName: "MSFS",
  isTest: false,
  startedAtMs: 1,
  endedAtMs: 2,
  entries: [],
};

describe("logs store", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useLogsStore.setState({ history: [] });
  });

  it("loads the stored sessions", async () => {
    mocked.getSessionHistory.mockResolvedValue([stored]);

    await useLogsStore.getState().load();

    expect(useLogsStore.getState().history).toEqual([stored]);
  });

  it("empties the history once the backend cleared it", async () => {
    useLogsStore.setState({ history: [stored] });
    mocked.clearSessionHistory.mockResolvedValue(null);

    await useLogsStore.getState().clear();

    expect(useLogsStore.getState().history).toEqual([]);
  });

  it("keeps the history and reports the error when clearing fails", async () => {
    useLogsStore.setState({ history: [stored] });
    mocked.clearSessionHistory.mockRejectedValue(new Error("disk"));

    await useLogsStore.getState().clear();

    expect(useLogsStore.getState().history).toEqual([stored]);
    expect(toast.error).toHaveBeenCalled();
  });
});
