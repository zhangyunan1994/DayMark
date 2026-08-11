use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, ToSql};

use crate::models::{
    AnalyticsSummary, DailySummary, DateRange, Report, ReportCreate, ReportTemplate,
    ReportTemplateInput, Settings, Subtask, Task, TaskInput, TaskUpdate, UserMessage,
};

pub const TZ_OFFSET_SECS: i32 = 8 * 3600;

pub fn utc8_now_str() -> String {
    chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::east_opt(TZ_OFFSET_SECS).unwrap())
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}

pub fn utc8_now() -> chrono::DateTime<chrono::FixedOffset> {
    chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(TZ_OFFSET_SECS).unwrap())
}

pub fn open_db(path: &Path) -> Result<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).context("创建数据目录失败")?;
    }
    let conn = Connection::open(path).context("打开数据库失败")?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .context("设置 WAL 失败")?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'todo',
            priority TEXT NOT NULL DEFAULT 'medium',
            assignee TEXT NOT NULL DEFAULT '',
            due_date TEXT,
            urgent INTEGER NOT NULL DEFAULT 0,
            important INTEGER NOT NULL DEFAULT 0,
            subtasks TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_type TEXT NOT NULL,
            title TEXT NOT NULL,
            template_id TEXT NOT NULL DEFAULT '',
            date_start TEXT,
            date_end TEXT,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );
        "#,
    )
    .context("初始化数据库结构失败")?;
    Ok(())
}

