# Windows 下 Pi 的 MCP 与 Skill 接入验证

关联提案：[Issue #7](https://github.com/aidget-ai/aidget/issues/7)。本次交付为验证记录和最小复现脚本，不包含产品代码修复。

**结论：在下述 SDK 与显式可执行文件条件下，仅 Skill 的 CLI 生命周期、位置保持、重启持久化及配置冲突保护通过；Windows MCP 尚不可判定为可用。** 仅 MCP 和 MCP＋Skill 两种场景都能发现 9 个工具并调用 `aidget_guide`，但 `aidget_list` 超时。独立 stdio 复现确认服务端出现 Tokio 嵌套运行时 panic，已记录为 [Issue #9](https://github.com/aidget-ai/aidget/issues/9)。相关 MCP 生命周期被阻塞，不能将本次结果记为全部通过。

## 环境与隔离

| 项目 | 实际环境 |
| --- | --- |
| 测试日期 | 2026-10-25，Asia/Hong_Kong |
| 系统 | Windows 11 家庭版中文版，x64，10.0.26200 |
| aidget | 开发版 `0.1.0`，commit `f930bef7dfef0a7219c1246e1160ea5a80815209` |
| 产品补丁 | 无；运行未修改的 `target/debug/aidget.exe` |
| Pi | 已安装的 `@earendil-works/pi-coding-agent` 1.3.0，使用其内置 MCP 支持 |
| 前端工具链 | Node.js 24.10.0，pnpm 12.15.0 |
| Rust 工具链 | 自定义 GNU 工具链；`rustc 1.99.1 (58cf8f9c3 2026-10-13)`，`cargo 1.99.1 (5f94df478 2026-08-27)` |
| 数据目录 | 可丢弃的 `AIDGET_HOME`、`PI_CODING_AGENT_DIR` 和工作区，两个进程继承同一测试环境 |

Pi 测试会话未加载个人 Agent 资源，aidget 使用可丢弃数据。应用检测页仍会读取其他已安装 Agent 的现有状态；接入配置的实际文件变更只发生在临时 Pi 目录，其他 Agent 配置未改动。

首次前台启动受 [Issue #8](https://github.com/aidget-ai/aidget/issues/8) 影响。本次先运行 `aidget ui --no-main-window`，等待 `aidget doctor --json` 返回 `uiRunning: true`，再运行 `aidget ui` 显示主窗口。此运行前提不表示 #8 已修复。

SDK 验证采用 Pi 1.3.0 自身的 Skill 解析器、MCP 配置解析器、stdio 传输和 AgentSession 工具调用管线。显式 ResourceLoader 排除个人目录资源及 prompt templates，因此不覆盖交互式 CLI 的默认资源发现。模型输出由本地确定性 provider 给出，不访问远程模型；真实 Pi 工具、aidget 进程和返回值均未模拟。这验证了调用管线，不证明模型能够自主选择正确工具。仅 Skill 的 CLI 操作显式使用源码构建的 `target/debug/aidget.exe`，未验证 Skill 文档默认个人目录 launcher 的安装或解析，也不证明真实模型能按 Skill 提示找到 launcher。

## 验证结果

| 场景 | 实际结果 | 状态 |
| --- | --- | --- |
| GUI 安装、关闭 Pi MCP / Skill | 分别完成仅 MCP、仅 Skill、双启用及卸载；MCP 写入当前可执行文件加 `["mcp"]`，Skill 文件与仓库版本 SHA-256 一致 | 通过 |
| 仅 MCP | 原生 `pi mcp list --json` 显示 connected；新 SDK 会话发现 9 个工具、无 aidget Skill；真实 `aidget_guide` 成功，`aidget_list` 10 秒超时 | 调用失败，生命周期阻塞 |
| 仅 Skill | `/skill:aidget` 展开；真实 Pi `write` / `powershell` 工具执行创建、读取、修改、移动、截图、删除 | 通过，限上述 SDK 与确定性模型条件 |
| MCP＋Skill | 新会话同时发现 Skill 和 9 个 MCP 工具，Skill 展开；真实 `aidget_guide` 成功，`aidget_list` 仍超时 | 调用失败，生命周期阻塞 |
| 手动拖动后修改内容 | 用户确认手动拖动；Pi patch 只推进内容 revision，整个 board 和 frame 保持不变 | 通过位置检查；有下述视觉异常 |
| 重启持久化 | 重启隔离 UI 后完整 document、board 相等；新 Pi 会话仍发现 Skill；之后删除成功 | 通过，限仅 Skill 场景 |
| 同名配置冲突 | GUI 分别拒绝覆盖不同的 `aidget` MCP entry 和 Skill，显示明确错误；三个文件 SHA-256 前后相同 | 通过 |
| 无关配置保留 | MCP 卸载仅删除 aidget entry，Skill 卸载保留无关 Skill；顶层标记和禁用的无关 MCP server 保留 | 通过 |

接入状态截图：[仅 MCP](evidence/gui-mcp-only.png)、[仅 Skill](evidence/gui-skill-only.png)、[双启用](evidence/gui-both.png)。[Pi MCP 调用证据](evidence/pi-mcp.json) 保留两种 MCP 场景的实际父子工具调用、参数和错误；发现工具和 CLI 成功均不能替代实际 MCP 操作成功。

## Skill 生命周期与位置

[Skill 调用证据](evidence/pi-skill.json) 包含 Skill 展开后的用户消息摘录、8 次真实 Pi 工具事件、实际 CLI 参数、revision、boardRevision 和 frame。5 次 `write` 准备工作区文件；3 个独立会话分别通过真实 `powershell` 执行创建和移动、拖动后的 patch、重启后的读取和删除，三个阶段均返回退出码 0。

| 阶段 | 内容 revision | boardRevision | frame `(x,y,w,h)` |
| --- | --- | --- | --- |
| 创建卡片 | 1 | 3 | `(456,96,360,220)` |
| 修改标题 | 2 | 3 | 不变 |
| 显式移动 | 2 | 4 | `(80,340,360,220)` |
| 原生拖动后的最终状态 | 2 | 13 | `(595,267,360,220)` |
| Pi patch `content` 区域 | 3 | 13 | 不变 |
| 重启隔离 UI | 3 | 13 | 不变 |
| 删除卡片 | — | 14 | 列表中不存在该卡片 |

创建前读取 authoring guide，修改前读取卡片及其 `agents` 指导。标题更新和 patch 均没有返回 frame，脚本比较确认它们不改变整个 board。重启前核对了测试 UI 的可执行文件路径，只结束该测试进程，再以相同环境启动新进程；完整 document 和 board 跨重启相等。

[初始截图](evidence/skill-before-drag.png)、[拖动并修改后的截图](evidence/skill-after-drag.png)、[重启后的截图](evidence/skill-after-restart.png) 保留原始图像。拖动并修改后的截图出现文字放大和裁切，重启后字号恢复正常、文字完整；本次未定位原因。位置检查通过不能证明渲染全程正确。

在排除了 prompt templates 的显式 SDK 配置中，`/skill:aidget` 展开为 Skill 内容，`/aidget` 仍为原文。这只是该配置下的观察；未验证原生交互式 Pi CLI 的简写支持，没有据此确认另一个 Bug。

## Pi 复测步骤与复现范围

下文的 stdio 脚本可独立自动复现 #9。Pi JSON 则是已经发生的会话证据；本次没有提交本地 SDK provider / ResourceLoader 夹具，不能仅靠这些 JSON 重放同一 SDK 会话。下面是人工复测产品行为的顺序，手工 CLI 命令不等同于重放 Pi 工具调用。

例如，在仓库根目录的新 PowerShell 终端创建隔离环境，并从该终端启动 aidget 和 Pi：

```powershell
$runRoot = Join-Path (Resolve-Path '.\target').Path ('windows-pi-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $runRoot
$env:AIDGET_HOME = Join-Path $runRoot 'aidget'
$env:PI_CODING_AGENT_DIR = Join-Path $runRoot 'pi'
New-Item -ItemType Directory -Path $env:AIDGET_HOME, $env:PI_CODING_AGENT_DIR, (Join-Path $runRoot 'workspace')
```

1. 使用 Pi 1.3.0、新建的可丢弃数据目录及工作区，为 aidget 和 Pi 设置相同的 `AIDGET_HOME`、`PI_CODING_AGENT_DIR`。按上文的 #8 启动前提打开 GUI，仅操作 Pi 开关。
2. 分别设为仅 MCP、仅 Skill、双启用；每次切换后使用新的隔离 Pi 会话。MCP 场景运行 `pi mcp list --json`，再实际调用 `aidget_guide` 和 `aidget_list`；Skill 场景输入 `/skill:aidget`，明确指定源码构建的 aidget 路径并允许 PowerShell 工具。确认会话不会加载个人资源；原生交互式 CLI 的默认资源来源需另行检查，不属于本报告已验证范围。
3. 仅 Skill 下按“创建 → 读取指导 → 更新 → 移动 → 截图 → 用户拖动 → 读取当前 revision → patch → 重启 → 读取 → 删除”复测。使用仓库内原样保存的 [index.html](fixtures/pi-skill/index.html)、[style.css](fixtures/pi-skill/style.css)、[agents.md](fixtures/pi-skill/agents.md)、[region.html](fixtures/pi-skill/region.html)；它们足以重建同一静态卡片素材。

以下命令在仓库根目录、已设置隔离环境且 UI 健康时执行。任一命令退出码非零即停止；revision 必须重新读取。可在 Pi 的真实 PowerShell 工具中请求执行并保留工具事件，也可在终端人工检查 CLI，后者只验证 CLI 路径。

```powershell
if (!$env:AIDGET_HOME -or !$env:PI_CODING_AGENT_DIR) { throw 'Set the disposable test environment first' }
$exe = (Resolve-Path '.\target\debug\aidget.exe').Path
$fixture = (Resolve-Path '.\docs\verification\fixtures\pi-skill').Path
& $exe guide --json
& $exe widget create issue7-pi-skill --title 'Issue 7 Pi Skill' --html-file "$fixture\index.html" --css-file "$fixture\style.css" --agents-file "$fixture\agents.md" --drag-mode all --json
$doc = & $exe widget show issue7-pi-skill --json | ConvertFrom-Json
$doc.agents
& $exe widget update issue7-pi-skill --revision $doc.meta.revision --title 'Issue 7 Pi Skill updated' --json
$board = & $exe board show --json | ConvertFrom-Json
& $exe board move issue7-pi-skill --revision $board.revision --x 80 --y 340 --w 360 --h 220 --json
& $exe screenshot issue7-pi-skill --scale 1 --out (Join-Path $env:AIDGET_HOME 'before-drag.png') --json
```

现在由用户拖动卡片，再读取新的 document / board 并执行 patch：

```powershell
$dragged = & $exe widget show issue7-pi-skill --json | ConvertFrom-Json
$draggedBoard = & $exe board show --json | ConvertFrom-Json
$dragged.agents
& $exe widget patch issue7-pi-skill content --revision $dragged.meta.revision --html-file "$fixture\region.html" --json
$patched = & $exe widget show issue7-pi-skill --json | ConvertFrom-Json
$patchedBoard = & $exe board show --json | ConvertFrom-Json
```

比较 `$draggedBoard` 与 `$patchedBoard` 的完整 JSON，预期相等；内容 revision 应增加 1，frame 不变。核对进程路径后只重启本次隔离 UI，保留环境变量，再执行以下命令。将返回的 document / board 与 `$patched` / `$patchedBoard` 比较，最后确认删除后的列表不含测试卡片。

```powershell
& $exe widget show issue7-pi-skill --json
& $exe board show --json
& $exe screenshot issue7-pi-skill --scale 1 --out (Join-Path $env:AIDGET_HOME 'after-restart.png') --json
& $exe widget delete issue7-pi-skill --json
& $exe widget list --json
```

## 配置冲突保护

先通过 GUI 移除临时 Pi 的 MCP 和 Skill，再放入不同的同名条目，分别从 GUI 尝试安装。页面明确显示 `invalid params`，具体原因分别为 `already has a different aidget MCP entry` 和 `already contains a different aidget skill`。测试后 `mcp.json`、冲突 Skill、无关 Skill 三个文件的 SHA-256 均与操作前相同，顶层标记及无关 MCP server 也保留。

[配置与哈希证据](evidence/config-conflicts.json) 包含脱敏错误原文、冲突夹具和前后哈希；另有 [MCP 冲突截图](evidence/gui-mcp-conflict.png)和 [Skill 冲突截图](evidence/gui-skill-conflict.png)。这些是临时 Pi 目录中的冲突，不涉及真实 Agent 配置写入。

## Windows MCP 失败与最小复现

仅 MCP 和双启用场景的真实调用路径均为 Pi AgentSession → codemode → `mcp__aidget__aidget_list`。嵌套工具返回 `MCP request aborted`，父调用报告 `Execution timed out after 10000 ms`，两者均为错误。`aidget_guide` 为成功的对照调用。

绕过 Pi 和模型，直接运行同一未修改的 `aidget.exe mcp` 仍可复现；[stdio 结果](evidence/windows-mcp.json) 如下：

| 请求 | 实际结果 |
| --- | --- |
| `initialize`，协议 `2025-03-26` | 56 ms 成功 |
| `tools/list` | 2 ms 成功，9 个工具 |
| `tools/call aidget_guide` | 成功 |
| `tools/call aidget_list` | 5 秒无响应，服务端 panic |

```text
thread 'tokio-rt-worker' panicked at src\transport\ipc.rs:242:17:
Cannot start a runtime from within a runtime.
```

堆栈及源码指向 MCP 异步处理 → `Client::is_running` / `ping` → Windows IPC `exchange` → 新建 Tokio runtime 并执行 `runtime.block_on`。预先启动健康 GUI 的独立诊断也出现相同 panic，因此该失败不依赖模型或 GUI 尚未启动。缺陷另见 [Issue #9](https://github.com/aidget-ai/aidget/issues/9)。

使用 [最小复现脚本](../../scripts/verification/windows-mcp.mjs)，在仓库根目录执行：

```powershell
$exe = (Resolve-Path '.\target\debug\aidget.exe').Path
$testHome = Join-Path (Resolve-Path '.\target').Path ('windows-mcp-' + [guid]::NewGuid().ToString('N'))
node .\scripts\verification\windows-mcp.mjs $exe $testHome
```

参数必须是绝对路径，数据目录必须尚不存在；脚本独占创建该目录，拒绝复用已有目录。它不需要 Pi 或模型，不修改 Agent 配置，不显式启动 GUI，每个请求超时 5 秒，仅清理自身 MCP 子进程，清理等待上限为 2 秒。桥接调用修复后，aidget 自身可能自动启动独立 UI，脚本不管理该 UI。测试目录和 JSON 输出保留供检查。本次命令退出码为 1，成功重现了产品失败；不是验证通过。

## 仓库检查与未测范围

以下命令均在上述 SHA 的 Windows checkout 执行，使用隔离安装的官方 pnpm 12.15.0。Rust 测试可执行文件未做事后修补。

| 实际命令 | 结果 |
| --- | --- |
| `pnpm install --frozen-lockfile` | 通过 |
| `pnpm test` | 通过，18 个测试文件、116 个测试 |
| `pnpm typecheck` | 通过 |
| `pnpm build:ui` | 通过 |
| `cargo fmt --check` | 通过 |
| `cargo check --lib`，环境变量 `AIDGET_CHECK_TARGET=1` | 通过；完成后清除变量 |
| `cargo test --no-run` | 编译通过；链接器警告 `.rsrc merge failure: multiple non-default manifests` |
| `cargo test` | 首个测试程序装载失败：`0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND`，未执行到用例 |
| `node --check scripts/verification/windows-mcp.mjs` | 通过 |
| `git diff --check` | 通过 |

`cargo test` 不能记为通过，也没有测试用例失败数量可报告。未测试 macOS、Linux、MSVC、发布安装包、打包流程、默认个人目录 launcher 安装与解析、交互式 Pi CLI 默认资源发现和远程模型自主调用。仅 MCP、双启用下的卡片生命周期及其重启、拖动行为均被 #9 阻塞；仅 Skill 的成功结果不替代这些场景。
