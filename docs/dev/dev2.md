# rcui -> rccui 前端移植计划

## Context

rccui 是一个多 AI 编程助手管理桌面应用（Tauri 2），后端已完整实现（53 个 Tauri 命令、4 种事件、SQLite、PTY、文件监听），但前端目前只有一个演示页面 `app.rs`，没有路由和功能页面。需要将 rcui（React/TypeScript Web 应用）的核心前端功能移植到 rccui 的 Leptos 0.8 + WASM 技术栈中，适配桌面环境。

**移植范围**: 核心功能（Sidebar + Chat + Shell + FileTree + CodeEditor + GitPanel + Settings + Onboarding）
**排除**: JWT 认证、Web Push、PWA、Task Master、Plugin 系统、i18n

## 模块结构

在 `src/` 下新增以下模块目录：

```
src/
├── main.rs                   # 入口（添加新 mod 声明）
├── app.rs                    # 重写：Router + 全局 providers + Layout
├── ui/                       # 85 个 UI 原子组件（不动）
├── hooks/                    # 26 个 hooks（不动）
├── utils/                    # 工具函数（不动，按需新增）
├── constants/                # 常量（不动，按需新增）
│
├── tauri/                    # 新建 — Tauri 交互层
│   ├── mod.rs
│   ├── commands.rs           # invoke() 类型封装（53 个命令）
│   ├── events.rs             # listen()/unlisten() 事件监听封装
│   └── types.rs              # 与后端 JSON 对应的 serde 类型
│
├── state/                    # 新建 — 全局状态管理
│   ├── mod.rs
│   ├── app_state.rs          # AppContext（项目、侧边栏、主题）
│   ├── chat_state.rs         # ChatContext（消息、流式、权限）
│   ├── session_state.rs      # SessionContext（会话列表）
│   └── shell_state.rs        # ShellContext（终端状态）
│
├── layout/                   # 新建 — 布局骨架
│   ├── mod.rs
│   ├── app_layout.rs         # SidenavWrapper + 主内容 + 编辑器侧边栏
│   ├── sidebar/              # 侧边栏（项目列表 + 会话列表）
│   │   ├── mod.rs
│   │   ├── sidebar.rs        # 侧边栏主组件
│   │   ├── project_list.rs   # 项目列表（展开/折叠）
│   │   ├── session_item.rs   # 会话条目（Provider图标、标题、时间）
│   │   └── sidebar_footer.rs # 底部（设置、主题切换）
│   ├── main_header.rs        # 项目名 + Tab 切换栏
│   └── editor_sidebar.rs     # 右侧编辑器面板（可折叠）
│
├── pages/                    # 新建 — 路由页面
│   ├── mod.rs
│   ├── dashboard.rs          # 首页/项目选择
│   ├── project.rs            # 项目视图（含 Tab 内容区）
│   ├── settings.rs           # 设置页面外壳
│   └── onboarding.rs         # 引导页面
│
└── features/                 # 新建 — 功能模块
    ├── mod.rs
    ├── chat/                 # 聊天功能
    │   ├── mod.rs
    │   ├── chat_panel.rs     # 聊天面板（消息列表 + 输入框）
    │   ├── message_list.rs   # 可滚动消息容器
    │   ├── message_item.rs   # 单条消息渲染（文本/工具/思考/错误）
    │   ├── chat_input.rs     # 输入栏（provider/model 选择、发送）
    │   ├── permission_dialog.rs  # 工具权限请求对话框
    │   └── quick_settings.rs # 模型/权限模式弹出面板
    │
    ├── shell/                # 终端功能
    │   ├── mod.rs
    │   └── terminal.rs       # 终端组件（Tauri shell 命令交互 + ANSI 渲染）
    │
    ├── files/                # 文件功能
    │   ├── mod.rs
    │   ├── file_tree.rs      # 层级文件浏览器
    │   ├── file_editor.rs    # CodeMirror 6 封装（JS interop）
    │   └── image_preview.rs  # 图片预览
    │
    ├── git/                  # Git 功能
    │   ├── mod.rs
    │   ├── git_panel.rs      # Git 面板（Changes/History/Branches）
    │   ├── changes_tab.rs    # 变更列表 + diff 查看
    │   ├── history_tab.rs    # 提交历史
    │   ├── branches_tab.rs   # 分支管理
    │   └── diff_viewer.rs    # Diff 渲染器
    │
    ├── settings/             # 设置功能
    │   ├── mod.rs
    │   ├── agents_settings.rs    # MCP 服务器管理
    │   ├── api_keys_settings.rs  # API Key 管理
    │   ├── credentials_settings.rs # 凭证管理
    │   ├── git_settings.rs       # Git 配置
    │   └── appearance_settings.rs # 外观（主题切换）
    │
    └── onboarding/           # 引导功能
        ├── mod.rs
        └── onboarding_wizard.rs  # Provider 检查 + Git 配置步骤
```

