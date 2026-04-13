# rcui → rccui 后端迁移开发计划

将 rcui (Axum Web 服务器) 后端完整迁移到 rccui (Tauri 桌面应用)。

## 架构变更

| 原 (rcui / Axum) | 新 (rccui / Tauri) |
|---|---|
| HTTP 路由 + 中间件 | `#[tauri::command]` |
| WebSocket 双向通信 | Tauri commands (客户端→后端) + events (后端→客户端) |
| `mpsc::UnboundedSender<ChatResponse>` | `AppHandle.emit("chat_response", ...)` |
| JWT 认证层 | 移除，桌面端使用 `LOCAL_USER_ID = 1` |
| `futures-util` SinkExt/StreamExt | 移除（不再需要 WebSocket 流处理） |

### 核心设计决策

- **AppHandle 存入 AppState**: `tauri::AppHandle` 是 `Send + Sync + Clone`，可安全从 `spawn_blocking` 中发送事件
- **单一 chat_execute 命令**: 接收 `ChatCommand` JSON 枚举并分派到对应 CLI spawner，而非每个变体一个命令
- **shell_init 直接返回 buffer**: 重连时通过命令响应返回 replay buffer，而非逐条发送 WebSocket 消息
- **File watcher 直接 emit**: 通过 `app_handle.emit("projects_updated", ...)` 替代 broadcast channel

---

## Phase 1: 基础设施

**提交**: `4e5dc0d feat: migrate rcui backend to Tauri Commands (Phase 1-2)`

### 迁移内容

| 模块 | 源文件 (rcui) | 目标文件 (rccui) |
|---|---|---|
| Cargo 依赖 | `Cargo.toml` | `src-tauri/Cargo.toml` |
| 配置 | `config.rs` | `src-tauri/src/config.rs` |
| 数据库 | `db/*.rs` | `src-tauri/src/db/*.rs` |
| 错误处理 | `error.rs` | `src-tauri/src/error.rs` |
| 状态管理 | `state.rs` | `src-tauri/src/state.rs` |
| 数据库迁移 | `migrations/` | `src-tauri/migrations/` |

### 关键改动

- `AppState` 添加 `app_handle: tauri::AppHandle` 字段
- `AppState::new()` 接受 `app_handle` 参数
- 移除所有 auth 中间件和 JWT 相关代码
- 数据库使用 SQLite (sqlx)

---

## Phase 2: 业务逻辑

**提交**: `4e5dc0d feat: migrate rcui backend to Tauri Commands (Phase 1-2)` (与 Phase 1 同一提交)

### 迁移内容

| 模块 | 说明 | 命令数量 |
|---|---|---|
| providers/ | Claude/Cursor/Codex/Gemini 适配器 | — |
| services/ | project_scanner, sessions 等 | — |
| commands/ | Tauri 命令封装 | 75+ |

### Tauri Command 模式

```rust
#[tauri::command]
pub async fn xxx(
    state: tauri::State<'_, Arc<AppState>>,
    /* params */
) -> Result<Value, String> {
    async fn _fn(state: &AppState, /* params */) -> Result<Value, AppError> {
        // 业务逻辑
    }
    _fn(&state, /* params */).await.map_err(|e| e.to_string())
}
```

### 命令模块

- `commands/api_keys.rs` — API 密钥管理
- `commands/commands_handler.rs` — 命令执行处理
- `commands/mcp.rs` — MCP 服务器管理
- `commands/projects.rs` — 项目管理
- `commands/sessions.rs` — 会话管理
- `commands/settings.rs` — 设置管理

---

## Phase 3: 事件驱动功能 (WebSocket → Tauri Events)

**提交**: `ffa4cc3 feat: convert WebSocket real-time features to Tauri Events (Phase 3)`

### 3.1 Chat 服务

| 源 | 目标 |
|---|---|
| `routes/ws.rs` (169 行) | `commands/chat.rs` (26 行) |
| `services/chat.rs` (1994 行) | `services/chat.rs` (~1950 行) |

**改动要点**:
- `tx.send(response)` → `app.emit("chat_response", response)`
- 单一 `chat_execute` 命令接收 `ChatCommand` JSON
- 支持 4 种 CLI spawner: Claude (SDK 模式 + control_request/response)、Cursor、Codex、Gemini
- 流式响应通过 `chat_response` Tauri 事件推送

