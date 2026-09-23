// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { EmptyItems } from "./EmptyItems";

afterEach(cleanup);

describe("EmptyItems", () => {
  it("invites the user to drag apps in", () => {
    render(<EmptyItems isElevated={false} onAddApp={vi.fn()} onAddUrl={vi.fn()} />);

    expect(screen.getByText(i18n.t("items.dragHint"))).toBeDefined();
  });

  it("explains that dragging is unavailable as administrator", () => {
    render(<EmptyItems isElevated onAddApp={vi.fn()} onAddUrl={vi.fn()} />);

    expect(screen.getByText(i18n.t("items.dragUnavailableAsAdmin"))).toBeDefined();
    expect(screen.queryByText(i18n.t("items.dragHint"))).toBeNull();
  });
});
