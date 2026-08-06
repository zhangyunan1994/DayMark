from __future__ import annotations

import httpx

from ..schemas import GitLabConfig


class GitLabError(RuntimeError):
    pass


def _headers(config: GitLabConfig) -> dict:
    headers = {"Content-Type": "application/json"}
    if config.token:
        headers["PRIVATE-TOKEN"] = config.token
    return headers


def _base(config: GitLabConfig) -> str:
    return config.base_url.rstrip("/")


async def _get(config: GitLabConfig, path: str, params: dict | None = None) -> list | dict:
    async with httpx.AsyncClient(timeout=30) as client:
        resp = await client.get(
            f"{_base(config)}{path}", params=params, headers=_headers(config)
        )
    if resp.status_code != 200:
        raise GitLabError(f"GitLab 请求失败 ({resp.status_code}): {resp.text[:300]}")
    try:
        return resp.json()
    except ValueError as e:
        raise GitLabError(f"GitLab 响应解析失败: {resp.text[:300]}") from e


async def _put(config: GitLabConfig, path: str, payload: dict) -> dict:
    async with httpx.AsyncClient(timeout=30) as client:
        resp = await client.put(
            f"{_base(config)}{path}", json=payload, headers=_headers(config)
        )
    if resp.status_code != 200:
        raise GitLabError(f"GitLab 更新失败 ({resp.status_code}): {resp.text[:300]}")
    try:
        return resp.json()
    except ValueError as e:
        raise GitLabError(f"GitLab 响应解析失败: {resp.text[:300]}") from e


async def test_connection(config: GitLabConfig) -> str:
    try:
        data = await _get(config, "/api/v4/user")
        return f"连接成功（{data.get('username', '')}）"
    except Exception as e:  # noqa: BLE001
        raise RuntimeError(f"连接失败: {e}") from e


async def list_projects(config: GitLabConfig) -> list[dict]:
    data = await _get(config, "/api/v4/projects", {"membership": True, "per_page": 100})
    return [p for p in data if isinstance(p, dict)] if isinstance(data, list) else []


async def list_merge_requests(
    config: GitLabConfig,
    updated_after: str | None = None,
    updated_before: str | None = None,
    state: str = "all",
    per_page: int = 2000,
) -> list[dict]:
    params: dict = {
        "state": state,
        "scope": "all",
        "order_by": "updated_at",
        "per_page": per_page,
    }
    if config.username:
        params["author_username"] = config.username
    if updated_after:
        params["updated_after"] = updated_after
    if updated_before:
        params["updated_before"] = updated_before
    data = await _get(config, "/api/v4/merge_requests", params)
    return [m for m in data if isinstance(m, dict)] if isinstance(data, list) else []


async def get_merge_request(config: GitLabConfig, project_id: int, merge_request_iid: int) -> dict:
    data = await _get(config, f"/api/v4/projects/{project_id}/merge_requests/{merge_request_iid}")
    if not isinstance(data, dict):
        raise GitLabError("GitLab 返回了异常数据")
    return data


async def list_merge_request_commits(
    config: GitLabConfig, project_id: int, merge_request_iid: int
) -> list[dict]:
    data = await _get(
        config, f"/api/v4/projects/{project_id}/merge_requests/{merge_request_iid}/commits"
    )
    return [c for c in data if isinstance(c, dict)] if isinstance(data, list) else []


async def update_merge_request(
    config: GitLabConfig, project_id: int, merge_request_iid: int, title: str, description: str
) -> dict:
    return await _put(
        config,
        f"/api/v4/projects/{project_id}/merge_requests/{merge_request_iid}",
        {"title": title, "description": description},
    )