**Tauri 事件**:
- `chat_response` — 聊天流式输出 (text/complete/error/permission_request 等)

### 3.2 Shell 服务

| 源 | 目标 |
|---|---|
| `routes/shell.rs` (516 行) | `commands/shell.rs` (~300 行) |

**命令**:
- `shell_init` — 创建/重连 PTY，返回 `{ sessionKey, buffer, reconnected }`
- `shell_input` — 写入 PTY stdin
- `shell_resize` — 调整 PTY 终端大小
- `shell_detach` — 分离并启动 30 分钟清理定时器

**Tauri 事件**:
- `shell_output` — PTY stdout 输出，payload: `{ session_key, data }`
- `shell_auth_url` — OAuth URL 检测，payload: `{ session_key, url }`

**改动要点**:
- 移除 `PtySession` 中的 `ws_tx: Option<mpsc::UnboundedSender<String>>`
- Reader 任务通过 `AppHandle.emit()` 发送输出（从 `spawn_blocking` 中调用，AppHandle 是 sync-safe 的）
- Buffer replay 通过命令响应直接返回

### 3.3 File Watcher

| 源 | 目标 |
|---|---|
| `services/file_watcher.rs` (110 行) | `services/file_watcher.rs` (~105 行) |

**改动要点**:
- broadcast channel → `app_handle.emit("projects_updated", ...)`
- 监听 `.claude/.cursor/.codex/.gemini` 目录变化
- 使用 `notify_debouncer_mini` 做 300ms 防抖

**Tauri 事件**:
- `projects_updated` — payload: `{ changeType, changedFile, watchProvider, projects }`

---

## Phase 4: 清理与验证

**提交**: `5d9d27f chore: Phase 4 cleanup — remove unused imports, dependencies, and stale comments`

### 清理项

| 文件 | 清理内容 |
|---|---|
| `Cargo.toml` | 移除未使用的 `futures-util = "0.3"` 依赖 |
| `state.rs` | 移除未使用的 `mpsc` 导入；更新 WebSocket → shell/detach 注释 |
| `file_watcher.rs` | 移除无效的 `rescan_in_progress` 变量（顺序异步循环中无意义） |

### 验证结果

- `cargo check` — 通过 (0 errors)
- `cargo clippy` — 通过 (0 errors, 仅 dead_code 警告 — 前端尚未调用全部 API)

---

## 文件结构总览

```
src-tauri/src/
├── lib.rs                      # Tauri 入口，注册命令，初始化状态
├── config.rs                   # 应用配置 (database_path, workspaces_root)
├── error.rs                    # AppError 统一错误类型
├── state.rs                    # AppState, ActiveSession, PtySession
├── db/                         # 数据库层
│   ├── mod.rs
│   ├── api_keys.rs
│   ├── app_config.rs
│   ├── credentials.rs
│   ├── push_subscriptions.rs
│   ├── session_names.rs
│   └── users.rs
├── commands/                   # Tauri 命令 (对外接口)
│   ├── mod.rs
│   ├── api_keys.rs
│   ├── chat.rs                 # chat_execute (单一入口)
│   ├── commands_handler.rs
│   ├── mcp.rs
│   ├── projects.rs
│   ├── sessions.rs
│   ├── settings.rs
│   └── shell.rs                # shell_init/input/resize/detach
├── services/                   # 业务服务层
│   ├── mod.rs
│   ├── chat.rs                 # CLI spawners (Claude/Cursor/Codex/Gemini)
│   ├── file_watcher.rs         # 文件变更监听
│   ├── project_scanner.rs
│   ├── pty_manager.rs
│   └── sessions.rs
├── providers/                  # Provider 适配器
│   ├── mod.rs
│   ├── claude/
│   ├── codex/
│   ├── cursor/
│   └── gemini/
└── migrations/                 # SQLite 迁移脚本
```

## Tauri 事件汇总

| 事件名 | 来源 | 用途 |
|---|---|---|
| `chat_response` | `services/chat.rs` | 聊天流式输出 (text/complete/error/permission) |
| `shell_output` | `commands/shell.rs` | PTY stdout 数据 |
| `shell_auth_url` | `commands/shell.rs` | OAuth URL 检测 |
| `projects_updated` | `services/file_watcher.rs` | 项目文件变更通知 |
