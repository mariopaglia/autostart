import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en.json";
import ptBR from "./locales/pt-BR.json";

export const defaultNamespace = "translation";

export const resources = {
  "pt-BR": { translation: ptBR },
  en: { translation: en },
} as const;

void i18n.use(initReactI18next).init({
  resources,
  lng: "pt-BR",
  fallbackLng: "pt-BR",
  interpolation: { escapeValue: false },
});

export { i18n };
