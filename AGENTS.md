# AGENTS.md

## Project Overview

Multi AI coding assistant management desktop app (rccui). Supports Claude, Cursor, Codex, Gemini providers with session management, real-time chat, PTY terminal, and project monitoring.

- **Language**: Rust (edition 2024, nightly toolchain)
- **Frontend**: Leptos 0.8 with CSR mode, compiled to WASM
- **Backend**: Tauri 2 with SQLite (sqlx), PTY (portable-pty), file watching (notify)
- **Styling**: Tailwind CSS v4 with OKLCH theme variables
- **Components**: rust/ui (85+ UI components + 26 hooks + utils + constants)

## Project Structure

```
src/                                # Frontend (Leptos WASM)
├── main.rs                         # Entry point: mod declarations + mount_to_body
├── app.rs                          # Root App component (leptos_fluent! + Router + context providers)
├── ui/                             # rust/ui components (85+ components)
│   ├── mod.rs
│   ├── button.rs, input.rs, card.rs, dialog.rs, ...
│   └── toast_custom/               # Directory-based component
├── hooks/                          # rust/ui hooks (26 hooks)
├── features/                       # Feature modules (business UI)
│   ├── chat/                       # Chat feature
│   │   ├── chat_panel.rs           # Main chat panel (assembles input + messages + permission)
│   │   ├── chat_input.rs           # Message input textarea
│   │   ├── message_list.rs         # Scrollable message container with auto-expand logic
│   │   ├── message_item.rs         # Individual message rendering (text/thinking/tool_use/tool_result)
│   │   ├── permission_dialog.rs    # Tool execution permission dialog
│   │   └── quick_settings.rs       # Provider selector dropdown
│   ├── shell/                      # Shell feature
│   │   ├── shell_panel.rs          # Terminal panel wrapper
│   │   └── terminal.rs             # xterm.js integration
│   ├── files/                      # File browser (placeholder)
│   ├── git/                        # Git operations (placeholder)
│   ├── onboarding/                 # First-time setup
│   └── settings/                   # Settings (placeholder)
├── state/                          # Reactive state management
│   ├── app_state.rs                # AppContext: global signals (projects, sessions, sidebar, etc.)
│   ├── chat_state.rs               # ChatContext: messages, streaming, event handling
│   ├── session_state.rs            # SessionContext: session list, active session
│   └── shell_state.rs              # ShellContext: terminal sessions
├── layout/                         # Layout components
│   ├── app_layout.rs               # Root layout (SidenavContainer + SidenavInset + MainHeader)
│   ├── main_header.rs              # Top header with tab navigation
│   ├── editor_sidebar.rs           # Right-side editor sidebar
│   └── sidebar/                    # Left sidebar
│       ├── sidebar.rs              # Sidebar container
│       ├── project_list.rs         # Project list with navigation
│       ├── session_item.rs         # Session list item
│       └── sidebar_footer.rs       # Sidebar footer (theme toggle, settings)
├── pages/                          # Route pages
│   ├── dashboard.rs                # Dashboard (project overview)
│   ├── project.rs                  # Project detail (integrates ChatPanel + ShellPanel)
│   ├── onboarding.rs               # Onboarding wizard
│   └── settings.rs                 # Settings page (API keys, language, theme)
├── tauri/                          # Tauri bridge (frontend ↔ backend)
│   ├── commands.rs                 # invoke() wrappers for backend commands
│   ├── events.rs                   # listen() wrappers for backend events
│   └── types.rs                    # Shared TypeScript-like types (ChatResponse, etc.)
├── utils/                          # Utilities (date, country, phone_number, query)
└── constants/                      # Constants (pagination)

src-tauri/src/                      # Backend (Tauri)
├── lib.rs                          # Tauri entry: register commands, init state & file watcher
├── config.rs                       # AppConfig (database_path, workspaces_root)
├── error.rs                        # AppError unified error type
├── state.rs                        # AppState, ActiveSession, PtySession
├── db/                             # Database layer (SQLite)
│   ├── api_keys.rs                 # API key CRUD
│   ├── app_config.rs               # Key-value app config
│   ├── credentials.rs              # Provider credentials
│   ├── notifications.rs            # Notification records
│   ├── push_subscriptions.rs       # Push subscriptions
│   ├── session_names.rs            # Session naming
│   └── users.rs                    # User management
├── commands/                       # Tauri commands (external interface)
│   ├── chat.rs                     # chat_execute (single entry, dispatches ChatCommand)
│   ├── shell.rs                    # shell_init / shell_input / shell_resize / shell_detach
│   ├── commands_handler.rs         # Command execution handler
│   ├── git.rs                      # Git operations
│   ├── mcp.rs                      # MCP server management
│   ├── projects.rs                 # Project management
│   ├── sessions.rs                 # Session management
│   ├── settings.rs                 # Settings management
│   └── user.rs                     # User operations
├── services/                       # Business logic layer
│   ├── chat.rs                     # CLI spawners (Claude/Cursor/Codex/Gemini)
│   ├── file_watcher.rs             # Provider directory watcher → Tauri events
│   ├── project_scanner.rs          # Project scanning & caching
│   └── pty_manager.rs              # PTY lifecycle management
├── providers/                      # Provider adapters
│   ├── claude/                     # Claude SDK protocol (control_request/response)
│   ├── cursor/                     # Cursor adapter
│   ├── codex/                      # Codex adapter
│   ├── gemini/                     # Gemini adapter
│   ├── types.rs                    # Shared provider types
│   └── utils.rs                    # Provider utilities
└── migrations/                     # SQLite migration scripts

public/                             # Static assets + JS dependencies
styles.css                          # Tailwind entry (source of truth for theme)
index.html                          # Trunk entry
docs/dev/                           # Development plans
```

