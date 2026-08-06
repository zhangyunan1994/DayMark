from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session

from ..database import get_db
from ..schemas import SettingsOut
from ..settings_store import load_settings, save_settings

router = APIRouter(prefix="/api/settings", tags=["settings"])


@router.get("", response_model=SettingsOut)
def get_settings_route(db: Session = Depends(get_db)):
    return load_settings(db)


@router.put("", response_model=SettingsOut)
def put_settings_route(payload: SettingsOut, db: Session = Depends(get_db)):
    return save_settings(db, payload)
