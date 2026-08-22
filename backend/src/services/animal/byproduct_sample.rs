// R53-3: 廢棄物再利用紀錄 Service
//
// 對應 migration 066 / TODO.md R53。Framework：byproduct reuse — 結案豬隻
// 組織/血液本將焚化廢棄，多採只是廢棄物的另一去向，非 PI 資產轉移，故 PI
// 看不到去向（R53-6 audit blacklist 配套，後續 sub-PR 落地）。
//
// Pattern：Service-driven audit（對齊 R26 / PR #155 protocol::submit 範本）：
// - actor.require_user() 強制 ActorContext::User（拒 Anonymous / System）
// - 單 tx 內完成「DB 變更 + log_activity_tx audit」
// - DataDiff::compute 取 before/after diff，自動填 changed_fields
//
// SQL 慣例：採樣表雖然需要 JOIN animals / protocols / users 取顯示名，但本
// service 只回 raw entity；handler 端要顯示名時用 join view 結構（後續
// sub-PR 視需求新增）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::audit_diff::{AuditRedact, DataDiff},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

/// 廢棄物再利用紀錄（DB row）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ByproductSample {
    pub id: Uuid,
    /// #445：強制安樂死單來源（None = 來自計劃內犧牲，無安樂死單）
    pub euthanasia_id: Option<Uuid>,
    pub animal_id: Uuid,
    pub source_protocol_id: Uuid,
    pub sampled_at: DateTime<Utc>,
    pub sample_content: String,
    pub requester_user_id: Option<Uuid>,
    /// R53-14: external requester 機構名（與 requester_contact_name 雙層）
    pub requester_org_name: Option<String>,
    /// R53-14: external requester 聯絡人
    pub requester_contact_name: Option<String>,
    pub collector_id: Uuid,
    pub notes: Option<String>,
    // R53-14 billing 欄位
    pub special_equipment_used: Option<String>,
    pub work_started_at: Option<DateTime<Utc>>,
    pub work_ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 採樣紀錄無欄位需 redact（requester / billing 自由文字屬內部稽核應留全值）
impl AuditRedact for ByproductSample {}