## Build & Run Commands

```bash
# Check compilation (fast feedback)
cargo check

# Frontend dev server (port 1420)
trunk serve

# Full desktop dev mode (runs trunk serve + tauri)
cargo tauri dev

# Production build
trunk build && cargo tauri build

# Compile Tailwind CSS manually
npx tailwindcss -i styles.css -o dist/tailwind.css
```

## Lint & Typecheck

```bash
# Rust type checking (use this after making changes)
cargo check

# Clippy lints
cargo clippy
```

## Component Usage

Components live at `crate::ui::*`, hooks at `crate::hooks::*`, utils at `crate::utils::*`.

```rust
// Import components
use crate::ui::button::Button;
use crate::ui::input::Input;
use crate::ui::card::{Card, CardHeader, CardTitle, CardContent, CardFooter};

// Use in view! macro
view! {
    <Card class="w-full max-w-md border-transparent shadow-none">
        <CardHeader>
            <CardTitle>"Title"</CardTitle>
            <CardDescription>"Description"</CardDescription>
        </CardHeader>
        <CardContent>
            <Input placeholder="Enter text..." bind_value=signal />
        </CardContent>
        <CardFooter>
            <Button attr:r#type="submit">"Submit"</Button>
        </CardFooter>
    </Card>
}
```

Key patterns:
- `bind_value` takes `RwSignal<String>` for two-way binding (requires `nightly` feature)
- Use `attr:r#type` for HTML type attribute (Rust keyword conflict)
- Components use `variants!` macro (from `leptos_ui`) for variant/size enums
- All styling uses Tailwind classes merged via `tw_merge`
- Use `border-transparent shadow-none` to remove card borders

## Adding New rust/ui Components

Option 1 - CLI (for individual components):
```bash
ui add <component_name> -y
```

Option 2 - Local copy (from rust/ui source repo):
```bash
cp /Users/lzmcoding/Code/ui/app_crates/registry/src/ui/<component>.rs src/ui/
# Add `pub mod <component>;` to src/ui/mod.rs
```

For hooks: copy from `app_crates/registry/src/hooks/` to `src/hooks/`.

## Tailwind CSS

- Entry file: `styles.css` (contains `@import "tailwindcss"`, theme variables, `@source` directive)
- `@source "./src/**/*.rs"` tells Tailwind v4 to scan Rust files for class names
- Compiled by Trunk pre_build hook: `npx tailwindcss -i styles.css -o dist/tailwind.css`
- `index.html` references `dist/tailwind.css` (compiled output, not raw source)
- Theme colors use OKLCH color space with CSS variables (`:root` for light, `.dark` for dark)

