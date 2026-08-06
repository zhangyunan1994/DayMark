use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use crate::models::LLMConfig;

pub fn chat_endpoint(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/').to_string();
    if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    }
}

fn extract_content(choice: &Value) -> Option<String> {
    if let Some(msg) = choice.get("message") {
        for key in ["content", "reasoning_content"] {
            if let Some(v) = msg.get(key).and_then(|v| v.as_str()) {
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    if let Some(v) = choice
        .get("delta")
        .and_then(|d| d.get("content"))
        .and_then(|v| v.as_str())
    {
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

pub async fn chat(
    client: &reqwest::Client,
    config: &LLMConfig,
    system_prompt: &str,
    user_message: &str,
    timeout: u64,
) -> Result<String> {
    let body = serde_json::json!({
        "model": config.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_message},
        ],
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
    });

    let mut request = client
        .post(chat_endpoint(&config.api_base_url))
        .timeout(std::time::Duration::from_secs(timeout))
        .json(&body);
    if !config.api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", config.api_key));
    }

    let resp = request.send().await.context("LLM 请求失败")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        bail!("LLM 接口错误 ({}): {}", status, truncate(&text, 500));
    }
    let data: Value = serde_json::from_str(&text).context(format!(
        "LLM 响应解析失败: {}",
        truncate(&text, 500)
    ))?;

    if let Some(err) = data.get("error") {
        let msg = err
            .get("message")
            .map(|m| m.to_string())
            .unwrap_or_else(|| err.to_string());
        bail!("API 错误: {msg}");
    }

    if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
        for choice in choices {
            if let Some(content) = extract_content(choice) {
                return Ok(content);
            }
        }
    }
    bail!("无响应内容")
}

pub async fn test_connection(client: &reqwest::Client, config: &LLMConfig) -> Result<String> {
    let result = chat(client, config, "You are a helpful assistant.", "Hello", 120)
        .await
        .map_err(|e| anyhow!("连接失败: {e}"))?;
    if result.is_empty() {
        bail!("连接失败: 无响应内容");
    }
    Ok("连接成功".to_string())
}

fn extract_json(text: &str) -> Result<Value> {
    let mut stripped = text.trim().to_string();
    if let Some(start) = stripped.find("```") {
        let after = &stripped[start + 3..];
        if let Some(end) = after.find("```") {
            let inner = after[..end].trim();
            let inner = inner
                .strip_prefix("json")
                .map(|s| s.trim())
                .unwrap_or(inner);
            stripped = inner.to_string();
        }
    }
    match serde_json::from_str::<Value>(&stripped) {
        Ok(v) => Ok(v),
        Err(_) => {
            let start = stripped.find('{');
            let end = stripped.rfind('}');
            match (start, end) {
                (Some(s), Some(e)) if e > s => {
                    serde_json::from_str(&stripped[s..=e])
                        .map_err(|_| anyhow!("LLM 未返回有效 JSON: {}", truncate(text, 300)))
                }
                _ => Err(anyhow!("LLM 未返回有效 JSON: {}", truncate(text, 300))),
            }
        }
    }
}

pub async fn rewrite_merge_request(
    client: &reqwest::Client,
    config: &LLMConfig,
    mr_title: &str,
    mr_description: &str,
    commits: &[Value],
) -> Result<(String, String)> {
    let mut commit_lines = Vec::new();
    for c in commits {
        let raw = c.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let lines: Vec<&str> = raw.trim().lines().collect();
        let first = lines.first().copied().unwrap_or("");
        let body = if lines.len() > 1 {
            lines[1..].join("\n").trim().to_string()
        } else {
            String::new()
        };
        let short_id = c.get("short_id").and_then(|v| v.as_str()).unwrap_or("");
        let mut line = format!("- [{short_id}] {first}");
        if !body.is_empty() {
            let clipped: String = body.chars().take(200).collect();
            line.push_str(&format!("\n  {clipped}"));
        }
        commit_lines.push(line);
    }

    let system_prompt = "你是一名资深软件工程师，擅长根据 commit 信息撰写规范的中文 Merge Request。\
        输出要求：\
        1. title：简洁概括改动主题，不超过 50 字，保留原有项目前缀风格（如 feat/fix/refactor 前缀）\
        2. description：使用 Markdown，包含「改动内容」要点列表，必要时补充说明\
        3. 必须只输出一个 JSON 对象，格式为 {\"title\": \"...\", \"description\": \"...\"}";

    let commits_text = if commit_lines.is_empty() {
        "(无)".to_string()
    } else {
        commit_lines.join("\n")
    };
    let user_message = format!(
        "当前 MR 标题: {mr_title}\n当前 MR 描述: {}\n\n关联 commits:\n{commits_text}\n\n请基于以上 commit 信息重写该 MR 的 title 和 description。",
        if mr_description.is_empty() { "(空)" } else { mr_description }
    );

    let content = chat(client, config, system_prompt, &user_message, 120).await?;
    let data = extract_json(&content)?;
    let title = data
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let description = data
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if title.is_empty() {
        bail!("LLM 未生成有效的 title");
    }
    Ok((title, description))
}

fn truncate(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        out.push_str("...");
    }
    out
}
