# 任务管理工具

个人项目（任务）管理工具：前端 React + Vite + TypeScript（Tailwind + shadcn/ui 风格），后端 FastAPI + SQLite。

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
backend/   FastAPI 后端（uv 管理依赖，SQLAlchemy + SQLite）
frontend/  React + Vite 前端（pnpm）
```

## 启动

后端（端口 8000，API 文档 http://localhost:8000/docs）：

```bash
cd backend
uv sync          # 首次安装依赖
uv run uvicorn app.main:app --reload
```

前端（端口 5173）：

```bash
cd frontend
pnpm install     # 首次安装依赖
pnpm dev
```

打开 http://localhost:5173 使用。

## API

| 方法 | 路径 | 说明 |
| ---- | ---- | ---- |
| GET    | /api/tasks        | 任务列表 |
| POST   | /api/tasks        | 新建任务 |
| GET    | /api/tasks/{id}   | 任务详情 |
| PUT    | /api/tasks/{id}   | 更新任务 |
| DELETE | /api/tasks/{id}   | 删除任务 |
| GET    | /api/settings     | 获取设置（LLM+GitLab） |
| PUT    | /api/settings     | 更新设置 |
| POST   | /api/llm/test     | 测试 LLM 连接 |
| POST   | /api/gitlab/test  | 测试 GitLab 连接 |
| GET    | /api/gitlab/projects | 获取 GitLab 项目列表 |
| POST   | /api/gitlab/merge-requests | 获取 GitLab MR 列表 |
| POST   | /api/reports/generate | 生成报告 |
| GET    | /api/reports      | 报告列表 |
| GET    | /api/reports/{id} | 报告详情 |
| DELETE | /api/reports/{id} | 删除报告 |

任务字段：title、description、status（todo/inProgress/review/done）、priority（low/medium/high）、assignee、due_date、tag、urgent、important、subtasks（JSON）。

数据库文件：`backend/tasks.db`（首次启动自动创建）。

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
