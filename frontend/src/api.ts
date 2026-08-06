export type TaskStatus = 'todo' | 'inProgress' | 'review' | 'done'
export type TaskPriority = 'low' | 'medium' | 'high'

export interface Subtask {
  id: string
  title: string
  completed: boolean
}

export interface Task {
  id: number
  title: string
  description: string
  status: TaskStatus
  priority: TaskPriority
  assignee: string
  due_date: string | null
  urgent: boolean
  important: boolean
  subtasks: Subtask[]
  created_at: string
  updated_at: string
}

export interface TaskInput {
  title: string
  description: string
  status: TaskStatus
  priority: TaskPriority
  assignee: string
  due_date: string | null
  urgent: boolean
  important: boolean
  subtasks: Subtask[]
}

export type TaskFilter = 'all' | TaskPriority

export interface LLMConfig {
  id?: string
  provider_name: string
  api_base_url: string
  api_key: string
  model: string
  temperature: number
  max_tokens: number
  is_active: boolean
}

export interface GitLabConfig {
  base_url: string
  username: string
  token: string
}

export interface Settings {
  llm_configs: LLMConfig[]
  gitlab: GitLabConfig
}

export interface Report {
  id: number
  report_type: 'daily' | 'weekly' | 'monthly'
  title: string
  template_id: string
  date_start: string | null
  date_end: string | null
  content: string
  created_at: string
}

export interface ReportTemplate {
  id: string
  report_type: 'daily' | 'weekly' | 'monthly'
  name: string
  description: string
  sections: string[]
}

export interface ReportTemplateInput {
  report_type: ReportTemplate['report_type']
  name: string
  description: string
  sections: string[]
}

export interface TestResult {
  ok: boolean
  message: string
}

export interface GitLabMR {
  id: number
  iid: number
  project_id: number
  title: string
  description: string | null
  state: string
  draft: boolean
  web_url: string
  source_branch: string
  target_branch: string
  author: { username: string; name: string }
  created_at: string
  updated_at: string
  merged_at: string | null
  labels: string[]
  milestone: { title: string } | null
  reviewers: { username: string; name: string }[]
}

export interface RewriteMRResult {
  title: string
  description: string
  web_url: string
}

const BASE_URL = import.meta.env.VITE_API_BASE ?? 'http://localhost:8000'

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  })
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(`${res.status} ${res.statusText}: ${body}`)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

export const api = {
  list: () => request<Task[]>('/api/tasks'),
  create: (input: TaskInput) =>
    request<Task>('/api/tasks', { method: 'POST', body: JSON.stringify(input) }),
  update: (id: number, input: Partial<TaskInput>) =>
    request<Task>(`/api/tasks/${id}`, { method: 'PUT', body: JSON.stringify(input) }),
  remove: (id: number) => request<void>(`/api/tasks/${id}`, { method: 'DELETE' }),

  getSettings: () => request<Settings>('/api/settings'),
  saveSettings: (settings: Settings) =>
    request<Settings>('/api/settings', { method: 'PUT', body: JSON.stringify(settings) }),

  testLlm: (config: LLMConfig) =>
    request<TestResult>('/api/llm/test', { method: 'POST', body: JSON.stringify({ config }) }),
  testGitlab: (config: GitLabConfig) =>
    request<TestResult>('/api/gitlab/test', { method: 'POST', body: JSON.stringify({ config }) }),
  gitlabMergeRequests: (config: GitLabConfig, updated_after: string, updated_before: string) =>
    request<Record<string, unknown>[]>('/api/gitlab/merge-requests', {
      method: 'POST',
      body: JSON.stringify({ config, updated_after, updated_before }),
    }),
  gitlabMrList: () => request<GitLabMR[]>('/api/gitlab/merge-requests'),
  rewriteMr: (projectId: number, mergeRequestIid: number) =>
    request<RewriteMRResult>('/api/gitlab/mr/rewrite', {
      method: 'POST',
      body: JSON.stringify({ project_id: projectId, merge_request_iid: mergeRequestIid }),
    }),

  generateReport: (input: {
    report_type: string
    template_name: string
    template_sections?: string[]
    date_start: string
    date_end: string
    include_gitlab: boolean
  }) => request<Report>('/api/reports/generate', { method: 'POST', body: JSON.stringify(input) }),
  listReports: () => request<Report[]>('/api/reports'),
  getReport: (id: number) => request<Report>(`/api/reports/${id}`),
  deleteReport: (id: number) => request<void>(`/api/reports/${id}`, { method: 'DELETE' }),

  listReportTemplates: () => request<ReportTemplate[]>('/api/report-templates'),
  createReportTemplate: (input: ReportTemplateInput) =>
    request<ReportTemplate>('/api/report-templates', { method: 'POST', body: JSON.stringify(input) }),
  updateReportTemplate: (id: string, input: ReportTemplateInput) =>
    request<ReportTemplate>(`/api/report-templates/${id}`, { method: 'PUT', body: JSON.stringify(input) }),
  deleteReportTemplate: (id: string) =>
    request<void>(`/api/report-templates/${id}`, { method: 'DELETE' }),
}
