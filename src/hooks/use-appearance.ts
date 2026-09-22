import { useEffect, useSyncExternalStore } from "react";
import { i18n } from "@/i18n";
import { resolveTheme, type ResolvedTheme } from "@/lib/theme";
import { useSettingsStore } from "@/stores/settings-store";

const darkSchemeQuery = window.matchMedia("(prefers-color-scheme: dark)");

function subscribeToColorScheme(onChange: () => void) {
  darkSchemeQuery.addEventListener("change", onChange);
  return () => {
    darkSchemeQuery.removeEventListener("change", onChange);
  };
}

function systemPrefersDark() {
  return darkSchemeQuery.matches;
}

export function useAppearance(): ResolvedTheme {
  const theme = useSettingsStore((state) => state.settings?.theme ?? "dark");
  const language = useSettingsStore((state) => state.settings?.language ?? "pt-BR");
  const prefersDark = useSyncExternalStore(subscribeToColorScheme, systemPrefersDark);
  const resolvedTheme = resolveTheme(theme, prefersDark);

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", resolvedTheme === "dark");
    root.style.colorScheme = resolvedTheme;
  }, [resolvedTheme]);

  useEffect(() => {
    document.documentElement.lang = language;
    void i18n.changeLanguage(language);
  }, [language]);

  return resolvedTheme;
}
