use std::sync::Mutex;

use chrono::Datelike;
use rusqlite::Connection;
use serde_json::Value;
use tauri::State;

use crate::db;
use crate::models::{
    AnalyticsSummary, GenerateReportRequest, GitLabConfig, LLMConfig, Report, ReportCreate,
    ReportTemplate, ReportTemplateInput, RewriteMRResult, Settings, Task, TaskInput, TaskUpdate,
    TestResult, UserMessage,
};
use crate::services::{gitlab, llm};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub client: reqwest::Client,
}

fn lock_db<'a>(state: &'a AppState) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    state.db.lock().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tasks(state: State<'_, AppState>) -> Result<Vec<Task>, String> {
    let db = lock_db(&state)?;
    db::list_tasks(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_task(state: State<'_, AppState>, input: TaskInput) -> Result<Task, String> {
    let db = lock_db(&state)?;
    db::create_task(&db, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task(
    state: State<'_, AppState>,
    id: i64,
    input: TaskUpdate,
) -> Result<Task, String> {
    let db = lock_db(&state)?;
    db::update_task(&db, id, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = lock_db(&state)?;
    db::delete_task(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let db = lock_db(&state)?;
    db::load_settings(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    let db = lock_db(&state)?;
    db::save_settings(&db, &settings).map_err(|e| e.to_string())?;
    db::load_settings(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_llm(
    state: State<'_, AppState>,
    config: LLMConfig,
) -> Result<TestResult, String> {
    match llm::test_connection(&state.client, &config).await {
        Ok(message) => Ok(TestResult { ok: true, message }),
        Err(e) => Ok(TestResult { ok: false, message: e.to_string() }),
    }
}

#[tauri::command]
pub async fn test_gitlab(
    state: State<'_, AppState>,
    config: GitLabConfig,
) -> Result<TestResult, String> {
    match gitlab::test_connection(&state.client, &config).await {
        Ok(message) => Ok(TestResult { ok: true, message }),
        Err(e) => Ok(TestResult { ok: false, message: e.to_string() }),
    }
}

fn saved_gitlab_config(state: &AppState) -> Result<GitLabConfig, String> {
    let db = lock_db(state)?;
    let settings = db::load_settings(&db).map_err(|e| e.to_string())?;
    if settings.gitlab.token.is_empty() {
        return Err("未配置 GitLab Token，请先在设置中配置".to_string());
    }
    Ok(settings.gitlab)
}

#[tauri::command]
pub async fn gitlab_projects(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let config = saved_gitlab_config(&state)?;
    gitlab::list_projects(&state.client, &config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn gitlab_merge_requests(
    state: State<'_, AppState>,
    config: GitLabConfig,
    updated_after: Option<String>,
    updated_before: Option<String>,
) -> Result<Vec<Value>, String> {
    gitlab::list_merge_requests(
        &state.client,
        &config,
        updated_after.as_deref(),
        updated_before.as_deref(),
        "all",
        2000,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn gitlab_mr_list(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let config = saved_gitlab_config(&state)?;
    gitlab::list_merge_requests(&state.client, &config, None, None, "all", 2000)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rewrite_mr(
    state: State<'_, AppState>,
    project_id: i64,
    merge_request_iid: i64,
) -> Result<RewriteMRResult, String> {
    let config = saved_gitlab_config(&state)?;
    let llm_config = {
        let db = lock_db(&state)?;
        db::active_llm_config(&db)
    }
    .ok_or_else(|| "未配置 LLM，请先在设置中配置 LLM".to_string())?;

    let mr = gitlab::get_merge_request(&state.client, &config, project_id, merge_request_iid)
        .await
        .map_err(|e| e.to_string())?;
    if mr.get("state").and_then(|v| v.as_str()) != Some("opened") {
        return Err("只有 opened 状态的 MR 才能重写".to_string());
    }
    let commits =
        gitlab::list_merge_request_commits(&state.client, &config, project_id, merge_request_iid)
            .await
            .map_err(|e| e.to_string())?;

    let mr_title = mr.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let mr_description = mr.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let (title, description) =
        llm::rewrite_merge_request(&state.client, &llm_config, mr_title, mr_description, &commits)
            .await
            .map_err(|e| e.to_string())?;

    let updated = gitlab::update_merge_request(
        &state.client,
        &config,
        project_id,
        merge_request_iid,
        &title,
        &description,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(RewriteMRResult {
        title: updated
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(&title)
            .to_string(),
        description: updated
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or(&description)
            .to_string(),
        web_url: updated
            .get("web_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn type_label(report_type: &str) -> String {
    match report_type {
        "daily" => "日报".to_string(),
        "weekly" => "周报".to_string(),
        "monthly" => "月报".to_string(),
        _ => "报告".to_string(),
    }
}

fn task_lines(tasks: &[Task]) -> Vec<String> {
    tasks
        .iter()
        .map(|t| {
            let status = match t.status.as_str() {
                "inProgress" => "[进行中]",
                "review" => "[审核中]",
                "done" => "[已完成]",
                _ => "[待办]",
            };
            let due = t
                .due_date
                .as_ref()
                .map(|d| format!("（截止 {d}）"))
                .unwrap_or_default();
            let subtask = if !t.subtasks.is_empty() {
                let done = t.subtasks.iter().filter(|s| s.completed).count();
                format!("（子任务 {done}/{}）", t.subtasks.len())
            } else {
                String::new()
            };
            format!("{status} {}{}{}", t.title, due, subtask)
                .trim()
                .to_string()
        })
        .collect()
}

fn gitlab_mr_lines(mrs: &[Value]) -> Vec<String> {
    mrs.iter()
        .map(|mr| {
            let merged = mr
                .get("merged_at")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            let state = if merged {
                "[已合并]".to_string()
            } else {
                match mr.get("state").and_then(|v| v.as_str()) {
                    Some("merged") => "[审核中]".to_string(),
                    Some("opened") => "[进行中]".to_string(),
                    Some(s) => format!("[{s}]"),
                    None => "[未知]".to_string(),
                }
            };
            let source = mr.get("source_branch").and_then(|v| v.as_str()).unwrap_or("");
            let target = mr.get("target_branch").and_then(|v| v.as_str()).unwrap_or("");
            let branch = if !source.is_empty() && !target.is_empty() {
                format!("（{source} → {target}）")
            } else {
                String::new()
            };
            let title = mr.get("title").and_then(|v| v.as_str()).unwrap_or("");
            format!("{state} {title}{branch}").trim().to_string()
        })
        .collect()
}

#[tauri::command]
pub async fn generate_report(
    state: State<'_, AppState>,
    req: GenerateReportRequest,
) -> Result<Report, String> {
    let settings = {
        let db = lock_db(&state)?;
        db::load_settings(&db).map_err(|e| e.to_string())?
    };
    let llm_config = {
        let db = lock_db(&state)?;
        db::active_llm_config(&db)
    };
    let tasks = {
        let db = lock_db(&state)?;
        db::list_tasks(&db).map_err(|e| e.to_string())?
    };

    let now = db::utc8_now();
    let current_date = now.format("%Y-%m-%d").to_string();
    let weekday = [
        "星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日",
    ][now.weekday().num_days_from_monday() as usize];
    let type_label = type_label(&req.report_type);

    let todos = task_lines(&tasks);

    let mut gitlab_mrs: Vec<Value> = Vec::new();
    let mut gitlab_mrs_lines: Vec<String> = Vec::new();
    if req.include_gitlab && !settings.gitlab.token.is_empty() {
        match gitlab::list_merge_requests(
            &state.client,
            &settings.gitlab,
            Some(&format!("{}T00:00:00Z", req.date_start)),
            Some(&format!("{}T23:59:59Z", req.date_end)),
            "all",
            2000,
        )
        .await
        {
            Ok(mrs) => {
                gitlab_mrs_lines = gitlab_mr_lines(&mrs);
                gitlab_mrs = mrs;
            }
            Err(e) => {
                gitlab_mrs_lines = vec![format!("（获取 GitLab 数据失败：{e}）")];
            }
        }
    }

    let title = format!("{} 至 {} {}", req.date_start, req.date_end, type_label);

    let content = if let Some(config) = &llm_config {
        let sections_text = if !req.template_sections.is_empty() {
            let numbered: Vec<String> = req
                .template_sections
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect();
            format!("\n模板结构：\n{}", numbered.join("\n"))
        } else {
            String::new()
        };
        let system_prompt = format!(
            "你是一个专业的工作汇报助手。当前日期：{current_date}，今天是{weekday}。\
             请根据用户提供的工作记录，生成一份{type_label}。\n\n\
             要求：\n\
             1. 使用 Markdown 格式\n\
             2. 语言简洁专业，突出成果和价值\n\
             3. 按照模板结构组织内容\n\
             4. 对工作内容进行归纳总结，不要简单罗列\n\
             5. 适当添加数据支撑（如果有）\n\
             6. 保持真实，不要编造内容\n\
             7. 注意时间范围，一周从星期一开始，到星期日结束\n\
             8. 生成的报告要注意内容的完整性\n\n\
             模板名称：{template_name}\n\
             时间范围：{date_start} 至 {date_end}{sections_text}",
            template_name = req.template_name,
            date_start = req.date_start,
            date_end = req.date_end,
        );
        let mut user_message = format!("请根据以下工作记录生成{type_label}：\n\n");
        if !todos.is_empty() {
            user_message.push_str("## 待办事项\n");
            for line in &todos {
                user_message.push_str(&format!("- {line}\n"));
            }
            user_message.push('\n');
        }
        if !gitlab_mrs_lines.is_empty() {
            user_message.push_str("## GitLab 合并请求\n");
            for line in &gitlab_mrs_lines {
                user_message.push_str(&format!("- {line}\n"));
            }
            user_message.push('\n');
        }
        if todos.is_empty() && gitlab_mrs.is_empty() {
            user_message
                .push_str("（暂无具体工作记录，请根据模板生成框架内容，标注需要填写的部分）\n");
        }
        llm::chat(&state.client, config, &system_prompt, &user_message, 120)
            .await
            .map_err(|e| e.to_string())?
    } else {
        let mut content = format!("# {title}\n\n**模板**：{}\n\n", req.template_name);
        if !todos.is_empty() {
            content.push_str("## 工作事项\n\n");
            for line in &todos {
                content.push_str(&format!("- {line}\n"));
            }
            content.push('\n');
        }
        if !gitlab_mrs_lines.is_empty() {
            content.push_str("## GitLab 合并请求\n\n");
            for line in &gitlab_mrs_lines {
                content.push_str(&format!("- {line}\n"));
            }
        }
        content
    };

    let db = lock_db(&state)?;
    db::create_report(
        &db,
        ReportCreate {
            report_type: req.report_type,
            title,
            template_id: req.template_name,
            date_start: Some(req.date_start),
            date_end: Some(req.date_end),
            content,
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_reports(state: State<'_, AppState>) -> Result<Vec<Report>, String> {
    let db = lock_db(&state)?;
    db::list_reports(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_report(state: State<'_, AppState>, id: i64) -> Result<Report, String> {
    let db = lock_db(&state)?;
    db::get_report(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_report(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = lock_db(&state)?;
    db::delete_report(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_report_templates(state: State<'_, AppState>) -> Result<Vec<ReportTemplate>, String> {
    let db = lock_db(&state)?;
    db::list_report_templates(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_report_template(
    state: State<'_, AppState>,
    input: ReportTemplateInput,
) -> Result<ReportTemplate, String> {
    let db = lock_db(&state)?;
    db::create_report_template(&db, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_report_template(
    state: State<'_, AppState>,
    id: String,
    input: ReportTemplateInput,
) -> Result<ReportTemplate, String> {
    let db = lock_db(&state)?;
    db::update_report_template(&db, &id, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_report_template(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = lock_db(&state)?;
    db::delete_report_template(&db, &id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn opencode_messages(
    date_start: Option<String>,
    date_end: Option<String>,
    directory: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<UserMessage>, String> {
    let ds = date_start.as_deref().filter(|s| !s.is_empty());
    let de = date_end.as_deref().filter(|s| !s.is_empty());
    let dir = directory.as_deref().filter(|s| !s.is_empty());
    log::info!("opencode_messages called with: date_start={:?}, date_end={:?}, directory={:?}, limit={:?}", 
        ds, de, dir, limit);
    let result = db::list_opencode_messages(
        ds,
        de,
        dir,
        limit.unwrap_or(500),
    );
    match &result {
        Ok(msgs) => log::info!("opencode_messages returned {} messages", msgs.len()),
        Err(e) => log::error!("opencode_messages error: {}", e),
    }
    result.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn opencode_summary(
    date_start: Option<String>,
    date_end: Option<String>,
) -> Result<AnalyticsSummary, String> {
    let ds = date_start.as_deref().filter(|s| !s.is_empty());
    let de = date_end.as_deref().filter(|s| !s.is_empty());
    log::info!("opencode_summary called with: date_start={:?}, date_end={:?}", ds, de);
    db::get_opencode_summary(ds, de).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_opencode_report(
    state: State<'_, AppState>,
    date_start: Option<String>,
    date_end: Option<String>,
    directory: Option<String>,
) -> Result<Report, String> {
    let llm_config = {
        let db = lock_db(&state)?;
        db::active_llm_config(&db)
    }.ok_or_else(|| "未配置 LLM，请先在设置中配置 LLM".to_string())?;

    let messages = db::list_opencode_messages(
        date_start.as_deref().filter(|s| !s.is_empty()),
        date_end.as_deref().filter(|s| !s.is_empty()),
        directory.as_deref().filter(|s| !s.is_empty()),
        500,
    ).map_err(|e| e.to_string())?;

    if messages.is_empty() {
        return Err("没有找到消息记录".to_string());
    }

    let user_content = messages.iter().map(|m| {
        format!("[{}] [{}] {}: {}", m.time_created, m.directory, m.title, m.user_text)
    }).collect::<Vec<_>>().join("\n\n");

    let system_prompt = "这是用户的 opencode 中 user 对话记录，分析用户在做什么?";
    let content = llm::chat(&state.client, &llm_config, system_prompt, &user_content, 120)
        .await
        .map_err(|e| e.to_string())?;

    let now = db::utc8_now_str();
    let date_range = match (&date_start, &date_end) {
        (Some(s), Some(e)) => format!("{} 至 {}", s, e),
        (Some(s), None) => format!("{} 至今", s),
        (None, Some(e)) => format!("至 {}", e),
        (None, None) => "全部".to_string(),
    };
    let title = format!("OpenCode 使用分析 {}", date_range);

    let db_conn = lock_db(&state).map_err(|e| e.to_string())?;
    let report = db::create_opencode_report(
        &db_conn,
        &title,
        &content,
        &date_start.filter(|s| !s.is_empty()),
        &date_end.filter(|s| !s.is_empty()),
    ).map_err(|e| e.to_string())?;

    Ok(report)
}