When adding new Tailwind classes in Rust code, they are automatically picked up on next build.

### Theme Variables

All colors are defined in `styles.css` as OKLCH CSS variables. This is the single source of truth for the design system.

**Available semantic color tokens** (usable as Tailwind classes like `bg-primary`, `text-success`, etc.):

| Token | Purpose | Example class |
|-------|---------|---------------|
| `background` / `foreground` | Page background & text | `bg-background text-foreground` |
| `card` / `card-foreground` | Card surfaces | `bg-card` |
| `primary` / `primary-foreground` | Primary actions | `bg-primary text-primary-foreground` |
| `secondary` / `secondary-foreground` | Secondary actions | `bg-secondary` |
| `muted` / `muted-foreground` | Subdued content | `text-muted-foreground` |
| `accent` / `accent-foreground` | Highlights | `bg-accent` |
| `destructive` | Danger/delete actions | `bg-destructive` |
| `success` / `success-foreground` | Success states | `bg-success text-success-foreground` |
| `warning` / `warning-foreground` | Warning states | `bg-warning text-warning-foreground` |
| `info` / `info-foreground` | Informational states | `bg-info text-info-foreground` |
| `border` / `input` / `ring` | Borders & focus rings | `border-border ring-ring` |
| `sidenav-*` | Sidebar navigation | `bg-sidenav text-sidenav-foreground` |
| `provider-claude` | Claude brand color | `text-provider-claude` |
| `provider-codex` | Codex brand color | `text-provider-codex` |
| `provider-gemini` | Gemini brand color | `text-provider-gemini` |

### Dark Mode

- Dark mode is driven by a `.dark` class on the `<html>` root element
- `ThemeMode` hook (`src/hooks/use_theme_mode.rs`) manages the signal, localStorage persistence, and DOM sync
- All theme variables have light (`:root`) and dark (`.dark`) values in `styles.css`
- Tailwind `dark:` prefix classes work automatically when `.dark` is present on root
- System preference is detected via `prefers-color-scheme` media query as fallback

### Style Guidelines

- **Always use theme variables** — never hardcode hex/rgb colors in Rust component code
- **Provider brand colors** use `text-provider-claude`, `text-provider-codex`, `text-provider-gemini` (not raw hex)
- **Scrollbar colors** are managed via `--scrollbar-thumb` / `--scrollbar-track` CSS variables
- **Hide scrollbars** using the `no__scrollbar` Tailwind utility (not inline styles)
- **Show scrollbar on hover** using `scrollbar__on_hover` utility with `group/scrollbar-on-hover` parent
- To add a new color token: define in `:root` + `.dark` in `styles.css`, map in `@theme inline`, then use as Tailwind class

## Backend Architecture

### Communication Pattern

Frontend calls backend via **Tauri Commands**, backend pushes real-time data via **Tauri Events**:

- Commands: `#[tauri::command]` functions registered in `lib.rs`
- Events: `app_handle.emit("event_name", payload)` from background tasks
- `AppHandle` is stored in `AppState` — it is `Send + Sync + Clone`, safe to use from `spawn_blocking`

### Tauri Command Pattern

All commands follow this wrapper pattern for error conversion:

```rust
#[tauri::command]
pub async fn some_command(
    state: tauri::State<'_, Arc<AppState>>,
    /* params */
) -> Result<Value, String> {
    async fn _fn(state: &AppState, /* params */) -> Result<Value, AppError> {
        // business logic
    }
    _fn(&state, /* params */).await.map_err(|e| e.to_string())
}
```

### Tauri Events

| Event | Source | Payload | Purpose |
|-------|--------|---------|---------|
| `chat_response` | `services/chat.rs` | `ChatResponse` | Streaming chat output (text/complete/error/permission_request) |
| `shell_output` | `commands/shell.rs` | `{ session_key, data }` | PTY stdout data |
| `shell_auth_url` | `commands/shell.rs` | `{ session_key, url }` | OAuth URL detection |
| `projects_updated` | `services/file_watcher.rs` | `{ changeType, changedFile, watchProvider, projects }` | Project file change notification |

