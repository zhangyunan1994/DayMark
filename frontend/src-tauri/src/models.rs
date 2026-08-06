use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assignee: String,
    pub due_date: Option<String>,
    pub urgent: bool,
    pub important: bool,
    pub subtasks: Vec<Subtask>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default)]
    pub assignee: String,
    pub due_date: Option<String>,
    #[serde(default)]
    pub urgent: bool,
    #[serde(default)]
    pub important: bool,
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<Option<String>>,
    pub urgent: Option<bool>,
    pub important: Option<bool>,
    pub subtasks: Option<Vec<Subtask>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub id: Option<String>,
    pub provider_name: String,
    pub api_base_url: String,
    #[serde(default)]
    pub api_key: String,
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i64,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    #[serde(default = "default_gitlab_base")]
    pub base_url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub token: String,
}

impl Default for GitLabConfig {
    fn default() -> Self {
        Self {
            base_url: default_gitlab_base(),
            username: String::new(),
            token: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub llm_configs: Vec<LLMConfig>,
    #[serde(default)]
    pub gitlab: GitLabConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub id: i64,
    pub report_type: String,
    pub title: String,
    pub template_id: String,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReportCreate {
    pub report_type: String,
    pub title: String,
    #[serde(default)]
    pub template_id: String,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateReportRequest {
    #[serde(default = "default_report_type")]
    pub report_type: String,
    pub template_name: String,
    #[serde(default)]
    pub template_sections: Vec<String>,
    pub date_start: String,
    pub date_end: String,
    #[serde(default = "default_true")]
    pub include_gitlab: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub id: String,
    pub report_type: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReportTemplateInput {
    pub report_type: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub ok: bool,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RewriteMRResult {
    pub title: String,
    pub description: String,
    pub web_url: String,
}

fn default_status() -> String {
    "todo".into()
}

fn default_priority() -> String {
    "medium".into()
}

fn default_temperature() -> f64 {
    0.7
}

fn default_max_tokens() -> i64 {
    2000
}

fn default_gitlab_base() -> String {
    "https://gitlab.com".into()
}

fn default_report_type() -> String {
    "weekly".into()
}

fn default_true() -> bool {
    true
}
