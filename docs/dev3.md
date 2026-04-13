# Frontend UI/UX Optimization Plan

## Context

rccui 是一个 Multi AI Coding Assistant 管理桌面应用，当前前端存在以下问题需要优化：

1. **styles.css 变量问题** -- `--primary`/`--secondary` 在 `:root` 和 `.dark` 中各重复定义两次（第一次是死代码）；缺少 `--destructive-foreground`；`--accent`/`--muted`/`--secondary` 区分度不足；暗色模式 card 与 background 对比过低
2. **侧边栏 UX** -- 搜索框无图标；session 文本 10px 低于无障碍标准；删除无确认；hover 操作按钮缺乏背景衬底
3. **Dashboard 页面** -- 仅有居中文字，无视觉层次和交互入口
4. **Header 标签页** -- 纯文本无图标；未实现的标签无视觉提示；点击目标偏小
5. **缺失 UX 模式** -- 错误仅输出到 console；无 toast 通知；无页面切换动画

优化目标：修复设计系统基础 -> 集成 toast 通知 -> 增强侧边栏体验 -> 改造 Dashboard -> 增强 Header -> 全局动画打磨。

---

## Implementation Order

```
Phase 1: styles.css (foundation)
Phase 2: Toast integration (app.rs + error sites)
Phase 3: Sidebar UX (project_list.rs, session_item.rs)
Phase 4: Header tabs (types.rs, main_header.rs)
Phase 5: Dashboard redesign (session_item.rs, dashboard.rs)
Phase 6: Global polish (project.rs)
```

---

## Phase 1: styles.css Cleanup & Color System Fix

**File**: `styles.css`

### 1.1 Remove duplicate variable declarations

In `:root` block, delete the first (dead) definitions of `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground` (lines 15-18). Keep the second set (lines 26-29) which has the actual blue/colored values.

In `.dark` block, delete the first (dead) definitions (lines 66-69: `--primary: oklch(0.87...)`, `--primary-foreground`, `--secondary: oklch(0.269...)`, `--secondary-foreground`). Keep lines 77-80.

### 1.2 Add missing `--destructive-foreground`

In `:root`, add after `--destructive`:
```css
--destructive-foreground: oklch(0.985 0 0);
```

In `.dark`, add after `--destructive`:
```css
--destructive-foreground: oklch(0.985 0 0);
```

### 1.3 Improve color trio differentiation

In `:root`:
- `--muted`: change from `oklch(0.97 0 0)` to `oklch(0.955 0 0)` (slightly darker)
- `--accent`: change from `oklch(0.97 0 0)` to `oklch(0.94 0.01 264)` (darker + faint blue tint)

In `.dark`:
- `--muted`: change from `oklch(0.269 0 0)` to `oklch(0.24 0 0)` (slightly darker)
- `--accent`: change from `oklch(0.371 0 0)` to `oklch(0.31 0.01 264)` (adjusted + blue tint)

### 1.4 Improve dark card-background contrast

In `.dark`:
- `--card`: change from `oklch(0.205 0 0)` to `oklch(0.22 0 0)`
- `--popover`: change from `oklch(0.205 0 0)` to `oklch(0.22 0 0)`

---

## Phase 2: Toast Integration

### 2.1 Mount Toaster in App root

**File**: `src/app.rs`

Add imports:
```rust
use crate::ui::toast_custom::toaster::{provide_toaster, Toaster};
```

In `App` component body, after `ThemeMode::init()` and before `AppContext::new()`:
```rust
provide_toaster();
```

In `view!` macro, add `<Toaster />` as sibling to `<Router>`:
```rust
view! {
    <Toaster />
    <Router> ... </Router>
}
```

### 2.2 Replace console.error with toast.error

**File**: `src/app.rs` (3 sites)

Add import: `use crate::ui::toast_custom::toaster::expect_toaster;`

Replace each `web_sys::console::error_1(&format!("...{e}").into())` with:
```rust
expect_toaster().error(format!("...{e}"));
```

Sites:
- Line ~36: "Failed to load projects"
- Line ~50: "Failed to check onboarding"
- Line ~67: "Failed to refresh projects"

**File**: `src/state/session_state.rs` (2 sites)

Add import: `use crate::ui::toast_custom::toaster::expect_toaster;`

Replace:
- Line 56-58: `web_sys::console::error_1(...)` -> `expect_toaster().error(format!("Failed to load sessions: {e}"));`
- Line 88-90: same pattern -> `expect_toaster().error(format!("Failed to load more sessions: {e}"));`

**File**: `src/layout/sidebar/project_list.rs` (3 sites)

Add import: `use crate::ui::toast_custom::toaster::expect_toaster;`

Replace all `web_sys::console::error_1(...)` with `expect_toaster().error(...)`.

### 2.3 Add success toasts

**File**: `src/layout/sidebar/project_list.rs`

After successful `delete_session` call (~line 256): add `expect_toaster().success("Session deleted");`

After successful `set_session_name` calls (~line 340 and ~370): add `expect_toaster().success("Session renamed");`