### Key Backend Concepts

- **Chat**: Single `chat_execute` command accepts `ChatCommand` JSON enum, dispatches to provider-specific CLI spawners (Claude SDK protocol / Cursor / Codex / Gemini). Responses stream back as `chat_response` events.
- **Shell**: `shell_init` creates/reconnects PTY, returns `{ sessionKey, buffer, reconnected }`. Reader task emits `shell_output` events. `shell_detach` starts 30-min cleanup timer. Buffer holds up to `PTY_BUFFER_CAP` (5000) lines.
- **File Watcher**: Uses `notify`/`notify_debouncer_mini` (300ms debounce) to watch `~/.claude/.cursor/.codex/.gemini` directories. Emits `projects_updated` events on changes.
- **No Auth**: Desktop app uses `LOCAL_USER_ID = 1`, no authentication layer.

### Chat Streaming Architecture

Frontend listens to `chat_response` Tauri events via `ChatContext` (`src/state/chat_state.rs`). Each event has a `kind` field dispatching to different handlers:

| Event Kind | Description | Message Kind |
|------------|-------------|--------------|
| `stream_delta` | Incremental text chunk | Accumulated in `stream_content` signal |
| `thinking` | Model thinking/reasoning content | `"thinking"` |
| `tool_use` | Tool invocation (name + input JSON) | `"tool_use"` |
| `tool_result` | Tool execution result | `"tool_result"` |
| `permission_request` | Permission dialog for tool execution | Sets `pending_permission` signal |
| `complete` | Stream finished | Flushes remaining text |
| `error` | Error occurred | `"error"` |
| `session_created` | New session ID assigned | Updates `active_session_id` |

**Critical pattern — stream content flush**: Between `stream_delta` events, text accumulates in `stream_content: RwSignal<String>`. Before any structural event (thinking, tool_use, tool_result, permission_request, error, complete), the accumulated text MUST be flushed to a `NormalizedMessage` with kind `"text"`. This is handled by `flush_stream_content()` helper. Forgetting to flush causes text summaries to be lost between tool operations.

**tool_result JSON extraction**: Backend sends `tool_result` as `Option<serde_json::Value>` in the form `{"content": "...", "isError": false}`. The frontend must extract the `content` field (not serialize the whole object), matching the history adapter's behavior in `providers/claude/adapter.rs`.

**Collapsible auto-expand**: During streaming, `MessageList` computes the last collapsible message index and passes `auto_expand=true` to that `MessageItem`. When streaming ends, all collapsibles default to collapsed.

### Flexbox Overflow Prevention

In flex layouts, children default to `min-width: auto`, allowing content (especially `<pre>` blocks) to push containers wider than the viewport. The fix is adding `min-w-0` at every level of the flex chain:

- `app_layout.rs`: `SidenavInset class="min-w-0"` + `<main class="flex-1 min-w-0 overflow-hidden">`
- `project.rs`: Container `<div class="... min-w-0 overflow-hidden">`
- `chat_panel.rs`: `<div class="... min-w-0">`
- `message_list.rs`: Scroll container `min-w-0 overflow-x-hidden`
- `message_item.rs`: Message bubble `min-w-0 overflow-hidden`

## Key Dependencies

### Frontend (src/)

| Crate | Purpose |
|-------|---------|
| `leptos` 0.8 (csr, nightly) | Frontend framework |
| `leptos_router` 0.8 | Router for Leptos |
| `leptos_ui` 0.3.22 | Component macros (variants!, clx!, void!) |
| `tw_merge` 0.1.21 | Tailwind class merging |
| `icons` 0.18.2 | Lucide-style icon components |
| `strum` 0.26 (derive) | Enum string conversion |
| `web-sys` 0.3 | DOM API bindings (40+ features enabled) |
| `leptos-fluent` 0.3 (nightly) | i18n (Fluent-based, tr! macro) |
| `validator` | Form validation |

### Backend (src-tauri/)

