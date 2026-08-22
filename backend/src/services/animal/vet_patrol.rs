// 獸醫巡場報告 Service

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::audit_diff::DataDiff,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

// R40-19：R39++ 三階段流程的 status 字串常數（取代散落的 magic strings）。
// SQL 內嵌字面值（`WHERE status = 'draft'`）保留為 SQL 語法本身的一部分不替換。
pub mod status {
    pub const DRAFT: &str = "draft";
    pub const AWAITING_ACK: &str = "awaiting_acknowledgement";
    pub const AWAITING_FOLLOW_UP: &str = "awaiting_follow_up";
    pub const COMPLETED: &str = "completed";
}

// ── 報告主表 ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetPatrolReport {
    pub id: Uuid,
    pub patrol_date: NaiveDate,
    pub week_start: Option<NaiveDate>,
    pub week_end: Option<NaiveDate>,
    pub accompanying_personnel: Option<String>,
    /// R39++ 三階段流程：'draft' / 'awaiting_acknowledgement' / 'awaiting_follow_up' / 'completed'
    pub status: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 獸醫指派的追蹤者（陪同人員）
    pub follow_up_user_id: Option<Uuid>,
    /// 追蹤者按「確認收到」的時間（R39++ 三階段流程的中間 milestone）
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// 追蹤者按「確認收到」時的使用者 id（通常等於 follow_up_user_id）
    pub acknowledged_by_id: Option<Uuid>,
    /// 追蹤者按「確認完成」的時間（流程結束 milestone）
    pub follow_up_submitted_at: Option<DateTime<Utc>>,
    /// 填寫報告的獸醫顯示名（= created_by 的 users.display_name；僅 list 查詢以子查詢帶出）。
    /// 其餘查詢未 SELECT 此欄，靠 sqlx(default) 補 None。
    #[sqlx(default)]
    pub created_by_name: Option<String>,
}

// R26-9: 巡場報告含觀察、建議、後續追蹤等醫療紀錄內容，為 GLP 研究資料本身，
// 需完整保留於 audit log。空 `redacted_fields()` 是主動決策。
impl crate::models::audit_diff::AuditRedact for VetPatrolReport {}

// ── Audit snapshot：包含 report + entries（GLP 醫療紀錄完整保留） ──────────────────────────────

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct VetPatrolEntrySnapshot {
    pub id: Uuid,
    pub category: String,
    pub animal_id: Option<Uuid>,
    pub observation: String,
    pub suggestion: String,
    pub follow_up: String,
    pub sort_order: i32,
}

impl crate::models::audit_diff::AuditRedact for VetPatrolEntrySnapshot {}

#[derive(Debug, Clone, Serialize)]
pub struct VetPatrolReportSnapshot {
    #[serde(flatten)]
    pub report: VetPatrolReport,
    pub entries: Vec<VetPatrolEntrySnapshot>,
}

impl crate::models::audit_diff::AuditRedact for VetPatrolReportSnapshot {}

async fn list_photos(pool: &PgPool, report_id: Uuid) -> Result<Vec<VetPatrolPhoto>> {
    let photos = sqlx::query_as::<_, VetPatrolPhoto>(
        r#"SELECT id, report_id, file_name, file_path, file_size, mime_type,
                  caption, sort_order, created_at
           FROM vet_patrol_photos
           WHERE report_id = $1
           ORDER BY sort_order, created_at"#,
    )
    .bind(report_id)
    .fetch_all(pool)
    .await?;
    Ok(photos)
}

async fn fetch_entry_snapshots(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    report_id: Uuid,
) -> Result<Vec<VetPatrolEntrySnapshot>> {
    let entries = sqlx::query_as::<_, VetPatrolEntrySnapshot>(
        r#"SELECT id, category, animal_id, observation, suggestion, follow_up, sort_order
           FROM vet_patrol_entries
           WHERE report_id = $1
           ORDER BY sort_order, created_at"#,
    )
    .bind(report_id)
    .fetch_all(&mut **tx)
    .await?;
    Ok(entries)
}

// ── 含耳號的條目（join animals） ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetPatrolEntryWithAnimal {
    pub id: Uuid,
    pub report_id: Uuid,
    pub category: String,
    /// R39+++ 多動物支援：第一隻動物（向後相容 PDF / advice sync），實際清單見 animal_ids
    pub animal_id: Option<Uuid>,
    /// R39+++ 多動物支援：同條觀察可掛多隻
    #[sqlx(skip)]
    pub animal_ids: Vec<Uuid>,
    /// R39+++ 多動物對應耳號清單（依 animal_ids 順序）
    #[sqlx(skip)]
    pub ear_tags: Vec<String>,
    /// @deprecated 向後相容：第一隻動物耳號；前端請改用 ear_tags
    #[sqlx(default)]
    pub ear_tag: Option<String>,
    pub observation: String,
    pub suggestion: String,
    pub follow_up: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

// ── 照片附件 ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetPatrolPhoto {
    pub id: Uuid,
    pub report_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub caption: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

// ── 報告 + 條目 + 照片合併回應 ──────────────────────────────

#[derive(Debug, Serialize)]
pub struct VetPatrolReportWithEntries {
    #[serde(flatten)]
    pub report: VetPatrolReport,
    pub entries: Vec<VetPatrolEntryWithAnimal>,
    pub photos: Vec<VetPatrolPhoto>,
    /// R39: entry-level 照片（前端按 entry_id 分組顯示）
    pub entry_photos: Vec<VetPatrolEntryPhoto>,
}

// ── 請求 ──────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateVetPatrolReportRequest {
    pub patrol_date: NaiveDate,
    #[serde(default)]
    pub accompanying_personnel: Option<String>,
    pub entries: Vec<CreateVetPatrolEntryRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVetPatrolReportRequest {
    pub patrol_date: Option<NaiveDate>,
    #[serde(default)]
    pub accompanying_personnel: Option<String>,
    pub entries: Option<Vec<CreateVetPatrolEntryRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVetPatrolEntryRequest {
    /// R39: 既有 entry 的 server-assigned UUID。
    /// `Some(uuid)` → diff-based update 命中既有列、photos 保留；
    /// `None` → 新增列；
    /// 沒在 request 裡出現的既有 id → DELETE（CASCADE 連帶 entry photos）。
    #[serde(default)]
    pub id: Option<Uuid>,
    pub category: String,
    /// @deprecated 向後相容：單一動物 id；前端請改傳 animal_ids
    #[serde(default)]
    pub animal_id: Option<Uuid>,
    /// R39+++ 多動物支援：同條觀察可掛多隻；空陣列代表無動物關聯（如「其他」類別）
    #[serde(default)]
    pub animal_ids: Vec<Uuid>,
    pub observation: String,
    pub suggestion: String,
    pub follow_up: String,
    pub sort_order: Option<i32>,
}

impl CreateVetPatrolEntryRequest {
    /// R39+++ 過渡期 helper：兼容前端只傳 animal_id（單）或 animal_ids（多）。
    /// 規則：animal_ids 非空 → 用 animal_ids；空 → fallback 到 animal_id（包成單元素或空）。
    pub fn resolved_animal_ids(&self) -> Vec<Uuid> {
        if !self.animal_ids.is_empty() {
            // 去重保序
            let mut seen = std::collections::HashSet::new();
            self.animal_ids
                .iter()
                .copied()
                .filter(|id| seen.insert(*id))
                .collect()
        } else if let Some(id) = self.animal_id {
            vec![id]
        } else {
            Vec::new()
        }
    }
}

/// R39: entry-level 照片附件（每筆觀察可掛多張）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetPatrolEntryPhoto {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub caption: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

impl crate::models::audit_diff::AuditRedact for VetPatrolEntryPhoto {}

/// R39: list 用的 filter，default = submitted（一般使用者不看別人的草稿）。
/// R40-15：直接 `Deserialize` 自 query string；URL 值用 snake_case，`MyFollowUps`
/// 顯式 rename 為 `my_followups`（保持與既有前端契約一致，不引入 `my_follow_ups`）。
#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VetPatrolListFilter {
    /// 已完成的報告（三階段都簽完）
    #[default]
    Completed,
    /// 我的草稿（自己創建、還沒送追蹤者的）
    MyDrafts,
    /// 待我確認收到（指派給我為追蹤者、status=awaiting_acknowledgement）
    /// 註：前端 tab 已移除（「確認收到」改為開啟 dialog 自動觸發），保留此 filter 維持 API 相容
    MyAcknowledgements,
    /// 待我回覆追蹤改善（指派給我為追蹤者、status ∈ awaiting_acknowledgement / awaiting_follow_up；
    /// 確認收到已自動化，兩種狀態對追蹤者皆屬「待我回覆」）
    #[serde(rename = "my_followups")]
    MyFollowUps,
    All,
    /// 單一列表預設視角：「與我相關」的全部歷史報告。三段聯集：
    ///   - 我的草稿（created_by = me AND status = draft）
    ///   - 我送出或被指派的在途報告（status ∈ awaiting_*，created_by 或 follow_up_user_id = me）
    ///   - 全系統已完成（status = completed）
    ///
    /// 前端抓此 filter 後，用各 row 的 status 在 client 端切「草稿/已送出/已完成」chip。
    Relevant,
}

/// R40-18：照片下載時從 DB 取出的最小資料集
pub struct VetPatrolPhotoDownloadInfo {
    pub file_path: String,
    pub file_name: String,
    pub mime_type: String,
}

pub struct VetPatrolReportService;

