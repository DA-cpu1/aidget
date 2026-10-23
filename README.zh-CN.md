<!-- prettier-ignore -->
<div align="center">

<img src="./icons/icon.png" alt="aidget 图标" width="96" />

# aidget

**把桌面创意交给 AI Agent**

[![CI](https://img.shields.io/github/actions/workflow/status/aidget-ai/aidget/check.yml?branch=main&label=CI&style=flat-square)](https://github.com/aidget-ai/aidget/actions/workflows/check.yml)
[![最新版本](https://img.shields.io/github/v/release/aidget-ai/aidget?style=flat-square)](https://github.com/aidget-ai/aidget/releases/latest)
[![Rust 1.85+](https://img.shields.io/badge/Rust-1.85%2B-b7410e?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)
[![MCP](https://img.shields.io/badge/MCP-stdio-6b7280?style=flat-square)](https://modelcontextprotocol.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-111111?style=flat-square)](LICENSE)

[快速开始](#快速开始) | [跨平台支持](#跨平台支持) | [开发](#开发) | [数据目录](#数据目录) | [安全模型](#安全模型)<br>
[English](README.md) | 简体中文

<img src="./screenshot.png" alt="aidget 截图" width="360" />

</div>

## aidget 适合你吗？

如果你的桌面死气沉沉，又恰好在使用 ChatGPT、Claude、Grok Bot 这样的桌面 Agent，那么 aidget 可以帮助你的 Agent 玩转桌面。

将 aidget 接入 Agent，用自然语言描述需求和创意，让桌面焕然一新。

一些典型场景：

- **便签纸**：让 Agent 在桌面创建个性化便签纸，记下每天的待办任务。
- **天气卡片**：让 Agent 复刻 macOS 天气小组件，实时展示当下天气预报。
- **桌面宠物**：让 Agent 照料你的电子宠物，观察它的一举一动。
- **纪念日卡**：让 Agent 创建一个纪念日卡片，记住重要的日子。
- **股市跟踪**：让 Agent 展示当前关注的股市信息，不错过任何交易时机。

更多创意待你发现！

## 快速开始

### 1. 安装

从 [GitHub Releases](https://github.com/aidget-ai/aidget/releases/latest) 下载并启动适用于你操作系统的版本。

> ⚠️ macOS 和 Windows 安装包目前没有签名，可能触发 Gatekeeper 或 SmartScreen 提示，从 GitHub 渠道下载请放心安装。

启动 aidget 会打开主窗口，也可以从系统托盘访问 aidget，首次打开会在桌面放置一个欢迎卡片，请留意。

### 2. 接入 Agent

> aidget 会检测本机已安装的常见 Agent 应用程序，任何支持 MCP 的客户端也可以手动接入。

1. 进入 **MCP 与技能** 页面。
2. 点击右侧图标为检测到的 Agent 接入「MCP 服务」和「Agent Skill」，建议同时接入。
3. 如果 Agent 没有检测到 MCP 或是 `aidget` Skill，重启 Agent 让它重新加载配置。

### 3. 让 Agent 创建第一个小组件

回到你熟悉的 Agent 对话框，用自然语言提出需求创意，比如：

```text
/aidget 在桌面放一张红桃 A 扑克牌卡片
```

或者一个定时任务：

```text
/aidget 创建一个股票走势跟踪卡片，每隔 5 分钟监控当前 Apple 股票详情。
```

## 跨平台支持

| 平台 | 窗口 / 桥接 | 能力与说明 |
| --- | --- | --- |
| macOS 12+ | WKWebView / Unix socket | 支持截图、CoreLocation 和原生 glass 材质。 |
| Windows x64 | WebView2 / 命名管道 | 支持截图和 WinRT 定位；脚本发起网络请求时使用 HTTPS。 |
| Linux x64 | WebKitGTK / Unix socket | 支持截图和 GeoClue；Wayland 下位置由合成器控制。 |

## 开发

### 工作原理

```mermaid
flowchart LR
    A["Agent"]
    A -->|"读取 Skill"| S["~/.aidget/skills"]
    A -->|"调用 MCP"| M["~/.aidget/aidget mcp"]
    A -->|"调用 CLI"| C["~/.aidget/aidget"]
    M --> U["aidget ui"]
    U --> W["widgets"]
```

### 环境要求

- Rust 1.85 或更高版本
- Node.js 20 或更高版本
- pnpm 12
- Tauri 2 对应平台的构建依赖
- Linux：`libwebkit2gtk-4.1-dev`、`libayatana-appindicator3-dev` 和 `librsvg2-dev`

### 本地运行

```bash
git clone https://github.com/aidget-ai/aidget.git && cd aidget
pnpm install
pnpm dev
```

> 开发、测试或便携运行时，可以通过 `AIDGET_HOME` 环境变量指向隔离目录：
>
> AIDGET_HOME=/tmp/aidget-dev pnpm dev

### 构建

```bash
pnpm build             # 生成 release 正式包
pnpm build:debug       # 生成 debug 应用包
```

### 检查

```bash
pnpm test
pnpm typecheck
cargo fmt --check
AIDGET_CHECK_TARGET=1 cargo check --lib
cargo test
```

### MCP 服务

| 工具                | 用途                                                       |
|---------------------|------------------------------------------------------------|
| `aidget_list`       | 列出小组件、revision、看板 revision、位置、损坏项和显示器名称。 |
| `aidget_get`        | 读取单个小组件的元数据、HTML、CSS、`agents.md` 和当前位置。    |
| `aidget_create`     | 创建包含 HTML、CSS、权限和作者说明的小组件。                  |
| `aidget_update`     | 基于 revision CAS 整体替换文件或元数据。                    |
| `aidget_patch`      | 原子替换一个或多个 `data-aidget-region` 的内部 HTML。       |
| `aidget_move`       | 在用户要求时以 boardRevision CAS 移动或缩放小组件。         |
| `aidget_screenshot` | 将渲染后的小组件截为 PNG，并等待页面就绪。                   |
| `aidget_delete`     | 将小组件移动到回收站。                                      |
| `aidget_guide`      | 读取完整创作规范。                                          |

### CLI 程序

```text
aidget ui [--background]
aidget mcp
aidget widget list|show|create|update|patch|delete
aidget board show|move
aidget displays
aidget template list|install
aidget screenshot <id> [--scale N] [--out FILE]
aidget doctor
aidget guide [--json]
```

## 数据目录

默认数据根目录是 `~/.aidget`。

```text
~/.aidget
├── aidget                    # CLI 入口（Windows 为 aidget.cmd）
├── board.json                # 卡片位置和尺寸
├── settings.toml             # 主程序配置
├── widgets/
│   └── <id>/
│       ├── config.json       # 标题、脚本开关、拖拽模式、权限和 revision
│       ├── index.html        # 小组件结构
│       ├── style.css         # 外观、尺寸和布局样式
│       ├── agents.md         # 面向 Agent 的说明文档
│       └── assets/           # 图片、字体等本地资源
├── state/
│   ├── screenshots/          # 小组件截图
│   ├── storage/
│   │   └── <id>.json         # 小组件持久化数据
│   └── widgets.json          # 小组件状态
├── skills/
│   └── aidget/
│       └── SKILL.md          # 分发给 Agent 的 Skill
└── trash/                    # 已归档的小组件
```

## 安全模型

aidget 与用户在同一账户下运行；`~/.aidget`、小组件和 Agent 配置都属于用户信任边界。

小组件是用户可控内容，不是可信扩展，因此再通过脚本权限和系统权限两层限制。

### 脚本权限

- `allowScripts` 默认关闭。关闭时，CSP 会阻止脚本、内联事件、嵌套网页、对象和 Worker，但 HTML、CSS 和静态资源仍可正常显示。
- 开启后，小组件运行在没有同源权限的沙箱中，不能读取其他小组件文件、访问 Tauri IPC 或控制 aidget 主界面。

### 系统权限

- 小组件必须在 `config.json` 的 `permissions` 中显式申请能力；未声明的调用返回 `permission_denied`。
- `location` 通过操作系统定位服务请求许可，不使用 IP 定位作为回退。
- `storage` 按小组件隔离保存字符串，上限为 256 KB，不与其他小组件共享。

### 安全声明

aidget 不保存模型或账户凭据，默认情况下，提示词、回复、密钥和账户不会经过 aidget。

安全漏洞处理响应流程见 [SECURITY.md](SECURITY.md)。

## 许可证

[MIT 开源许可证](LICENSE)
