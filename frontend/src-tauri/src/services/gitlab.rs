use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};

use crate::models::GitLabConfig;

fn base_url(config: &GitLabConfig) -> String {
    config.base_url.trim_end_matches('/').to_string()
}

fn headers(config: &GitLabConfig) -> Vec<(String, String)> {
    let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
    if !config.token.is_empty() {
        headers.push(("PRIVATE-TOKEN".to_string(), config.token.clone()));
    }
    headers
}

pub async fn get(
    client: &reqwest::Client,
    config: &GitLabConfig,
    path: &str,
    params: Vec<(&str, String)>,
) -> Result<Value> {
    let mut request = client
        .get(format!("{}{}", base_url(config), path))
        .timeout(std::time::Duration::from_secs(30))
        .query(&params);
    for (k, v) in headers(config) {
        request = request.header(k, v);
    }
    let resp = request.send().await.context("GitLab 请求失败")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if status != 200 {
        bail!("GitLab 请求失败 ({}): {}", status, truncate(&text, 300));
    }
    serde_json::from_str(&text)
        .map_err(|_| anyhow!("GitLab 响应解析失败: {}", truncate(&text, 300)))
}

pub async fn put(
    client: &reqwest::Client,
    config: &GitLabConfig,
    path: &str,
    payload: &Value,
) -> Result<Value> {
    let mut request = client
        .put(format!("{}{}", base_url(config), path))
        .timeout(std::time::Duration::from_secs(30))
        .json(payload);
    for (k, v) in headers(config) {
        request = request.header(k, v);
    }
    let resp = request.send().await.context("GitLab 请求失败")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if status != 200 {
        bail!("GitLab 更新失败 ({}): {}", status, truncate(&text, 300));
    }
    serde_json::from_str(&text)
        .map_err(|_| anyhow!("GitLab 响应解析失败: {}", truncate(&text, 300)))
}

fn as_list(data: Value) -> Vec<Value> {
    data.as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|v| v.is_object())
        .collect()
}

pub async fn test_connection(client: &reqwest::Client, config: &GitLabConfig) -> Result<String> {
    let data = get(client, config, "/api/v4/user", vec![])
        .await
        .map_err(|e| anyhow!("连接失败: {e}"))?;
    let username = data
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Ok(format!("连接成功（{username}）"))
}

pub async fn list_projects(client: &reqwest::Client, config: &GitLabConfig) -> Result<Vec<Value>> {
    let data = get(
        client,
        config,
        "/api/v4/projects",
        vec![("membership", "true".into()), ("per_page", "100".into())],
    )
    .await?;
    Ok(as_list(data))
}

pub async fn list_merge_requests(
    client: &reqwest::Client,
    config: &GitLabConfig,
    updated_after: Option<&str>,
    updated_before: Option<&str>,
    state: &str,
    per_page: i64,
) -> Result<Vec<Value>> {
    let mut params: Vec<(&str, String)> = vec![
        ("state", state.to_string()),
        ("scope", "all".to_string()),
        ("order_by", "updated_at".to_string()),
        ("per_page", per_page.to_string()),
    ];
    if !config.username.is_empty() {
        params.push(("author_username", config.username.clone()));
    }
    if let Some(v) = updated_after {
        params.push(("updated_after", v.to_string()));
    }
    if let Some(v) = updated_before {
        params.push(("updated_before", v.to_string()));
    }
    let data = get(client, config, "/api/v4/merge_requests", params).await?;
    Ok(as_list(data))
}

pub async fn get_merge_request(
    client: &reqwest::Client,
    config: &GitLabConfig,
    project_id: i64,
    merge_request_iid: i64,
) -> Result<Value> {
    let data = get(
        client,
        config,
        &format!("/api/v4/projects/{project_id}/merge_requests/{merge_request_iid}"),
        vec![],
    )
    .await?;
    if !data.is_object() {
        bail!("GitLab 返回了异常数据");
    }
    Ok(data)
}

pub async fn list_merge_request_commits(
    client: &reqwest::Client,
    config: &GitLabConfig,
    project_id: i64,
    merge_request_iid: i64,
) -> Result<Vec<Value>> {
    let data = get(
        client,
        config,
        &format!("/api/v4/projects/{project_id}/merge_requests/{merge_request_iid}/commits"),
        vec![],
    )
    .await?;
    Ok(as_list(data))
}

pub async fn update_merge_request(
    client: &reqwest::Client,
    config: &GitLabConfig,
    project_id: i64,
    merge_request_iid: i64,
    title: &str,
    description: &str,
) -> Result<Value> {
    put(
        client,
        config,
        &format!("/api/v4/projects/{project_id}/merge_requests/{merge_request_iid}"),
        &json!({"title": title, "description": description}),
    )
    .await
}

fn truncate(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        out.push_str("...");
    }
    out
}
