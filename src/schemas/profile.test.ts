import { describe, expect, it } from "vitest";
import { launchItemSchema, profileSchema } from "./profile";

const PROFILE_ID = "6f1c1d52-8f8e-4c4b-9a53-2f0a5f3a1d10";
const ITEM_ID = "0b6f0c2e-3f4d-4a8e-9b1a-7c2d5e6f7a8b";

const MSFS_2024 = { processName: "FlightSimulator2024.exe", label: "MSFS 2024" };
const MSFS_2020 = { processName: "FlightSimulator.exe", label: "MSFS 2020" };

function buildProfile(overrides: Record<string, unknown> = {}) {
  return {
    id: PROFILE_ID,
    name: "Live MSFS 2024",
    triggers: [MSFS_2024],
    ...overrides,
  };
}

describe("profileSchema", () => {
  it("applies defaults to a minimal profile", () => {
    const profile = profileSchema.parse(buildProfile());

    expect(profile.items).toEqual([]);
    expect(profile.enabled).toBe(true);
  });

  it("rejects a blank name", () => {
    const result = profileSchema.safeParse(buildProfile({ name: "   " }));

    expect(result.success).toBe(false);
    expect(result.error?.issues[0]?.path).toEqual(["name"]);
  });

  it("rejects a trigger that is not an executable", () => {
    const result = profileSchema.safeParse(
      buildProfile({ triggers: [{ processName: "FlightSimulator", label: "MSFS" }] }),
    );

    expect(result.success).toBe(false);
    expect(result.error?.issues[0]?.path).toEqual(["triggers", 0, "processName"]);
  });

  it("accepts several distinct triggers", () => {
    const profile = profileSchema.parse(buildProfile({ triggers: [MSFS_2020, MSFS_2024] }));

    expect(profile.triggers).toEqual([MSFS_2020, MSFS_2024]);
  });

  it("rejects no triggers, more than five or a repeated one", () => {
    const six = ["A", "B", "C", "D", "E", "F"].map((name) => ({
      processName: `${name}.exe`,
      label: name,
    }));
    const repeated = [MSFS_2024, { processName: "flightsimulator2024.EXE", label: "Copy" }];

    expect(profileSchema.safeParse(buildProfile({ triggers: [] })).success).toBe(false);
    expect(profileSchema.safeParse(buildProfile({ triggers: six })).success).toBe(false);
    const result = profileSchema.safeParse(buildProfile({ triggers: repeated }));
    expect(result.success).toBe(false);
    expect(result.error?.issues[0]?.message).toBe("validation.triggerDuplicate");
  });

  it("converts the single trigger of files exported by v0.2.0", () => {
    const exported = { id: PROFILE_ID, name: "Live MSFS 2020", trigger: MSFS_2020 };

    const profile = profileSchema.parse(exported);

    expect(profile.triggers).toEqual([MSFS_2020]);
    expect(profile).not.toHaveProperty("trigger");
  });
});

