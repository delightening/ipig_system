"""Schemas — copied from `pdf-service/app/schemas/` so this service is drop-in.

If you change the source-of-truth schemas in pdf-service, copy them here too
(or replace with `pip install` shared package once that exists).
"""
from .audit_log import AuditLogPayload
from .aup_protocol import AupProtocolPayload
from .blood_test import BloodTestPayload
from .medical_record import MedicalRecordPayload
from .pig_approval import PigApprovalPayload
from .review_reply import ReviewReplyPayload
from .review_result import ReviewResultPayload
from .surgery import SurgeryPayload
from .vet_patrol import VetPatrolPayload
from .vet_patrol_report import VetPatrolReportPayload
from .warehouse import WarehouseReportPayload

__all__ = [
    "AuditLogPayload",
    "AupProtocolPayload",
    "BloodTestPayload",
    "MedicalRecordPayload",
    "PigApprovalPayload",
    "ReviewReplyPayload",
    "ReviewResultPayload",
    "SurgeryPayload",
    "VetPatrolPayload",
    "VetPatrolReportPayload",
    "WarehouseReportPayload",
]
