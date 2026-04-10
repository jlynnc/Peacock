import { createI18n, type Composer } from "vue-i18n";
import zhCN from "./zh-CN";
import en from "./en";

// Detect browser/system language
function getDefaultLocale(): string {
  const lang = navigator.language || "zh-CN";
  if (lang.startsWith("zh")) return "zh-CN";
  return "en";
}

// Try to load saved language preference
function getSavedLocale(): string | null {
  try {
    return localStorage.getItem("peacock-locale");
  } catch {
    return null;
  }
}

export const i18n = createI18n({
  legacy: false,
  locale: getSavedLocale() || getDefaultLocale(),
  fallbackLocale: "en",
  messages: {
    "zh-CN": zhCN,
    en: en,
  },
});

export function setLocale(locale: string) {
  (i18n.global as unknown as Composer).locale.value = locale;
  localStorage.setItem("peacock-locale", locale);
}

export function getLocale(): string {
  return (i18n.global as unknown as Composer).locale.value;
}