describe("launchItemSchema", () => {
  it("parses an app item with defaults", () => {
    const item = launchItemSchema.parse({
      type: "app",
      id: ITEM_ID,
      name: "Volanta",
      exePath: "C:\\Apps\\Volanta\\Launcher.exe",
      processName: "Volanta.exe",
      args: "",
    });

    expect(item).toMatchObject({
      type: "app",
      delayMs: 800,
      runAsAdmin: false,
      onClose: "graceful",
      enabled: true,
    });
    expect(item.type === "app" && item.args).toBeUndefined();
  });

  it("accepts an app item saved by v0.1.0 with the new launch options defaulted", () => {
    const itemFromV010 = {
      type: "app",
      id: ITEM_ID,
      name: "Volanta",
      exePath: "C:\\Apps\\Volanta\\Launcher.exe",
      processName: "Volanta.exe",
      delayMs: 0,
      runAsAdmin: false,
      onClose: "graceful",
      enabled: true,
    };

    expect(profileSchema.parse(buildProfile({ items: [itemFromV010] })).items[0]).toMatchObject({
      processNameMode: "auto",
      startMinimized: false,
      waitForSimConnect: false,
      restartOnCrash: false,
    });
  });

  it("accepts web urls and Steam launch links only", () => {
    const base = { type: "url", id: ITEM_ID, name: "SimBrief" };
    const accepted = [
      "https://simbrief.com",
      "http://localhost:8080/map",
      "steam://rungameid/1234560",
      "steam://run/1250410",
    ];
    const rejected = [
      "ftp://example.com",
      "not a url",
      "steam://uninstall/123",
      "steam://rungameid/",
      "steam://rungameid/12a",
      "steam://rungameid/1/extra",
      "ms-msdt:/id PCWDiagnostic",
      "file:///C:/Windows/notepad.exe",
      "search-ms:query=x",
    ];

    for (const url of accepted) {
      expect(launchItemSchema.safeParse({ ...base, url }).success, url).toBe(true);
    }
    for (const url of rejected) {
      expect(launchItemSchema.safeParse({ ...base, url }).success, url).toBe(false);
    }
  });

  it("rejects a delay outside the allowed range", () => {
    const result = launchItemSchema.safeParse({
      type: "url",
      id: ITEM_ID,
      name: "Charts",
      url: "https://charts.navigraph.com",
      delayMs: 60_001,
    });

    expect(result.success).toBe(false);
  });

  it("rejects an unknown item type", () => {
    const result = launchItemSchema.safeParse({ type: "script", id: ITEM_ID, name: "x" });

    expect(result.success).toBe(false);
  });

  it("accepts executables, Steam links and Store apps as launch targets", () => {
    const accepted = [
      "D:\\X-Plane 12\\X-Plane.exe",
      "C:\\Games\\msfs.EXE",
      "steam://rungameid/2537590",
      "shell:AppsFolder\\Microsoft.Limitless_8wekyb3d8bbwe!App",
    ];
    const rejected = [
      "X-Plane 12\\X-Plane.exe",
      "X-Plane.exe",
      "C:\\Games\\readme.txt",
      "ms-msdt:/id PCWDiagnostic",
      "https://example.com/setup.exe",
      "steam://uninstall/2537590",
      "shell:AppsFolder\\Microsoft.Limitless_8wekyb3d8bbwe",
      "shell:AppsFolder\\..\\evil!App",
      "shell:AppsFolder\\My App!App",
    ];
    const withTarget = (launchTarget: string) =>
      buildProfile({ triggers: [{ ...MSFS_2024, launchTarget }] });

    for (const target of accepted) {
      expect(profileSchema.safeParse(withTarget(target)).success, target).toBe(true);
    }
    for (const target of rejected) {
      expect(profileSchema.safeParse(withTarget(target)).success, target).toBe(false);
    }
  });

  it("restricts items only to triggers of the profile, without repeats", () => {
    const withOnlyFor = (onlyForTriggers: string[]) =>
      buildProfile({
        triggers: [MSFS_2020, MSFS_2024],
        items: [{ type: "url", id: ITEM_ID, name: "Site", url: "https://a.com", onlyForTriggers }],
      });

    expect(profileSchema.safeParse(withOnlyFor([])).success).toBe(true);
    expect(profileSchema.safeParse(withOnlyFor(["flightsimulator2024.EXE"])).success).toBe(true);
    const unknown = profileSchema.safeParse(withOnlyFor(["X-Plane.exe"]));
    expect(unknown.error?.issues[0]?.path).toEqual(["items", 0, "onlyForTriggers"]);
    expect(
      profileSchema.safeParse(withOnlyFor(["FlightSimulator.exe", "flightsimulator.exe"])).success,
    ).toBe(false);
  });

  it("does not let an item open before the simulator and wait for SimConnect", () => {
    const withApp = (waitForSimConnect: boolean) =>
      buildProfile({
        items: [
          {
            type: "app",
            id: ITEM_ID,
            name: "TrackIR",
            exePath: "C:\\TrackIR\\TrackIR5.exe",
            processName: "TrackIR5.exe",
            launchBeforeSimulator: true,
            waitForSimConnect,
          },
        ],
      });

    expect(profileSchema.safeParse(withApp(false)).success).toBe(true);
    expect(profileSchema.safeParse(withApp(true)).success).toBe(false);
  });

  it("imports a v0.3.1 profile without the new fields", () => {
    const profile = profileSchema.parse(
      buildProfile({
        items: [
          {
            type: "app",
            id: ITEM_ID,
            name: "Volanta",
            exePath: "C:\\Volanta\\Volanta.exe",
            processName: "Volanta.exe",
          },
        ],
      }),
    );

    expect(profile.triggers[0]?.launchTarget).toBeUndefined();
    expect(profile.items[0]).toMatchObject({ launchBeforeSimulator: false, onlyForTriggers: [] });
  });
});
