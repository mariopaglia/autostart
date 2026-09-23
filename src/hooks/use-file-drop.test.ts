// @vitest-environment jsdom
import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { windowEvents, type FileDropEvent } from "@/lib/tauri";
import { useFileDrop } from "./use-file-drop";

vi.mock("@/lib/tauri", () => ({ windowEvents: { onFileDrop: vi.fn() } }));

let emit: (event: FileDropEvent) => void = () => undefined;

beforeEach(() => {
  vi.mocked(windowEvents.onFileDrop).mockImplementation((handler) => {
    emit = handler;
    return Promise.resolve(() => undefined);
  });
});

afterEach(() => {
  document.body.innerHTML = "";
});

function send(event: FileDropEvent) {
  act(() => {
    emit(event);
  });
}

describe("useFileDrop", () => {
  it("shows the overlay while files are dragged over the window", () => {
    const { result } = renderHook(() => useFileDrop({ enabled: true, onDrop: vi.fn() }));

    send({ kind: "enter" });
    expect(result.current).toBe(true);

    send({ kind: "leave" });
    expect(result.current).toBe(false);
  });

  it("hands the dropped paths over and hides the overlay", () => {
    const onDrop = vi.fn();
    const { result } = renderHook(() => useFileDrop({ enabled: true, onDrop }));

    send({ kind: "enter" });
    send({ kind: "drop", paths: ["C:\\Volanta.lnk"] });

    expect(onDrop).toHaveBeenCalledWith(["C:\\Volanta.lnk"]);
    expect(result.current).toBe(false);
  });

  it("ignores drops while a dialog is open", () => {
    const onDrop = vi.fn();
    const { result } = renderHook(() => useFileDrop({ enabled: true, onDrop }));
    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    document.body.append(dialog);

    send({ kind: "enter" });
    expect(result.current).toBe(false);
    send({ kind: "drop", paths: ["C:\\Volanta.lnk"] });

    expect(onDrop).not.toHaveBeenCalled();
  });

  it("ignores drops while disabled", () => {
    const onDrop = vi.fn();
    renderHook(() => useFileDrop({ enabled: false, onDrop }));

    send({ kind: "drop", paths: ["C:\\Volanta.lnk"] });

    expect(onDrop).not.toHaveBeenCalled();
  });
});
