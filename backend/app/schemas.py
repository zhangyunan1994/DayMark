from datetime import datetime

from pydantic import BaseModel, ConfigDict, Field


class LLMConfig(BaseModel):
    id: str | None = None
    provider_name: str = Field(min_length=1, max_length=50)
    api_base_url: str = Field(min_length=1)
    api_key: str = Field(default="", max_length=500)
    model: str = Field(min_length=1, max_length=100)
    temperature: float = Field(default=0.7, ge=0, le=2)
    max_tokens: int = Field(default=2000, ge=100)
    is_active: bool = False


class GitLabConfig(BaseModel):
    base_url: str = Field(default="https://gitlab.com", min_length=1)
    username: str = ""
    token: str = ""


class SettingsOut(BaseModel):
    llm_configs: list[LLMConfig] = Field(default_factory=list)
    gitlab: GitLabConfig = GitLabConfig()


class ReportCreate(BaseModel):
    report_type: str = Field(pattern="^(daily|weekly|monthly)$")
    title: str = Field(min_length=1, max_length=200)
    template_id: str = ""
    date_start: str | None = None
    date_end: str | None = None
    content: str = ""


class ReportOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    report_type: str
    title: str
    template_id: str
    date_start: str | None
    date_end: str | None
    content: str
    created_at: datetime


class GenerateReportRequest(BaseModel):
    report_type: str = Field(default="weekly", pattern="^(daily|weekly|monthly)$")
    template_name: str = Field(min_length=1, max_length=100)
    template_sections: list[str] = Field(default_factory=list)
    date_start: str = Field(pattern=r"^\d{4}-\d{2}-\d{2}$")
    date_end: str = Field(pattern=r"^\d{4}-\d{2}-\d{2}$")
    include_gitlab: bool = True


class TestConnectionResult(BaseModel):
    ok: bool
    message: str = ""


class ReportTemplate(BaseModel):
    id: str
    report_type: str = Field(pattern="^(daily|weekly|monthly)$")
    name: str = Field(min_length=1, max_length=50)
    description: str = Field(default="", max_length=200)
    sections: list[str] = Field(default_factory=list)


class ReportTemplateInput(BaseModel):
    report_type: str = Field(pattern="^(daily|weekly|monthly)$")
    name: str = Field(min_length=1, max_length=50)
    description: str = Field(default="", max_length=200)
    sections: list[str] = Field(default_factory=list)


# Re-export task schemas keep working
class Subtask(BaseModel):
    id: str
    title: str = Field(min_length=1, max_length=200)
    completed: bool = False


class TaskBase(BaseModel):
    title: str = Field(min_length=1, max_length=200)
    description: str = Field(default="", max_length=2000)
    status: str = Field(default="todo", pattern="^(todo|inProgress|review|done)$")
    priority: str = Field(default="medium", pattern="^(low|medium|high)$")
    assignee: str = Field(default="", max_length=50)
    due_date: str | None = Field(default=None, pattern=r"^\d{4}-\d{2}-\d{2}( \d{2}:\d{2})?$")
    urgent: bool = False
    important: bool = False
    subtasks: list[Subtask] = Field(default_factory=list)


class TaskCreate(TaskBase):
    pass


class TaskUpdate(BaseModel):
    title: str | None = Field(default=None, min_length=1, max_length=200)
    description: str | None = Field(default=None, max_length=2000)
    status: str | None = Field(default=None, pattern="^(todo|inProgress|review|done)$")
    priority: str | None = Field(default=None, pattern="^(low|medium|high)$")
    assignee: str | None = Field(default=None, max_length=50)
    due_date: str | None = Field(default=None, pattern=r"^\d{4}-\d{2}-\d{2}( \d{2}:\d{2})?$")
    urgent: bool | None = None
    important: bool | None = None
    subtasks: list[Subtask] | None = None


class TaskOut(TaskBase):
    model_config = ConfigDict(from_attributes=True)

    id: int
    created_at: datetime
    updated_at: datetime
