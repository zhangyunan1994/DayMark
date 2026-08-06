from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.orm import Session

from ..database import get_db
from ..schemas import GitLabConfig, TestConnectionResult
from ..services import gitlab, llm
from ..settings_store import active_llm_config, load_settings

router = APIRouter(prefix="/api/gitlab", tags=["gitlab"])


class TestRequest(BaseModel):
    config: GitLabConfig


class MRQuery(BaseModel):
    config: GitLabConfig
    updated_after: str | None = None
    updated_before: str | None = None
    per_page: int = 2000


class RewriteMRRequest(BaseModel):
    project_id: int
    merge_request_iid: int


def _saved_config(db: Session) -> GitLabConfig:
    settings = load_settings(db)
    if not settings.gitlab.token:
        raise HTTPException(status_code=400, detail="未配置 GitLab Token，请先在设置中配置")
    return settings.gitlab


@router.post("/test", response_model=TestConnectionResult)
async def test_connection(req: TestRequest):
    try:
        message = await gitlab.test_connection(req.config)
        return TestConnectionResult(ok=True, message=message)
    except Exception as e:  # noqa: BLE001
        return TestConnectionResult(ok=False, message=str(e))


@router.get("/projects")
async def list_projects(db: Session = Depends(get_db)):
    config = _saved_config(db)
    try:
        return await gitlab.list_projects(config)
    except Exception as e:  # noqa: BLE001
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/merge-requests")
async def list_merge_requests(req: MRQuery, db: Session = Depends(get_db)):
    try:
        return await gitlab.list_merge_requests(
            req.config, updated_after=req.updated_after, updated_before=req.updated_before, per_page=req.per_page
        )
    except Exception as e:  # noqa: BLE001
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.get("/merge-requests")
async def list_merge_requests_saved(db: Session = Depends(get_db)):
    config = _saved_config(db)
    try:
        return await gitlab.list_merge_requests(config)
    except Exception as e:  # noqa: BLE001
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/mr/rewrite")
async def rewrite_merge_request(req: RewriteMRRequest, db: Session = Depends(get_db)):
    config = _saved_config(db)
    llm_config = active_llm_config(db)
    if not llm_config:
        raise HTTPException(status_code=400, detail="未配置 LLM，请先在设置中配置 LLM")
    try:
        mr = await gitlab.get_merge_request(config, req.project_id, req.merge_request_iid)
        if mr.get("state") != "opened":
            raise HTTPException(status_code=400, detail="只有 opened 状态的 MR 才能重写")
        commits = await gitlab.list_merge_request_commits(config, req.project_id, req.merge_request_iid)
        title, description = await llm.rewrite_merge_request(
            llm_config, mr.get("title", ""), mr.get("description", "") or "", commits
        )
        updated = await gitlab.update_merge_request(
            config, req.project_id, req.merge_request_iid, title, description
        )
        return {
            "title": updated.get("title", title),
            "description": updated.get("description", description),
            "web_url": updated.get("web_url"),
        }
    except HTTPException:
        raise
    except Exception as e:  # noqa: BLE001
        raise HTTPException(status_code=502, detail=str(e)) from e