fn task_from_row(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    let subtasks_json: String = row.get("subtasks")?;
    let subtasks: Vec<Subtask> =
        serde_json::from_str(&subtasks_json).unwrap_or_else(|_| vec![]);
    Ok(Task {
        id: row.get("id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        status: row.get("status")?,
        priority: row.get("priority")?,
        assignee: row.get("assignee")?,
        due_date: row.get("due_date")?,
        urgent: row.get::<_, i64>("urgent")? != 0,
        important: row.get::<_, i64>("important")? != 0,
        subtasks,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list_tasks(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn
        .prepare("SELECT * FROM tasks ORDER BY id DESC")
        .context("查询任务失败")?;
    let rows = stmt
        .query_map([], task_from_row)
        .context("读取任务失败")?;
    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

pub fn create_task(conn: &Connection, input: TaskInput) -> Result<Task> {
    let now = utc8_now_str();
    let subtasks = serde_json::to_string(&input.subtasks)?;
    conn.execute(
        "INSERT INTO tasks (title, description, status, priority, assignee, due_date, urgent, important, subtasks, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            input.title,
            input.description,
            input.status,
            input.priority,
            input.assignee,
            input.due_date,
            input.urgent as i64,
            input.important as i64,
            subtasks,
            now,
            now,
        ],
    )
    .context("创建任务失败")?;
    let id = conn.last_insert_rowid();
    get_task(conn, id)
}

pub fn get_task(conn: &Connection, id: i64) -> Result<Task> {
    conn.query_row("SELECT * FROM tasks WHERE id = ?1", [id], task_from_row)
        .optional()
        .context("查询任务失败")?
        .ok_or_else(|| anyhow::anyhow!("Task not found"))
}

pub fn update_task(conn: &Connection, id: i64, input: TaskUpdate) -> Result<Task> {
    let now = utc8_now_str();
    let mut sql = String::from("UPDATE tasks SET updated_at = ?");
    let mut values: Vec<Box<dyn ToSql>> = vec![Box::new(&now)];

    if let Some(v) = &input.title {
        sql.push_str(", title = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = &input.description {
        sql.push_str(", description = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = &input.status {
        sql.push_str(", status = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = &input.priority {
        sql.push_str(", priority = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = &input.assignee {
        sql.push_str(", assignee = ?");
        values.push(Box::new(v));
    }
    if let Some(Some(v)) = &input.due_date {
        sql.push_str(", due_date = ?");
        values.push(Box::new(v));
    }
    if let Some(None) = &input.due_date {
        sql.push_str(", due_date = ?");
        values.push(Box::new(None::<String>));
    }
    if let Some(v) = &input.urgent {
        sql.push_str(", urgent = ?");
        values.push(Box::new(*v as i64));
    }
    if let Some(v) = &input.important {
        sql.push_str(", important = ?");
        values.push(Box::new(*v as i64));
    }
    if let Some(v) = &input.subtasks {
        let s = serde_json::to_string(v)?;
        sql.push_str(", subtasks = ?");
        values.push(Box::new(s));
    }

    sql.push_str(" WHERE id = ?");
    values.push(Box::new(id));

    let changed = conn
        .execute(&sql, params_from_iter(values.iter()))
        .context("更新任务失败")?;
    if changed == 0 {
        anyhow::bail!("Task not found");
    }
    get_task(conn, id)
}

pub fn delete_task(conn: &Connection, id: i64) -> Result<()> {
    let changed = conn
        .execute("DELETE FROM tasks WHERE id = ?1", [id])
        .context("删除任务失败")?;
    if changed == 0 {
        anyhow::bail!("Task not found");
    }
    Ok(())
}

fn get_setting(conn: &Connection, key: &str) -> Result<Option<serde_json::Value>> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .context("读取设置失败")?;
    match raw {
        Some(v) => Ok(Some(serde_json::from_str(&v).unwrap_or(serde_json::Value::Null))),
        None => Ok(None),
    }
}

fn set_setting(conn: &Connection, key: &str, value: &serde_json::Value) -> Result<()> {
    let raw = serde_json::to_string(value)?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, raw],
    )
    .context("保存设置失败")?;
    Ok(())
}

pub fn load_settings(conn: &Connection) -> Result<Settings> {
    let mut settings = Settings {
        llm_configs: vec![],
        gitlab: Default::default(),
    };
    if let Some(serde_json::Value::Array(list)) = get_setting(conn, "llm_configs")? {
        settings.llm_configs = list
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
    }
    if let Some(serde_json::Value::Object(map)) = get_setting(conn, "gitlab")? {
        settings.gitlab =
            serde_json::from_value(serde_json::Value::Object(map)).unwrap_or_default();
    }
    Ok(settings)
}

pub fn save_settings(conn: &Connection, settings: &Settings) -> Result<()> {
    set_setting(conn, "llm_configs", &serde_json::to_value(&settings.llm_configs)?)?;
    set_setting(conn, "gitlab", &serde_json::to_value(&settings.gitlab)?)?;
    Ok(())
}

pub fn active_llm_config(conn: &Connection) -> Option<crate::models::LLMConfig> {
    let settings = load_settings(conn).ok()?;
    settings
        .llm_configs
        .iter()
        .find(|c| c.is_active)
        .cloned()
        .or_else(|| settings.llm_configs.last().cloned())
}

pub fn list_report_templates(conn: &Connection) -> Result<Vec<ReportTemplate>> {
    match get_setting(conn, "report_templates")? {
        Some(serde_json::Value::Array(list)) => Ok(list
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect()),
        _ => Ok(vec![]),
    }
}

pub fn save_report_templates(conn: &Connection, templates: &[ReportTemplate]) -> Result<()> {
    set_setting(conn, "report_templates", &serde_json::to_value(templates)?)?;
    Ok(())
}

pub fn create_report_template(conn: &Connection, input: ReportTemplateInput) -> Result<ReportTemplate> {
    let id = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
    let template = ReportTemplate {
        id,
        report_type: input.report_type,
        name: input.name,
        description: input.description,
        sections: input.sections,
    };
    let mut templates = list_report_templates(conn)?;
    templates.push(template.clone());
    save_report_templates(conn, &templates)?;
    Ok(template)
}

pub fn update_report_template(
    conn: &Connection,
    id: &str,
    input: ReportTemplateInput,
) -> Result<ReportTemplate> {
    let mut templates = list_report_templates(conn)?;
    for t in templates.iter_mut() {
        if t.id == id {
            t.report_type = input.report_type;
            t.name = input.name;
            t.description = input.description;
            t.sections = input.sections;
            let updated = t.clone();
            save_report_templates(conn, &templates)?;
            return Ok(updated);
        }
    }
    anyhow::bail!("模版不存在")
}

pub fn delete_report_template(conn: &Connection, id: &str) -> Result<()> {
    let templates = list_report_templates(conn)?;
    let filtered: Vec<ReportTemplate> = templates
        .into_iter()
        .filter(|t| t.id != id)
        .collect();
    if filtered.len() == 0 {
        anyhow::bail!("模版不存在");
    }
    save_report_templates(conn, &filtered)
}

fn report_from_row(row: &rusqlite::Row) -> rusqlite::Result<Report> {
    Ok(Report {
        id: row.get("id")?,
        report_type: row.get("report_type")?,
        title: row.get("title")?,
        template_id: row.get("template_id")?,
        date_start: row.get("date_start")?,
        date_end: row.get("date_end")?,
        content: row.get("content")?,
        created_at: row.get("created_at")?,
    })
}

pub fn list_reports(conn: &Connection) -> Result<Vec<Report>> {
    let mut stmt = conn
        .prepare("SELECT * FROM reports ORDER BY id DESC")
        .context("查询报告失败")?;
    let rows = stmt
        .query_map([], report_from_row)
        .context("读取报告失败")?;
    let mut reports = Vec::new();
    for row in rows {
        reports.push(row?);
    }
    Ok(reports)
}

pub fn get_report(conn: &Connection, id: i64) -> Result<Report> {
    conn.query_row("SELECT * FROM reports WHERE id = ?1", [id], report_from_row)
        .optional()
        .context("查询报告失败")?
        .ok_or_else(|| anyhow::anyhow!("Report not found"))
}

pub fn create_report(conn: &Connection, input: ReportCreate) -> Result<Report> {
    let now = utc8_now_str();
    conn.execute(
        "INSERT INTO reports (report_type, title, template_id, date_start, date_end, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            input.report_type,
            input.title,
            input.template_id,
            input.date_start,
            input.date_end,
            input.content,
            now,
        ],
    )
    .context("保存报告失败")?;
    let id = conn.last_insert_rowid();
    get_report(conn, id)
}

pub fn delete_report(conn: &Connection, id: i64) -> Result<()> {
    let changed = conn
        .execute("DELETE FROM reports WHERE id = ?1", [id])
        .context("删除报告失败")?;
    if changed == 0 {
        anyhow::anyhow!("Report not found");
    }
    Ok(())
}

// OpenCode Analytics functions

fn open_opencode_db() -> Result<Connection> {
    let home = dirs::home_dir().context("无法获取用户主目录")?;
    let db_path = home.join(".local/share/opencode/opencode.db");
    log::info!("Opening opencode db at: {:?}", db_path);
    if !db_path.exists() {
        anyhow::bail!("opencode.db 不存在于 {:?}", db_path);
    }
    let conn = Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .context("打开 opencode.db 失败")?;
    Ok(conn)
}

pub fn list_opencode_messages(
    date_start: Option<&str>,
    date_end: Option<&str>,
    directory: Option<&str>,
    limit: usize,
) -> Result<Vec<UserMessage>> {
    let conn = open_opencode_db()?;
    let mut sql = String::from(
        "SELECT
            datetime(p.time_created / 1000, 'unixepoch', '+8 hours') as time_created,
            s.directory,
            s.id as session_id,
            s.title,
            COALESCE(CAST(json_extract(p.data, '$.text') AS TEXT), '') as user_text
        FROM message m
        JOIN part p ON p.message_id = m.id
        JOIN session s ON s.id = m.session_id
        WHERE json_extract(m.data, '$.role') = 'user'
          AND json_extract(p.data, '$.type') = 'text'"
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(start) = date_start {
        sql.push_str(" AND datetime(p.time_created / 1000, 'unixepoch', '+8 hours') >= ?");
        params.push(Box::new(format!("{start} 00:00:00")));
    }
    if let Some(end) = date_end {
        sql.push_str(" AND datetime(p.time_created / 1000, 'unixepoch', '+8 hours') <= ?");
        params.push(Box::new(format!("{end} 23:59:59")));
    }
    if let Some(dir) = directory {
        sql.push_str(" AND s.directory LIKE ?");
        params.push(Box::new(format!("%{dir}%")));
    }

    sql.push_str(" ORDER BY p.time_created DESC LIMIT ?");
    params.push(Box::new(limit as i64));

    log::info!("SQL: {}", sql);
    log::info!("Params count: {}", params.len());

    let mut stmt = conn.prepare(&sql).context("查询 opencode 消息失败")?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut messages = Vec::new();
    let mut rows = stmt.query(param_refs.as_slice()).context("查询 opencode 消息失败")?;
    while let Some(row) = rows.next()? {
        let user_text = match row.get_ref(4)? {
            rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
            rusqlite::types::ValueRef::Blob(b) => String::from_utf8_lossy(b).to_string(),
            _ => String::new(),
        };
        messages.push(UserMessage {
            time_created: row.get(0)?,
            directory: row.get(1)?,
            session_id: row.get(2)?,
            title: row.get(3)?,
            user_text,
        });
    }
    log::info!("Found {} messages", messages.len());
    Ok(messages)
}

pub fn get_opencode_summary(
    date_start: Option<&str>,
    date_end: Option<&str>,
) -> Result<AnalyticsSummary> {
    let conn = open_opencode_db()?;
    let mut sql = String::from(
        "SELECT
            datetime(p.time_created / 1000, 'unixepoch', '+8 hours') as time_created,
            s.directory,
            s.id as session_id
        FROM message m
        JOIN part p ON p.message_id = m.id
        JOIN session s ON s.id = m.session_id
        WHERE json_extract(m.data, '$.role') = 'user'
          AND json_extract(p.data, '$.type') = 'text'"
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(start) = date_start {
        sql.push_str(" AND datetime(p.time_created / 1000, 'unixepoch', '+8 hours') >= ?");
        params.push(Box::new(format!("{start} 00:00:00")));
    }
    if let Some(end) = date_end {
        sql.push_str(" AND datetime(p.time_created / 1000, 'unixepoch', '+8 hours') <= ?");
        params.push(Box::new(format!("{end} 23:59:59")));
    }

    log::info!("Summary SQL: {}", sql);
    log::info!("Summary params count: {}", params.len());

    let mut stmt = conn.prepare(&sql).context("查询 opencode 摘要失败")?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut rows = stmt.query(param_refs.as_slice()).context("查询 opencode 摘要失败")?;

    let mut total_messages = 0usize;
    let mut sessions: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut directories: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut daily_data: std::collections::HashMap<String, DailyData> = std::collections::HashMap::new();
    let mut all_dates: Vec<String> = Vec::new();

    struct DailyData {
        message_count: usize,
        session_ids: std::collections::HashSet<String>,
        directories: std::collections::HashSet<String>,
    }

    while let Some(row) = rows.next()? {
        let time_created = match row.get_ref(0)? {
            rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
            _ => String::new(),
        };
        let directory = match row.get_ref(1)? {
            rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
            _ => String::new(),
        };
        let session_id = match row.get_ref(2)? {
            rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
            _ => String::new(),
        };
        total_messages += 1;
        sessions.insert(session_id.clone());
        directories.insert(directory.clone());

        let date_str = &time_created[..10];
        all_dates.push(date_str.to_string());

        let entry = daily_data
            .entry(date_str.to_string())
            .or_insert_with(|| DailyData {
                message_count: 0,
                session_ids: std::collections::HashSet::new(),
                directories: std::collections::HashSet::new(),
            });
        entry.message_count += 1;
        entry.session_ids.insert(session_id);
        entry.directories.insert(directory);
    }

    let mut daily_summaries: Vec<DailySummary> = daily_data
        .into_iter()
        .map(|(date, data)| DailySummary {
            date,
            message_count: data.message_count,
            session_count: data.session_ids.len(),
            directories: data.directories.into_iter().collect(),
        })
        .collect();
    daily_summaries.sort_by(|a, b| b.date.cmp(&a.date));

    let date_range = if all_dates.is_empty() {
        None
    } else {
        all_dates.sort();
        Some(DateRange {
            earliest: all_dates.first().unwrap().clone(),
            latest: all_dates.last().unwrap().clone(),
        })
    };

    Ok(AnalyticsSummary {
        total_messages,
        total_sessions: sessions.len(),
        total_directories: directories.len(),
        date_range,
        daily_summaries,
    })
}