/// 新增採樣紀錄 — body 不帶 animal_id / source_protocol_id，由 service 從來源（euthanasia
/// 單或 animal）內部推導，避免 caller 傳錯（或惡意傳）造成 IDOR 或資料不一致。
/// `euthanasia_id`：`Some` = 強制安樂死單路徑（從單推導）；`None` = 計劃內犧牲路徑（從 animal 推導）。
#[derive(Debug, Deserialize)]
pub struct CreateByproductSampleRequest {
    pub euthanasia_id: Option<Uuid>,
    pub sampled_at: DateTime<Utc>,
    pub sample_content: String,
    /// in-system 研究人員 FK；與 (org + contact) 兩層擇一
    pub requester_user_id: Option<Uuid>,
    /// R53-14: external requester 機構名（與 requester_contact_name 必須同時填）
    pub requester_org_name: Option<String>,
    /// R53-14: external requester 聯絡人姓名
    pub requester_contact_name: Option<String>,
    pub notes: Option<String>,
    // R53-14 billing 欄位
    pub special_equipment_used: Option<String>,
    pub work_started_at: Option<DateTime<Utc>>,
    pub work_ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateByproductSampleRequest {
    pub sampled_at: Option<DateTime<Utc>>,
    pub sample_content: Option<String>,
    pub requester_user_id: Option<Uuid>,
    pub requester_org_name: Option<String>,
    pub requester_contact_name: Option<String>,
    pub notes: Option<String>,
    pub special_equipment_used: Option<String>,
    pub work_started_at: Option<DateTime<Utc>>,
    pub work_ended_at: Option<DateTime<Utc>>,
}

pub struct ByproductSampleService;

impl ByproductSampleService {
    /// 列出特定 euthanasia 的所有採樣紀錄（不含已 soft delete）
    pub async fn list_by_euthanasia(
        pool: &PgPool,
        euthanasia_id: Uuid,
    ) -> Result<Vec<ByproductSample>> {
        let rows = sqlx::query_as::<_, ByproductSample>(
            "SELECT * FROM euthanasia_byproduct_samples \
             WHERE euthanasia_id = $1 AND deleted_at IS NULL \
             ORDER BY sampled_at DESC, created_at DESC",
        )
        .bind(euthanasia_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 列出特定動物的所有採樣紀錄（跨多 euthanasia 通常只一筆）
    pub async fn list_by_animal(pool: &PgPool, animal_id: Uuid) -> Result<Vec<ByproductSample>> {
        let rows = sqlx::query_as::<_, ByproductSample>(
            "SELECT * FROM euthanasia_byproduct_samples \
             WHERE animal_id = $1 AND deleted_at IS NULL \
             ORDER BY sampled_at DESC, created_at DESC",
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 列出特定來源計畫的所有採樣紀錄
    pub async fn list_by_protocol(
        pool: &PgPool,
        protocol_id: Uuid,
    ) -> Result<Vec<ByproductSample>> {
        let rows = sqlx::query_as::<_, ByproductSample>(
            "SELECT * FROM euthanasia_byproduct_samples \
             WHERE source_protocol_id = $1 AND deleted_at IS NULL \
             ORDER BY sampled_at DESC, created_at DESC",
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 取單筆採樣紀錄
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<ByproductSample> {
        let row = sqlx::query_as::<_, ByproductSample>(
            "SELECT * FROM euthanasia_byproduct_samples \
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Byproduct sample not found".into()))?;
        Ok(row)
    }

    /// 新增採樣紀錄 — Service-driven audit (ANIMAL / BYPRODUCT_SAMPLE_CREATE)
    ///
    /// **IDOR 守衛**：`animal_id` 與 `source_protocol_id` 由 service 從
    /// `euthanasia_id` 內部推導（euthanasia_orders.animal_id → animals.iacuc_no
    /// → protocols.id），caller 不能傳。避免「caller 傳 euthanasia_id=A 但
    /// animal_id=B、protocol_id=C」這種 cross-entity 不一致情境。
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateByproductSampleRequest,
    ) -> Result<ByproductSample> {
        let user = actor.require_user()?;
        Self::validate_requester(
            req.requester_user_id,
            req.requester_org_name.as_deref(),
            req.requester_contact_name.as_deref(),
        )?;
        Self::validate_work_time(req.work_started_at, req.work_ended_at)?;

        let euthanasia_id = req
            .euthanasia_id
            .ok_or_else(|| AppError::Validation("此路徑須提供安樂死單（euthanasia_id）".into()))?;
        let mut tx = pool.begin().await?;
        let (animal_id, source_protocol_id) =
            Self::resolve_fks_from_euthanasia_tx(&mut tx, euthanasia_id).await?;
        let row = Self::insert_byproduct_sample_tx(
            &mut tx,
            req,
            Some(euthanasia_id),
            animal_id,
            source_protocol_id,
            user.id,
        )
        .await?;
        Self::audit_create_tx(&mut tx, actor, &row).await?;
        tx.commit().await?;
        Ok(row)
    }

    /// 新增採樣紀錄（計劃內犧牲路徑，#445）— Service-driven audit。
    ///
    /// 與 `create` 對稱，但來源是 **animal**（不是安樂死單）：byproduct 主要來自 SD 填的
    /// 「犧牲單」，那條路徑沒有安樂死單。閘門：動物須已犧牲（`status = euthanized`，由犧牲
    /// 確認 / 安樂死執行設定）。`euthanasia_id` 寫 NULL。
    /// **IDOR 守衛**：`source_protocol_id` 由 service 從 `animals.iacuc_no` 推導，caller 不傳。
    pub async fn create_for_animal(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateByproductSampleRequest,
    ) -> Result<ByproductSample> {
        let user = actor.require_user()?;
        Self::validate_requester(
            req.requester_user_id,
            req.requester_org_name.as_deref(),
            req.requester_contact_name.as_deref(),
        )?;
        Self::validate_work_time(req.work_started_at, req.work_ended_at)?;

        let mut tx = pool.begin().await?;
        let source_protocol_id =
            Self::resolve_protocol_for_sacrificed_animal_tx(&mut tx, animal_id).await?;
        // euthanasia_id 強制 None（animal-path 不綁安樂死單，即使 body 偷塞也忽略）。
        let row = Self::insert_byproduct_sample_tx(
            &mut tx,
            req,
            None,
            animal_id,
            source_protocol_id,
            user.id,
        )
        .await?;
        Self::audit_create_tx(&mut tx, actor, &row).await?;
        tx.commit().await?;
        Ok(row)
    }

    /// 計劃內犧牲路徑（#445）：從 animal 推導 source_protocol_id，並驗動物已犧牲。
    /// 鎖動物列 → 驗 `status = euthanized`（犧牲確認 / 安樂死執行設定）→ iacuc_no → protocol。
    async fn resolve_protocol_for_sacrificed_animal_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        animal_id: Uuid,
    ) -> Result<Uuid> {
        let row: Option<(crate::models::AnimalStatus, Option<String>)> = sqlx::query_as(
            "SELECT status, iacuc_no FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(animal_id)
        .fetch_optional(&mut **tx)
        .await?;
        let (status, iacuc_no) =
            row.ok_or_else(|| AppError::NotFound("Animal not found".into()))?;
        if status != crate::models::AnimalStatus::Euthanized {
            return Err(AppError::BadRequest(
                "動物尚未犧牲 / 安樂死，無法新增採樣紀錄".into(),
            ));
        }
        let iacuc_no = iacuc_no.ok_or_else(|| {
            AppError::Validation(
                "Animal has no IACUC no.；無法定位來源計畫，無法建立採樣紀錄".into(),
            )
        })?;
        let source_protocol_id: Uuid =
            sqlx::query_scalar("SELECT id FROM protocols WHERE iacuc_no = $1")
                .bind(&iacuc_no)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Source protocol not found".into()))?;
        Ok(source_protocol_id)
    }

    /// `create` private helper：tx 內 INSERT（拆出避免 `create` 超 50 行）。
    async fn insert_byproduct_sample_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        req: &CreateByproductSampleRequest,
        euthanasia_id: Option<Uuid>,
        animal_id: Uuid,
        source_protocol_id: Uuid,
        collector_id: Uuid,
    ) -> Result<ByproductSample> {
        let row = sqlx::query_as::<_, ByproductSample>(
            r#"
            INSERT INTO euthanasia_byproduct_samples (
                id, euthanasia_id, animal_id, source_protocol_id,
                sampled_at, sample_content,
                requester_user_id, requester_org_name, requester_contact_name,
                collector_id, notes,
                special_equipment_used, work_started_at, work_ended_at,
                created_at, updated_at, created_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                $12, $13, $14,
                NOW(), NOW(), $10
            )
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(euthanasia_id)
        .bind(animal_id)
        .bind(source_protocol_id)
        .bind(req.sampled_at)
        .bind(&req.sample_content)
        .bind(req.requester_user_id)
        .bind(req.requester_org_name.as_deref())
        .bind(req.requester_contact_name.as_deref())
        .bind(collector_id)
        .bind(req.notes.as_deref())
        .bind(req.special_equipment_used.as_deref())
        .bind(req.work_started_at)
        .bind(req.work_ended_at)
        .fetch_one(&mut **tx)
        .await?;
        Ok(row)
    }

    /// `create` private helper：tx 內寫入 audit log（拆出避免 `create` 超 50 行）。
    async fn audit_create_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        row: &ByproductSample,
    ) -> Result<()> {
        let display = Self::display_name(row);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "BYPRODUCT_SAMPLE_CREATE",
                entity: Some(AuditEntity::new("byproduct_sample", row.id, &display)),
                data_diff: Some(DataDiff::create_only(row)),
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    /// 更新採樣紀錄 — Service-driven audit (ANIMAL / BYPRODUCT_SAMPLE_UPDATE)
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateByproductSampleRequest,
    ) -> Result<ByproductSample> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;
        let before = Self::lock_byproduct_sample_tx(&mut tx, id).await?;
        Self::validate_merged_invariants(&before, req)?;
        let after = Self::update_byproduct_sample_tx(&mut tx, id, req, &before).await?;
        Self::audit_update_tx(&mut tx, actor, &before, &after).await?;
        tx.commit().await?;
        Ok(after)
    }

    /// `update` / `delete` private helper：SELECT FOR UPDATE 鎖列 + soft-delete 過濾。
    async fn lock_byproduct_sample_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
    ) -> Result<ByproductSample> {
        sqlx::query_as::<_, ByproductSample>(
            "SELECT * FROM euthanasia_byproduct_samples \
             WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Byproduct sample not found".into()))
    }