| Crate | Purpose |
|-------|---------|
| `tauri` 2 | Desktop runtime + commands + events |
| `sqlx` 0.8 (sqlite) | SQLite database |
| `tokio` 1 (full) | Async runtime |
| `portable-pty` 0.9 | Pseudo-terminal for shell sessions |
| `notify` 7 / `notify-debouncer-mini` 0.5 | File system watching |
| `dashmap` 6 | Concurrent hash map (active sessions) |
| `chrono` 0.4 | Date/time handling |
| `regex` 1 | Pattern matching (stream parsing) |
| `tracing` 0.1 | Structured logging |
| `thiserror` 2 / `anyhow` 1 | Error handling |
| `async-trait` 0.1 | Async trait support |

## Applying DESIGN.md Design Specifications

Design specification files are stored in `docs/designmds/` (e.g. `opencode-DESIGN.md`, `cursor-DESIGN.md`, `xAi-DESIGN.md`). Each defines a complete visual language (colors, typography, radius, shadows, spacing). Below is the workflow and architecture knowledge needed to apply any DESIGN.md to this project.

### Theme Architecture (How Styling Propagates)

```
styles.css (:root / .dark CSS variables)
    ↓
styles.css (@theme inline → maps to Tailwind color/radius tokens)
    ↓
Tailwind CSS compilation (npx tailwindcss -i styles.css -o dist/tailwind.css)
    ↓
src/ui/*.rs components (use semantic classes: bg-primary, text-foreground, border-border, etc.)
    ↓
src/layout/*.rs + src/pages/*.rs (app-level code, also uses semantic classes)
```

**Key insight**: Changing CSS variables in `styles.css` automatically propagates to ~90% of the UI because rust/ui components use semantic Tailwind tokens (not hardcoded colors). Only layout/page files with explicit Tailwind utility classes (e.g. `rounded-lg`, `shadow-sm`) need manual adjustment.

### Files to Modify (Ordered by Impact)

| Priority | File | What to Change | Impact |
|----------|------|---------------|--------|
| 1 | `styles.css` | `:root` + `.dark` CSS variables, `@theme inline` overrides | ~90% visual change |
| 2 | `index.html` | Font loading (`<link>` tags in `<head>`, before Trunk CSS link) | Typography |
| 3 | `src/layout/*.rs` | Explicit utility classes (`rounded-*`, `shadow-*`, `backdrop-blur-*`) | Layout polish |
| 4 | `src/pages/*.rs` | Same as above | Page polish |
| 5 | `src/ui/toast_custom/_template_styles.rs` | Hardcoded `--leptoaster-font-family` and color values | Toast notifications |

### styles.css Variable Groups

When updating `styles.css`, replace values in these groups for **both** `:root` (light) and `.dark` (dark):

1. **Core surfaces**: `--background`, `--foreground`, `--card`, `--card-foreground`, `--popover`, `--popover-foreground`
2. **Interactive**: `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground`
3. **Muted/Accent**: `--muted`, `--muted-foreground`, `--accent`, `--accent-foreground`
4. **Semantic**: `--destructive`, `--success`, `--warning`, `--info` (each with `-foreground`)
5. **Borders**: `--border`, `--input`, `--ring`
6. **Sidenav**: `--sidenav`, `--sidenav-foreground`, `--sidenav-primary`, `--sidenav-accent`, `--sidenav-border`, `--sidenav-ring` (each with `-foreground`)
7. **Provider brands**: `--provider-claude`, `--provider-codex`, `--provider-gemini`
8. **Charts**: `--chart-1` through `--chart-5`, `--sidebar-primary`
9. **Scrollbar**: `--scrollbar-thumb`, `--scrollbar-track`
10. **Radius**: `--radius` (single value, controls all via formulas in `@theme inline`)

### Radius Control

`--radius` is the single base value. The `@theme inline` block calculates derived values:

```
--radius-sm = --radius - 4px   → rounded-sm
--radius-md = --radius - 2px   → rounded-md  (most components use this)
--radius-lg = --radius          → rounded-lg  (inputs, containers)
--radius-xl = --radius + 4px   → rounded-xl
```

To target a specific `rounded-md` value: set `--radius` = desired + 2px. Example: want `rounded-md` = 4px → set `--radius: 0.375rem` (6px).

### Shadow Control via @theme inline

Add these to `@theme inline` to globally disable specific shadow levels:

