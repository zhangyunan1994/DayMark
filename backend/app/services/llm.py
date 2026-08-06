from __future__ import annotations

import json
import re

import httpx

from ..schemas import LLMConfig


def chat_endpoint(base_url: str) -> str:
    base = base_url.rstrip("/")
    if base.endswith("/v1"):
        return f"{base}/chat/completions"
    return f"{base}/v1/chat/completions"


def _extract_content(choice: dict) -> str:
    msg = choice.get("message") or {}
    for key in ("content", "reasoning_content"):
        value = msg.get(key)
        if value:
            return value
    delta = choice.get("delta") or {}
    value = delta.get("content")
    if value:
        return value
    return ""


async def chat(config: LLMConfig, system_prompt: str, user_message: str, timeout: float = 120) -> str:
    request = {
        "model": config.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_message},
        ],
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
    }

    headers = {"Content-Type": "application/json"}
    if config.api_key:
        headers["Authorization"] = f"Bearer {config.api_key}"

    async with httpx.AsyncClient(timeout=timeout) as client:
        resp = await client.post(
            chat_endpoint(config.api_base_url), json=request, headers=headers
        )
        body = resp.text

    if resp.status_code != 200:
        raise RuntimeError(f"LLM 接口错误 ({resp.status_code}): {body[:500]}")

    try:
        data = resp.json()
    except ValueError as e:
        raise RuntimeError(f"LLM 响应解析失败: {body[:500]}") from e

    if data.get("error"):
        raise RuntimeError(f"API 错误: {data['error'].get('message', data['error'])}")

    for choice in data.get("choices", []):
        content = _extract_content(choice)
        if content:
            return content
    raise RuntimeError("无响应内容")


async def test_connection(config: LLMConfig) -> str:
    try:
        result = await chat(config, "You are a helpful assistant.", "Hello")
        return "连接成功"
    except Exception as e:  # noqa: BLE001
        raise RuntimeError(f"连接失败: {e}") from e


def _extract_json(text: str) -> dict:
    stripped = text.strip()
    fenced = re.search(r"```(?:json)?\s*(.*?)```", stripped, re.DOTALL)
    if fenced:
        stripped = fenced.group(1).strip()
    try:
        data = json.loads(stripped)
    except ValueError:
        start = stripped.find("{")
        end = stripped.rfind("}")
        if start == -1 or end <= start:
            raise RuntimeError(f"LLM 未返回有效 JSON: {text[:300]}")
        data = json.loads(stripped[start : end + 1])
    if not isinstance(data, dict):
        raise RuntimeError(f"LLM 返回格式异常: {text[:300]}")
    return data


async def rewrite_merge_request(
    config: LLMConfig,
    mr_title: str,
    mr_description: str,
    commits: list[dict],
) -> tuple[str, str]:
    commit_lines = []
    for c in commits:
        message = (c.get("message") or "").strip().split("\n")
        first = message[0] if message else ""
        body = "\n".join(message[1:]).strip() if len(message) > 1 else ""
        line = f"- [{c.get('short_id', '')}] {first}"
        if body:
            line += f"\n  {body[:200]}"
        commit_lines.append(line)

    system_prompt = (
        "你是一名资深软件工程师，擅长根据 commit 信息撰写规范的中文 Merge Request。"
        "输出要求：\n"
        "1. title：简洁概括改动主题，不超过 50 字，保留原有项目前缀风格（如 feat/fix/refactor 前缀）\n"
        "2. description：使用 Markdown，包含「改动内容」要点列表，必要时补充说明\n"
        "3. 必须只输出一个 JSON 对象，格式为 {\"title\": \"...\", \"description\": \"...\"}"
    )
    user_message = (
        f"当前 MR 标题: {mr_title}\n"
        f"当前 MR 描述: {mr_description or '(空)'}\n\n"
        f"关联 commits:\n{chr(10).join(commit_lines) or '(无)'}\n\n"
        "请基于以上 commit 信息重写该 MR 的 title 和 description。"
    )
    content = await chat(config, system_prompt, user_message)
    data = _extract_json(content)
    title = str(data.get("title") or "").strip()
    description = str(data.get("description") or "").strip()
    if not title:
        raise RuntimeError("LLM 未生成有效的 title")
    return title, description