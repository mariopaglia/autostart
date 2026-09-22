import type { Theme } from "@/bindings/Theme";

export type ResolvedTheme = "light" | "dark";

export function resolveTheme(theme: Theme, systemPrefersDark: boolean): ResolvedTheme {
  if (theme === "system") return systemPrefersDark ? "dark" : "light";
  return theme;
}
