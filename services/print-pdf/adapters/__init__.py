"""Adapters — translate raw backend JSON (DB query shape) → schema."""
from . import (
    aup_protocol,
    audit_log,
    blood_test,
    medical_record,
    review_reply,
    review_result,
    surgery,
    vet_patrol,
    vet_patrol_report,
    warehouse,
)

__all__ = [
    "aup_protocol",
    "audit_log",
    "blood_test",
    "medical_record",
    "review_reply",
    "review_result",
    "surgery",
    "vet_patrol",
    "vet_patrol_report",
    "warehouse",
]