```css
--shadow-xs: 0 0 #0000;   /* neutralizes shadow-xs (inputs, small elements) */
--shadow-sm: 0 0 #0000;   /* neutralizes shadow-sm (cards, buttons, sidenav, tabs) */
--shadow: 0 0 #0000;       /* neutralizes shadow (badges) */
/* shadow-md through shadow-2xl are preserved for floating elements (dialogs, dropdowns, tooltips) */
```

### Font Override via @theme inline

```css
--font-sans: 'Font Name', fallback1, fallback2, monospace;
--font-mono: 'Font Name', fallback1, fallback2, monospace;
```

Also add explicit `font-family` in the `@layer base` body rule for full coverage.

### Component Shadow/Radius Audit

Components in `src/ui/` that have embedded shadow/radius classes (cannot be changed via CSS variables alone):

| Component | Shadow | Radius | Notes |
|-----------|--------|--------|-------|
| `button.rs` | `shadow-xs` | `rounded-md` | Controllable via `@theme inline` |
| `card.rs` | `shadow-sm` | `rounded-xl` | Shadow controllable; radius stays at `--radius + 4px` |
| `input.rs` | `shadow-xs` | `rounded-md` | Both controllable |
| `sidenav.rs` | `shadow-sm` | `rounded-lg/md` | Shadow controllable |
| `tabs.rs` | `shadow-sm` | `rounded-lg/md` | Shadow controllable |
| `dialog.rs` | `shadow-lg` | `rounded-2xl` | Floating element, usually keep shadow |
| `tooltip.rs` | `shadow-lg` | none | Floating element |
| `badge.rs` | `shadow` | `rounded-md` | Both controllable |

### Layout Files with Hardcoded Utility Classes

These files in `src/layout/` and `src/pages/` contain explicit Tailwind classes that may need manual adjustment per design spec:

- `src/layout/main_header.rs`: Tab container (`rounded-lg`), active tab (`shadow-sm`)
- `src/layout/sidebar/session_item.rs`: Hover actions (`backdrop-blur-sm`)
- `src/layout/sidebar/project_list.rs`: Search input (long inline class string)
- `src/pages/dashboard.rs`: Provider cards (`rounded-lg`), empty state icon (`rounded-lg`)

### Toast System (_template_styles.rs)

`src/ui/toast_custom/_template_styles.rs` contains a hardcoded CSS string (not Tailwind). It has its own `:root` and `.dark` blocks with:
- `--leptoaster-font-family` (default: `Arial`)
- `--leptoaster-{success,warn,error}-background-color` and `-border-color` (OKLCH values)
- `--leptoaster-info-*` colors reference `var(--background)` / `var(--foreground)` (auto-follows theme)

Update font and success/warn/error colors to match the design spec.

### Verification Checklist

After applying a DESIGN.md:
1. `cargo check` — Rust compilation (catches syntax errors in string edits)
2. `npx tailwindcss -i styles.css -o dist/tailwind.css` — CSS compilation (catches invalid variable syntax)
3. `cargo tauri dev` — Visual verification in both light and dark modes

### Lessons Learned

- **Do NOT modify `src/ui/*.rs` component files** unless absolutely necessary. They are from a shared library. Use CSS variable overrides and `@theme inline` to control their appearance.
- **Color conversion**: DESIGN.md specs typically use hex colors. Convert to OKLCH for `styles.css`. Include the warm/cool hue in neutral colors (e.g. hue ~25 for warm neutrals, hue ~260 for cool). Pure `0` hue = achromatic gray.
- **Light + dark mode**: If a DESIGN.md is dark-only, create a complementary light mode by inverting luminosity values while keeping the same hue family.
- **Always preview the result** with `cargo tauri dev` before considering the work done. CSS variable changes can have unexpected cascading effects on component contrast and readability.

## Important Notes

- This project uses Rust **nightly** toolchain (configured in `rust-toolchain.toml`)
- Rust edition is **2024** (required for `let chains` syntax used in components)
- The `src/ui/` and `src/hooks/` modules are at the crate root level (components reference `crate::ui::` paths)
- Do NOT move components into a `components/` subdirectory - it will break internal `crate::ui::` imports
- JS dependency files in `public/app/` and `public/hooks/` are required by some components (drawer, OTP, action_bar, shimmer, etc.)
- `ui_config.toml` uses relative path: `base_path_components = "src/components"` (no leading slash)
- Backend commands are registered in `src-tauri/src/lib.rs` via `invoke_handler(tauri::generate_handler![...])`
- When adding new commands, remember to both create the function and register it in `lib.rs`