## 路由设计

使用 `leptos_router` 0.8，路由结构：

| 路由 | 组件 | 说明 |
|------|------|------|
| `/` | Dashboard | 项目选择页（或重定向到上次项目） |
| `/onboarding` | Onboarding | 首次启动引导 |
| `/project/:name` | ProjectView | 项目视图（默认 Chat tab） |
| `/project/:name/:tab` | ProjectView | 深链到具体 Tab |
| `/settings` | Settings | 设置页 |
| `/settings/:section` | Settings | 设置子页 |

`AppLayout` 在路由外层，侧边栏在所有页面间持久化，只有主内容区随路由切换。

## 状态架构

用 Leptos `provide_context`/`use_context` 替代 React Context：

```
App（provide AppContext）
  └── AppLayout
       ├── Sidebar（读 AppContext.projects, SessionContext）
       └── Main Content
            └── ProjectView（provide ChatContext, SessionContext）
                 ├── ChatPanel（读写 ChatContext）
                 ├── ShellPanel（provide ShellContext）
                 ├── FilesPanel
                 └── GitPanel
```

**AppContext**: `projects`, `selected_project`, `sidebar_open`, `theme`, `onboarding_complete`
**ChatContext**: `messages`, `active_session_id`, `is_streaming`, `provider`, `model`, `permission_mode`, `pending_permission`
**SessionContext**: `sessions`, `selected_session`, `total`, `has_more`
**ShellContext**: `session_key`, `is_connected`

## Tauri 交互层

### 命令调用 (`tauri/commands.rs`)
复用已有的 `wasm_bindgen` invoke 模式，封装为类型安全的 async 函数：
```rust
pub async fn list_projects(refresh: bool) -> Result<Vec<Project>, String> {
    let args = serde_wasm_bindgen::to_value(&ListProjectsArgs { refresh }).unwrap();
    let result = invoke("list_projects", args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}
```

### 事件监听 (`tauri/events.rs`)
绑定 `window.__TAURI__.event.listen`，返回 unlisten handle：
- `listen_chat_response(callback)` → 监听 `"chat_response"`
- `listen_shell_output(callback)` → 监听 `"shell_output"`
- `listen_shell_auth_url(callback)` → 监听 `"shell_auth_url"`
- `listen_projects_updated(callback)` → 监听 `"projects_updated"`

全局事件（`projects_updated`）在 App 层设置；作用域事件（`chat_response`）在功能组件中按 `session_id` 过滤。

## 迭代计划

### Phase 0: 基础框架（路由 + 布局 + Tauri 交互层） ✅ 已完成

**目标**: 应用能启动，路由工作，三栏布局骨架显示，Tauri 命令可调用。

**创建/修改文件**:
- `src/main.rs` — 添加 `mod tauri; mod state; mod layout; mod pages; mod features;`
- `src/app.rs` — 重写：Router + AppContext provider + AppLayout
- `src/tauri/mod.rs`, `commands.rs`, `events.rs`, `types.rs` — 交互层骨架（先实现 `list_projects`, `get_onboarding_status`）
- `src/state/mod.rs`, `app_state.rs` — AppContext（projects, selected_project, sidebar_open, theme）
- `src/layout/mod.rs`, `app_layout.rs` — 使用 `crate::ui::sidenav::*` 构建三栏布局
- `src/layout/sidebar/mod.rs`, `sidebar.rs` — 侧边栏骨架
- `src/layout/main_header.rs` — Tab 切换栏骨架
- `src/pages/mod.rs`, `dashboard.rs`, `project.rs` — 页面占位

**使用的 UI 组件**: `SidenavWrapper`, `Sidenav`, `SidenavHeader`, `SidenavContent`, `SidenavFooter`, `SidenavMenuButton`, `SidenavTrigger`, `Button`, `Input`, `Tabs`

**验证**: `cargo check` 通过 → `cargo tauri dev` 显示三栏布局 → 点击路由切换正常

---

### Phase 1: 侧边栏 + 项目列表 + 会话管理

**目标**: 侧边栏完整功能——项目发现、会话列表、搜索、分页、展开/折叠。

**创建/修改文件**:
- `src/state/session_state.rs` — SessionContext
- `src/layout/sidebar/project_list.rs` — 项目列表（展开/折叠子会话）
- `src/layout/sidebar/session_item.rs` — 会话条目（Provider 图标、标题、时间戳）
- `src/layout/sidebar/sidebar_footer.rs` — 底部（折叠按钮、设置入口、主题切换）
- `src/tauri/commands.rs` — 添加 `list_sessions`, `rename_session`, `delete_session`, `set_session_name`
- `src/tauri/events.rs` — 设置 `projects_updated` 监听，自动刷新项目列表
- `src/pages/project.rs` — 完善项目视图，Tab 切换

