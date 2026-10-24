import { describe, expect, it } from "vitest";
import { systemLocale, translate } from "../src/main-locale";

describe("main window locale", () => {
  it("follows simplified Chinese when any system language is Chinese", () => {
    expect(systemLocale(["en-US", "zh-Hans-CN"])).toBe("zh");
    expect(systemLocale(["en-US"])).toBe("en");
  });

  it("translates widget filters in both supported languages", () => {
    expect(translate("en", "filterAll")).toBe("All");
    expect(translate("zh", "filterEnabled")).toBe("已打开");
    expect(translate("zh", "filterDisabled")).toBe("已关闭");
    expect(translate("zh", "recycleBin")).toBe("回收站");
    expect(translate("zh", "sortNewestDescription")).toBe("按创建时间倒序排列");
  });

  it("labels integrations, local widgets, and navigation groups", () => {
    expect(translate("zh", "integrations")).toBe("接入");
    expect(translate("zh", "mcpAndSkills")).toBe("MCP 与技能");
    expect(translate("zh", "mcpStepTitle")).toBe("第一步：接入 MCP 服务");
    expect(translate("zh", "skillsStepTitle")).toBe("第二步：接入 Agent Skill");
    expect(translate("zh", "widgetsGroup")).toBe("小组件");
    expect(translate("zh", "localWidgets")).toBe("本地组件");
    expect(translate("zh", "communityWidgets")).toBe("社区组件");
    expect(translate("en", "widgetsGroup")).toBe("Widgets");
    expect(translate("zh", "agentAll")).toBe("全部");
    expect(translate("zh", "integrationNoAgents")).toBe("没有检测到可接入此资源的 Agent。");
    expect(translate("en", "integrationRestart")).toContain("Restart");
  });

  it("labels appearance, behavior, and tray click options", () => {
    expect(translate("zh", "appearance")).toBe("外观");
    expect(translate("zh", "behavior")).toBe("行为");
    expect(translate("zh", "trayClickAction")).toBe("点击托盘图标");
    expect(translate("zh", "trayMenu")).toBe("打开快捷菜单");
    expect(translate("zh", "trayWindow")).toBe("打开主窗口");
    expect(translate("en", "appearance")).toBe("Appearance");
    expect(translate("en", "behavior")).toBe("Behavior");
    expect(translate("en", "trayClickAction")).toBe("Click tray icon");
    expect(translate("en", "trayMenu")).toBe("Open quick menu");
    expect(translate("en", "trayWindow")).toBe("Open main window");
  });
});
