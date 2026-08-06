# DayMark

个人项目（任务）管理桌面客户端：Vue 3 + Vite + TypeScript（Tailwind + shadcn/ui 风格）前端，Tauri v2 + Rust 原生后端，内置 SQLite。

## 功能

- 看板 / 四象限 / 列表 / 甘特图四种视图
- 任务字段：标题、描述、状态、优先级、负责人、截止日期、标签、紧急/重要、子任务
- 按优先级筛选、状态统计
- **LLM 集成**：支持 OpenAI API 兼容厂商（DeepSeek、通义千问、Moonshot 等），配置 URL + Key + Model
- **GitLab 集成**：配置实例地址、用户名、Personal Access Token，获取提交和合并请求数据
- **报告生成**：支持日报/周报/月报，内置多种模板（成果导向、三句话、TOP3、周进展等），LLM 智能生成 Markdown 报告
- **报告历史**：已生成的报告自动保存，支持查看和删除

## 目录结构

```
frontend/           Vue 3 + Vite 前端（pnpm）
frontend/src-tauri/ Tauri v2 + Rust 后端（rusqlite 内置 SQLite）
backend/            旧版 FastAPI 后端（已弃用，仅作数据迁移来源）
```

## 构建与运行

开发模式（热重载）：

```bash
cd frontend
pnpm install   # 首次安装依赖
pnpm tauri dev
```

构建桌面应用（产出 .app / .dmg）：

```bash
cd frontend
pnpm tauri build
```

数据库位置：`~/.local/share/daymark/daymark.db`（首次安装自动创建）。

## 架构说明

原 HTTP API 已全部改为 Tauri 原生命令（`invoke` 直调，无网络层），前端 `src/api.ts` 保持原有接口签名不变：

- `list_tasks` / `create_task` / `update_task` / `delete_task`
- `get_settings` / `save_settings`
- `test_llm` / `test_gitlab`
- `gitlab_projects` / `gitlab_merge_requests` / `gitlab_mr_list` / `rewrite_mr`
- `generate_report` / `list_reports` / `get_report` / `delete_report`
- `list_report_templates` / `create_report_template` / `update_report_template` / `delete_report_template`

任务字段：title、description、status（todo/inProgress/review/done）、priority（low/medium/high）、assignee、due_date、tag、urgent、important、subtasks（JSON）。
| GET    | /api/settings     | 获取设置（LLM+GitLab） |
| PUT    | /api/settings     | 更新设置 |
| POST   | /api/llm/test     | 测试 LLM 连接 |
| POST   | /api/gitlab/test  | 测试 GitLab 连接 |
| GET    | /api/gitlab/projects | 获取 GitLab 项目列表 |
| POST   | /api/gitlab/merge-requests | 获取 GitLab MR 列表 |
任务字段：title、description、status（todo/inProgress/review/done）、priority（low/medium/high）、assignee、due_date、tag、urgent、important、subtasks（JSON）。

## 设置

### LLM

在侧边栏底部「设置」按钮中配置：
- 名称（如 DeepSeek、OpenAI）
- API 地址（如 https://api.deepseek.com）
- API Key（选填）
- 模型（如 deepseek-chat、gpt-4o）
- 温度（0-2，默认 0.7）
- Max Tokens（默认 2000）
- 支持添加多个配置，设为「当前使用」的那个

支持 OpenAI API 兼容厂商：OpenAI、DeepSeek、通义千问、Moonshot 等。

### GitLab

在设置页面 GitLab 选项卡中配置：
- 实例地址（如 https://gitlab.com）
- 用户名
- Personal Access Token（需要 read_api 权限）
