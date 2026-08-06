from fastapi import APIRouter
from pydantic import BaseModel

from ..schemas import LLMConfig, TestConnectionResult
from ..services import llm

router = APIRouter(prefix="/api/llm", tags=["llm"])


class TestRequest(BaseModel):
    config: LLMConfig


@router.post("/test", response_model=TestConnectionResult)
async def test_connection(req: TestRequest):
    try:
        message = await llm.test_connection(req.config)
        return TestConnectionResult(ok=True, message=message)
    except Exception as e:  # noqa: BLE001
        return TestConnectionResult(ok=False, message=str(e))