**使用的 UI 组件**: `ScrollArea`, `Skeleton`, `Badge`, `ContextMenu`, `Tooltip`, `Input`（搜索）, `Separator`

**验证**: 项目列表加载 → 展开显示会话 → 搜索过滤 → 右键菜单操作 → 文件监听自动更新

---

### Phase 2: 聊天功能

**目标**: 完整的实时流式聊天，支持 4 个 Provider，工具渲染，权限处理。

**创建/修改文件**:
- `src/state/chat_state.rs` — ChatContext
- `src/features/chat/` — 全部 7 个文件
- `src/tauri/commands.rs` — 添加 `chat_execute`, `get_session_messages`, `get_session_token_usage`
- `src/tauri/events.rs` — 设置 `chat_response` 监听
- `src/tauri/types.rs` — 添加 `ChatCommand`, `ChatResponse`, `NormalizedMessage` 等类型

**数据流**:
1. 用户输入 → `chat_execute(ChatCommand)` Tauri 命令
2. 后端 spawn CLI → 流式解析 → emit `chat_response` 事件
3. 前端监听 → 按 `session_id` 过滤 → 更新 `ChatContext.messages` signal
4. `message_list.rs` 响应式渲染新消息
5. 权限请求 → 显示 `permission_dialog.rs` → 用户操作 → `ClaudePermissionResponse`

**消息渲染**:
- 文本消息：保留空白格式，后续可接入 Markdown 渲染（pulldown-cmark）
- 工具调用：可折叠 Card，显示工具名 + JSON 输入
- 工具结果：可折叠 Card，显示输出
- 思考块：折叠的 "thinking" 块
- 错误：destructive Alert 组件
- 流式增量：追加到当前消息内容

**使用的 UI 组件**: `Card`, `Button`, `Textarea`, `ScrollArea`, `Dialog`, `DropdownMenu`, `Collapsible`, `Alert`, `Spinner`, `Select`, `Popover`, `Badge`

**验证**: 发送消息 → 流式响应显示 → 工具调用显示权限对话框 → 切换 Provider → 加载历史会话

---

### Phase 3: 终端功能

**目标**: 使用 Tauri shell 命令实现交互式终端，ANSI 渲染。

**创建/修改文件**:
- `src/features/shell/mod.rs`, `terminal.rs`
- `src/state/shell_state.rs` — ShellContext
- `src/tauri/commands.rs` — 添加 `shell_init`, `shell_input`, `shell_resize`, `shell_detach`
- `src/tauri/events.rs` — 设置 `shell_output`, `shell_auth_url` 监听
- `public/app/xterm.bundle.js` — xterm.js 打包（用于 ANSI 终端渲染）
- `index.html` — 添加 xterm.js 的 `<script>` 和 CSS 引用

**实现方式**:
后端 PTY 已完整实现，前端通过 Tauri 命令交互（非 Tauri shell 插件）：
1. 组件渲染 `<div>` 容器
2. mount 时通过 JS interop 初始化 xterm.js Terminal
3. 调用 `shell_init` 获取/创建 PTY 会话
4. `shell_output` 事件 → `terminal.write(data)` 渲染 ANSI
5. xterm `onData` → `shell_input` 发送用户输入
6. resize → `terminal.fit()` + `shell_resize` 命令
7. 离开时 → `shell_detach`（30分钟超时清理）
8. 重连时 → 重放 buffer 恢复终端状态

**使用的 UI 组件**: `Card`（连接状态覆盖层）, `Button`（重连）, `Spinner`（连接中）

**验证**: Shell Tab → 终端显示 shell prompt → 输入命令有输出 → 切换 Tab 再切回 → 重连恢复

---

### Phase 4: 文件功能

**目标**: 文件树浏览器 + CodeMirror 6 代码编辑器 + 图片预览。

**创建/修改文件**:
- `src/features/files/` — 全部 3 个文件
- `src/layout/editor_sidebar.rs` — 右侧编辑器面板
- `src/tauri/commands.rs` — 添加 `list_files`, `read_file`, `save_file`, `create_file`, `rename_file`, `delete_file`, `upload_files`, `read_file_content`, `read_raw_file`
- `public/app/codemirror.bundle.js` — CodeMirror 6 打包
- `index.html` — 添加 CodeMirror CSS/JS 引用

**文件树**: 递归组件，使用 `Collapsible` 展开/折叠目录，`ContextMenu` 右键操作
**编辑器**: JS interop 初始化 CodeMirror，语法高亮自动检测，保存调用 `save_file` 命令
**图片预览**: `read_file_content` 返回 base64 → `<img>` 渲染

**使用的 UI 组件**: `Collapsible`, `ContextMenu`, `Dialog`, `Input`, `ScrollArea`, `Image`