---

## Phase 3: Sidebar UX Enhancement

### 3.1 Search icon

**File**: `src/layout/sidebar/project_list.rs`

Add import: `use icons::Search;`

Wrap the search `<input>` (lines 38-47) in a relative container, add Search icon:

```rust
<div class="px-2 pb-2">
    <div class="relative">
        <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground pointer-events-none" />
        <input
            // ... existing props ...
            class="w-full h-8 shadow-none bg-background ... pl-8 pr-3 ..."
            // Change px-3 to pl-8 pr-3 to make room for icon
        />
    </div>
</div>
```

### 3.2 Session text size increase

**File**: `src/layout/sidebar/session_item.rs`
- Line 52: change `text-[10px]` to `text-[11px]` (time/count span)
- Line 55: change `text-[10px]` to `text-[11px]` (message count badge)

**File**: `src/layout/sidebar/project_list.rs`
- Line ~181: change `text-[10px]` to `text-[11px]` (project session count badge)
- Line ~292: change `text-[10px]` to `text-[11px]` (load more button)

### 3.3 Delete confirmation dialog

**File**: `src/layout/sidebar/project_list.rs`

In `SessionList` component, add signals:
```rust
let deleting_session_id = RwSignal::new(Option::<String>::None);
let deleting_provider = RwSignal::new(Option::<String>::None);
```

Change `on_delete` callback: instead of immediately deleting, set pending state:
```rust
let on_delete = {
    let provider = provider.clone();
    Callback::new(move |id: String| {
        deleting_session_id.set(Some(id));
        deleting_provider.set(Some(provider.clone()));
    })
};
```

Add confirmation modal (following the existing rename modal pattern at lines 312-391):
```rust
{move || {
    deleting_session_id.get().map(|session_id| {
        let project_name = project_name.clone();
        view! {
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80">
                <div class="bg-background border border-border rounded-lg p-4 shadow-lg w-80">
                    <p class="text-sm font-medium mb-1">"Delete Session"</p>
                    <p class="text-xs text-muted-foreground mb-4">
                        "This session will be permanently deleted. This action cannot be undone."
                    </p>
                    <div class="flex justify-end gap-2">
                        <button
                            class="px-3 py-1.5 text-xs rounded-md border border-border text-muted-foreground hover:text-foreground transition-colors"
                            on:click=move |_| deleting_session_id.set(None)
                        >
                            "Cancel"
                        </button>
                        <button
                            class="px-3 py-1.5 text-xs rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors"
                            on:click=move |_| {
                                // actual delete logic here (extracted from old on_delete)
                                let provider = deleting_provider.get_untracked().unwrap_or_default();
                                let pn = project_name.clone();
                                let sid = session_id.clone();
                                spawn_local(async move { ... });
                                deleting_session_id.set(None);
                            }
                        >
                            "Delete"
                        </button>
                    </div>
                </div>
            </div>
        }
    })
}}
```

### 3.4 Hover action backdrop

**File**: `src/layout/sidebar/session_item.rs`

Line 67: change the hover actions container class from:
```
"absolute top-0.5 right-1 hidden group-hover/session:flex items-center gap-0.5"
```
to:
```
"absolute top-0.5 right-1 hidden group-hover/session:flex items-center gap-0.5 bg-sidenav/90 backdrop-blur-sm rounded px-0.5"
```

---

## Phase 4: Header Tab Enhancement

### 4.1 Add `is_implemented` method to AppTab

**File**: `src/tauri/types.rs`

Add after `label()` method (~line 264):
```rust
pub fn is_implemented(&self) -> bool {
    matches!(self, AppTab::Chat)
}
```

### 4.2 Add tab icons and coming-soon indicator

**File**: `src/layout/main_header.rs`

Add imports:
```rust
use icons::{MessageSquare, SquareTerminal, FolderOpen, GitBranch};
use crate::ui::tooltip::{Tooltip, TooltipContent, TooltipPosition};
```

Add helper component:
```rust
#[component]
fn TabIcon(tab: AppTab) -> impl IntoView {
    match tab {
        AppTab::Chat => view! { <MessageSquare class="size-3.5" /> }.into_any(),
        AppTab::Shell => view! { <SquareTerminal class="size-3.5" /> }.into_any(),
        AppTab::Files => view! { <FolderOpen class="size-3.5" /> }.into_any(),
        AppTab::Git => view! { <GitBranch class="size-3.5" /> }.into_any(),
    }
}
```

Update tab button rendering:
- Increase tap target: change `px-2.5 py-1` to `px-3 py-1.5`
- Increase nav container: change `h-8` to `h-9`
- Add icon before label: `<span class="flex items-center gap-1.5"><TabIcon tab=tab />{label}</span>`
- For unimplemented tabs (non-active state): add `opacity-50` to class
- Wrap unimplemented tab buttons in `<Tooltip>` with `<TooltipContent position=TooltipPosition::Bottom>"Coming soon"</TooltipContent>`

---

## Phase 5: Dashboard Redesign

