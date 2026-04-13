# AGENTS.md

## Project Overview

Tauri 2 + Leptos 0.8 CSR desktop application with rust/ui component library (shadcn-ui style copy pattern).

- **Language**: Rust (edition 2024, nightly toolchain)
- **Frontend**: Leptos 0.8 with CSR mode, compiled to WASM
- **Desktop**: Tauri 2
- **Styling**: Tailwind CSS v4 with OKLCH theme variables
- **Components**: rust/ui (85+ UI components + 26 hooks + utils + constants)

## Project Structure

```
src/
├── main.rs             # Entry point: mod declarations + mount_to_body
├── app.rs              # Root App component
├── ui/                 # rust/ui components (83 components)
│   ├── mod.rs
│   ├── button.rs, input.rs, card.rs, dialog.rs, ...
│   └── toast_custom/   # Directory-based component
├── hooks/              # rust/ui hooks (26 hooks)
├── utils/              # Utilities (date, country, query)
└── constants/          # Constants (pagination)
src-tauri/              # Tauri backend (Rust)
public/                 # Static assets + JS dependencies
styles.css              # Tailwind entry (source of truth for theme)
index.html              # Trunk entry
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

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `leptos` 0.8 (csr, nightly) | Frontend framework |
| `leptos_router` 0.8 | Router for Leptos |
| `leptos_ui` 0.3.22 | Component macros (variants!, clx!, void!) |
| `tw_merge` 0.1.21 | Tailwind class merging |
| `icons` 0.18.2 | Lucide-style icon components |
| `strum` 0.26 (derive) | Enum string conversion |
| `web-sys` 0.3 | DOM API bindings (40+ features enabled) |
| `tauri` 2 | Desktop runtime (in src-tauri/) |
| `time` / `chrono` | Date/time handling |
| `validator` | Form validation |
| `serde_json` | JSON serialization |

## Important Notes

- This project uses Rust **nightly** toolchain (configured in `rust-toolchain.toml`)
- Rust edition is **2024** (required for `let chains` syntax used in components)
- The `src/ui/` and `src/hooks/` modules are at the crate root level (components reference `crate::ui::` paths)
- Do NOT move components into a `components/` subdirectory - it will break internal `crate::ui::` imports
- JS dependency files in `public/app/` and `public/hooks/` are required by some components (drawer, OTP, action_bar, shimmer, etc.)
- `ui_config.toml` uses relative path: `base_path_components = "src/components"` (no leading slash)
