from typing import Callable

from pydantic import BaseModel

from .schemas.blood_test import BloodTestPayload


class DocType(BaseModel):
    template: str
    schema_cls: type[BaseModel]
    filename_fn: Callable[[BaseModel], str]

    model_config = {"arbitrary_types_allowed": True}


def _blood_test_filename(payload: BaseModel) -> str:
    assert isinstance(payload, BloodTestPayload)
    iacuc = payload.animal_iacuc_no or "unassigned"
    return f"blood_test_{iacuc}_{payload.animal_ear_tag}.pdf"


REGISTRY: dict[str, DocType] = {
    "blood_test": DocType(
        template="blood_test.html",
        schema_cls=BloodTestPayload,
        filename_fn=_blood_test_filename,
    ),
}
