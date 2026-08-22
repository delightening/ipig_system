"""R32-A8i: backend handler 預先組好的 audit log payload → `AuditLogPayload`。

Backend `export_activity_logs_pdf` (handlers/audit.rs) 端組 meta + summary +
entries[] + signature 後 POST 過來；adapter 主要做型別轉換 + None 防禦。
"""

from __future__ import annotations

from typing import Any

from schemas.audit_log import (
    AuditLogEntry,
    AuditLogMeta,
    AuditLogPayload,
    AuditLogSignature,
    AuditLogSummary,
)


def _entry(raw: Any) -> AuditLogEntry:
    if not isinstance(raw, dict):
        return AuditLogEntry()
    return AuditLogEntry(
        timestamp=str(raw.get("timestamp") or ""),
        user=str(raw.get("user") or ""),
        action=str(raw.get("action") or ""),
        resource=str(raw.get("resource") or ""),
        ip=str(raw.get("ip") or ""),
        change_summary=str(raw.get("change_summary") or ""),
    )


def _as_dict(v: Any) -> dict[str, Any]:
    return v if isinstance(v, dict) else {}


def _as_int(v: Any) -> int:
    try:
        return int(v)
    except (TypeError, ValueError):
        return 0


def from_export_data(data: dict[str, Any]) -> AuditLogPayload:
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict, got {type(data).__name__}")
    meta_raw = _as_dict(data.get("meta"))
    summary_raw = _as_dict(data.get("summary"))
    signature_raw = _as_dict(data.get("signature"))
    entries_raw = data.get("entries")
    return AuditLogPayload(
        meta=AuditLogMeta(
            system_name=str(meta_raw.get("system_name") or "豬博士動物實驗管理系統"),
            period_from=str(meta_raw.get("period_from") or ""),
            period_to=str(meta_raw.get("period_to") or ""),
            exported_by=str(meta_raw.get("exported_by") or ""),
            export_time=str(meta_raw.get("export_time") or ""),
        ),
        summary=AuditLogSummary(
            total_count=_as_int(summary_raw.get("total_count")),
            user_count=_as_int(summary_raw.get("user_count")),
            failure_count=_as_int(summary_raw.get("failure_count")),
            admin_count=_as_int(summary_raw.get("admin_count")),
        ),
        entries=[_entry(e) for e in entries_raw] if isinstance(entries_raw, list) else [],
        signature=AuditLogSignature(
            admin_name=str(signature_raw.get("admin_name") or ""),
            admin_signature=str(signature_raw.get("admin_signature") or ""),
        ),
    )
