import { systemLocale, type Locale } from "./main-locale";

export type Theme = "system" | "light" | "dark";

const THEME_KEY = "aidget.main.theme";
const LOCALE_KEY = "aidget.main.locale";

export function storedTheme(): Theme {
  const value = localStorage.getItem(THEME_KEY);
  return value === "light" || value === "dark" ? value : "system";
}

export function storedLocale(): Locale {
  const value = localStorage.getItem(LOCALE_KEY);
  return value === "en" || value === "zh-CN" ? value : "system";
}

export function saveTheme(theme: Theme) {
  localStorage.setItem(THEME_KEY, theme);
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme =
    theme === "system" ? "light dark" : theme;
}

export function saveLocale(locale: Locale) {
  localStorage.setItem(LOCALE_KEY, locale);
  document.documentElement.lang = locale === "system"
    ? systemLocale(navigator.languages) === "zh" ? "zh-CN" : "en"
    : locale;
}

export function resolvedLocale(locale: Locale): "en" | "zh" {
  if (locale === "system") return systemLocale(navigator.languages);
  return locale === "zh-CN" ? "zh" : "en";
}
