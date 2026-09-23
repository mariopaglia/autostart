import { vi } from "vitest";

// jsdom lacks the layout APIs that Radix and cmdk call while rendering.
vi.stubGlobal(
  "ResizeObserver",
  class {
    observe() {}
    unobserve() {}
    disconnect() {}
  },
);
Element.prototype.scrollIntoView = vi.fn();
