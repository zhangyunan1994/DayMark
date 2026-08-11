import { invoke } from '@tauri-apps/api/core'

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
  report_type: 'daily' | 'weekly' | 'monthly' | 'opencode'
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

export interface UserMessage {
  time_created: string
  directory: string
  session_id: string
  title: string
  user_text: string
}

export interface DailySummary {
  date: string
  message_count: number
  session_count: number
  directories: string[]
}

export interface DateRange {
  earliest: string
  latest: string
}

export interface AnalyticsSummary {
  total_messages: number
  total_sessions: number
  total_directories: number
  date_range: DateRange | null
  daily_summaries: DailySummary[]
}

export const api = {
  list: () => invoke<Task[]>('list_tasks'),
  create: (input: TaskInput) => invoke<Task>('create_task', { input }),
  update: (id: number, input: Partial<TaskInput>) =>
    invoke<Task>('update_task', { id, input }),
  remove: (id: number) => invoke<void>('delete_task', { id }),

  getSettings: () => invoke<Settings>('get_settings'),
  saveSettings: (settings: Settings) => invoke<Settings>('save_settings', { settings }),

  testLlm: (config: LLMConfig) => invoke<TestResult>('test_llm', { config }),
  testGitlab: (config: GitLabConfig) => invoke<TestResult>('test_gitlab', { config }),
  gitlabProjects: () => invoke<Record<string, unknown>[]>('gitlab_projects'),
  gitlabMergeRequests: (config: GitLabConfig, updated_after: string, updated_before: string) =>
    invoke<Record<string, unknown>[]>('gitlab_merge_requests', {
      config,
      updatedAfter: updated_after,
      updatedBefore: updated_before,
    }),
  gitlabMrList: () => invoke<GitLabMR[]>('gitlab_mr_list'),
  rewriteMr: (projectId: number, mergeRequestIid: number) =>
    invoke<RewriteMRResult>('rewrite_mr', {
      projectId,
      mergeRequestIid,
    }),

  generateReport: (input: {
    report_type: string
    template_name: string
    template_sections?: string[]
    date_start: string
    date_end: string
    include_gitlab: boolean
  }) => invoke<Report>('generate_report', { req: input }),
  listReports: () => invoke<Report[]>('list_reports'),
  getReport: (id: number) => invoke<Report>('get_report', { id }),
  deleteReport: (id: number) => invoke<void>('delete_report', { id }),

  listReportTemplates: () => invoke<ReportTemplate[]>('list_report_templates'),
  createReportTemplate: (input: ReportTemplateInput) =>
    invoke<ReportTemplate>('create_report_template', { input }),
  updateReportTemplate: (id: string, input: ReportTemplateInput) =>
    invoke<ReportTemplate>('update_report_template', { id, input }),
  deleteReportTemplate: (id: string) =>
    invoke<void>('delete_report_template', { id }),

  opencodeMessages: (params: {
    date_start?: string
    date_end?: string
    directory?: string
    limit?: number
  }) => invoke<UserMessage[]>('opencode_messages', params),
  opencodeSummary: (params: { date_start?: string; date_end?: string }) =>
    invoke<AnalyticsSummary>('opencode_summary', params),
  generateOpencodeReport: (params: {
    date_start?: string
    date_end?: string
    directory?: string
  }) => invoke<Report>('generate_opencode_report', params),
}
