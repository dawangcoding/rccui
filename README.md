# rccui

多 AI 编码助手管理桌面应用，支持 Claude、Cursor、Codex、Gemini 四大 Provider 的会话管理、实时交互和项目监控。

## 技术栈

| 层 | 技术 | 说明 |
|---|---|---|
| 前端框架 | Leptos 0.8 (CSR) | Rust 响应式前端，编译为 WASM |
| 桌面运行时 | Tauri 2 | 原生桌面应用 |
| UI 组件 | rust/ui | shadcn-ui 风格，85+ 组件 + 26 hooks |
| 样式 | Tailwind CSS v4 | OKLCH 主题变量 |
| 数据库 | SQLite (sqlx) | 本地持久化 |
| 语言 | Rust (edition 2024, nightly) | 前后端统一 |

## 核心功能

- **多 Provider 聊天**: 通过 CLI spawner 与 Claude/Cursor/Codex/Gemini 实时交互，流式输出
- **终端会话管理**: PTY 终端创建、输入、调整大小、分离/重连，带 5000 行回放缓冲
- **项目文件监控**: 监听 Provider 目录变更，自动刷新项目列表
- **API 密钥管理**: 多 Provider 凭据存储与管理
- **MCP 服务器管理**: Model Context Protocol 服务器配置
- **会话持久化**: 会话历史记录与命名

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
├── app.rs                      # 根组件
├── ui/                         # rust/ui 组件库 (85+ 组件)
├── hooks/                      # 自定义 hooks (26 个)
├── utils/                      # 工具函数
└── constants/                  # 常量定义

src-tauri/src/                  # 后端 (Tauri)
├── lib.rs                      # Tauri 入口，注册命令，初始化状态
├── config.rs                   # 应用配置
├── error.rs                    # 统一错误类型
├── state.rs                    # AppState, ActiveSession, PtySession
├── db/                         # 数据库层 (SQLite)
│   ├── api_keys.rs
│   ├── credentials.rs
│   ├── session_names.rs
│   └── users.rs
├── commands/                   # Tauri 命令 (对外接口)
│   ├── api_keys.rs             # API 密钥管理
│   ├── chat.rs                 # 聊天命令入口
│   ├── commands_handler.rs     # 命令执行处理
│   ├── mcp.rs                  # MCP 服务器管理
│   ├── projects.rs             # 项目管理
│   ├── sessions.rs             # 会话管理
│   ├── settings.rs             # 设置管理
│   └── shell.rs                # 终端命令 (init/input/resize/detach)
├── services/                   # 业务服务层
│   ├── chat.rs                 # CLI spawners (Claude/Cursor/Codex/Gemini)
│   ├── file_watcher.rs         # 文件变更监听
│   ├── project_scanner.rs      # 项目扫描
│   ├── pty_manager.rs          # PTY 管理
│   └── sessions.rs             # 会话服务
└── providers/                  # Provider 适配器
    ├── claude/                 # Claude SDK 协议 (control_request/response)
    ├── cursor/                 # Cursor 适配
    ├── codex/                  # Codex 适配
    └── gemini/                 # Gemini 适配

public/                         # 静态资源
styles.css                      # Tailwind 入口文件
index.html                      # HTML 模板
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
| `chat_response` | `services/chat.rs` | 聊天流式输出 |
| `shell_output` | `commands/shell.rs` | 终端 stdout 数据 |
| `shell_auth_url` | `commands/shell.rs` | OAuth URL 检测 |
| `projects_updated` | `services/file_watcher.rs` | 项目文件变更通知 |

## 开发指南

- 查看 [AGENTS.md](./AGENTS.md) 获取编码规范和项目配置说明
- 查看 [docs/dev/](./docs/dev/) 获取开发计划文档
- 组件使用 `bind_value` 进行双向绑定（需要 nightly feature）
- 使用 `attr:r#type` 代替 `attr:type`（Rust 关键字冲突）
- Tailwind 类通过 `tw_merge` 合并

## 许可证

MIT
