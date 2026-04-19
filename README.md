# rccui

多 AI 编码助手管理桌面应用，支持 Claude、Cursor、Codex、Gemini 四大 Provider 的会话管理、实时交互和项目监控。

## 技术栈

| 层 | 技术 | 说明 |
|---|---|---|
| 前端框架 | Leptos 0.8 (CSR) | Rust 响应式前端，编译为 WASM |
| 桌面运行时 | Tauri 2 | 原生桌面应用 |
| UI 组件 | rust/ui | shadcn-ui 风格，85+ 组件 + 26 hooks |
| 样式 | Tailwind CSS v4 | OKLCH 主题变量，支持 light/dark 双主题 |
| 数据库 | SQLite (sqlx) | 本地持久化 |
| 国际化 | leptos-fluent 0.3 | 基于 Fluent 的中英文切换，localStorage 持久化 |
| 语言 | Rust (edition 2024, nightly) | 前后端统一 |

## 核心功能

- **多 Provider 实时聊天**: 通过 CLI spawner 与 Claude/Cursor/Codex/Gemini 实时交互，支持流式文本输出、思考过程展示、工具调用/结果渲染、权限确认对话
- **会话管理**: 会话历史记录持久化与命名，点击会话条目即可加载完整历史消息
- **交互式终端**: 基于 xterm.js + PTY 的全功能终端，支持 ANSI 渲染、分离/重连、5000 行回放缓冲、OAuth URL 检测
- **文件浏览与编辑**: 递归文件树、CodeMirror 文本编辑（失败自动回退 textarea）、语法高亮、图片预览、未保存状态提示、保存成功/失败 toast、`Cmd/Ctrl + S` 快捷保存
- **项目文件监控**: 监听 Provider 目录变更 (`~/.claude/.cursor/.codex/.gemini`)，自动刷新项目列表
- **API 密钥管理**: 多 Provider 凭据存储与管理
- **MCP 服务器管理**: Model Context Protocol 服务器配置
- **中英文切换**: 界面支持中文/英文双语，Settings 页面切换，默认中文

## 快速开始

### 前置要求

