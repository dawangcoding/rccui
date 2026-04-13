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
├── app.rs                          # Root App component
├── ui/                             # rust/ui components (85+ components)
│   ├── mod.rs
│   ├── button.rs, input.rs, card.rs, dialog.rs, ...
│   └── toast_custom/               # Directory-based component
├── hooks/                          # rust/ui hooks (26 hooks)
├── utils/                          # Utilities (date, country, query)
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

## Important Notes

- This project uses Rust **nightly** toolchain (configured in `rust-toolchain.toml`)
- Rust edition is **2024** (required for `let chains` syntax used in components)
- The `src/ui/` and `src/hooks/` modules are at the crate root level (components reference `crate::ui::` paths)
- Do NOT move components into a `components/` subdirectory - it will break internal `crate::ui::` imports
- JS dependency files in `public/app/` and `public/hooks/` are required by some components (drawer, OTP, action_bar, shimmer, etc.)
- `ui_config.toml` uses relative path: `base_path_components = "src/components"` (no leading slash)
- Backend commands are registered in `src-tauri/src/lib.rs` via `invoke_handler(tauri::generate_handler![...])`
- When adding new commands, remember to both create the function and register it in `lib.rs`
