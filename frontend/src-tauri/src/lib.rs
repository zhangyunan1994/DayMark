mod commands;
mod db;
mod models;
mod services;

use std::sync::Mutex;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let db_path = dirs::home_dir()
                .map(|home| home.join(".local").join("share").join("daymark").join("daymark.db"))
                .ok_or_else(|| "无法获取用户目录".to_string())?;
            let conn = db::open_db(&db_path)
                .map_err(|e| format!("初始化数据库失败: {e}"))?;
            let client = reqwest::Client::builder()
                .build()
                .map_err(|e| format!("初始化 HTTP 客户端失败: {e}"))?;
            app.manage(AppState {
                db: Mutex::new(conn),
                client,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::get_settings,
            commands::save_settings,
            commands::test_llm,
            commands::test_gitlab,
            commands::gitlab_projects,
            commands::gitlab_merge_requests,
            commands::gitlab_mr_list,
            commands::rewrite_mr,
            commands::generate_report,
            commands::list_reports,
            commands::get_report,
            commands::delete_report,
            commands::list_report_templates,
            commands::create_report_template,
            commands::update_report_template,
            commands::delete_report_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