    /// `update` private helper：caller partial-update 後 merged value 仍需符合
    /// requester 雙路徑 + work_time 區間 invariant（避免 caller 只送一半欄位
    /// 把 row 推到 CHECK violation 邊界）。
    fn validate_merged_invariants(
        before: &ByproductSample,
        req: &UpdateByproductSampleRequest,
    ) -> Result<()> {
        let (user_id, org, contact) = Self::resolve_requester(before, req);
        Self::validate_requester(user_id, org.as_deref(), contact.as_deref())?;

        let start = req.work_started_at.or(before.work_started_at);
        let end = req.work_ended_at.or(before.work_ended_at);
        Self::validate_work_time(start, end)?;
        Ok(())
    }

    /// 計算 update 後最終 requester 三元組（#443）。
    ///
    /// 同時支援部分更新（PATCH）與正確的模式切換（in-system ↔ external）：
    /// 1. 送了 `requester_user_id` → 切換至系統內，清空 external 兩欄。
    /// 2. 送了任一 external 欄位 → 切換至 / 更新 external，清空 user_id，
    ///    未提供的 external 欄位以 before 值 fallback（保留部分更新語意）。
    /// 3. 皆未提供 → 三欄保留 before 原值。
    ///
    /// 逐欄 COALESCE 無法在切換時清空另一型別殘留（#443）；validate 與 write 共用此解析避免漂移。
    fn resolve_requester(
        before: &ByproductSample,
        req: &UpdateByproductSampleRequest,
    ) -> (Option<Uuid>, Option<String>, Option<String>) {
        if req.requester_user_id.is_some() {
            (req.requester_user_id, None, None)
        } else if req.requester_org_name.is_some() || req.requester_contact_name.is_some() {
            (
                None,
                req.requester_org_name
                    .clone()
                    .or_else(|| before.requester_org_name.clone()),
                req.requester_contact_name
                    .clone()
                    .or_else(|| before.requester_contact_name.clone()),
            )
        } else {
            (
                before.requester_user_id,
                before.requester_org_name.clone(),
                before.requester_contact_name.clone(),
            )
        }
    }

