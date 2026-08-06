from datetime import datetime, timedelta, timezone

from sqlalchemy import Boolean, DateTime, JSON, String, Text
from sqlalchemy.orm import Mapped, mapped_column

from .database import Base

TZ_CN = timezone(timedelta(hours=8))


def utc8_now() -> datetime:
    return datetime.now(TZ_CN).replace(tzinfo=None)


class Task(Base):
    __tablename__ = "tasks"

    id: Mapped[int] = mapped_column(primary_key=True, index=True)
    title: Mapped[str] = mapped_column(String(200))
    description: Mapped[str] = mapped_column(String(2000), default="")
    status: Mapped[str] = mapped_column(String(20), default="todo")  # todo / inProgress / review / done
    priority: Mapped[str] = mapped_column(String(10), default="medium")  # low / medium / high
    assignee: Mapped[str] = mapped_column(String(50), default="")
    due_date: Mapped[str | None] = mapped_column(String(16), nullable=True)  # YYYY-MM-DD 或 YYYY-MM-DD HH:mm
    urgent: Mapped[bool] = mapped_column(Boolean, default=False)
    important: Mapped[bool] = mapped_column(Boolean, default=False)
    subtasks: Mapped[list] = mapped_column(JSON, default=list)  # [{id, title, completed}]
    created_at: Mapped[datetime] = mapped_column(DateTime, default=utc8_now)
    updated_at: Mapped[datetime] = mapped_column(DateTime, default=utc8_now, onupdate=utc8_now)


class Setting(Base):
    __tablename__ = "settings"

    key: Mapped[str] = mapped_column(String(50), primary_key=True)
    value: Mapped[dict] = mapped_column(JSON, default=dict)


class Report(Base):
    __tablename__ = "reports"

    id: Mapped[int] = mapped_column(primary_key=True, index=True)
    report_type: Mapped[str] = mapped_column(String(20))  # daily / weekly / monthly
    title: Mapped[str] = mapped_column(String(200))
    template_id: Mapped[str] = mapped_column(String(50), default="")
    date_start: Mapped[str | None] = mapped_column(String(10), nullable=True)
    date_end: Mapped[str | None] = mapped_column(String(10), nullable=True)
    content: Mapped[str] = mapped_column(Text, default="")
    created_at: Mapped[datetime] = mapped_column(DateTime, default=utc8_now)
