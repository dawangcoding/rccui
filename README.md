# Tauri + Leptos + Rust/UI

基于 Tauri 2 和 Leptos 0.8 的桌面应用模板，集成 rust/ui 组件库（shadcn-ui 风格）。

## 技术栈

- **Rust** (edition 2024, nightly toolchain)
- **Leptos** 0.8 - 响应式前端框架 (CSR 模式)
- **Tauri** 2 - 桌面应用运行时
- **Tailwind CSS** v4 - 样式系统
- **rust/ui** - 85+ UI 组件 + 26 hooks

## 快速开始

### 前置要求

- [Rust](https://rustup.rs/) (nightly toolchain)
- [Node.js](https://nodejs.org/) (用于 Tailwind CSS)
- [Tauri CLI](https://tauri.app/start/prerequisites/)

### 安装依赖

```bash
# 安装 pnpm 依赖 (Tailwind CSS)
pnpm install

# Rust 依赖会自动通过 Cargo 安装
```

### 开发模式

```bash
# 启动桌面应用 (推荐)
cargo tauri dev

# 仅启动前端开发服务器
trunk serve
```

### 构建生产版本

```bash
# 构建桌面应用
cargo tauri build
```

## 项目结构

```
src/
├── main.rs          # 应用入口
├── app.rs           # 根组件
├── ui/              # rust/ui 组件库 (85+ 组件)
├── hooks/           # 自定义 hooks (26 个)
├── utils/           # 工具函数
└── constants/       # 常量定义
src-tauri/           # Tauri 后端代码
public/              # 静态资源
├── app/             # JS 依赖文件
└── hooks/           # JS hooks
styles.css           # Tailwind 入口文件
index.html           # HTML 模板
Trunk.toml           # Trunk 配置
```

## 使用 rust/ui 组件

```rust
use crate::ui::{
    button::Button,
    input::Input,
    card::{Card, CardContent, CardFooter},
};

view! {
    <Card class="w-full max-w-md">
        <CardContent>
            <Input placeholder="输入内容..." bind_value=signal />
        </CardContent>
        <CardFooter>
            <Button attr:r#type="submit">"提交"</Button>
        </CardFooter>
    </Card>
}
```

## 开发指南

- 查看 [AGENTS.md](./AGENTS.md) 获取详细的编码规范和项目配置说明
- 组件使用 `bind:value` 进行双向绑定（需要 nightly feature）
- 使用 `attr:r#type` 代替 `attr:type`（Rust 关键字冲突）
- Tailwind 类通过 `tw_merge` 合并

## 许可证

MIT