## Leptos 0.8 CSR Pitfalls & Solutions

### spawn_local 中不要调用 expect_context / expect_toaster

**问题**: 在 `spawn_local` 的 async 块中调用 `expect_context::<T>()` 或 `expect_toaster()`（内部也是 `expect_context`）可能因为缺失 reactive owner 而 panic，导致整个 async 块中止，后续代码（如信号更新）不会执行。

**症状**: 后端操作成功（数据已保存），但 UI 不刷新，toast 也不显示。

**正确做法**: 在 `spawn_local` 外部（组件作用域或事件处理闭包的同步部分）捕获 context，然后通过 clone 传入 async 块：

```rust
// 在组件作用域中捕获（有效的 reactive 上下文）
let ctx = expect_context::<AppContext>();   // Copy，可直接捕获
let toaster = expect_toaster();            // Clone，需要显式 clone

// 事件处理中
let toaster = toaster.clone();  // clone 给 spawn_local
spawn_local(async move {
    // 这里直接使用 ctx (Copy) 和 toaster (已 clone 进来)
    if let Err(e) = some_command().await {
        toaster.error(format!("Failed: {e}"));
    } else {
        ctx.some_signal.set(new_value);
        toaster.success("Done");
    }
});
```

**注意**: `AppContext` 是 `#[derive(Clone, Copy)]`（内部全是 `RwSignal`，本身 `Copy`），无需 clone。`ToasterContext` 包含 `Arc<Mutex<...>>`，是 `Clone` 但不是 `Copy`，需要显式 clone。

### 信号更新触发 UI 刷新：set() vs update()

- `ctx.projects.set(new_vec)` — 替换整个值，始终触发通知，推荐用于需要确保 UI 刷新的场景
- `ctx.projects.update(|v| { ... })` — 原地修改，也会触发通知，但在某些边界情况下可能不如 `set()` 可靠
- **推荐模式**（clone-modify-set）：
  ```rust
  let mut projects = ctx.projects.get_untracked();
  // 修改 projects...
  ctx.projects.set(projects);  // 显式 set 触发所有订阅者
  ```

### collect_view() 列表渲染

`collect_view()` 不带 key，父级响应式闭包重新执行时会**整体替换** DOM 子树（不是 diff）。对于信号驱动的列表刷新这是可行的，但要确保信号确实被正确更新（参见上面的 spawn_local 陷阱）。如果需要高性能的增量更新，使用 `<For each=... key=...>` 组件。

## CSS z-index 与 Stacking Context

### Sidebar 弹出框被页面内容遮挡

**问题**: Sidebar（`SidenavContainer`）使用 `fixed z-10` 创建了 stacking context，内部弹出框的 `z-50` 受限于父级的 `z-10`。如果页面内容区也有 `relative z-10`，由于 DOM 顺序（`SidenavInset` 在 `Sidenav` 之后），页面内容会覆盖 sidebar 的弹出框。

**解决方案**: 移除页面内容区不必要的 `z-10`（如 `src/pages/dashboard.rs` 中的内容 section），依靠 DOM 源顺序保证层叠关系。不要给 sidebar 内的弹出框使用 `<Portal>`（会导致 `FnOnce` vs `Fn` 编译错误，因为 Portal children 要求 `Fn`）。

## Non-Copy 类型在 view! 多闭包中的传递

`view!` 宏中多个 `{move || { ... }}` 闭包会各自捕获变量。对于 `Clone` 但非 `Copy` 的类型（如 `ToasterContext`、`String`），需要在 `view!` 之前为每个闭包准备独立的 clone：

```rust
let toaster = expect_toaster();
let toaster_for_dialog_a = toaster.clone();
let toaster_for_dialog_b = toaster;  // 最后一个可以直接 move

view! {
    {move || { /* 使用 toaster_for_dialog_a */ }}
    {move || { /* 使用 toaster_for_dialog_b */ }}
}
```