    /// `update` private helper：tx 內 partial UPDATE（拆出避免 `update` 超 50 行）。
    async fn update_byproduct_sample_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
        req: &UpdateByproductSampleRequest,
        before: &ByproductSample,
    ) -> Result<ByproductSample> {
        // requester 群組整組覆寫（#443）：以 resolve_requester 算出的最終三元組直接寫入
        // （keep 分支已回填 before 值，故可直接賦值；不用 COALESCE 才能正確清空切換殘留）。
        let (req_user_id, req_org, req_contact) = Self::resolve_requester(before, req);
        let row = sqlx::query_as::<_, ByproductSample>(
            r#"
            UPDATE euthanasia_byproduct_samples SET
                sampled_at             = COALESCE($2, sampled_at),
                sample_content         = COALESCE($3, sample_content),
                requester_user_id      = $4,
                requester_org_name     = $5,
                requester_contact_name = $6,
                notes                  = COALESCE($7, notes),
                special_equipment_used = COALESCE($8, special_equipment_used),
                work_started_at        = COALESCE($9, work_started_at),
                work_ended_at          = COALESCE($10, work_ended_at),
                updated_at             = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.sampled_at)
        .bind(req.sample_content.as_deref())
        .bind(req_user_id)
        .bind(req_org.as_deref())
        .bind(req_contact.as_deref())
        .bind(req.notes.as_deref())
        .bind(req.special_equipment_used.as_deref())
        .bind(req.work_started_at)
        .bind(req.work_ended_at)
        .fetch_one(&mut **tx)
        .await?;
        Ok(row)
    }