**验证**: Files Tab → 文件树显示 → 点击文件 → 编辑器打开 → 编辑保存 → 图片预览

---

### Phase 5: Git 功能

**目标**: Git 面板，Changes/History/Branches 三个子 Tab。

**创建/修改文件**:
- `src/features/git/` — 全部 6 个文件
- `src/tauri/commands.rs` — 添加全部 18 个 `git_*` 命令

**子 Tab**:
- **Changes**: `git_status` → 文件列表 + `git_diff` 点击查看 → `git_commit` 提交
- **History**: `git_commits` → 提交列表 + `git_commit_diff` 展开 diff
- **Branches**: `git_branches` → 列表 + 切换/创建/删除 + `git_fetch`/`git_pull`/`git_push`

**Diff 渲染**: 解析 unified diff → 行号 + 颜色编码（红删绿增），后续可用 CodeMirror diff 扩展

**使用的 UI 组件**: `Tabs`, `Button`, `Input`, `Badge`, `Alert`, `ScrollArea`, `Separator`, `Dialog`

**验证**: Git Tab → 状态加载 → 点击文件看 diff → 提交 → 分支操作

---

### Phase 6: 设置 + 引导

**目标**: 设置页面全部子模块 + 首次启动引导向导。

**创建/修改文件**:
- `src/features/settings/` — 全部 5 个文件
- `src/features/onboarding/` — 全部 2 个文件
- `src/pages/settings.rs`, `src/pages/onboarding.rs` — 页面路由
- `src/tauri/commands.rs` — 添加 settings/MCP/user 相关命令

**设置子页**:
- **Agents (MCP)**: `mcp_cli_list/add/remove` + `cursor_mcp_list/add/remove` + `mcp_all_servers`
- **API Keys**: `list_api_keys/create_api_key/toggle_api_key/delete_api_key`
- **Credentials**: `list_credentials/create_credential/toggle_credential/delete_credential`
- **Git**: `get_git_config/update_git_config`
- **Appearance**: `crate::ui::theme_toggle` 组件 + 本地存储

**引导向导**:
1. 检查 Provider CLI 安装（claude/cursor/codex/gemini 是否可用）
2. Git 配置（name + email）
3. 完成 → `complete_onboarding` → 重定向到 Dashboard

**使用的 UI 组件**: `Tabs`, `Form`, `Input`, `Button`, `Switch`, `AlertDialog`, `ThemeToggle`, `Card`, `Label`

**验证**: `/settings` → 各子页功能正常 → 首次启动引导 → 完成后进入主页

## 各阶段使用的后端命令和事件

| Phase | 使用的 Tauri 命令 | 监听的事件 |
|-------|-------------------|-----------|
| 0 | `list_projects`, `get_onboarding_status` | — |
| 1 | `list_sessions`, `rename_session`, `delete_session`, `set_session_name` | `projects_updated` |
| 2 | `chat_execute`, `get_session_messages`, `get_session_token_usage` | `chat_response` |
| 3 | `shell_init`, `shell_input`, `shell_resize`, `shell_detach` | `shell_output`, `shell_auth_url` |
| 4 | `list_files`, `read_file`, `save_file`, `create_file`, `rename_file`, `delete_file`, `upload_files`, `read_file_content`, `read_raw_file` | — |
| 5 | 全部 18 个 `git_*` 命令 | — |
| 6 | `mcp_*` (9), `list_api_keys/create/delete/toggle`, `list_credentials/create/delete/toggle`, `get_git_config/update_git_config`, `complete_onboarding/get_onboarding_status` | — |

## 关键技术决策

1. **终端渲染**: 后端 PTY 已完整实现，前端通过 Tauri 命令（shell_init/input/resize/detach）+ 事件（shell_output）交互。ANSI 渲染使用 xterm.js via JS interop（`public/app/xterm.bundle.js`）。
2. **代码编辑器**: CodeMirror 6 via JS interop（`public/app/codemirror.bundle.js`），语法高亮、搜索、保存功能。
3. **Markdown 渲染**: Phase 2 先用纯文本，后续可引入 `pulldown-cmark` crate 或 JS interop。
4. **无认证**: 桌面应用，`LOCAL_USER_ID = 1`，无 JWT。
5. **无 WebSocket**: 所有实时通信通过 Tauri Events（`listen`/`emit`），替代 rcui 的 WebSocket。
6. **JS 依赖**: xterm.js 和 CodeMirror 6 的 bundle 放在 `public/app/` 下，通过 `<script>` 加载（与现有 drawer/resizable 等 JS 依赖模式一致）。

## 验证方式

每个 Phase 完成后：
1. `cargo check` — 编译通过
2. `cargo tauri dev` — 启动应用，视觉验证功能
3. 功能测试：按该 Phase 的验证项逐一测试
4. 边界测试：空项目列表、100+ 会话分页、大文件、无 Git 仓库等
