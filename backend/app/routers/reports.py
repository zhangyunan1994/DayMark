from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session

from .. import models, schemas
from ..database import get_db
from ..schemas import GenerateReportRequest, ReportOut, TestConnectionResult
from ..services import gitlab, llm
from ..models import utc8_now
from ..settings_store import active_llm_config, load_settings

router = APIRouter(prefix="/api/reports", tags=["reports"])

TYPE_LABEL = {"daily": "日报", "weekly": "周报", "monthly": "月报"}


def _task_lines(tasks: list[models.Task]) -> list[str]:
    lines = []
    for t in tasks:
        status = {"todo": "[待办]", "inProgress": "[进行中]", "review": "[审核中]", "done": "[已完成]"}.get(
            t.status, "[待办]"
        )
        due = f"（截止 {t.due_date}）" if t.due_date else ""
        subtask = ""
        if t.subtasks:
            done = sum(1 for s in t.subtasks if s.get("completed")) if isinstance(t.subtasks, list) else 0
            subtask = f"（子任务 {done}/{len(t.subtasks)}）" if isinstance(t.subtasks, list) else ""
        lines.append(f"{status} {t.title}{due}{subtask}".strip())
    return lines


def _gitlab_mr_lines(mrs: list[dict]) -> list[str]:
    lines = []
    for mr in mrs:
        state = "[已合并]" if mr.get("merged_at") else "[审核中]" if mr.get("state") == "merged" else "[进行中]" if mr.get("state") == "opened" else f"[{mr.get('state')}]"
        source = mr.get("source_branch") or ""
        target = mr.get("target_branch") or ""
        branch = f"（{source} → {target}）" if source and target else ""
        lines.append(f"{state} {mr.get('title')}{branch}".strip())
    return lines


@router.post("/generate", response_model=ReportOut)
async def generate_report(req: GenerateReportRequest, db: Session = Depends(get_db)):
    settings = load_settings(db)
    config = active_llm_config(db)

    now = utc8_now()
    current_date = now.strftime("%Y-%m-%d")
    weekday = ["星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日"][now.weekday()]

    type_label = TYPE_LABEL.get(req.report_type, "报告")

    # Collect work records
    tasks = db.query(models.Task).all()
    todos = _task_lines(tasks)

    gitlab_mrs: list[str] = []
    if req.include_gitlab and settings.gitlab.token:
        try:
            mrs = await gitlab.list_merge_requests(
                settings.gitlab,
                updated_after=f"{req.date_start}T00:00:00Z",
                updated_before=f"{req.date_end}T23:59:59Z",
            )
            gitlab_mrs = _gitlab_mr_lines(mrs)
        except Exception as e:  # noqa: BLE001
            gitlab_mrs = [f"（获取 GitLab 数据失败：{e}）"]

    title = f"{req.date_start} 至 {req.date_end} {type_label}"

    if config:
        sections_text = ""
        if req.template_sections:
            numbered = "\n".join(f"{i + 1}. {s}" for i, s in enumerate(req.template_sections))
            sections_text = f"\n模板结构：\n{numbered}"
        system_prompt = (
            f"你是一个专业的工作汇报助手。当前日期：{current_date}，今天是{weekday}。"
            f"请根据用户提供的工作记录，生成一份{type_label}。\n\n"
            "要求：\n"
            "1. 使用 Markdown 格式\n"
            "2. 语言简洁专业，突出成果和价值\n"
            "3. 按照模板结构组织内容\n"
            "4. 对工作内容进行归纳总结，不要简单罗列\n"
            "5. 适当添加数据支撑（如果有）\n"
            "6. 保持真实，不要编造内容\n"
            "7. 注意时间范围，一周从星期一开始，到星期日结束\n"
            "8. 生成的报告要注意内容的完整性\n\n"
            f"模板名称：{req.template_name}\n"
            f"时间范围：{req.date_start} 至 {req.date_end}"
            f"{sections_text}"
        )
        user_message = f"请根据以下工作记录生成{type_label}：\n\n"
        if todos:
            user_message += "## 待办事项\n" + "\n".join(f"- {line}" for line in todos) + "\n\n"
        if gitlab_mrs:
            user_message += "## GitLab 合并请求\n" + "\n".join(f"- {line}" for line in gitlab_mrs) + "\n\n"
        if not todos and not gitlab_mrs:
            user_message += "（暂无具体工作记录，请根据模板生成框架内容，标注需要填写的部分）\n"
        content = await llm.chat(config, system_prompt, user_message)
    else:
        # Fallback template-based generation without LLM
        content = f"# {title}\n\n**模板**：{req.template_name}\n\n"
        if todos:
            content += "## 工作事项\n\n" + "\n".join(f"- {line}" for line in todos) + "\n\n"
        if gitlab_mrs:
            content += "## GitLab 合并请求\n\n" + "\n".join(f"- {line}" for line in gitlab_mrs) + "\n\n"

    report = models.Report(
        report_type=req.report_type,
        title=title,
        template_id=req.template_name,
        date_start=req.date_start,
        date_end=req.date_end,
        content=content,
    )
    db.add(report)
    db.commit()
    db.refresh(report)
    return report


@router.get("", response_model=list[ReportOut])
def list_reports(db: Session = Depends(get_db)):
    return db.query(models.Report).order_by(models.Report.id.desc()).all()


@router.get("/{report_id}", response_model=ReportOut)
def get_report(report_id: int, db: Session = Depends(get_db)):
    report = db.get(models.Report, report_id)
    if not report:
        raise HTTPException(status_code=404, detail="Report not found")
    return report


@router.delete("/{report_id}", status_code=204)
def delete_report(report_id: int, db: Session = Depends(get_db)):
    report = db.get(models.Report, report_id)
    if not report:
        raise HTTPException(status_code=404, detail="Report not found")
    db.delete(report)
    db.commit()