"""R32-A3: 系統稽核日誌報表 docxtpl payload schema。

對應模板：`templates/audit_log.docx`
對應 mapping：`pdf-service/scripts/mappings/audit_log.py`
原始 skeleton：`templates/audit_log_skeleton.docx`（python-docx 程式化建立）

用途：GLP / IACUC 稽核時匯出系統 `user_activity_logs` 紀錄為正式報表。
資料來源（Rust 端）：`UserActivityLog` model + 相關 metadata（HMAC chain 已有）。

結構：
- T0 報表 metadata（系統 / 期間 / 匯出者 / 匯出時間）
- T1 統計摘要（總筆數 / 使用者數 / 失敗事件數 / 管理操作數）
- T2 稽核紀錄 loop（時間 / 使用者 / 動作 / 對象 / IP / 變更摘要）
- T3 系統管理員簽章
"""

from typing import Optional

from pydantic import BaseModel, Field


class AuditLogMeta(BaseModel):
    """T0 報表 metadata。"""

    system_name: str = "豬博士動物實驗管理系統"
    period_from: str = ""           # YYYY-MM-DD HH:MM
    period_to: str = ""
    exported_by: str = ""           # 匯出者使用者名稱
    export_time: str = ""           # 匯出當下時間


class AuditLogSummary(BaseModel):
    """T1 統計摘要。"""

    total_count: int = 0
    user_count: int = 0
    failure_count: int = 0          # event_type 含 FAILED / DENIED 等
    admin_count: int = 0            # 系統管理員執行的操作


class AuditLogEntry(BaseModel):
    """T2 一筆稽核紀錄。"""

    timestamp: str = ""             # YYYY-MM-DD HH:MM:SS
    user: str = ""                  # 使用者名稱（系統使用者 / SYSTEM / Anonymous）
    action: str = ""                # event_type（CREATE_ANIMAL / LOGIN_FAILED 等）
    resource: str = ""              # 操作對象（如 animals/{id}）
    ip: str = ""                    # 來源 IP
    change_summary: str = ""        # before/after diff 摘要


class AuditLogSignature(BaseModel):
    """T3 系統管理員簽章。"""

    admin_name: str = ""
    admin_signature: str = ""       # 「簽章 / 日期」整段


class AuditLogPayload(BaseModel):
    """系統稽核日誌報表完整 payload。"""

    meta: AuditLogMeta = Field(default_factory=AuditLogMeta)
    summary: AuditLogSummary = Field(default_factory=AuditLogSummary)
    entries: list[AuditLogEntry] = Field(default_factory=list)
    signature: AuditLogSignature = Field(default_factory=AuditLogSignature)
