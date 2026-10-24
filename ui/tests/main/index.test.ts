/// @vitest-environment happy-dom

import { readFileSync } from "node:fs";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { MainState } from "../../src/main/model";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(async () => () => {}),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    close: vi.fn(async () => {}),
    minimize: vi.fn(async () => {}),
    startDragging: vi.fn(async () => {}),
    toggleMaximize: vi.fn(async () => {}),
  }),
}));
vi.mock("@tauri-apps/plugin-autostart", () => ({
  disable: vi.fn(async () => {}),
  enable: vi.fn(async () => {}),
  isEnabled: vi.fn(async () => false),
}));
vi.mock("lucide", () => {
  const icon = {};
  return {
    Archive: icon,
    ArrowUpDown: icon,
    Info: icon,
    LayoutGrid: icon,
    Minus: icon,
    RotateCcw: icon,
    ServerCog: icon,
    Settings: icon,
    Square: icon,
    Trash: icon,
    UsersRound: icon,
    X: icon,
    createElement: () => document.createElement("svg"),
    createIcons: vi.fn(),
  };
});

const mainHtml = readFileSync("ui/main.html", "utf8");

function state(lastPage: MainState["lastPage"]): MainState {
  return {
    widgets: [],
    deletedWidgets: [],
    version: "0.1.0",
    theme: "system",
    language: "system",
    trayClickAction: "window",
    lastPage,
  };
}

function prepareDocument() {
  const bodyStart = mainHtml.indexOf("<body>") + "<body>".length;
  const bodyEnd = mainHtml.lastIndexOf("</body>");
  document.body.innerHTML = mainHtml
    .slice(bodyStart, bodyEnd)
    .replace('<script type="module" src="./src/main/index.ts"></script>', "");
  vi.stubGlobal("ResizeObserver", class {
    observe() {}
  });
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    callback(0);
    return 1;
  });
}

async function loadMain(initial: MainState) {
  mocks.invoke.mockImplementation((command: string, args?: { preferences?: unknown }) => {
    if (command === "main_state") return Promise.resolve(initial);
    if (command === "main_painted") return Promise.resolve(true);
    if (command === "set_preferences") return Promise.resolve();
    return Promise.resolve(args);
  });
  await import("../../src/main");
  await vi.waitFor(() => {
    expect(document.querySelector(`[data-page="${initial.lastPage}"]`)?.classList)
      .toContain("is-active");
  });
}

function pageButton(page: string) {
  return document.querySelector<HTMLButtonElement>(`[data-page="${page}"]`)!;
}

beforeEach(() => {
  vi.resetModules();
  mocks.invoke.mockReset();
  mocks.listen.mockClear();
  prepareDocument();
});

describe("main window navigation", () => {
  it("restores the persisted page before the first reveal", async () => {
    await loadMain(state("about"));

    expect(pageButton("about").getAttribute("aria-current")).toBe("page");
    expect(document.querySelector<HTMLElement>('[data-view="about"]')!.hidden).toBe(false);
  });

  it("serializes rapid navigation writes so the last page wins", async () => {
    let finishFirstWrite: (() => void) | undefined;
    const written: string[] = [];
    mocks.invoke.mockImplementation((command: string, args?: {
      preferences?: { lastPage: string };
    }) => {
      if (command === "main_state") return Promise.resolve(state("integrations"));
      if (command === "main_painted") return Promise.resolve(true);
      if (command === "integration_state") {
        return Promise.resolve({
          agents: [],
          mcpArgs: [],
          mcpCommand: "aidget mcp",
          skillContent: "",
          skillDirectory: "",
        });
      }
      if (command === "set_preferences") {
        written.push(args!.preferences!.lastPage);
        if (written.length === 1) {
          return new Promise<void>((resolve) => {
            finishFirstWrite = resolve;
          });
        }
        return Promise.resolve();
      }
      return Promise.resolve();
    });

    await import("../../src/main");
    await vi.waitFor(() => {
      expect(pageButton("integrations").classList).toContain("is-active");
    });

    pageButton("local-widgets").click();
    await vi.waitFor(() => expect(written).toHaveLength(1));

    pageButton("about").click();
    await vi.waitFor(() => expect(pageButton("about").classList).toContain("is-active"));

    finishFirstWrite!();
    await vi.waitFor(() => expect(written).toHaveLength(2));
    expect(written.at(-1)).toBe("about");
  });
});