    /// `update` private helper：tx 內寫入 audit log（before/after diff）。
    async fn audit_update_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        before: &ByproductSample,
        after: &ByproductSample,
    ) -> Result<()> {
        let display = Self::display_name(after);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "BYPRODUCT_SAMPLE_UPDATE",
                entity: Some(AuditEntity::new("byproduct_sample", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(before), Some(after))),
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    /// 刪除採樣紀錄（soft delete）— Service-driven audit (ANIMAL / BYPRODUCT_SAMPLE_DELETE)
    pub async fn delete(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;
        let before = Self::lock_byproduct_sample_tx(&mut tx, id).await?;

        sqlx::query(
            "UPDATE euthanasia_byproduct_samples \
             SET deleted_at = NOW(), updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        let display = Self::display_name(&before);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "BYPRODUCT_SAMPLE_DELETE",
                entity: Some(AuditEntity::new("byproduct_sample", before.id, &display)),
                // #443：用 delete_only(before) 記錄完整刪除前快照，而非 compute(before, after)
                // 把 soft-delete 誤記成只改 deleted_at/updated_at 的 UPDATE（與 create_only 對稱）。
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// R53-14: requester 雙路徑驗證 — in-system FK 或 (org + contact 兩欄都非空)。
    /// migration CHECK 也擋，但 service 端先給乾淨錯誤（vs sqlx CHECK violation）。
    fn validate_requester(
        user_id: Option<Uuid>,
        org: Option<&str>,
        contact: Option<&str>,
    ) -> Result<()> {
        if user_id.is_some() {
            return Ok(());
        }
        let org_present = org.map(|s| !s.trim().is_empty()).unwrap_or(false);
        let contact_present = contact.map(|s| !s.trim().is_empty()).unwrap_or(false);
        if !org_present || !contact_present {
            return Err(AppError::Validation(
                "requester：須提供 in-system FK 或同時填入機構（requester_org_name）+ 聯絡人（requester_contact_name）"
                    .into(),
            ));
        }
        Ok(())
    }

    /// R53-14: 工作時間區間驗證 — 兩欄都有值時 end >= start（單欄為 NULL 視為合法）。
    /// migration CHECK 也擋，但 service 端先給乾淨錯誤。
    fn validate_work_time(
        started_at: Option<DateTime<Utc>>,
        ended_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        if let (Some(s), Some(e)) = (started_at, ended_at) {
            if e < s {
                return Err(AppError::Validation(
                    "work_ended_at 不可早於 work_started_at".into(),
                ));
            }
        }
        Ok(())
    }

    /// 從 euthanasia_id 推導 (animal_id, source_protocol_id)。IDOR 守衛：
    /// caller 不傳 FK，避免 cross-entity 不一致。
    ///
    /// - euthanasia_orders.animal_id → animal_id
    /// - animals.iacuc_no → protocols.iacuc_no → source_protocol_id
    ///
    /// 任一 lookup 失敗回 `AppError::NotFound`（含 animal 已 soft-delete 或
    /// 動物無 iacuc_no、無對應 protocol 等情境）。
    async fn resolve_fks_from_euthanasia_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        euthanasia_id: Uuid,
    ) -> Result<(Uuid, Uuid)> {
        // Step 1: euthanasia_orders → animal_id
        let animal_id: Uuid =
            sqlx::query_scalar("SELECT animal_id FROM euthanasia_orders WHERE id = $1")
                .bind(euthanasia_id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Euthanasia order not found".into()))?;

        // Step 2: animals → iacuc_no（同時驗證 animal 未 soft-delete）
        let iacuc_no: Option<String> =
            sqlx::query_scalar("SELECT iacuc_no FROM animals WHERE id = $1 AND deleted_at IS NULL")
                .bind(animal_id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Animal not found".into()))?;
        let iacuc_no = iacuc_no.ok_or_else(|| {
            AppError::Validation(
                "Animal has no IACUC no.；無法定位來源計畫，無法建立採樣紀錄".into(),
            )
        })?;

        // Step 3: protocols → id
        let source_protocol_id: Uuid =
            sqlx::query_scalar("SELECT id FROM protocols WHERE iacuc_no = $1")
                .bind(&iacuc_no)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Source protocol not found".into()))?;

        Ok((animal_id, source_protocol_id))
    }

    fn display_name(row: &ByproductSample) -> String {
        format!(
            "sample#{} / animal#{} / protocol#{}",
            row.id, row.animal_id, row.source_protocol_id
        )
    }

    /// R53-15: 月結報表 — 時間區間內的 byproduct samples（JOIN enriched）
    ///
    /// R86-7：這是財務紀錄，可見性真相源是 `ebs.deleted_at`（WHERE 子句），
    /// 不可讓 `animals.deleted_at` 連帶湊入 JOIN 條件——動物軟刪不代表已計費的
    /// 採樣紀錄要跟著從月結報表消失（軟刪動物在 `animals` 仍有列，JOIN 本身不受影響）。
    pub async fn list_monthly_report(
        pool: &PgPool,
        start: Option<chrono::NaiveDate>,
        end: Option<chrono::NaiveDate>,
    ) -> Result<Vec<ByproductMonthlyRow>> {
        let rows = sqlx::query_as::<_, ByproductMonthlyRow>(
            r#"
            SELECT
                ebs.sampled_at,
                pr.iacuc_no,
                pr.title AS protocol_title,
                a.ear_tag,
                CASE
                    WHEN ebs.requester_user_id IS NOT NULL THEN ru.display_name
                    ELSE COALESCE(ebs.requester_org_name, '') || '／' || COALESCE(ebs.requester_contact_name, '')
                END AS requester_display,
                ebs.sample_content,
                cu.display_name AS collector_name,
                ebs.id
            FROM euthanasia_byproduct_samples ebs
            JOIN animals a    ON ebs.animal_id = a.id
            JOIN protocols pr ON ebs.source_protocol_id = pr.id
            JOIN users cu     ON ebs.collector_id = cu.id
            LEFT JOIN users ru ON ebs.requester_user_id = ru.id
            WHERE ebs.deleted_at IS NULL
              AND ($1::date IS NULL OR ebs.sampled_at >= $1::date::timestamptz)
              AND ($2::date IS NULL OR ebs.sampled_at < ($2::date + 1)::timestamptz)
            ORDER BY ebs.sampled_at DESC
            LIMIT 5000
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

/// R53-15: 月結報表 row（JOIN enriched）
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ByproductMonthlyRow {
    pub sampled_at: DateTime<Utc>,
    pub iacuc_no: Option<String>,
    pub protocol_title: Option<String>,
    pub ear_tag: String,
    pub requester_display: Option<String>,
    pub sample_content: String,
    pub collector_name: Option<String>,
    pub id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn validate_requester_rejects_all_empty() {
        let r = ByproductSampleService::validate_requester(None, None, None);
        assert!(r.is_err());
    }

    #[test]
    fn validate_requester_rejects_org_only() {
        // 機構填了但聯絡人沒填 — 帳務無法對人，拒絕
        let r = ByproductSampleService::validate_requester(None, Some("國防醫學大學"), None);
        assert!(r.is_err());
    }

    #[test]
    fn validate_requester_rejects_contact_only() {
        // 聯絡人填了但機構沒填 — 帳務無法對單位，拒絕
        let r = ByproductSampleService::validate_requester(None, None, Some("王教授"));
        assert!(r.is_err());
    }

    #[test]
    fn validate_requester_rejects_whitespace_only() {
        let r = ByproductSampleService::validate_requester(None, Some("   "), Some("\t"));
        assert!(r.is_err());
    }

    #[test]
    fn validate_requester_accepts_user_id_only() {
        // in-system FK 不需要 org / contact（從 user 推導）
        let r = ByproductSampleService::validate_requester(Some(Uuid::new_v4()), None, None);
        assert!(r.is_ok());
    }

    #[test]
    fn validate_requester_accepts_org_plus_contact() {
        let r =
            ByproductSampleService::validate_requester(None, Some("國防醫學大學"), Some("王教授"));
        assert!(r.is_ok());
    }

    #[test]
    fn validate_requester_accepts_user_id_overrides_external_fields() {
        // 即使 org / contact 沒填，只要有 FK 就 OK
        let r =
            ByproductSampleService::validate_requester(Some(Uuid::new_v4()), Some(""), Some(""));
        assert!(r.is_ok());
    }

    #[test]
    fn validate_work_time_allows_both_none() {
        let r = ByproductSampleService::validate_work_time(None, None);
        assert!(r.is_ok());
    }

    #[test]
    fn validate_work_time_allows_single_endpoint() {
        let now = Utc::now();
        assert!(ByproductSampleService::validate_work_time(Some(now), None).is_ok());
        assert!(ByproductSampleService::validate_work_time(None, Some(now)).is_ok());
    }

    #[test]
    fn validate_work_time_accepts_end_after_start() {
        let start = Utc::now();
        let end = start + Duration::hours(2);
        let r = ByproductSampleService::validate_work_time(Some(start), Some(end));
        assert!(r.is_ok());
    }

    #[test]
    fn validate_work_time_accepts_equal_start_end() {
        // 邊界：end == start 視為合法（短於 1 秒的採樣）
        let t = Utc::now();
        let r = ByproductSampleService::validate_work_time(Some(t), Some(t));
        assert!(r.is_ok());
    }

    #[test]
    fn validate_work_time_rejects_end_before_start() {
        let start = Utc::now();
        let end = start - Duration::hours(1);
        let r = ByproductSampleService::validate_work_time(Some(start), Some(end));
        assert!(r.is_err());
    }

    // ── #443 requester 整組覆寫（resolve_requester）─────────────────────

    /// 測試用：建一個指定 requester 的 before row（其餘欄位填預設）。
    fn sample_with_requester(
        user_id: Option<Uuid>,
        org: Option<&str>,
        contact: Option<&str>,
    ) -> ByproductSample {
        let now = Utc::now();
        ByproductSample {
            id: Uuid::new_v4(),
            euthanasia_id: Some(Uuid::new_v4()),
            animal_id: Uuid::new_v4(),
            source_protocol_id: Uuid::new_v4(),
            sampled_at: now,
            sample_content: "before content".to_string(),
            requester_user_id: user_id,
            requester_org_name: org.map(String::from),
            requester_contact_name: contact.map(String::from),
            collector_id: Uuid::new_v4(),
            notes: None,
            special_equipment_used: None,
            work_started_at: None,
            work_ended_at: None,
            created_at: now,
            updated_at: now,
            created_by: Uuid::new_v4(),
            deleted_at: None,
        }
    }

    fn empty_update() -> UpdateByproductSampleRequest {
        UpdateByproductSampleRequest {
            sampled_at: None,
            sample_content: None,
            requester_user_id: None,
            requester_org_name: None,
            requester_contact_name: None,
            notes: None,
            special_equipment_used: None,
            work_started_at: None,
            work_ended_at: None,
        }
    }

    #[test]
    fn resolve_requester_switch_external_to_internal_clears_org_contact() {
        // before = external（org+contact），req 切到 internal（只送 user_id）
        let before = sample_with_requester(None, Some("國防醫學大學"), Some("王教授"));
        let uid = Uuid::new_v4();
        let req = UpdateByproductSampleRequest {
            requester_user_id: Some(uid),
            ..empty_update()
        };
        let (u, org, contact) = ByproductSampleService::resolve_requester(&before, &req);
        assert_eq!(u, Some(uid));
        assert_eq!(org, None, "切到 internal 應清空 org（不殘留）");
        assert_eq!(contact, None, "切到 internal 應清空 contact（不殘留）");
    }

    #[test]
    fn resolve_requester_switch_internal_to_external_clears_user_id() {
        // before = internal（user_id），req 切到 external（送 org+contact）
        let before = sample_with_requester(Some(Uuid::new_v4()), None, None);
        let req = UpdateByproductSampleRequest {
            requester_org_name: Some("台大醫院".to_string()),
            requester_contact_name: Some("李醫師".to_string()),
            ..empty_update()
        };
        let (u, org, contact) = ByproductSampleService::resolve_requester(&before, &req);
        assert_eq!(u, None, "切到 external 應清空 user_id（不殘留）");
        assert_eq!(org.as_deref(), Some("台大醫院"));
        assert_eq!(contact.as_deref(), Some("李醫師"));
    }

    #[test]
    fn resolve_requester_partial_external_update_preserves_other_field() {
        // before = external（org+contact），req 只更新 contact → org 應保留（PATCH 部分更新語意）
        let before = sample_with_requester(None, Some("國防醫學大學"), Some("王教授"));
        let req = UpdateByproductSampleRequest {
            requester_contact_name: Some("陳教授".to_string()),
            ..empty_update()
        };
        let (u, org, contact) = ByproductSampleService::resolve_requester(&before, &req);
        assert_eq!(u, None);
        assert_eq!(
            org.as_deref(),
            Some("國防醫學大學"),
            "部分更新時應保留未提供的 org 欄位"
        );
        assert_eq!(contact.as_deref(), Some("陳教授"));
    }

    #[test]
    fn resolve_requester_no_requester_fields_keeps_before() {
        // req 完全沒送 requester 欄位（只改其他） → 整組保留 before
        let before = sample_with_requester(None, Some("國防醫學大學"), Some("王教授"));
        let req = UpdateByproductSampleRequest {
            sample_content: Some("new content".to_string()),
            ..empty_update()
        };
        let (u, org, contact) = ByproductSampleService::resolve_requester(&before, &req);
        assert_eq!(u, None);
        assert_eq!(org.as_deref(), Some("國防醫學大學"));
        assert_eq!(contact.as_deref(), Some("王教授"));
    }
}
