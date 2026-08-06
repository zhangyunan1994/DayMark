import uuid

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session

from ..database import get_db
from ..schemas import ReportTemplate, ReportTemplateInput
from ..settings_store import load_report_templates, save_report_templates

router = APIRouter(prefix="/api/report-templates", tags=["report-templates"])


@router.get("", response_model=list[ReportTemplate])
def list_templates(db: Session = Depends(get_db)):
    return load_report_templates(db)


@router.post("", response_model=ReportTemplate, status_code=201)
def create_template(payload: ReportTemplateInput, db: Session = Depends(get_db)):
    templates = load_report_templates(db)
    template = ReportTemplate(id=uuid.uuid4().hex[:12], **payload.model_dump())
    templates.append(template.model_dump())
    save_report_templates(db, templates)
    return template


@router.put("/{template_id}", response_model=ReportTemplate)
def update_template(template_id: str, payload: ReportTemplateInput, db: Session = Depends(get_db)):
    templates = load_report_templates(db)
    for i, t in enumerate(templates):
        if t.get("id") == template_id:
            updated = ReportTemplate(id=template_id, **payload.model_dump())
            templates[i] = updated.model_dump()
            save_report_templates(db, templates)
            return updated
    raise HTTPException(status_code=404, detail="模版不存在")


@router.delete("/{template_id}", status_code=204)
def delete_template(template_id: str, db: Session = Depends(get_db)):
    templates = load_report_templates(db)
    new_templates = [t for t in templates if t.get("id") != template_id]
    if len(new_templates) == len(templates):
        raise HTTPException(status_code=404, detail="模版不存在")
    save_report_templates(db, new_templates)
