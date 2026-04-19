# ── 通用 ──
page-not-found = 页面未找到
coming-soon = 即将推出
loading = 加载中...

# ── 操作 ──
action-cancel = 取消
action-delete = 删除
action-save = 保存
action-rename = 重命名

# ── 标签页 ──
tab-chat = 聊天
tab-shell = 终端
tab-files = 文件
tab-git = Git

# ── 侧边栏 ──
sidebar-projects = 项目
select-project = 选择项目
search-projects-placeholder = 搜索项目...
no-projects-found = 未找到项目。使用 Claude、Cursor、Codex 或 Gemini 后，会话将自动出现。
no-matching-projects = 没有匹配的项目
load-more-sessions = 加载更多会话

# ── 侧边栏底部 ──
tooltip-settings = 设置
tooltip-toggle-theme = 切换主题

# ── 会话操作 ──
dialog-delete-session-title = 删除会话
dialog-delete-session-desc = 此会话将被永久删除。此操作无法撤消。
dialog-rename-session-title = 重命名会话
rename-session-placeholder = 输入新名称...

# ── 提示消息 ──
toast-session-deleted = 会话已删除
toast-session-renamed = 会话名称已更改

# ── 相对时间 ──
time-just-now = 刚刚
time-minutes = { $value }分钟前
time-hours = { $value }小时前
time-days = { $value }天前
time-weeks = { $value }周前

# ── 仪表盘 ──
app-subtitle = 多 AI 编程助手管理器
dashboard-no-projects = 暂无项目
dashboard-no-projects-desc = 使用 Claude、Cursor、Codex 或 Gemini 后，会话将自动出现在这里。
dashboard-projects-found = 发现 { $count } 个项目。从侧边栏选择一个开始使用。

# ── 项目页 ──
project-select-prompt = 从侧边栏选择一个项目
project-session-label = 会话：
project-chat-coming = 聊天 - 第2阶段推出
project-shell-coming = 终端 - 第3阶段推出
project-files-coming = 文件 - 第4阶段推出
project-git-coming = Git - 第5阶段推出

# ── 聊天 ──
chat-input-placeholder = 输入消息...
chat-enter-to-send = Enter 发送，Shift+Enter 换行
chat-stop-btn = 停止生成
chat-empty-state = 选择一个会话或发送消息开始新对话
chat-empty-hint = 从侧边栏选择一个会话，或直接输入内容开始
chat-thinking-indicator = 正在思考...
chat-thinking = 思考过程
chat-tool-result = 工具结果
chat-tool-error = 工具错误
chat-error-title = 错误
chat-unknown-error = 未知错误
chat-permission-title = 工具权限请求
chat-permission-desc = 助手请求使用以下工具，是否允许？
chat-permission-allow = 允许
chat-permission-deny = 拒绝
chat-perm-ask = 询问
chat-perm-auto-accept = 自动允许
chat-perm-auto-deny = 自动拒绝

# ── 设置页 ──
settings-title = 设置
settings-language-title = 语言
settings-language-desc = 选择界面显示语言

# ── 引导页 ──
onboarding-coming = 引导 - 第6阶段推出

# ── 终端 ──
shell-empty-state = 选择一个项目以启动终端
shell-connecting = 连接中...
shell-connected = 已连接
shell-disconnected = 已断开
shell-reconnect = 重新连接
shell-auth-required = 需要登录认证
shell-open-link = 打开链接
toast-shell-init-failed = 初始化终端失败

