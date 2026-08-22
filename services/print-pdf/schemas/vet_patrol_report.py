"""R39: 獸醫巡場報告 docx 渲染 payload schema.

替代原本走 backend `vet_patrol_report.html` + Gotenberg HTML→PDF 路徑。
與 `vet_patrol.py`（欄位狀態表 xlsx）不同 — 這是巡場觀察報告 docx 範本。

Photos 為 entry-level（每筆觀察可掛多張）+ report-level（整體環境照）。
photos 欄位的 `src` 為 backend 端讀檔後 base64 編碼的 data URL；adapter 端
解碼為 BytesIO 再包成 docxtpl `InlineImage` 注入範本（範本用
`{%p for pair in cat.photos | batch(2) %}` nested loop 排版）。
"""

from __future__ import annotations

from pydantic import BaseModel, Field


class PhotoEntry(BaseModel):
    """單張照片：src 為 data URL（如 'data:image/jpeg;base64,...'）。"""

    src: str
    caption: str = ""


class PhotoGroup(BaseModel):
    """同隻豬（或同個來源）的照片群組。caption 通常為 '#耳號' 或整體照說明。"""

    caption: str = ""
    description: str = ""
    # 群組層級的 src 清單。**已被 `photos` 取代**，保留是為了相容尚未同步部署的 backend
    # （backend 與本服務非原子部署）。兩者都有時以 `photos` 為準。
    srcs: list[str] = Field(default_factory=list)
    # 每張照片各自的說明。`srcs` 只是字串陣列、結構上放不下 per-photo caption，
    # 導致使用者在前端填的圖片說明印不進 PDF（同組每張都只印同一行耳號）。
    photos: list[PhotoEntry] = Field(default_factory=list)


class SubEntry(BaseModel):
    """單筆觀察（每隻豬一行）。"""

    observation: str = ""
    suggestion: str = ""
    follow_up: str = ""


class EntryRow(BaseModel):
    """單個類別（豬隻狀況 / 防疫及消毒計畫 / 病歷紀錄 / 其他）。"""

    label: str
    observation: str = ""
    suggestion: str = ""
    follow_up: str = ""
    entries: list[SubEntry] = Field(default_factory=list)
    photos: list[PhotoEntry] = Field(default_factory=list)


class VetPatrolReportPayload(BaseModel):
    """巡場報告 docx 渲染 payload."""

    # 基本資訊
    vet_name: str = ""
    companion: str = ""
    patrol_date: str  # 'YYYY-MM-DD'
    patrol_date_display: str = ""  # 'YYYY年MM月DD日'

    # 4 類觀察條目（順序固定）
    categories: list[EntryRow] = Field(default_factory=list)

    # 整體環境照（report-level，無特定 entry）。
    photos: list[PhotoEntry] = Field(default_factory=list)

    # R39-D2：以「同隻豬一組」為單位的照片群組。每組對應一個 entry（或一張整體照），
    # 內含多張 srcs（data URL）。Adapter 解碼成 list[InlineImage]，範本用
    # `{%p for pair in photo_groups | batch(2) %}` 配對成兩格一列，每格內再用
    # 內層 loop 把同組多張照片疊在同一 cell。
    photo_groups: list[PhotoGroup] = Field(default_factory=list)
