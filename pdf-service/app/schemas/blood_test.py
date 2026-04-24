from typing import Optional

from pydantic import BaseModel, Field


class BloodTestRow(BaseModel):
    test_date: str
    lab_name: Optional[str] = None
    item_count: int = 0
    abnormal_count: int = 0
    status: Optional[str] = None
    created_by_name: Optional[str] = None


class BloodTestPayload(BaseModel):
    animal_ear_tag: str
    animal_iacuc_no: Optional[str] = None
    export_date: str
    tests: list[BloodTestRow] = Field(default_factory=list)
