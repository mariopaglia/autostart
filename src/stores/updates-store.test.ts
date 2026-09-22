import { beforeEach, describe, expect, it, vi } from "vitest";
import { notifyError, notifySuccess } from "@/lib/notify";
import { system, updater, type Update } from "@/lib/tauri";
import { useUpdatesStore } from "@/stores/updates-store";

vi.mock("@/lib/tauri", () => ({
  updater: { check: vi.fn() },
  system: { logWarning: vi.fn(), relaunch: vi.fn() },
}));

vi.mock("@/lib/notify", () => ({ notifyError: vi.fn(), notifySuccess: vi.fn() }));

const mockedUpdater = vi.mocked(updater);
const mockedSystem = vi.mocked(system);

function fakeUpdate(downloadAndInstall: Update["downloadAndInstall"]): Update {
  return { version: "0.2.0", currentVersion: "0.1.0", downloadAndInstall } as Update;
}

describe("updates store", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useUpdatesStore.setState({ status: "idle", update: null });
  });

  it("fails silently on a startup check without network", async () => {
    mockedUpdater.check.mockRejectedValue(new Error("offline"));

    await useUpdatesStore.getState().check({ silent: true });

    expect(notifyError).not.toHaveBeenCalled();
    expect(mockedSystem.logWarning).toHaveBeenCalled();
    expect(useUpdatesStore.getState().status).toBe("idle");
  });

  it("shows an error on a manual check without network", async () => {
    mockedUpdater.check.mockRejectedValue(new Error("offline"));

    await useUpdatesStore.getState().check({ silent: false });

    expect(notifyError).toHaveBeenCalled();
  });

  it("tells the user when a manual check finds nothing new", async () => {
    mockedUpdater.check.mockResolvedValue(null);

    await useUpdatesStore.getState().check({ silent: false });

    expect(notifySuccess).toHaveBeenCalled();
  });

  it("keeps the update available and reports when installing fails", async () => {
    const update = fakeUpdate(vi.fn().mockRejectedValue(new Error("invalid signature")));
    mockedUpdater.check.mockResolvedValue(update);
    await useUpdatesStore.getState().check({ silent: true });

    await useUpdatesStore.getState().install();

    expect(useUpdatesStore.getState().status).toBe("available");
    expect(notifyError).toHaveBeenCalled();
    expect(mockedSystem.relaunch).not.toHaveBeenCalled();
  });

  it("relaunches after a successful install", async () => {
    mockedUpdater.check.mockResolvedValue(fakeUpdate(vi.fn().mockResolvedValue(undefined)));
    await useUpdatesStore.getState().check({ silent: true });

    await useUpdatesStore.getState().install();

    expect(mockedSystem.relaunch).toHaveBeenCalled();
  });
});