- [Rust](https://rustup.rs/) (nightly toolchain)
- [Node.js](https://nodejs.org/) (用于 Tailwind CSS)
- [Tauri CLI](https://tauri.app/start/prerequisites/)

### 安装与运行

```bash
# 安装前端依赖
pnpm install

# 开发模式 (推荐)
cargo tauri dev

# 仅前端开发服务器
trunk serve

# 生产构建
cargo tauri build
```

### 检查与验证

```bash
cargo check          # 编译检查
cargo clippy         # Lint 检查
```

## 项目结构

```
src/                            # 前端 (Leptos WASM)
├── main.rs                     # 应用入口
├── app.rs                      # 根组件 (Router + Context Providers)
├── ui/                         # rust/ui 组件库 (85+ 组件)
├── hooks/                      # 自定义 hooks (26 个)
├── features/                   # 功能模块
│   ├── chat/                   # 聊天功能 (面板、消息列表、消息项、输入框、权限对话、Provider 选择)
│   ├── shell/                  # 终端功能 (xterm.js 集成、PTY 会话管理)
│   ├── files/                  # 文件树、文本编辑器、图片预览
│   ├── git/                    # Git 操作
│   ├── onboarding/             # 引导流程
│   └── settings/               # 设置
├── state/                      # 响应式状态管理
│   ├── app_state.rs            # 全局状态 (项目、会话、侧边栏)
│   ├── chat_state.rs           # 聊天状态 (消息、流式处理、事件分发)
│   ├── file_state.rs           # 文件状态 (树、选中文件、编辑内容、dirty 状态)
│   ├── session_state.rs        # 会话状态
│   └── shell_state.rs          # 终端状态
├── layout/                     # 布局组件 (应用布局、头部、侧边栏、编辑器侧栏)
├── pages/                      # 路由页面 (仪表盘、项目详情、设置、引导)
├── tauri/                      # Tauri 桥接 (命令调用、事件监听、类型定义)
├── utils/                      # 工具函数
└── constants/                  # 常量定义

src-tauri/src/                  # 后端 (Tauri)
├── lib.rs                      # Tauri 入口，注册命令，初始化状态
├── config.rs                   # 应用配置
├── error.rs                    # 统一错误类型
├── state.rs                    # AppState, ActiveSession, PtySession
├── db/                         # 数据库层 (SQLite)
├── commands/                   # Tauri 命令 (对外接口)
│   ├── chat.rs                 # 聊天命令入口 (ChatCommand 分发)
│   ├── shell.rs                # 终端命令 (init/input/resize/detach)
│   ├── projects.rs             # 项目管理
│   ├── sessions.rs             # 会话管理
│   ├── settings.rs             # 设置管理
│   ├── mcp.rs                  # MCP 服务器管理
│   └── git.rs                  # Git 操作
├── services/                   # 业务服务层
│   ├── chat.rs                 # CLI spawners (Claude/Cursor/Codex/Gemini)
│   ├── file_watcher.rs         # 文件变更监听
│   ├── project_scanner.rs      # 项目扫描
│   └── pty_manager.rs          # PTY 管理
└── providers/                  # Provider 适配器
    ├── claude/                 # Claude SDK 协议 (control_request/response)
    ├── cursor/                 # Cursor 适配
    ├── codex/                  # Codex 适配
    └── gemini/                 # Gemini 适配

public/                         # 静态资源
├── app/                        # JS 依赖 (xterm/codemirror bundles + css)
├── hooks/                      # JS hooks 依赖
scripts/                        # 构建脚本
├── xterm_entry.js              # xterm.js esbuild 入口
├── codemirror_entry.js         # CodeMirror esbuild 入口
styles.css                      # Tailwind 入口文件 (主题变量源)
index.html                      # HTML 模板 (Trunk 入口)
docs/dev/                       # 开发文档
```

## 架构

### 通信模式

前端通过 **Tauri Commands** 调用后端，后端通过 **Tauri Events** 推送实时数据：

```
前端 (Leptos/WASM)                后端 (Tauri/Rust)
      │                                  │
      │── invoke("chat_execute") ───────>│── spawn CLI process
      │                                  │
      │<── event("chat_response") ──────│── 流式输出
      │<── event("chat_response") ──────│── 流式输出
      │<── event("chat_response") ──────│── 完成
      │                                  │
      │── invoke("shell_init") ─────────>│── 创建 PTY
      │<── { buffer, sessionKey } ──────│
      │                                  │
      │<── event("shell_output") ───────│── PTY stdout
      │── invoke("shell_input") ────────>│── PTY stdin
```

### Tauri 事件

| 事件 | 来源 | 用途 |
|---|---|---|
| `chat_response` | `services/chat.rs` | 聊天流式输出 (text/thinking/tool_use/tool_result/permission/complete/error) |
| `shell_output` | `commands/shell.rs` | 终端 stdout 数据 |
| `shell_auth_url` | `commands/shell.rs` | OAuth URL 检测 |
| `projects_updated` | `services/file_watcher.rs` | 项目文件变更通知 |

### 聊天流式处理

`chat_response` 事件通过 `kind` 字段分发到不同处理器，前端 `ChatContext` (`src/state/chat_state.rs`) 负责：

1. **文本累积**: `stream_delta` 事件将文本追加到 `stream_content` 信号
2. **内容刷新**: 在结构化事件 (thinking/tool_use/tool_result/permission_request/error/complete) 之前，通过 `flush_stream_content()` 将累积文本刷新为独立消息
3. **消息渲染**: `MessageItem` 组件根据消息 `kind` 渲染不同 UI (文本/思考过程/工具调用/工具结果)，可折叠区域在流式过程中自动展开当前步骤

### 终端集成

前端通过 xterm.js (esbuild 打包为 IIFE) 渲染终端，通过 `wasm_bindgen` JS interop 与 `window.XtermBridge` 通信：

```
xterm.js (@xterm/xterm + addon-fit + addon-web-links)
    ↓ esbuild (scripts/xterm_entry.js → public/app/xterm.bundle.js)
window.XtermBridge (create/write/onData/onResize/fit/dispose)
    ↓ wasm_bindgen extern "C" (js_namespace = ["window", "XtermBridge"])
terminal.rs (ShellTerminal 组件)
    ↓ ShellContext (session_key + terminal_id 双 ID)
Tauri Commands (shell_init/shell_input/shell_resize/shell_detach)
```

- **双 ID 设计**: 后端 `session_key` (项目路径+会话+命令的哈希) 用于 Tauri 命令寻址；前端 `terminal_id` (`term_{project_name}`) 用于 xterm.js 实例寻址
- **事件监听**: `shell_output` / `shell_auth_url` 事件在 `app.rs` 根组件通过 `std::mem::forget` 模式保持 Closure 存活
- **xterm.js 主题**: 从 CSS 自定义属性自动读取，跟随 light/dark 模式切换

### 文件编辑器

当前文件编辑器位于 `src/features/files/file_editor.rs`，默认使用 CodeMirror 6，并在初始化失败时自动回退到原生 `textarea`。现阶段能力包括：

- 文本文件读取、编辑与保存
- 图片文件预览（不进入文本编辑器）
- `Cmd/Ctrl + S` 保存
- 保存成功/失败 toast
- 未保存状态提示
- 语法高亮（按文件扩展名映射语言）
- 主题跟随应用明暗模式切换（浅色 `github-light` / 深色 `github-dark`）

如果后续继续增强编辑器功能，建议单独迭代，不要与终端或布局修复混在同一次改动里。

## 开发指南

- 查看 [AGENTS.md](./AGENTS.md) 获取编码规范和项目配置说明
- 查看 [docs/dev/](./docs/dev/) 获取开发计划文档
- 组件使用 `bind_value` 进行双向绑定（需要 nightly feature）
- 使用 `attr:r#type` 代替 `attr:type`（Rust 关键字冲突）
- Tailwind 类通过 `tw_merge` 合并
- 所有颜色使用 OKLCH 主题变量（定义在 `styles.css`），不要在组件中硬编码 hex/rgb 值
- 暗黑模式通过 `<html>` 根元素的 `.dark` class 驱动，`ThemeMode` hook 管理信号、持久化和 DOM 同步
- 可用语义化颜色：`primary`, `secondary`, `muted`, `accent`, `destructive`, `success`, `warning`, `info`, `provider-claude/codex/gemini` 等
- 需要准确保存反馈时，不要在 UI 里“先调用保存、再立刻弹成功 toast”；应等待保存命令真实返回结果后再更新 toast 和 `editor_dirty`

## 许可证

MIT