### 5.1 Make ProviderIcon public

**File**: `src/layout/sidebar/session_item.rs`

Change `fn ProviderIcon` to `pub fn ProviderIcon` (line ~102).

### 5.2 Rewrite dashboard

**File**: `src/pages/dashboard.rs`

New imports:
```rust
use crate::state::AppContext;
use crate::layout::sidebar::session_item::ProviderIcon;
```

New layout structure:

```
centered container (max-w-2xl, gap-8, h-full)
  |
  +-- Header section
  |     "RCCUI" (text-3xl font-bold tracking-tight)
  |     "Multi AI Coding Assistant Manager" (text-base text-muted-foreground)
  |
  +-- Provider stats grid (grid grid-cols-2 sm:grid-cols-4 gap-3)
  |     4x provider cards:
  |       - ProviderIcon (size-5) + provider name (text-xs uppercase tracking-wider)
  |       - Session count (text-2xl font-bold tabular-nums)
  |       - Card bg: bg-muted/50 border-transparent shadow-none
  |       - Provider brand color: text-provider-{name}
  |
  +-- Bottom section (conditional)
        if projects > 0: "Select a project from the sidebar"
        if projects == 0: Empty state component
```

Provider stats computation (reactive):
```rust
let stats = move || {
    let projects = ctx.projects.get();
    let claude = projects.iter().map(|p| p.sessions.len()).sum::<usize>();
    let cursor = projects.iter().map(|p| p.cursor_sessions.len()).sum::<usize>();
    let codex = projects.iter().map(|p| p.codex_sessions.len()).sum::<usize>();
    let gemini = projects.iter().map(|p| p.gemini_sessions.len()).sum::<usize>();
    (claude, cursor, codex, gemini)
};
```

Each provider card:
```rust
<div class="flex flex-col items-center gap-2 rounded-lg bg-muted/50 p-4">
    <ProviderIcon provider="claude".to_string() />  // but larger, wrap in size-5 container
    <span class="text-xs text-muted-foreground uppercase tracking-wider">"Claude"</span>
    <span class="text-2xl font-bold tabular-nums text-provider-claude">{count}</span>
</div>
```

Empty state (when 0 projects):
```rust
<div class="flex flex-col items-center gap-3 text-center px-4">
    <div class="size-12 rounded-lg bg-muted flex items-center justify-center">
        // folder icon SVG
    </div>
    <p class="text-sm font-medium text-foreground">"No projects yet"</p>
    <p class="text-xs text-muted-foreground max-w-sm">
        "Sessions will appear automatically as you use Claude, Cursor, Codex, or Gemini in your projects."
    </p>
</div>
```

---

## Phase 6: Global Polish

### 6.1 Tab content fade-in animation

**File**: `src/pages/project.rs`

Add `animate-in fade-in duration-150` class to each tab content wrapper div (the outermost div in each match arm). Since `tw-animate-css` is already imported in styles.css, these animation classes are available.

For example:
```rust
AppTab::Chat => view! {
    <div class="flex flex-col items-center justify-center h-full gap-3 text-sm text-muted-foreground animate-in fade-in duration-150">
        ...
    </div>
}.into_any(),
```

Apply the same to Shell, Files, Git match arms.

---

## Files Modified Summary

| File | Changes |
|------|---------|
| `styles.css` | Remove duplicate vars, add destructive-foreground, fix contrast, fix dark card |
| `src/app.rs` | Mount Toaster, provide_toaster(), replace console.error with toast |
| `src/state/session_state.rs` | Replace 2x console.error with toast.error |
| `src/layout/sidebar/project_list.rs` | Search icon, text size, delete confirmation, toast success |
| `src/layout/sidebar/session_item.rs` | Text size, hover backdrop, make ProviderIcon pub |
| `src/tauri/types.rs` | Add is_implemented() to AppTab |
| `src/layout/main_header.rs` | Tab icons, coming-soon indicator, larger tap targets |
| `src/pages/dashboard.rs` | Complete redesign with provider stats + empty state |
| `src/pages/project.rs` | Add fade-in animation to tab content |

---

## Verification

1. **After Phase 1**: `cargo check` -- no Rust changes, only CSS. Rebuild with `npx tailwindcss -i styles.css -o dist/tailwind.css` or rely on Trunk auto-rebuild
2. **After Phase 2**: `cargo check` -- verify no import errors. Run app and trigger an error to see toast
3. **After Phase 3**: `cargo check` -- verify Search icon import works. Visual check: search icon visible, text larger, delete shows confirmation
4. **After Phase 4**: `cargo check` -- verify icon imports (MessageSquare, SquareTerminal, FolderOpen, GitBranch exist in icons crate). Visual check: tabs have icons, unimplemented tabs show opacity + tooltip
5. **After Phase 5**: `cargo check` -- verify ProviderIcon is public and importable. Visual check: dashboard shows 4 provider stat cards
6. **After Phase 6**: `cargo check` -- verify animate classes compile. Visual check: tab content fades in

Final end-to-end: `cargo tauri dev` and test all interactions.
