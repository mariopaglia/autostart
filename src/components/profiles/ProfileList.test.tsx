// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createProfile } from "@/lib/profile-factory";
import { useProfilesStore } from "@/stores/profiles-store";
import { useUiStore } from "@/stores/ui-store";
import { ProfileList } from "./ProfileList";

vi.mock("@/lib/tauri", () => ({
  commands: {},
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

vi.mock("./profile-files", () => ({
  exportProfileToFile: vi.fn(),
  readProfileFromFile: vi.fn(),
}));

const first = createProfile("First");
const second = createProfile("Second");

function profileButton(name: string) {
  return screen.getByRole("button", { name: new RegExp(`^${name}`) });
}

beforeEach(() => {
  useProfilesStore.setState({ profiles: [first, second], selectedId: first.id });
  useUiStore.setState({ screen: "settings" });
});

afterEach(cleanup);

describe("ProfileList", () => {
  it("opens the Profiles screen with the clicked profile from another screen", () => {
    render(<ProfileList />);

    fireEvent.click(profileButton("Second"));

    expect(useUiStore.getState().screen).toBe("main");
    expect(useProfilesStore.getState().selectedId).toBe(second.id);
    expect(profileButton("Second").getAttribute("aria-current")).toBe("page");
  });

  it("highlights the selected profile only while the Profiles screen is open", () => {
    render(<ProfileList />);
    expect(profileButton("First").getAttribute("aria-current")).toBeNull();

    fireEvent.click(profileButton("First"));

    expect(profileButton("First").getAttribute("aria-current")).toBe("page");
  });
});
