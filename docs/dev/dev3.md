# 文件编辑器修复记录

## 背景

文件面板已经能够正确读取文件内容，日志中可见：

- `read_file` 返回成功
- `editor_content` 已写入文本内容
- `FileEditor` 的同步日志显示 `current=0chars, next=...chars`

但在 Tauri/WebKit 环境下，右侧编辑区域仍然出现“内容已加载但画面空白”的现象。

## 排查结论

问题根因不在文件读取链路，而在富文本编辑器渲染链路。

前期尝试过两类修复：

1. 为 CodeMirror 增加 `ResizeObserver`
2. 通过 JS 把父容器实际像素尺寸直接同步到编辑器根节点

这些修复能改善高度计算问题，但在当前 Tauri/WebKit 场景下仍然不够稳定。即使实例创建成功、内容同步成功，视图仍可能空白。

## 当前稳定方案

当前主路径已切换为原生 `textarea` 编辑器，位置：

- `src/features/files/file_editor.rs`

这样做的目标不是提供最终形态，而是先保证以下核心能力稳定可用：

- 文本文件内容展示
- 文本编辑
- `Cmd/Ctrl + S` 保存
- 保存成功/失败 toast
- 未保存状态提示
- `Tab` 自动插入 4 个空格
- `tab-size: 4`
- 图片文件继续走 `image_preview.rs`

## 相关实现要点

### 保存反馈

保存入口统一走 `save_current_file()`，避免 UI 出现“先弹成功 toast，后面才异步失败”的假成功状态。

正确模式是：

1. 调用 `commands::save_file()`
2. 等待结果返回
3. 成功后清除 `editor_dirty` 并弹 success toast
4. 失败后保留 dirty 状态并弹 error toast

### 未保存状态

未保存状态通过 `FileContext.editor_dirty` 统一驱动：

- 编辑器头部显示醒目的未保存徽标
- 保存按钮在 dirty 时使用强调样式
- 底部状态栏显示“未保存 / 已保存”

### 输入体验

为避免浏览器默认行为影响代码输入，当前实现对 `Tab` 做了拦截并插入 4 个空格，不再让焦点跳出编辑器。

## 文档同步

以下文档已同步更新：

- `AGENTS.md`
- `README.md`

它们现在都说明了：

- 文件模块不再是 placeholder
- 当前稳定编辑器是原生 `textarea`
- Tauri/WebKit 下的 CodeMirror 空白问题属于已知经验

## 后续建议

如果后续要重新接回增强编辑器，建议单独开一轮迭代，不要与终端、布局或文件树变更混在一起。

推荐顺序：

1. 保持当前 `textarea` 主路径稳定
2. 单独做增强编辑器实验分支
3. 先验证渲染稳定性，再逐步补语法高亮、搜索、快捷键等能力
4. 只有当增强编辑器在 Tauri/WebKit 下稳定后，再替换当前主路径
