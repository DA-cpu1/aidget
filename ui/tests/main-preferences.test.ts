import { afterEach, describe, expect, it, vi } from "vitest";
import { resolvedLocale, saveLocale, saveTheme, storedLocale, storedTheme } from "../src/main-preferences";

const values = new Map<string, string>();
const root = { dataset: {} as Record<string, string>, style: { colorScheme: "" }, lang: "" };

vi.stubGlobal("localStorage", {
  getItem: (key: string) => values.get(key) ?? null,
  setItem: (key: string, value: string) => values.set(key, value),
});
vi.stubGlobal("document", { documentElement: root });
vi.stubGlobal("navigator", { languages: ["zh-CN"] });

afterEach(() => {
  values.clear();
  root.dataset = {};
  root.style.colorScheme = "";
  root.lang = "";
});

describe("main window preferences", () => {
  it("defaults to system appearance and language", () => {
    expect(storedTheme()).toBe("system");
    expect(storedLocale()).toBe("system");
    expect(resolvedLocale("system")).toBe("zh");
  });

  it("persists a selected color theme and updates the root color scheme", () => {
    saveTheme("dark");
    expect(storedTheme()).toBe("dark");
    expect(root.dataset.theme).toBe("dark");
    expect(root.style.colorScheme).toBe("dark");
  });

  it("persists the locale and resolves explicit Chinese", () => {
    saveLocale("zh-CN");
    expect(storedLocale()).toBe("zh-CN");
    expect(root.lang).toBe("zh-CN");
    expect(resolvedLocale("zh-CN")).toBe("zh");
  });
});