# ── 文件 ──
files-empty-state = 选择一个项目以浏览文件
files-no-files = 没有找到文件
files-loading = 加载文件中...
files-toolbar-new-file = 新建文件
files-toolbar-new-folder = 新建文件夹
files-toolbar-refresh = 刷新
files-create-file-title = 新建文件
files-create-folder-title = 新建文件夹
files-name-placeholder = 输入名称...
files-rename-title = 重命名
files-delete-title = 删除
files-delete-confirm = 确定要删除「{ $name }」吗？此操作无法撤消。
files-editor-unsaved = 未保存
files-editor-save = 保存
files-editor-close = 关闭
toast-file-saved = 文件已保存
toast-file-created = 已创建「{ $name }」
toast-file-renamed = 已重命名为「{ $name }」
toast-file-deleted = 已删除「{ $name }」
toast-file-save-failed = 保存文件失败
toast-file-load-failed = 加载文件失败
toast-file-create-failed = 创建失败
toast-file-delete-failed = 删除失败
toast-file-rename-failed = 重命名失败

# ── Git ──
git-changes = 变更
git-history = 历史
git-branches = 分支
git-branch-label = 分支：
git-no-changes = 没有变更
git-no-commits = 暂无提交记录
git-no-branches = 没有分支
git-modified = 已修改
git-added = 已添加
git-deleted = 已删除
git-untracked = 未追踪
git-commit-placeholder = 输入提交消息...
git-commit-btn = 提交
git-commit-all = 提交全部
git-initial-commit = 初始提交
git-push = 推送
git-pull = 拉取
git-fetch = 获取
git-publish = 发布
git-discard = 撤销更改
git-delete-untracked = 删除文件
git-checkout = 切换
git-create-branch = 新建分支
git-delete-branch = 删除分支
git-delete-branch-title = 删除分支
git-delete-branch-confirm = 确定要删除分支「{ $branch }」吗？此操作不可撤销。
git-branch-name-placeholder = 输入分支名称...
git-create-branch-title = 新建分支
git-revert-commit = 撤销上次提交
git-empty-state = 选择一个项目以查看 Git 状态
git-not-a-repo = 该项目不是 Git 仓库
git-ahead = 领先 { $count } 个提交
git-behind = 落后 { $count } 个提交
git-up-to-date = 与远程一致
git-no-remote = 无远程仓库
git-diff-truncated = Diff 内容过大，已截断
git-loading = 加载中...
git-files-changed = 个文件变更
git-select-file-diff = 选择文件查看差异
git-select-commit-diff = 选择提交查看详情
git-local = 本地
git-remote = 远程
git-current = 当前
toast-git-committed = 提交成功
toast-git-pushed = 推送成功
toast-git-pulled = 拉取成功
toast-git-fetched = 获取成功
toast-git-published = 分支已发布
toast-git-checked-out = 已切换到分支「{ $branch }」
toast-git-branch-created = 已创建分支「{ $branch }」
toast-git-branch-deleted = 已删除分支「{ $branch }」
toast-git-discarded = 已撤销更改
toast-git-reverted = 已撤销上次提交
toast-git-error = Git 操作失败

# ── GitHub ──
git-github = GitHub
git-github-empty = 无法获取 GitHub 信息
git-github-empty-hint = 请确保已安装 gh CLI 且仓库关联了 GitHub 远程。
git-github-visibility-public = 公开
git-github-visibility-private = 私有
git-github-stars = 星标
git-github-forks = 复刻
git-github-language = 语言
git-github-license = 许可证
git-github-default-branch = 默认分支
git-github-homepage = 主页
git-github-disk-usage = 大小
git-github-topics = 标签
git-github-created = 创建于
git-github-updated = 更新于
git-github-open = 在 GitHub 打开
git-github-no-description = 暂无描述
git-github-no-license = 无许可证
git-github-refresh = 刷新
git-github-no-remote-title = 尚未关联 GitHub 远程仓库
git-github-no-remote-hint = 创建一个 GitHub 仓库并将本地代码推送上去。
git-github-repo-name = 仓库名称
git-github-repo-name-placeholder = 输入仓库名称...
git-github-repo-desc = 描述
git-github-repo-desc-placeholder = 仓库描述（可选）...
git-github-create-repo = 创建仓库
git-github-creating = 创建中...
toast-github-created = GitHub 仓库创建成功
