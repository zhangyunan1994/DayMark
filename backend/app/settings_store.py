from . import models
from .schemas import GitLabConfig, LLMConfig, SettingsOut


def load_settings(db) -> SettingsOut:
    rows = {s.key: s.value for s in db.query(models.Setting).all()}
    llm_configs = [
        LLMConfig.model_validate(c) for c in rows.get("llm_configs", []) if isinstance(c, dict)
    ]
    gitlab = rows.get("gitlab") or {}
    return SettingsOut(
        llm_configs=llm_configs,
        gitlab=GitLabConfig.model_validate(gitlab) if isinstance(gitlab, dict) else GitLabConfig(),
    )


def save_settings(db, payload: SettingsOut) -> SettingsOut:
    for key, value in {
        "llm_configs": [c.model_dump() for c in payload.llm_configs],
        "gitlab": payload.gitlab.model_dump(),
    }.items():
        row = db.get(models.Setting, key)
        if row:
            row.value = value
        else:
            db.add(models.Setting(key=key, value=value))
    db.commit()
    return load_settings(db)


def active_llm_config(db) -> LLMConfig | None:
    settings = load_settings(db)
    for cfg in settings.llm_configs:
        if getattr(cfg, "is_active", False):
            return cfg
    return settings.llm_configs[-1] if settings.llm_configs else None


def load_report_templates(db) -> list[dict]:
    row = db.get(models.Setting, "report_templates")
    if row and isinstance(row.value, list):
        return row.value
    return []


def save_report_templates(db, templates: list[dict]) -> None:
    row = db.get(models.Setting, "report_templates")
    if row:
        row.value = templates
    else:
        db.add(models.Setting(key="report_templates", value=templates))
    db.commit()