impl VetPatrolReportService {
    /// R40-17：上傳前驗報告存在且未軟刪（避免照片寫到孤兒紀錄）
    pub async fn ensure_report_exists(pool: &PgPool, report_id: Uuid) -> Result<()> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM vet_patrol_reports WHERE id = $1 AND deleted_at IS NULL)",
        )
        .bind(report_id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(AppError::NotFound("找不到巡場報告".to_string()));
        }
        Ok(())
    }

    /// R40-17：上傳前驗 entry 存在且其 report 未軟刪。
    /// #363：並驗父報告未鎖定（status=completed 為 GLP 不可變醫療紀錄）——否則
    /// vet 可對已完成報告新增照片證據，繞過 update() 的 completed lock。
    pub async fn ensure_entry_exists(pool: &PgPool, entry_id: Uuid) -> Result<()> {
        let report_status: Option<String> = sqlx::query_scalar(
            r#"SELECT r.status::text FROM vet_patrol_entries e
                   JOIN vet_patrol_reports r ON r.id = e.report_id
                   WHERE e.id = $1 AND r.deleted_at IS NULL"#,
        )
        .bind(entry_id)
        .fetch_optional(pool)
        .await?;
        let report_status =
            report_status.ok_or_else(|| AppError::NotFound("找不到觀察條目或報告".to_string()))?;
        if report_status == status::COMPLETED {
            return Err(AppError::BusinessRule(
                "報告已完成（鎖定），不可再異動照片".to_string(),
            ));
        }
        Ok(())
    }

    /// R75-12 核心：巡場照片異動授權——報告未鎖定（status != completed）**且**操作者為
    /// 報告建立者（或 admin）。巡場為全場內部福利文件，限建立者異動其照片，避免任一
    /// vet.recommend 持有者污染/刪除他人報告的照片證據（GLP 完整性）。
    fn check_report_photo_writable(
        status: &str,
        created_by: Option<Uuid>,
        user: &CurrentUser,
    ) -> Result<()> {
        if status == status::COMPLETED {
            return Err(AppError::BusinessRule(
                "報告已完成（鎖定），不可再異動照片".to_string(),
            ));
        }
        if !user.is_admin() && created_by != Some(user.id) {
            return Err(AppError::Forbidden(
                "僅報告建立者可異動巡場照片".to_string(),
            ));
        }
        Ok(())
    }

    /// R75-12：依 report_id 驗照片可寫（report-level 上傳用；補 report-level 原缺的 completed-lock）。
    pub async fn ensure_report_photo_writable(
        pool: &PgPool,
        report_id: Uuid,
        user: &CurrentUser,
    ) -> Result<()> {
        let row: Option<(String, Option<Uuid>)> = sqlx::query_as(
            "SELECT status::text, created_by FROM vet_patrol_reports WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(report_id)
        .fetch_optional(pool)
        .await?;
        let (status, created_by) =
            row.ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;
        Self::check_report_photo_writable(&status, created_by, user)
    }

    /// R75-12：依 report-level photo_id 反查報告驗可寫（caption/delete 用）。
    pub async fn ensure_report_photo_writable_by_photo(
        pool: &PgPool,
        photo_id: Uuid,
        user: &CurrentUser,
    ) -> Result<()> {
        let row: Option<(String, Option<Uuid>)> = sqlx::query_as(
            r#"SELECT r.status::text, r.created_by FROM vet_patrol_photos p
               JOIN vet_patrol_reports r ON r.id = p.report_id
               WHERE p.id = $1 AND r.deleted_at IS NULL"#,
        )
        .bind(photo_id)
        .fetch_optional(pool)
        .await?;
        let (status, created_by) =
            row.ok_or_else(|| AppError::NotFound("找不到照片".to_string()))?;
        Self::check_report_photo_writable(&status, created_by, user)
    }

    /// R75-12：依 entry_id 反查報告驗可寫（entry 照片上傳用；含 completed-lock + 限建立者）。
    pub async fn ensure_entry_photo_writable(
        pool: &PgPool,
        entry_id: Uuid,
        user: &CurrentUser,
    ) -> Result<()> {
        let row: Option<(String, Option<Uuid>)> = sqlx::query_as(
            r#"SELECT r.status::text, r.created_by FROM vet_patrol_entries e
               JOIN vet_patrol_reports r ON r.id = e.report_id
               WHERE e.id = $1 AND r.deleted_at IS NULL"#,
        )
        .bind(entry_id)
        .fetch_optional(pool)
        .await?;
        let (status, created_by) =
            row.ok_or_else(|| AppError::NotFound("找不到觀察條目或報告".to_string()))?;
        Self::check_report_photo_writable(&status, created_by, user)
    }

    /// R75-12：依 entry-level photo_id 反查報告驗可寫（entry 照片 caption/delete 用）。
    pub async fn ensure_entry_photo_writable_by_photo(
        pool: &PgPool,
        photo_id: Uuid,
        user: &CurrentUser,
    ) -> Result<()> {
        let row: Option<(String, Option<Uuid>)> = sqlx::query_as(
            r#"SELECT r.status::text, r.created_by FROM vet_patrol_entry_photos ep
               JOIN vet_patrol_entries e ON e.id = ep.entry_id
               JOIN vet_patrol_reports r ON r.id = e.report_id
               WHERE ep.id = $1 AND r.deleted_at IS NULL"#,
        )
        .bind(photo_id)
        .fetch_optional(pool)
        .await?;
        let (status, created_by) =
            row.ok_or_else(|| AppError::NotFound("找不到照片".to_string()))?;
        Self::check_report_photo_writable(&status, created_by, user)
    }

    /// R40-18：下載 report-level 照片時取 metadata。
    /// JOIN vet_patrol_reports 並加 `deleted_at IS NULL` — 軟刪 report 的照片
    /// 不應仍可下載（vet_patrol_photos 是 hard FK + ON DELETE CASCADE，但軟刪
    /// 不觸發 cascade，故 photo 仍存在 → 沒此 filter 就洩漏）。
    pub async fn find_photo_for_download(
        pool: &PgPool,
        photo_id: Uuid,
    ) -> Result<VetPatrolPhotoDownloadInfo> {
        let row: Option<(String, String, String)> = sqlx::query_as(
            r#"SELECT p.file_path, p.file_name, p.mime_type
               FROM vet_patrol_photos p
               JOIN vet_patrol_reports r ON r.id = p.report_id
               WHERE p.id = $1 AND r.deleted_at IS NULL"#,
        )
        .bind(photo_id)
        .fetch_optional(pool)
        .await?;
        let (file_path, file_name, mime_type) =
            row.ok_or_else(|| AppError::NotFound("找不到照片".to_string()))?;
        Ok(VetPatrolPhotoDownloadInfo {
            file_path,
            file_name,
            mime_type,
        })
    }

    /// R40-18：下載 entry-level 照片時取 metadata（同上邏輯，多一層 JOIN entries）
    pub async fn find_entry_photo_for_download(
        pool: &PgPool,
        photo_id: Uuid,
    ) -> Result<VetPatrolPhotoDownloadInfo> {
        let row: Option<(String, String, String)> = sqlx::query_as(
            r#"SELECT ep.file_path, ep.file_name, ep.mime_type
               FROM vet_patrol_entry_photos ep
               JOIN vet_patrol_entries e ON e.id = ep.entry_id
               JOIN vet_patrol_reports r ON r.id = e.report_id
               WHERE ep.id = $1 AND r.deleted_at IS NULL"#,
        )
        .bind(photo_id)
        .fetch_optional(pool)
        .await?;
        let (file_path, file_name, mime_type) =
            row.ok_or_else(|| AppError::NotFound("找不到照片".to_string()))?;
        Ok(VetPatrolPhotoDownloadInfo {
            file_path,
            file_name,
            mime_type,
        })
    }

    /// 列出巡場報告（不含條目），R39+ 兩階段流程 filter
    pub async fn list(
        pool: &PgPool,
        filter: VetPatrolListFilter,
        current_user_id: Uuid,
    ) -> Result<Vec<VetPatrolReport>> {
        let reports = match filter {
            VetPatrolListFilter::Completed => {
                sqlx::query_as::<_, VetPatrolReport>(
                    r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                              created_by, updated_by, created_at, updated_at,
                              follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at,
                              (SELECT u.display_name FROM users u WHERE u.id = vet_patrol_reports.created_by) AS created_by_name
                       FROM vet_patrol_reports
                       WHERE deleted_at IS NULL AND status = 'completed'
                       ORDER BY patrol_date DESC, created_at DESC"#,
                )
                .fetch_all(pool)
                .await?
            }
            VetPatrolListFilter::MyDrafts => {
                sqlx::query_as::<_, VetPatrolReport>(
                    r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                              created_by, updated_by, created_at, updated_at,
                              follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at,
                              (SELECT u.display_name FROM users u WHERE u.id = vet_patrol_reports.created_by) AS created_by_name
                       FROM vet_patrol_reports
                       WHERE deleted_at IS NULL AND status = 'draft' AND created_by = $1
                       ORDER BY updated_at DESC"#,
                )
                .bind(current_user_id)
                .fetch_all(pool)
                .await?
            }
            VetPatrolListFilter::MyAcknowledgements => {
                sqlx::query_as::<_, VetPatrolReport>(
                    r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                              created_by, updated_by, created_at, updated_at,
                              follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at,
                              (SELECT u.display_name FROM users u WHERE u.id = vet_patrol_reports.created_by) AS created_by_name
                       FROM vet_patrol_reports
                       WHERE deleted_at IS NULL
                         AND status = 'awaiting_acknowledgement'
                         AND follow_up_user_id = $1
                       ORDER BY updated_at DESC"#,
                )
                .bind(current_user_id)
                .fetch_all(pool)
                .await?
            }
            VetPatrolListFilter::MyFollowUps => {
                sqlx::query_as::<_, VetPatrolReport>(
                    r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                              created_by, updated_by, created_at, updated_at,
                              follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at,
                              (SELECT u.display_name FROM users u WHERE u.id = vet_patrol_reports.created_by) AS created_by_name
                       FROM vet_patrol_reports
                       WHERE deleted_at IS NULL
                         AND status IN ('awaiting_acknowledgement', 'awaiting_follow_up')
                         AND follow_up_user_id = $1
                       ORDER BY updated_at DESC"#,
                )
                .bind(current_user_id)
                .fetch_all(pool)
                .await?
            }
            VetPatrolListFilter::All => {
                sqlx::query_as::<_, VetPatrolReport>(
                    r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                              created_by, updated_by, created_at, updated_at,
                              follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at,
                              (SELECT u.display_name FROM users u WHERE u.id = vet_patrol_reports.created_by) AS created_by_name
                       FROM vet_patrol_reports
                       WHERE deleted_at IS NULL
                       ORDER BY patrol_date DESC, created_at DESC"#,
                )
                .fetch_all(pool)
                .await?
            }
            VetPatrolListFilter::Relevant => {
                sqlx::query_as::<_, VetPatrolReport>(
                    r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                              created_by, updated_by, created_at, updated_at,
                              follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at,
                              (SELECT u.display_name FROM users u WHERE u.id = vet_patrol_reports.created_by) AS created_by_name
                       FROM vet_patrol_reports
                       WHERE deleted_at IS NULL
                         AND (
                               status = 'completed'
                               OR (status = 'draft' AND created_by = $1)
                               OR (status IN ('awaiting_acknowledgement', 'awaiting_follow_up')
                                   AND (created_by = $1 OR follow_up_user_id = $1))
                             )
                       ORDER BY patrol_date DESC, created_at DESC"#,
                )
                .bind(current_user_id)
                .fetch_all(pool)
                .await?
            }
        };
        Ok(reports)
    }

    /// 取得單一報告（含條目 + 耳號）
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<VetPatrolReportWithEntries>> {
        let report = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                      created_by, updated_by, created_at, updated_at,
                      follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        let Some(report) = report else {
            return Ok(None);
        };

        let mut entries = sqlx::query_as::<_, VetPatrolEntryWithAnimal>(
            r#"SELECT e.id, e.report_id, e.category, e.animal_id,
                      a.ear_tag,
                      e.observation, e.suggestion, e.follow_up,
                      e.sort_order, e.created_at
               FROM vet_patrol_entries e
               LEFT JOIN animals a ON a.id = e.animal_id
               WHERE e.report_id = $1
               ORDER BY e.sort_order, e.created_at"#,
        )
        .bind(report.id)
        .fetch_all(pool)
        .await?;

        // R39+++ 補上多動物資訊：從 junction table 撈每個 entry 的所有 (animal_id, ear_tag)
        let junction_rows: Vec<(Uuid, Uuid, Option<String>)> = sqlx::query_as(
            r#"SELECT ea.entry_id, ea.animal_id, a.ear_tag
               FROM vet_patrol_entry_animals ea
               JOIN vet_patrol_entries e ON e.id = ea.entry_id
               JOIN animals a ON a.id = ea.animal_id
               WHERE e.report_id = $1
               ORDER BY ea.entry_id, a.ear_tag"#,
        )
        .bind(report.id)
        .fetch_all(pool)
        .await?;
        let mut by_entry: std::collections::HashMap<Uuid, (Vec<Uuid>, Vec<String>)> =
            std::collections::HashMap::new();
        for (entry_id, animal_id, ear_tag) in junction_rows {
            let bucket = by_entry.entry(entry_id).or_default();
            bucket.0.push(animal_id);
            if let Some(tag) = ear_tag {
                bucket.1.push(tag);
            }
        }
        for e in entries.iter_mut() {
            if let Some((ids, tags)) = by_entry.remove(&e.id) {
                e.animal_ids = ids;
                e.ear_tags = tags;
            }
        }

        let photos = list_photos(pool, report.id).await?;
        let entry_photos = Self::list_entry_photos_for_report(pool, report.id).await?;

        Ok(Some(VetPatrolReportWithEntries {
            report,
            entries,
            photos,
            entry_photos,
        }))
    }

    /// 建立巡場報告（含條目）+ 自動同步到動物獸醫師建議
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateVetPatrolReportRequest,
    ) -> Result<VetPatrolReport> {
        let user_id = actor.require_user()?.id;
        let mut tx = pool.begin().await?;

        let report = sqlx::query_as::<_, VetPatrolReport>(
            r#"INSERT INTO vet_patrol_reports (patrol_date, accompanying_personnel, created_by, updated_by)
               VALUES ($1, $2, $3, $3)
               RETURNING id, patrol_date, week_start, week_end, accompanying_personnel, status,
                         created_by, updated_by, created_at, updated_at,
                         follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at"#,
        )
        .bind(req.patrol_date)
        .bind(req.accompanying_personnel.as_deref())
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        for (i, entry) in req.entries.iter().enumerate() {
            let animal_ids = entry.resolved_animal_ids();
            // 寫入主表時 animal_id 取第一隻（向後相容 PDF / 既有讀路徑）
            let primary_animal_id = animal_ids.first().copied();

            // R39+++ 防護：建立階段（必為 draft）忽略 follow_up 欄位 — 該欄保留給追蹤者
            let entry_id: Uuid = sqlx::query_scalar(
                r#"INSERT INTO vet_patrol_entries
                       (report_id, category, animal_id, observation, suggestion, follow_up, sort_order)
                   VALUES ($1, $2, $3, $4, $5, '', $6)
                   RETURNING id"#,
            )
            .bind(report.id)
            .bind(&entry.category)
            .bind(primary_animal_id)
            .bind(&entry.observation)
            .bind(&entry.suggestion)
            .bind(entry.sort_order.unwrap_or(i as i32))
            .fetch_one(&mut *tx)
            .await?;

            // R39+++ junction：寫入多動物關聯
            for animal_id in &animal_ids {
                sqlx::query(
                    "INSERT INTO vet_patrol_entry_animals (entry_id, animal_id) VALUES ($1, $2) \
                     ON CONFLICT DO NOTHING",
                )
                .bind(entry_id)
                .bind(animal_id)
                .execute(&mut *tx)
                .await?;
            }

            // 建立/草稿階段「不」同步到病歷獸醫師建議——同步改在 complete_followup()
            // （報告三階段完成後才歸位），避免草稿/半成品污染病歷。見 sync_advice_on_complete。
        }

        let entry_snapshots = fetch_entry_snapshots(&mut tx, report.id).await?;
        let snapshot = VetPatrolReportSnapshot {
            report: report.clone(),
            entries: entry_snapshots,
        };
        let display = format!("巡場報告 {}", report.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_CREATED",
                entity: Some(AuditEntity::new("vet_patrol_reports", report.id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(report)
    }

    /// 更新巡場報告（條目全部替換）
    ///
    /// 回傳 (report, files_to_unlink)：caller 端在 commit 後對 files_to_unlink 執行 FileService::delete，
    /// 避免 (a) tx rollback 時檔案也被砍 (b) tx 內呼叫 async file I/O 增加 lock 持有時間。
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateVetPatrolReportRequest,
    ) -> Result<(VetPatrolReport, Vec<String>)> {
        let user_id = actor.require_user()?.id;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                      created_by, updated_by, created_at, updated_at,
                      follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;
        let before_entries = fetch_entry_snapshots(&mut tx, id).await?;

        // R39+ 兩階段流程權限檢查
        let is_admin = actor
            .as_user()
            .map(|u| u.has_permission("messaging.admin_view"))
            .unwrap_or(false);
        match before.status.as_str() {
            status::DRAFT => {
                // 草稿期：只有 created_by 可改全部
                if before.created_by != Some(user_id) && !is_admin {
                    return Err(AppError::Forbidden("只有發起者可編輯草稿".into()));
                }
            }
            status::AWAITING_ACK => {
                // 待確認收到：tracker 必須先按「確認收到」才能進入填寫階段
                return Err(AppError::BusinessRule(
                    "請先按「確認收到」進入填寫階段".into(),
                ));
            }
            status::AWAITING_FOLLOW_UP => {
                // 待追蹤：只允許指派的追蹤者修改 follow_up 欄位
                // 在 service 層用判斷限制 — 若 caller 改了 observation/suggestion/animal_id 也擋
                if before.follow_up_user_id != Some(user_id) && !is_admin {
                    return Err(AppError::Forbidden(
                        "目前處於待追蹤狀態，只有指派的追蹤者可修改".into(),
                    ));
                }
                // #955（PR review）：追蹤者（可能非獸醫）在待追蹤階段僅能補填各條目的
                // 「追蹤改善」，不得改報告基本資訊。只在「實際變動」時擋（前端 echo 相同值放行）。
                if let Some(new_date) = req.patrol_date {
                    if new_date != before.patrol_date {
                        return Err(AppError::BusinessRule(
                            "待追蹤狀態下不可修改巡場日期".into(),
                        ));
                    }
                }
                if let Some(new_ap) = req.accompanying_personnel.as_deref() {
                    // 對齊下方 UPDATE 的三態語義：空字串等同清為 NULL
                    let normalized = (!new_ap.is_empty()).then_some(new_ap);
                    if normalized != before.accompanying_personnel.as_deref() {
                        return Err(AppError::BusinessRule(
                            "待追蹤狀態下不可修改陪同人員".into(),
                        ));
                    }
                }
                // entry update：只允許 follow_up 欄位變動，其他欄位需與 before 一致。
                // #378：新增 entry（id=None）在待追蹤階段一律拒絕（只能補既有觀察的追蹤改善，
                // 不能新增觀察）；多動物關聯 animal_ids 的篡改另由寫入路徑跳過 junction 改寫防堵
                // （見下方 is_follow_up_phase 分支），因為 before_entries 快照只含單一 animal_id、
                // 無法在此可靠比對多動物集合。
                if let Some(entries) = &req.entries {
                    for new_e in entries {
                        let Some(new_id) = new_e.id else {
                            return Err(AppError::BusinessRule(
                                "待追蹤狀態下不可新增觀察條目，只能修改既有條目的「追蹤改善」欄位"
                                    .into(),
                            ));
                        };
                        if let Some(old_e) = before_entries.iter().find(|e| e.id == new_id) {
                            // 待追蹤階段追蹤者僅可補填 follow_up；此檢查擋「連同改了 observation /
                            // suggestion / category」。**動物關聯刻意不在此比對**：
                            //   1. 讀取路徑回傳的 animal_ids 依 ear_tag 排序（見本檔 list 端
                            //      `ORDER BY ea.entry_id, a.ear_tag`），與 DB 既有單一 animal_id
                            //      （建立時依「選取順序」取的主要動物）不必然同序；逐一或比主要動物
                            //      都會對「多動物條目」誤判 422（陪同人員存不了追蹤改善。#959 首修只比
                            //      主要動物，對多動物仍不足——ear_tag 序首 ≠ 選取序首）。
                            //   2. 更根本地，寫入路徑於待追蹤階段只 `UPDATE ... SET follow_up`、完全
                            //      跳過 animal_id/junction（見下方 is_follow_up_phase 分支），故追蹤者
                            //      即使送不同動物集也是 no-op；比對動物零防護價值、只會誤擋合法儲存。
                            if new_e.observation != old_e.observation
                                || new_e.suggestion != old_e.suggestion
                                || new_e.category != old_e.category
                            {
                                return Err(AppError::BusinessRule(
                                    "待追蹤狀態下，只能修改「追蹤改善」欄位".into(),
                                ));
                            }
                        }
                    }
                }
            }
            status::COMPLETED => {
                // 已完成：lock（GLP 不可變醫療紀錄）。admin 也不能透過 update 改
                return Err(AppError::BusinessRule(
                    "報告已完成（鎖定），不可再修改".into(),
                ));
            }
            other => {
                return Err(AppError::Internal(format!("非預期 status: {other}")));
            }
        }

        // accompanying_personnel 三態語義：
        //   $3 IS NULL                → 未傳，保留原值
        //   $3 = '' (空字串)          → user 主動清空，寫 NULL
        //   $3 = '值'                 → 寫入新值
        // 與 frontend trim() 後送字串/不送 配合（PR #361 移除 || null 後 caller 永遠送字串）
        let report = sqlx::query_as::<_, VetPatrolReport>(
            r#"UPDATE vet_patrol_reports SET
                   patrol_date            = COALESCE($2, patrol_date),
                   accompanying_personnel = CASE
                       WHEN $3::text IS NULL THEN accompanying_personnel
                       ELSE NULLIF($3, '')
                   END,
                   updated_by             = $4,
                   updated_at             = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, patrol_date, week_start, week_end, accompanying_personnel, status,
                         created_by, updated_by, created_at, updated_at,
                         follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at"#,
        )
        .bind(id)
        .bind(req.patrol_date)
        .bind(req.accompanying_personnel.as_deref())
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
        let after = report.clone();

        // R39: 改 diff-based 更新（保留既有 entries 的 entry_photos）
        // - request 帶 id 的 → UPDATE 既有列（photos 保留）
        // - request 沒 id 的 → INSERT 新列
        // - 既有但不在 request 的 id → DELETE（CASCADE 清掉該列的 photos）
        let mut to_unlink: Vec<String> = Vec::new();
        if let Some(entries) = &req.entries {
            let existing_ids: Vec<Uuid> = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM vet_patrol_entries WHERE report_id = $1",
            )
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

            let request_ids: std::collections::HashSet<Uuid> =
                entries.iter().filter_map(|e| e.id).collect();

            // 防資料流失：若 request 帶了不屬於本報告的 entry_id（例如 stale data /
            // 跨報告的錯字），原本的 fallback 是「視為新增」— 但這會讓 to_delete
            // 把所有合法 existing entries 砍掉、然後 INSERT 一筆 stale 資料。
            // 改為 upfront validate：request_ids 必須是 existing_ids 的子集。
            let existing_set: std::collections::HashSet<Uuid> =
                existing_ids.iter().copied().collect();
            for req_id in &request_ids {
                if !existing_set.contains(req_id) {
                    return Err(AppError::Validation(format!(
                        "entry_id {req_id} 不存在或不屬於此巡場報告（請重新同步報告狀態）"
                    )));
                }
            }

            // #378（bot follow-up）：待追蹤階段不可刪除既有觀察條目。lock check 已擋新增
            // （id=None）與欄位變更，但「省略既有 id」會落入下方 to_delete 把該 entry 連同
            // 照片 CASCADE 砍掉（entries:[] 即可清空報告）。故要求 request 必含所有 existing id。
            if before.status == status::AWAITING_FOLLOW_UP {
                for ex_id in &existing_ids {
                    if !request_ids.contains(ex_id) {
                        return Err(AppError::BusinessRule(
                            "待追蹤狀態下不可刪除既有觀察條目，只能修改「追蹤改善」欄位".into(),
                        ));
                    }
                }
            }

            let to_delete: Vec<Uuid> = existing_ids
                .iter()
                .filter(|x| !request_ids.contains(*x))
                .copied()
                .collect();

            // 在 DELETE 前先撈出將被 CASCADE 清掉的 entry_photos file_path —
            // CASCADE 只清 DB row，實體檔案會殘留在 uploads/ 造成 disk leak。
            // tx commit 後由 caller 對這些 path 跑 FileService::delete。
            if !to_delete.is_empty() {
                to_unlink = sqlx::query_scalar::<_, String>(
                    "SELECT file_path FROM vet_patrol_entry_photos WHERE entry_id = ANY($1)",
                )
                .bind(&to_delete)
                .fetch_all(&mut *tx)
                .await?;

                sqlx::query("DELETE FROM vet_patrol_entries WHERE id = ANY($1)")
                    .bind(&to_delete)
                    .execute(&mut *tx)
                    .await?;
            }

            // R39+++ 防護：draft 階段 follow_up 一律忽略（保留給追蹤者）；
            // awaiting_follow_up 階段才接受 client 傳的 follow_up（service 開頭已驗權限為追蹤者）。
            let is_draft_phase = before.status == status::DRAFT;
            let is_follow_up_phase = before.status == status::AWAITING_FOLLOW_UP;

            for (i, entry) in entries.iter().enumerate() {
                // #378：待追蹤階段只准改 follow_up — 不動 animal_id / 其他欄位 / junction，
                // 避免追蹤者送 animal_ids=[不同動物集] 篡改 GLP 動物關聯（新增 entry 已於 lock check 擋下）。
                if is_follow_up_phase {
                    if let Some(entry_id) = entry.id {
                        sqlx::query(
                            "UPDATE vet_patrol_entries SET follow_up = $2 \
                             WHERE id = $1 AND report_id = $3",
                        )
                        .bind(entry_id)
                        .bind(&entry.follow_up)
                        .bind(id)
                        .execute(&mut *tx)
                        .await?;
                    }
                    continue;
                }

                let sort_order = entry.sort_order.unwrap_or(i as i32);
                let animal_ids = entry.resolved_animal_ids();
                let primary_animal_id = animal_ids.first().copied();
                let effective_follow_up: &str = if is_draft_phase { "" } else { &entry.follow_up };
                let resolved_entry_id: Uuid = if let Some(entry_id) = entry.id {
                    // 嘗試 UPDATE，若 id 屬於別份 report 或不存在 → 視為 INSERT
                    let updated = sqlx::query(
                        r#"UPDATE vet_patrol_entries
                              SET category = $2, animal_id = $3, observation = $4,
                                  suggestion = $5, follow_up = $6, sort_order = $7
                            WHERE id = $1 AND report_id = $8"#,
                    )
                    .bind(entry_id)
                    .bind(&entry.category)
                    .bind(primary_animal_id)
                    .bind(&entry.observation)
                    .bind(&entry.suggestion)
                    .bind(effective_follow_up)
                    .bind(sort_order)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                    if updated.rows_affected() == 0 {
                        // 上面的 upfront validate 已排除 stale id；這裡 0 rows 必然是
                        // 同一個 tx 內某種非預期競爭（極不該發生）→ 立刻錯出讓 caller 重試
                        return Err(AppError::Internal(format!(
                            "entry_id {entry_id} UPDATE 0 rows，無法解釋的狀態"
                        )));
                    }
                    entry_id
                } else {
                    sqlx::query_scalar::<_, Uuid>(
                        r#"INSERT INTO vet_patrol_entries
                               (report_id, category, animal_id, observation, suggestion, follow_up, sort_order)
                           VALUES ($1, $2, $3, $4, $5, $6, $7)
                           RETURNING id"#,
                    )
                    .bind(id)
                    .bind(&entry.category)
                    .bind(primary_animal_id)
                    .bind(&entry.observation)
                    .bind(&entry.suggestion)
                    .bind(effective_follow_up)
                    .bind(sort_order)
                    .fetch_one(&mut *tx)
                    .await?
                };

                // R39+++ junction：每次 UPDATE/INSERT 都全量 replace 多動物關聯
                // 先清掉舊的，再寫新的（最簡單、避免 diff 邏輯）
                sqlx::query("DELETE FROM vet_patrol_entry_animals WHERE entry_id = $1")
                    .bind(resolved_entry_id)
                    .execute(&mut *tx)
                    .await?;
                for animal_id in &animal_ids {
                    sqlx::query(
                        "INSERT INTO vet_patrol_entry_animals (entry_id, animal_id) VALUES ($1, $2) \
                         ON CONFLICT DO NOTHING",
                    )
                    .bind(resolved_entry_id)
                    .bind(animal_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        let after_entries = fetch_entry_snapshots(&mut tx, id).await?;

        // R39: draft → draft 的更新不寫 audit（避免 auto-save 噪音）
        // 只有 submitted report 的更新才寫 audit；submit 動作另由 submit() 寫一筆
        // R39+ 兩階段：draft 期間 auto-save 不寫 audit（避免噪音）
        // awaiting_follow_up / completed 都寫 audit（vet 已送出後的任何修改都該留痕）
        if before.status != status::DRAFT {
            let before_snapshot = VetPatrolReportSnapshot {
                report: before,
                entries: before_entries,
            };
            let after_snapshot = VetPatrolReportSnapshot {
                report: after.clone(),
                entries: after_entries,
            };
            let display = format!("巡場報告 {}", after.patrol_date);
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "ANIMAL",
                    event_type: "VET_PATROL_REPORT_UPDATED",
                    entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                    data_diff: Some(DataDiff::compute(
                        Some(&before_snapshot),
                        Some(&after_snapshot),
                    )),
                    request_context: None,
                },
            )
            .await?;
        }

        tx.commit().await?;
        Ok((report, to_unlink))
    }

    /// R39++ 三階段流程 phase 1：獸醫送出給追蹤者 → status='awaiting_acknowledgement'
    ///
    /// 條件（R40-20：created_by only + admin override，對齊本檔 5 處 is_admin 既有 pattern）：
    ///   - actor 必須是 created_by，**或**具備 admin override 權限（代勞情境）
    ///   - 必須指派 follow_up_user_id（追蹤者，通常為陪同人員）
    ///   - status 必須是 draft
    ///
    /// 寫一筆 VET_PATROL_REPORT_SUBMITTED_FOR_FOLLOWUP audit。
    /// caller 端（handler）負責後續發站內信 / email / notification 給追蹤者。
    pub async fn submit_for_followup(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        follow_up_user_id: Uuid,
    ) -> Result<VetPatrolReport> {
        let user_id = actor.require_user()?.id;
        let is_admin = actor
            .as_user()
            .map(|u| u.has_permission("messaging.admin_view"))
            .unwrap_or(false);
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                      created_by, updated_by, created_at, updated_at,
                      follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;

        if before.status != status::DRAFT {
            return Err(AppError::Validation(format!(
                "報告狀態為 {}，無法送出（只有草稿可送出）",
                before.status
            )));
        }
        if before.created_by != Some(user_id) && !is_admin {
            return Err(AppError::Forbidden("只有發起的獸醫可送出報告".into()));
        }
        if follow_up_user_id == user_id {
            return Err(AppError::Validation("追蹤者不能是自己".into()));
        }

        let after = sqlx::query_as::<_, VetPatrolReport>(
            r#"UPDATE vet_patrol_reports
                  SET status = 'awaiting_acknowledgement',
                      follow_up_user_id = $2,
                      submitted_at = NOW(),
                      updated_at = NOW(),
                      updated_by = $3
                WHERE id = $1 AND status = 'draft'
                RETURNING id, patrol_date, week_start, week_end, accompanying_personnel, status,
                          created_by, updated_by, created_at, updated_at,
                          follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at"#,
        )
        .bind(id)
        .bind(follow_up_user_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validation("報告狀態變動：另一請求已送出此報告".into()))?;

        let entries = fetch_entry_snapshots(&mut tx, id).await?;
        let snapshot = VetPatrolReportSnapshot {
            report: after.clone(),
            entries,
        };
        // 置頂待辦與狀態轉換同一個 tx。原本由 handler 在 commit 之後 best-effort 建立，
        // 有兩個問題：
        // (1) INSERT 失敗時只 warn —— 獸醫看到送出成功、追蹤者卻永遠收不到待辦，
        //     而本 PR 的對帳只找「殘留」不找「漏建」，這個方向沒有任何兜底。
        // (2) 與 retract / delete 的 tx 內解除形成競態：送出 commit 後、pin 建立前，
        //     撤回可能剛好取得列鎖並解除（此時 pin 尚未存在，掃不到），
        //     隨後 pin 才被建立 → 孤兒。
        // 放進 tx 後兩者都消失：報告列鎖序列化了狀態轉換，pin 與狀態同生共死。
        // 位置同其餘三條終態路徑，排在 audit 之前（不握全域 audit 鎖再跨表寫入）。
        Self::create_followup_pin_tx(&mut tx, &after, follow_up_user_id, actor).await?;

        let display = format!("巡場報告 {}", after.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_SUBMITTED_FOR_FOLLOWUP",
                entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// 在 `submit_for_followup` 的 tx 內建立「需您填寫追蹤改善」置頂待辦。
    ///
    /// 與報告狀態轉換 all-or-nothing：送出成功就一定有待辦，rollback 就一定沒有。
    async fn create_followup_pin_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        report: &VetPatrolReport,
        follow_up_user_id: Uuid,
        actor: &ActorContext,
    ) -> Result<()> {
        let actor_label = actor
            .as_user()
            .map(|u| u.email.clone())
            .unwrap_or_else(|| "系統".to_string());
        crate::services::NotificationService::create_pinned_notification_tx(
            tx,
            crate::models::CreateNotificationRequest {
                user_id: follow_up_user_id,
                notification_type: crate::models::NotificationType::VetRecommendation,
                title: format!("[巡場報告] {} 需您填寫追蹤改善", report.patrol_date),
                content: Some(format!(
                    "{} 已將「{}」巡場報告指派給您追蹤；請填寫各條目的「追蹤改善」欄位後送出完成。",
                    actor_label, report.patrol_date,
                )),
                related_entity_type: Some("vet_patrol_reports".to_string()),
                related_entity_id: Some(report.id),
            },
        )
        .await?;
        Ok(())
    }

    /// 撤回到草稿：把「已送出但未完成」的報告退回 draft，讓填報獸醫修正後重送。
    ///
    /// 條件：
    ///   - status 必須是 awaiting_acknowledgement 或 awaiting_follow_up（送出後、未完成前；
    ///     draft 已是草稿、completed 為 GLP 鎖定終態，皆不可撤回）
    ///   - actor 必須是 created_by（填報獸醫），或 admin
    ///   - reset 工作流欄位（follow_up_user_id / submitted_at / acknowledged_* /
    ///     follow_up_submitted_at 清空），回到乾淨草稿供重新編輯 + 重送
    ///
    /// 保留報告本體（非刪除），並寫 VET_PATROL_REPORT_RETRACTED audit — 以軌跡保紀錄
    /// 完整性、同時允許修錯，符合 GLP。
    pub async fn retract_to_draft(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<VetPatrolReport> {
        let user_id = actor.require_user()?.id;
        let is_admin = actor.as_user().map(|u| u.is_admin()).unwrap_or(false);
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                      created_by, updated_by, created_at, updated_at,
                      follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;

        if before.status != status::AWAITING_ACK && before.status != status::AWAITING_FOLLOW_UP {
            return Err(AppError::Validation(format!(
                "報告狀態為 {}，無法撤回（僅已送出未完成的報告可撤回成草稿）",
                before.status
            )));
        }
        if before.created_by != Some(user_id) && !is_admin {
            return Err(AppError::Forbidden(
                "只有填報的獸醫或管理員可撤回報告".into(),
            ));
        }

        let after = sqlx::query_as::<_, VetPatrolReport>(
            r#"UPDATE vet_patrol_reports
                  SET status = 'draft',
                      follow_up_user_id = NULL,
                      submitted_at = NULL,
                      acknowledged_at = NULL,
                      acknowledged_by_id = NULL,
                      follow_up_submitted_at = NULL,
                      updated_at = NOW(),
                      updated_by = $2
                WHERE id = $1 AND status IN ('awaiting_acknowledgement', 'awaiting_follow_up')
                RETURNING id, patrol_date, week_start, week_end, accompanying_personnel, status,
                          created_by, updated_by, created_at, updated_at,
                          follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at"#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validation("報告狀態變動：另一請求已改變此報告".into()))?;

        let entries = fetch_entry_snapshots(&mut tx, id).await?;
        let snapshot = VetPatrolReportSnapshot {
            report: after.clone(),
            entries,
        };
        // 撤回清空 follow_up_user_id → 原追蹤者不再被指派，解除其置頂待辦。同 tx。
        //
        // ⚠️ 位置刻意排在 `log_activity_tx` **之前**：後者會取
        // `pg_advisory_xact_lock(hashtext('audit_log_chain'))` —— 全系統唯一的 audit
        // 序列化鎖，且持有到 tx 結束。把 notifications 的 UPDATE 排在它後面，等於握著
        // 全域 audit 鎖再做一次跨表寫入，會拉長所有人的 audit 寫入臨界區。
        // 本檔四條終態路徑一律採此順序。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "vet_patrol_reports",
            id,
        )
        .await?;

        let display = format!("巡場報告 {}", after.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_RETRACTED",
                entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// R39++ 三階段流程 phase 2：追蹤者按「確認收到」 → status='awaiting_follow_up'
    ///
    /// 條件：
    ///   - actor 必須是 follow_up_user_id（指派的追蹤者）
    ///   - status 必須是 awaiting_acknowledgement
    ///
    /// 寫一筆 VET_PATROL_REPORT_ACKNOWLEDGED audit。
    ///
    /// 回傳 `(report, newly_acknowledged)`：`newly_acknowledged=false` 代表冪等 no-op
    /// （已由本人確認過），呼叫端據此避免重複發通知。
    pub async fn acknowledge_receipt(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<(VetPatrolReport, bool)> {
        let user_id = actor.require_user()?.id;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                      created_by, updated_by, created_at, updated_at,
                      follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;

        // 冪等保護：前端「打開即自動確認」可能因 StrictMode 雙觸發 / 多分頁 race 重入。
        // 若報告已是 awaiting_follow_up 且已由本人確認，視為成功 no-op（不重複寫 audit）。
        if before.status == status::AWAITING_FOLLOW_UP && before.acknowledged_by_id == Some(user_id)
        {
            return Ok((before, false));
        }
        if before.status != status::AWAITING_ACK {
            return Err(AppError::Validation(format!(
                "報告狀態為 {}，無法確認收到（只有 awaiting_acknowledgement 可確認）",
                before.status
            )));
        }
        let is_admin = actor
            .as_user()
            .map(|u| u.has_permission("messaging.admin_view"))
            .unwrap_or(false);
        if before.follow_up_user_id != Some(user_id) && !is_admin {
            return Err(AppError::Forbidden("只有指派的追蹤者可確認收到".into()));
        }

        let after = sqlx::query_as::<_, VetPatrolReport>(
            r#"UPDATE vet_patrol_reports
                  SET status = 'awaiting_follow_up',
                      acknowledged_at = NOW(),
                      acknowledged_by_id = $2,
                      updated_at = NOW(),
                      updated_by = $2
                WHERE id = $1 AND status = 'awaiting_acknowledgement'
                RETURNING id, patrol_date, week_start, week_end, accompanying_personnel, status,
                          created_by, updated_by, created_at, updated_at,
                          follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at"#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validation("報告狀態變動：另一請求已確認此報告".into()))?;

        let entries = fetch_entry_snapshots(&mut tx, id).await?;
        let snapshot = VetPatrolReportSnapshot {
            report: after.clone(),
            entries,
        };
        let display = format!("巡場報告 {}", after.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_ACKNOWLEDGED",
                entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok((after, true))
    }

    /// R39++ 三階段流程 phase 3：追蹤者按「確認完成」 → status='completed' (locked)
    ///
    /// 條件：
    ///   - actor 必須是 follow_up_user_id（指派的追蹤者）
    ///   - status 必須是 awaiting_follow_up
    ///
    /// 寫一筆 VET_PATROL_REPORT_COMPLETED audit。
    /// 完成後 status=completed，任何人都不能再改（GLP 不可變醫療紀錄）。
    pub async fn complete_followup(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<VetPatrolReport> {
        let user_id = actor.require_user()?.id;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                      created_by, updated_by, created_at, updated_at,
                      follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;

        if before.status != status::AWAITING_FOLLOW_UP {
            return Err(AppError::Validation(format!(
                "報告狀態為 {}，無法完成追蹤（只有 awaiting_follow_up 可完成）",
                before.status
            )));
        }
        let is_admin = actor
            .as_user()
            .map(|u| u.has_permission("messaging.admin_view"))
            .unwrap_or(false);
        if before.follow_up_user_id != Some(user_id) && !is_admin {
            return Err(AppError::Forbidden("只有指派的追蹤者可完成追蹤".into()));
        }

        let after = sqlx::query_as::<_, VetPatrolReport>(
            r#"UPDATE vet_patrol_reports
                  SET status = 'completed',
                      follow_up_submitted_at = NOW(),
                      updated_at = NOW(),
                      updated_by = $2
                WHERE id = $1 AND status = 'awaiting_follow_up'
                RETURNING id, patrol_date, week_start, week_end, accompanying_personnel, status,
                          created_by, updated_by, created_at, updated_at,
                          follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at"#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validation("報告狀態變動：另一請求已完成此報告".into()))?;

        // 三階段完成 → 歸位病歷「獸醫師建議」：每個「掛豬的 entry × 每隻豬」upsert 一筆
        // （觀察 + 建議 + 追蹤改善），連結 source_vet_patrol_entry_id。set-based 一條 SQL，
        // 依部分唯一索引 upsert 去重（重跑冪等）。source 非 NULL → 命中 ux_vet_advice_source_entry_animal。
        // 場級觀察（無掛豬，如防疫/病歷/其他）沒有 junction 列 → 自然不同步。
        // created_by/updated_by 取報告建立者（填寫觀察/建議的獸醫）。
        let synced = sqlx::query(
            r#"INSERT INTO animal_vet_advice_records
                   (animal_id, advice_date, observation, suggested_treatment, follow_up,
                    source_vet_patrol_entry_id, created_by, updated_by)
               SELECT ea.animal_id, $2, e.observation, e.suggestion, e.follow_up, e.id, $3, $3
                 FROM vet_patrol_entries e
                 JOIN vet_patrol_entry_animals ea ON ea.entry_id = e.id
                WHERE e.report_id = $1
                  AND (e.observation <> '' OR e.suggestion <> '' OR e.follow_up <> '')
               ON CONFLICT (source_vet_patrol_entry_id, animal_id)
                   WHERE source_vet_patrol_entry_id IS NOT NULL
               DO UPDATE SET observation         = EXCLUDED.observation,
                             suggested_treatment = EXCLUDED.suggested_treatment,
                             follow_up           = EXCLUDED.follow_up,
                             advice_date         = EXCLUDED.advice_date,
                             updated_by          = EXCLUDED.updated_by,
                             updated_at          = NOW(),
                             deleted_at          = NULL"#,
        )
        .bind(id)
        .bind(after.patrol_date)
        .bind(after.created_by)
        .execute(&mut *tx)
        .await?;
        tracing::info!(
            report_id = %id,
            advice_synced = synced.rows_affected(),
            "vet patrol completed: synced advice to animal medical records"
        );

        let entries = fetch_entry_snapshots(&mut tx, id).await?;
        let snapshot = VetPatrolReportSnapshot {
            report: after.clone(),
            entries,
        };
        // 追蹤改善完成 → 解除追蹤者的置頂待辦。同 tx，且排在 audit 之前
        // （理由見 retract_to_draft：不要握著全域 audit 鎖再跨表寫入）。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "vet_patrol_reports",
            id,
        )
        .await?;

        let display = format!("巡場報告 {}", after.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_COMPLETED",
                entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// R39 GC：刪除 7 天未動的草稿（scheduler nightly 呼叫）
    ///
    /// 設計考量：
    /// - **批次處理**（LIMIT 100 / 輪）：避免一次 DELETE 鎖死 vet_patrol_reports / entries /
    ///   entry_photos / photos 多張表造成 prod 短暫無法寫入；多輪迴圈直到清完
    /// - **檔案 unlink**：CASCADE 只清 DB row，實體檔案需另外處理；先撈所有
    ///   entry_photos + report photos 的 file_path → DELETE → unlink files
    /// - **ORDER BY updated_at**：優先清最舊的 draft，避免邊清邊有新的「7 天前」湧入
    ///
    /// 回傳：(刪除的 report 數, unlink 失敗的 file_path 數量)
    pub async fn cleanup_stale_drafts(pool: &PgPool) -> Result<(u64, usize)> {
        const BATCH_SIZE: i64 = 100;
        let mut total_deleted: u64 = 0;
        let mut unlink_failures: usize = 0;

        loop {
            // 整批選 + 鎖 + 撈檔案路徑 + 刪除全部在同一 tx 內，避免 SELECT 後使用者剛好
            // auto-save / submit 而 GC 仍把該行刪掉造成資料流失。SKIP LOCKED 讓 GC 不
            // 會卡到正被別的 tx 持有的列；DELETE 端再次 assert stale 條件 double safety。
            let mut tx = pool.begin().await?;

            let stale_ids: Vec<Uuid> = sqlx::query_scalar(
                r#"SELECT id FROM vet_patrol_reports
                   WHERE status = 'draft'
                     AND deleted_at IS NULL
                     AND updated_at < NOW() - INTERVAL '7 days'
                   ORDER BY updated_at
                   LIMIT $1
                   FOR UPDATE SKIP LOCKED"#,
            )
            .bind(BATCH_SIZE)
            .fetch_all(&mut *tx)
            .await?;

            if stale_ids.is_empty() {
                tx.commit().await?;
                break;
            }

            let entry_photo_paths: Vec<String> = sqlx::query_scalar(
                r#"SELECT ep.file_path
                   FROM vet_patrol_entry_photos ep
                   JOIN vet_patrol_entries e ON e.id = ep.entry_id
                   WHERE e.report_id = ANY($1)"#,
            )
            .bind(&stale_ids)
            .fetch_all(&mut *tx)
            .await?;

            let report_photo_paths: Vec<String> = sqlx::query_scalar(
                "SELECT file_path FROM vet_patrol_photos WHERE report_id = ANY($1)",
            )
            .bind(&stale_ids)
            .fetch_all(&mut *tx)
            .await?;

            // DELETE 端 assert 條件再次 — defense in depth；FOR UPDATE 已鎖列，理論上條件不會變
            let result = sqlx::query(
                r#"DELETE FROM vet_patrol_reports
                   WHERE id = ANY($1)
                     AND status = 'draft'
                     AND deleted_at IS NULL
                     AND updated_at < NOW() - INTERVAL '7 days'"#,
            )
            .bind(&stale_ids)
            .execute(&mut *tx)
            .await?;

            // 硬刪為終態：row 一旦消失，任何指向它的置頂待辦就再也沒有業務路徑能解除。
            // 草稿正常不帶置頂待辦（送出才建立），但「送出 → 撤回 → 擱置 7 天」這條路徑
            // 上若 retract 的解除將來回歸，這裡是最後一道防線。同 tx，與刪除同生共死。
            for id in &stale_ids {
                crate::services::NotificationService::resolve_pinned_notifications_tx(
                    &mut tx,
                    "vet_patrol_reports",
                    *id,
                )
                .await?;
            }

            tx.commit().await?;
            total_deleted += result.rows_affected();

            // 4. unlink 實體檔案（commit 後才動，避免 rollback 時檔案也被砍）
            for path in entry_photo_paths.iter().chain(report_photo_paths.iter()) {
                if let Err(e) = crate::services::FileService::delete(path).await {
                    tracing::warn!("vet_patrol GC unlink failed path={path} err={e}");
                    unlink_failures += 1;
                }
            }

            if (stale_ids.len() as i64) < BATCH_SIZE {
                break;
            }
        }

        Ok((total_deleted, unlink_failures))
    }

    /// R39+++ 棄置草稿（hard delete + unlink 照片實體檔）
    ///
    /// 用於：使用者開了「新增」dialog → 預建 draft → 沒按存草稿就關閉 → 整筆連同上傳的照片
    /// 一併丟掉。比 soft delete 嚴格：
    ///   - 直接 DELETE row（CASCADE 會清 entries / entry_photos / entry_animals / photos）
    ///   - 撈 file_path 在 commit 後 unlink 實體檔（避免 disk leak）
    ///
    /// 條件：
    ///   - status 必須是 draft（已送出 / 已完成 不可棄置）
    ///   - actor 必須是 created_by（或 admin）
    ///
    /// 寫一筆 VET_PATROL_REPORT_DISCARDED audit（GLP：草稿被棄也要留軌跡）。
    /// 回傳 unlink 失敗的檔案數量（用來 log 警告，不阻擋）。
    pub async fn discard_draft(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<usize> {
        let user_id = actor.require_user()?.id;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                      created_by, updated_by, created_at, updated_at,
                      follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;

        if before.status != status::DRAFT {
            return Err(AppError::Validation(format!(
                "只有 draft 可棄置；目前狀態 {}",
                before.status
            )));
        }
        let is_admin = actor
            .as_user()
            .map(|u| u.has_permission("messaging.admin_view"))
            .unwrap_or(false);
        if before.created_by != Some(user_id) && !is_admin {
            return Err(AppError::Forbidden("只有發起者可棄置自己的草稿".into()));
        }

        // 撈所有相關檔案 file_path（entry photos + report photos）
        let entry_photo_paths: Vec<String> = sqlx::query_scalar(
            r#"SELECT ep.file_path
               FROM vet_patrol_entry_photos ep
               JOIN vet_patrol_entries e ON e.id = ep.entry_id
               WHERE e.report_id = $1"#,
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let report_photo_paths: Vec<String> =
            sqlx::query_scalar("SELECT file_path FROM vet_patrol_photos WHERE report_id = $1")
                .bind(id)
                .fetch_all(&mut *tx)
                .await?;

        let before_entries = fetch_entry_snapshots(&mut tx, id).await?;

        // HARD DELETE — CASCADE 會清 entries / entry_animals / entry_photos / photos
        sqlx::query("DELETE FROM vet_patrol_reports WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 硬刪除為終態。草稿正常不會有置頂待辦（送出才建立），但「送出 → 撤回 → 棄置」
        // 這條路徑上若 retract 的解除將來回歸，這裡是最後一道防線——row 一旦硬刪，
        // 任何指向它的置頂待辦就再也沒有業務路徑能解除它。同 tx，理由見 retract_to_draft。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "vet_patrol_reports",
            id,
        )
        .await?;

        // 寫 audit 軌跡（草稿雖被刪，動作仍留痕）
        let snapshot = VetPatrolReportSnapshot {
            report: before.clone(),
            entries: before_entries,
        };
        let display = format!("巡場報告 {}", before.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_DISCARDED",
                entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                data_diff: Some(DataDiff::delete_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // commit 後才 unlink 實體檔（避免 rollback 時誤刪）
        let mut unlink_failures: usize = 0;
        for path in entry_photo_paths.iter().chain(report_photo_paths.iter()) {
            if let Err(e) = crate::services::FileService::delete(path).await {
                tracing::warn!("vet_patrol discard_draft unlink failed path={path} err={e}");
                unlink_failures += 1;
            }
        }

        Ok(unlink_failures)
    }

    /// 刪除巡場報告（soft delete）
    pub async fn delete(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, accompanying_personnel, status,
                      created_by, updated_by, created_at, updated_at,
                      follow_up_user_id, acknowledged_at, acknowledged_by_id, follow_up_submitted_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;
        // B2：非 admin 只能刪草稿；已送出/已完成的報告為 GLP 流程中/鎖定紀錄，限 admin 刪除。
        let is_admin = actor.as_user().map(|u| u.is_admin()).unwrap_or(false);
        if before.status != status::DRAFT && !is_admin {
            return Err(AppError::Forbidden(
                "已送出/已完成的報告限管理員刪除（GLP 鎖定）".into(),
            ));
        }

        let before_entries = fetch_entry_snapshots(&mut tx, id).await?;

        sqlx::query(
            "UPDATE vet_patrol_reports SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 報告為軟刪除（deleted_at）——entry 仍在，故 animal_vet_advice_records 的
        // source_vet_patrol_entry_id FK ON DELETE CASCADE 不會觸發。手動軟刪本報告已同步到
        // 病歷的建議，避免報告刪除後留下孤兒建議（gemini/CodeRabbit #972 review）。
        sqlx::query(
            r#"UPDATE animal_vet_advice_records
                  SET deleted_at = NOW()
                WHERE deleted_at IS NULL
                  AND source_vet_patrol_entry_id IN
                      (SELECT id FROM vet_patrol_entries WHERE report_id = $1)"#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        let snapshot = VetPatrolReportSnapshot {
            report: before.clone(),
            entries: before_entries,
        };
        // 報告已軟刪 → 追蹤者不可能再操作它，解除置頂待辦。同 tx，且排在 audit 之前
        // （理由見 retract_to_draft：不要握著全域 audit 鎖再跨表寫入）。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "vet_patrol_reports",
            id,
        )
        .await?;

        let display = format!("巡場報告 {}", before.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_DELETED",
                entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                data_diff: Some(DataDiff::delete_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// R32-A3b：查詢動物清單供巡視欄位狀態表 xlsx 套版用。
    ///
    /// 回傳 JSON array：`[{"pen_location":..., "ear_tag":..., "status":...}, ...]`，
    /// 給 pdf-service `/render-vet-patrol/from-animals` body.animals 直接使用。
    ///
    /// 篩選條件對齊 codebase 標準的「在欄活體」定義（見
    /// `services/animal/transfer.rs`、`repositories/pen.rs`）：
    /// `deleted_at IS NULL AND status NOT IN ('euthanized','sudden_death','transferred')`。
    /// Handler 不再直接寫 SQL / 組 JSON（per CLAUDE.md「Handler 禁止 SQL」規範）。
    pub async fn list_animals_for_patrol(pool: &PgPool) -> Result<serde_json::Value> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            r#"SELECT COALESCE(pen_location, '') as pen_location,
                      ear_tag,
                      COALESCE(status::text, '') as status
               FROM animals
               WHERE deleted_at IS NULL
                 AND status NOT IN ('euthanized', 'sudden_death', 'transferred')
                 AND pen_location IS NOT NULL
               ORDER BY pen_location, ear_tag"#,
        )
        .fetch_all(pool)
        .await?;

        let arr: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|(pen, ear_tag, status)| {
                serde_json::json!({
                    "pen_location": pen,
                    "ear_tag": ear_tag,
                    "status": status,
                })
            })
            .collect();
        Ok(serde_json::Value::Array(arr))
    }

    /// 列出某份報告的所有照片
    pub async fn list_photos(pool: &PgPool, report_id: Uuid) -> Result<Vec<VetPatrolPhoto>> {
        list_photos(pool, report_id).await
    }

    /// 新增照片附件
    ///
    /// sort_order 用 INSERT 內部 sub-select 計算，避免「先 SELECT MAX 再 INSERT」
    /// 兩段式 query 之間的 race condition。
    ///
    /// R40-16 follow-up（CodeRabbit PR #407 review）：INSERT 用 `... SELECT ... FROM
    /// vet_patrol_reports WHERE id = $1 AND deleted_at IS NULL` 形式，把「parent
    /// 存在且未軟刪」收進同一個 atomic SQL。原本 handler 端 `ensure_report_exists`
    /// preflight 與 disk write 之間的 race window 仍可能讓 report 被 soft-delete，
    /// 此處 0 rows 即視為 NotFound（caller 端 upload_and_insert_photo 會接著 unlink）。
    pub async fn insert_photo(
        pool: &PgPool,
        report_id: Uuid,
        created_by: Uuid,
        file_name: &str,
        file_path: &str,
        file_size: i64,
        mime_type: &str,
        caption: &str,
    ) -> Result<VetPatrolPhoto> {
        let photo = sqlx::query_as::<_, VetPatrolPhoto>(
            r#"INSERT INTO vet_patrol_photos
                   (report_id, file_name, file_path, file_size, mime_type, caption, sort_order, created_by)
               SELECT
                   $1, $2, $3, $4, $5, $6,
                   (SELECT COALESCE(MAX(sort_order), -1) + 1
                    FROM vet_patrol_photos WHERE report_id = $1),
                   $7
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, report_id, file_name, file_path, file_size, mime_type,
                         caption, sort_order, created_at"#,
        )
        .bind(report_id)
        .bind(file_name)
        .bind(file_path)
        .bind(file_size)
        .bind(mime_type)
        .bind(caption)
        .bind(created_by)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;
        Ok(photo)
    }

    /// R40-16：上傳實體檔 + DB insert 一條龍；DB 失敗時自動 rollback unlink。
    /// 將原本散在 handler 的「upload → match insert → unlink on err」pattern 下沉到 service。
    /// 不開 transaction（insert_photo 為單 statement，rollback 動作即 unlink）。
    pub async fn upload_and_insert_photo(
        pool: &PgPool,
        report_id: Uuid,
        user_id: Uuid,
        file_name: &str,
        content_type: &str,
        data: &[u8],
        caption: &str,
    ) -> Result<VetPatrolPhoto> {
        let category = crate::services::FileCategory::AnimalPhoto;
        let upload = crate::services::FileService::upload(
            category,
            file_name,
            content_type,
            data,
            Some(&report_id.to_string()),
        )
        .await?;

        match Self::insert_photo(
            pool,
            report_id,
            user_id,
            &upload.file_name,
            &upload.file_path,
            upload.file_size,
            &upload.mime_type,
            caption,
        )
        .await
        {
            Ok(photo) => Ok(photo),
            Err(e) => {
                if let Err(unlink_err) =
                    crate::services::FileService::delete(&upload.file_path).await
                {
                    tracing::warn!(
                        "vet_patrol photo rollback unlink 失敗 path={} err={unlink_err}",
                        upload.file_path
                    );
                }
                Err(e)
            }
        }
    }

    /// 更新照片的解說文字
    pub async fn update_photo_caption(
        pool: &PgPool,
        photo_id: Uuid,
        caption: &str,
    ) -> Result<VetPatrolPhoto> {
        let photo = sqlx::query_as::<_, VetPatrolPhoto>(
            r#"UPDATE vet_patrol_photos SET caption = $2
               WHERE id = $1
               RETURNING id, report_id, file_name, file_path, file_size, mime_type,
                         caption, sort_order, created_at"#,
        )
        .bind(photo_id)
        .bind(caption)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到照片".to_string()))?;
        Ok(photo)
    }

    /// 刪除照片（回傳 file_path 供 caller 清檔案）
    pub async fn delete_photo(pool: &PgPool, photo_id: Uuid) -> Result<String> {
        let path: Option<String> =
            sqlx::query_scalar("DELETE FROM vet_patrol_photos WHERE id = $1 RETURNING file_path")
                .bind(photo_id)
                .fetch_optional(pool)
                .await?;
        path.ok_or_else(|| AppError::NotFound("找不到照片".to_string()))
    }

    // ── R39: entry-level 照片 ──────────────────────────────

    /// 列出某個 entry 的所有照片
    pub async fn list_entry_photos(
        pool: &PgPool,
        entry_id: Uuid,
    ) -> Result<Vec<VetPatrolEntryPhoto>> {
        let photos = sqlx::query_as::<_, VetPatrolEntryPhoto>(
            r#"SELECT id, entry_id, file_name, file_path, file_size, mime_type,
                      caption, sort_order, created_at
               FROM vet_patrol_entry_photos
               WHERE entry_id = $1
               ORDER BY sort_order, created_at"#,
        )
        .bind(entry_id)
        .fetch_all(pool)
        .await?;
        Ok(photos)
    }

    /// 列出某個 report 的所有 entry 照片（按 entry_id 分組用，handler 端組裝 nested 結構）
    pub async fn list_entry_photos_for_report(
        pool: &PgPool,
        report_id: Uuid,
    ) -> Result<Vec<VetPatrolEntryPhoto>> {
        let photos = sqlx::query_as::<_, VetPatrolEntryPhoto>(
            r#"SELECT ep.id, ep.entry_id, ep.file_name, ep.file_path, ep.file_size,
                      ep.mime_type, ep.caption, ep.sort_order, ep.created_at
               FROM vet_patrol_entry_photos ep
               JOIN vet_patrol_entries e ON e.id = ep.entry_id
               WHERE e.report_id = $1
               ORDER BY ep.entry_id, ep.sort_order, ep.created_at"#,
        )
        .bind(report_id)
        .fetch_all(pool)
        .await?;
        Ok(photos)
    }

    /// 新增 entry 照片
    ///
    /// sort_order 用 INSERT 內部 sub-select 計算（同 insert_photo），race-safe。
    ///
    /// R40-16 follow-up（CodeRabbit PR #407 review）：INSERT 用 `... SELECT FROM
    /// vet_patrol_entries JOIN vet_patrol_reports WHERE deleted_at IS NULL`，把
    /// 「entry 存在且其 report 未軟刪」收進同一個 atomic SQL，0 rows 即 NotFound。
    pub async fn insert_entry_photo(
        pool: &PgPool,
        entry_id: Uuid,
        created_by: Uuid,
        file_name: &str,
        file_path: &str,
        file_size: i64,
        mime_type: &str,
        caption: &str,
    ) -> Result<VetPatrolEntryPhoto> {
        let photo = sqlx::query_as::<_, VetPatrolEntryPhoto>(
            r#"INSERT INTO vet_patrol_entry_photos
                   (entry_id, file_name, file_path, file_size, mime_type, caption, sort_order, created_by)
               SELECT
                   $1, $2, $3, $4, $5, $6,
                   (SELECT COALESCE(MAX(sort_order), -1) + 1
                    FROM vet_patrol_entry_photos WHERE entry_id = $1),
                   $7
               FROM vet_patrol_entries e
               JOIN vet_patrol_reports r ON r.id = e.report_id
               WHERE e.id = $1 AND r.deleted_at IS NULL
                 AND r.status <> 'completed'
               RETURNING id, entry_id, file_name, file_path, file_size, mime_type,
                         caption, sort_order, created_at"#,
        )
        .bind(entry_id)
        .bind(file_name)
        .bind(file_path)
        .bind(file_size)
        .bind(mime_type)
        .bind(caption)
        .bind(created_by)
        .fetch_optional(pool)
        .await?
        // #363（bot TOCTOU）：status<>'completed' 內嵌於 INSERT，與 ensure_entry_exists
        // preflight 構成原子守衛 —— 若報告在 preflight 後競態轉 completed，此處 0 rows。
        .ok_or_else(|| AppError::NotFound("找不到觀察條目或報告（或報告已鎖定）".to_string()))?;
        Ok(photo)
    }

    /// R40-16：上傳 entry 照片實體檔 + DB insert 一條龍；DB 失敗時自動 rollback unlink。
    pub async fn upload_and_insert_entry_photo(
        pool: &PgPool,
        entry_id: Uuid,
        user_id: Uuid,
        file_name: &str,
        content_type: &str,
        data: &[u8],
        caption: &str,
    ) -> Result<VetPatrolEntryPhoto> {
        let category = crate::services::FileCategory::AnimalPhoto;
        let upload = crate::services::FileService::upload(
            category,
            file_name,
            content_type,
            data,
            Some(&entry_id.to_string()),
        )
        .await?;

        match Self::insert_entry_photo(
            pool,
            entry_id,
            user_id,
            &upload.file_name,
            &upload.file_path,
            upload.file_size,
            &upload.mime_type,
            caption,
        )
        .await
        {
            Ok(photo) => Ok(photo),
            Err(e) => {
                if let Err(unlink_err) =
                    crate::services::FileService::delete(&upload.file_path).await
                {
                    tracing::warn!(
                        "vet_patrol entry photo rollback unlink 失敗 path={} err={unlink_err}",
                        upload.file_path
                    );
                }
                Err(e)
            }
        }
    }

    /// #363：取 entry-photo 所屬報告 status（不存在 / 已軟刪 → NotFound）。
    /// 用於 caption / delete 前驗鎖定狀態。
    async fn entry_photo_report_status(pool: &PgPool, photo_id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT r.status::text FROM vet_patrol_entry_photos p
                   JOIN vet_patrol_entries e ON e.id = p.entry_id
                   JOIN vet_patrol_reports r ON r.id = e.report_id
                   WHERE p.id = $1 AND r.deleted_at IS NULL"#,
        )
        .bind(photo_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到照片".to_string()))
    }

    /// 更新 entry 照片解說。#363：報告鎖定（completed）時拒絕。
    pub async fn update_entry_photo_caption(
        pool: &PgPool,
        photo_id: Uuid,
        caption: &str,
    ) -> Result<VetPatrolEntryPhoto> {
        if Self::entry_photo_report_status(pool, photo_id).await? == status::COMPLETED {
            return Err(AppError::BusinessRule(
                "報告已完成（鎖定），不可再異動照片".to_string(),
            ));
        }
        // #363（bot TOCTOU）：status<>'completed' 內嵌於 UPDATE，與上方 preflight 構成原子守衛
        // —— 若報告在 preflight 後競態轉 completed，此 UPDATE 0 rows。
        let photo = sqlx::query_as::<_, VetPatrolEntryPhoto>(
            r#"UPDATE vet_patrol_entry_photos p SET caption = $2
               WHERE p.id = $1
                 AND EXISTS (
                     SELECT 1 FROM vet_patrol_entries e
                     JOIN vet_patrol_reports r ON r.id = e.report_id
                     WHERE e.id = p.entry_id
                       AND r.deleted_at IS NULL AND r.status <> 'completed'
                 )
               RETURNING p.id, p.entry_id, p.file_name, p.file_path, p.file_size, p.mime_type,
                         p.caption, p.sort_order, p.created_at"#,
        )
        .bind(photo_id)
        .bind(caption)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到照片（或報告已鎖定）".to_string()))?;
        Ok(photo)
    }

    /// 刪除 entry 照片（回傳 file_path）。
    /// #363：(a) 報告鎖定（completed）時拒絕，防繞過 GLP lock 刪除照片證據；
    /// (b) 對非 draft 報告（已送出）的刪除寫 audit log，與 update() 的 audit 門檻一致——
    /// 不可逆刪除 GLP 附掛證據須留痕。
    pub async fn delete_entry_photo(
        pool: &PgPool,
        actor: &ActorContext,
        photo_id: Uuid,
    ) -> Result<String> {
        let mut tx = pool.begin().await?;
        // 鎖列 + 取所屬報告 status / file 資訊
        let row: Option<(String, String, String)> = sqlx::query_as(
            r#"SELECT r.status::text, p.file_path, p.file_name
                   FROM vet_patrol_entry_photos p
                   JOIN vet_patrol_entries e ON e.id = p.entry_id
                   JOIN vet_patrol_reports r ON r.id = e.report_id
                   WHERE p.id = $1 AND r.deleted_at IS NULL
                   FOR UPDATE OF p, r"#,
        )
        .bind(photo_id)
        .fetch_optional(&mut *tx)
        .await?;
        let (report_status, file_path, file_name) =
            row.ok_or_else(|| AppError::NotFound("找不到照片".to_string()))?;
        if report_status == status::COMPLETED {
            return Err(AppError::BusinessRule(
                "報告已完成（鎖定），不可再異動照片".to_string(),
            ));
        }

        sqlx::query("DELETE FROM vet_patrol_entry_photos WHERE id = $1")
            .bind(photo_id)
            .execute(&mut *tx)
            .await?;

        if report_status != status::DRAFT {
            let display = format!("巡場報告觀察照片 {file_name}");
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "ANIMAL",
                    event_type: "VET_PATROL_ENTRY_PHOTO_DELETED",
                    entity: Some(AuditEntity::new(
                        "vet_patrol_entry_photos",
                        photo_id,
                        &display,
                    )),
                    data_diff: None,
                    request_context: None,
                },
            )
            .await?;
        }

        tx.commit().await?;
        Ok(file_path)
    }
}