在嵌套闭包链中（`move || { .map(|x| { on:click={ move |_| { spawn_local(async move { }) } } }) }`），每一层 move 都需要一次 clone。

## 国际化 (i18n) — leptos-fluent

项目使用 `leptos-fluent 0.3` 实现中英文国际化，基于 Mozilla Fluent `.ftl` 翻译文件。

### 架构概览

```
locales/
├── zh-CN/main.ftl          # 中文翻译（默认语言）
└── en/main.ftl              # 英文翻译

src/app.rs                   # leptos_fluent! 宏初始化（必须在其他 context provider 之前）
src/pages/settings.rs        # 语言切换器 UI
```

### 初始化

`leptos_fluent!` 宏必须在 `App` 组件中**最先调用**（在 `ThemeMode::init` 等其他 context 之前），因为它创建 `I18n` context：

```rust
use leptos_fluent::leptos_fluent;

leptos_fluent! {
    locales: "./locales",
    default_language: "zh-CN",
    set_language_to_local_storage: true,
    initial_language_from_local_storage: true,
    local_storage_key: "lang",
    sync_html_tag_lang: true,
};
```

### tr! 宏的关键限制

**`tr!` 宏只接受字符串字面量作为 key，不接受变量或表达式。**

```rust
// 错误 — 编译失败
let key = "tab-chat";
tr!(key)

// 错误 — 编译失败
tr!(tab.label_key())

// 正确 — 使用 match + 字面量
match tab {
    AppTab::Chat => tr!("tab-chat"),
    AppTab::Shell => tr!("tab-shell"),
    AppTab::Files => tr!("tab-files"),
    AppTab::Git => tr!("tab-git"),
}
```

带参数的翻译同理，key 必须是字面量：

```rust
// .ftl 文件: time-minutes = { $value }分钟前
tr!("time-minutes", { "value" => minutes_count })
```

### spawn_local 中使用 tr!

`tr!()` 内部调用 `expect_context::<I18n>()`，在 `spawn_local` 的 async 块中可能 panic。必须在 async 块外预捕获翻译字符串：

```rust
// 在同步上下文中预捕获
let msg_success = tr!("toast-session-deleted");
let msg_error = tr!("toast-delete-failed");

spawn_local(async move {
    match delete_session(id).await {
        Ok(_) => toaster.success(msg_success),
        Err(e) => toaster.error(msg_error),
    }
});
```

### 原生 HTML 属性不用 attr: 前缀

Leptos `view!` 宏中，原生 HTML 元素的原生属性（`placeholder`、`title` 等）直接写，不加 `attr:` 前缀。`attr:` 仅用于自定义属性或 Leptos 组件的属性透传：

```rust
// 错误 — 编译失败
<input attr:placeholder={move || tr!("search-placeholder")} />
<button attr:title={move || tr!("tooltip-text")} />

// 正确
<input placeholder={move || tr!("search-placeholder")} />
<button title={move || tr!("tooltip-text")} />
```

### 获取 I18n context

`leptos_fluent` 没有 `expect_i18n` 函数，直接使用 Leptos 的 `expect_context`：

```rust
let i18n = expect_context::<leptos_fluent::I18n>();

// 获取可用语言列表
for lang in i18n.languages {
    // lang.name — 原生语言名（如 "中文（简体）"、"English"）
    // lang.id — 语言标识符（如 "zh-CN"、"en"）
}

// 切换语言
i18n.language.set(target_lang);
```

### 添加新翻译 key 的步骤

1. 在 `locales/zh-CN/main.ftl` 和 `locales/en/main.ftl` 中同时添加 key
2. 在 Rust 代码中使用 `tr!("new-key")` 或带参数的 `tr!("new-key", { "param" => value })`
3. `cargo check` 验证编译

### Fluent .ftl 语法要点

```ftl
# 简单文本
sidebar-projects = 项目

# 带参数（参数名用 $）
time-minutes = { $value }分钟前
dashboard-projects-found = 发现 { $count } 个项目

# 多行文本用缩进
long-text =
    这是第一行
    这是第二行
```
