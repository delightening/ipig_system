use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::{AuditRedact, DataDiff},
        AnimalBloodTest, AnimalBloodTestItem, AnimalBloodTestWithItems, BloodTestListItem,
        BloodTestPanel, BloodTestPanelWithItems, BloodTestPreset, BloodTestTemplate,
        CreateBloodTestPanelRequest, CreateBloodTestPresetRequest, CreateBloodTestRequest,
        CreateBloodTestTemplateRequest, UpdateBloodTestPanelItemsRequest,
        UpdateBloodTestPanelRequest, UpdateBloodTestPresetRequest, UpdateBloodTestRequest,
        UpdateBloodTestTemplateRequest,
    },
    repositories,
    services::{
        access,
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, SignatureService,
    },
    AppError, Result,
};

/// R30-41: panel ↔ template join 表的快照（給 PANEL_TEMPLATE_CHANGE audit 用）。
/// 兩種視角各一個 struct，before/after 透過 DataDiff::compute 比對 template_ids。
#[derive(Serialize)]
struct PanelMembershipSnapshot {
    panel_id: Uuid,
    template_ids: Vec<Uuid>,
}
impl AuditRedact for PanelMembershipSnapshot {}

#[derive(Serialize)]
struct TemplateMembershipSnapshot {
    template_id: Uuid,
    panel_ids: Vec<Uuid>,
}
impl AuditRedact for TemplateMembershipSnapshot {}

/// `attach_panel_items` 批次查詢的中繼列：panel_id 用來把結果分回各組合，
/// template 以 `#[sqlx(flatten)]` 承接 `t.*`。
/// 定義在模組層級而非函式內，以將 `attach_panel_items` 控制在 CLAUDE.md 的
/// 「函數 ≤50 行」門檻內（原為 51 行）。
#[derive(sqlx::FromRow)]
struct PanelItemRow {
    panel_id: Uuid,
    #[sqlx(flatten)]
    template: BloodTestTemplate,
}

/// R30-41: PANEL_TEMPLATE_CHANGE event_type 常數（CLAUDE.md「魔術字串必為 const」）
const EVT_PANEL_TEMPLATE_CHANGE: &str = "PANEL_TEMPLATE_CHANGE";

/// R32-A8h: blood_test 匯出 SQL row tuple alias（避免 clippy::type_complexity）
/// 順序：test_date / item_name / result_value / result_unit / reference_range
///       / is_abnormal / created_by_name
type BloodTestExportRow = (
    chrono::NaiveDate,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<bool>,
    Option<String>,
);

/// R30-41: 對 Vec<Uuid> 保序去重（先出現先保留）。
/// 用於 audit `after` snapshot — DB INSERT ... ON CONFLICT DO NOTHING 會去重，
/// audit 須與實際 DB 狀態一致（GLP §11.10(b)）。
fn dedup_preserving_order(input: &[Uuid]) -> Vec<Uuid> {
    let mut seen = std::collections::HashSet::with_capacity(input.len());
    input.iter().filter(|u| seen.insert(**u)).copied().collect()
}

pub struct AnimalBloodTestService;

impl AnimalBloodTestService {
    // ============================================
    // 血液檢查管理
    // ============================================

    /// 列出血液檢查紀錄（支援資料隔離）
    pub async fn list_blood_tests(
        pool: &PgPool,
        animal_id: Uuid,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<BloodTestListItem>> {
        let tests = sqlx::query_as::<_, BloodTestListItem>(
            r#"
            SELECT 
                bt.id, bt.animal_id, bt.test_date, bt.lab_name, bt.status,
                bt.remark, bt.vet_read, bt.created_at,
                u.display_name as created_by_name,
                COUNT(bti.id) as item_count,
                COUNT(CASE WHEN bti.is_abnormal THEN 1 END) as abnormal_count
            FROM animal_blood_tests bt
            LEFT JOIN animal_blood_test_items bti
                   ON bti.blood_test_id = bt.id
                  AND bti.superseded_by_id IS NULL  -- R30-16: 只計 current items
            LEFT JOIN users u ON u.id = bt.created_by
            WHERE bt.animal_id = $1 AND bt.deleted_at IS NULL
              AND ($2::timestamptz IS NULL OR bt.created_at > $2)
            GROUP BY bt.id, bt.animal_id, bt.test_date, bt.lab_name, bt.status,
                     bt.remark, bt.vet_read, bt.created_at, u.display_name
            ORDER BY bt.test_date DESC, bt.created_at DESC
            "#,
        )
        .bind(animal_id)
        .bind(after)
        .fetch_all(pool)
        .await?;

        Ok(tests)
    }

    /// R32-A8h：彙整單一動物所有血檢記錄 + 每筆 item 為 flat per-item rows，
    /// 供 PDF 匯出（pdf-service `render-blood-test/from-blood-test-data`）使用。
    ///
    /// 形狀：
    /// ```json
    /// {
    ///   "animal_id": "...",
    ///   "items": [
    ///     {"test_date": "2026-04-01", "item_name": "WBC", "result_value": "8.2",
    ///      "result_unit": "10^3/uL", "reference_range": "5.5-22.0",
    ///      "is_abnormal": false, "created_by_name": "陳獸醫"},
    ///     ...
    ///   ]
    /// }
    /// ```
    /// 排序：blood_test 依 test_date DESC + created_at DESC，items 依 sort_order。
    /// 只回 current items（superseded_by_id IS NULL；對齊 R30-16 append-only 設計）。
    pub async fn list_blood_test_export_rows(
        pool: &PgPool,
        scope: access::Scoped<access::AnimalWrite>,
    ) -> Result<Vec<serde_json::Value>> {
        let animal_id = scope.id();
        let rows = Self::fetch_blood_test_export_rows(pool, animal_id).await?;
        Ok(rows.into_iter().map(Self::map_export_row).collect())
    }

    async fn fetch_blood_test_export_rows(
        pool: &PgPool,
        animal_id: Uuid,
    ) -> Result<Vec<BloodTestExportRow>> {
        // INNER JOIN 確保只回有 current item 的 blood_test row（避免空 product 列）
        let rows: Vec<BloodTestExportRow> = sqlx::query_as(
            r#"
            SELECT
                bt.test_date,
                bti.item_name,
                bti.result_value,
                bti.result_unit,
                bti.reference_range,
                bti.is_abnormal,
                u.display_name AS created_by_name
            FROM animal_blood_tests bt
            INNER JOIN animal_blood_test_items bti
                   ON bti.blood_test_id = bt.id
                  AND bti.superseded_by_id IS NULL
            LEFT JOIN users u ON u.id = bt.created_by
            WHERE bt.animal_id = $1 AND bt.deleted_at IS NULL
            ORDER BY bt.test_date DESC, bt.created_at DESC,
                     bti.sort_order NULLS LAST, bti.created_at
            "#,
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    fn map_export_row(row: BloodTestExportRow) -> serde_json::Value {
        let (date, item_name, value, unit, range, abnormal, by) = row;
        // 合併 result_value + result_unit（"8.2 10^3/uL"）；空值不留前導空格
        let value_with_unit = match (value.as_deref(), unit.as_deref()) {
            (Some(v), Some(u)) if !u.trim().is_empty() => format!("{} {}", v, u),
            (Some(v), _) => v.to_string(),
            _ => String::new(),
        };
        serde_json::json!({
            "test_date": date.to_string(),
            "item_name": item_name.unwrap_or_default(),
            "result_value": value_with_unit,
            "reference_range": range.unwrap_or_default(),
            "is_abnormal": abnormal.unwrap_or(false),
            "created_by_name": by.unwrap_or_default(),
        })
    }

    /// 取得單筆血液檢查（含明細項目）
    pub async fn get_blood_test_by_id(pool: &PgPool, id: Uuid) -> Result<AnimalBloodTestWithItems> {
        let blood_test = sqlx::query_as::<_, AnimalBloodTest>(
            "SELECT * FROM animal_blood_tests WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("血液檢查紀錄不存在".to_string()))?;

        // R30-16: 預設只回 current items（superseded_by_id IS NULL）；
        // 完整修正歷史見 get_blood_test_history_by_id。
        let items = sqlx::query_as::<_, AnimalBloodTestItem>(
            "SELECT * FROM animal_blood_test_items \
             WHERE blood_test_id = $1 AND superseded_by_id IS NULL \
             ORDER BY sort_order, created_at",
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let created_by_name = match blood_test.created_by {
            Some(uid) => repositories::user::find_user_display_name_by_id(pool, uid).await?,
            None => None,
        };

        Ok(AnimalBloodTestWithItems {
            blood_test,
            items,
            created_by_name,
        })
    }

    /// 建立血液檢查 — Service-driven audit
    pub async fn create_blood_test(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateBloodTestRequest,
    ) -> Result<AnimalBloodTestWithItems> {
        let user = actor.require_user()?;
        let created_by = user.id;

        // 取得動物資訊用於 audit 顯示（Gemini PR #178：顯示 IACUC + 耳號 而非 UUID）
        let animal_info = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT ear_tag, iacuc_no FROM animals WHERE id = $1",
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;

        let mut tx = pool.begin().await?;

        // 建立主表
        let blood_test = sqlx::query_as::<_, AnimalBloodTest>(
            r#"
            INSERT INTO animal_blood_tests (animal_id, test_date, lab_name, remark, status, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'completed', $5, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(animal_id)
        .bind(req.test_date)
        .bind(&req.lab_name)
        .bind(&req.remark)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // 建立明細項目
        let mut items = Vec::new();
        for item_input in &req.items {
            let item = sqlx::query_as::<_, AnimalBloodTestItem>(
                r#"
                INSERT INTO animal_blood_test_items
                    (blood_test_id, template_id, item_name, result_value, result_unit, reference_range, is_abnormal, remark, sort_order, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                RETURNING *
                "#
            )
            .bind(blood_test.id)
            .bind(item_input.template_id)
            .bind(&item_input.item_name)
            .bind(&item_input.result_value)
            .bind(&item_input.result_unit)
            .bind(&item_input.reference_range)
            .bind(item_input.is_abnormal)
            .bind(&item_input.remark)
            .bind(item_input.sort_order)
            .fetch_one(&mut *tx)
            .await?;
            items.push(item);
        }

        let display = match animal_info {
            Some((ear_tag, iacuc_no)) => {
                let iacuc = iacuc_no.unwrap_or_else(|| "未指派".to_string());
                format!("[{}] {}", iacuc, ear_tag)
            }
            None => format!("血液檢查紀錄 (animal: {})", animal_id),
        };

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "BLOOD_TEST_CREATE",
                entity: Some(AuditEntity::new("animal_blood_test", animal_id, &display)),
                data_diff: Some(DataDiff::create_only(&blood_test)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        let created_by_name =
            repositories::user::find_user_display_name_by_id(pool, created_by).await?;

        Ok(AnimalBloodTestWithItems {
            blood_test,
            items,
            created_by_name,
        })
    }

    /// 更新血液檢查 — Service-driven audit
    pub async fn update_blood_test(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateBloodTestRequest,
    ) -> Result<AnimalBloodTestWithItems> {
        actor.require_user()?;

        // C1 (GLP) fail-fast：簽章後鎖定的血液檢查拒絕修改
        SignatureService::ensure_not_locked_uuid(pool, "blood_test", id).await?;

        let mut tx = pool.begin().await?;

        // C1 atomic：tx 內以 FOR UPDATE 再次驗證
        SignatureService::ensure_not_locked_uuid_tx(&mut tx, "blood_test", id).await?;

        // 取得 before 狀態（FOR UPDATE 鎖定）
        let before = sqlx::query_as::<_, AnimalBloodTest>(
            "SELECT * FROM animal_blood_tests WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("血液檢查紀錄不存在".to_string()))?;

        let after = sqlx::query_as::<_, AnimalBloodTest>(
            r#"
            UPDATE animal_blood_tests SET
                test_date = COALESCE($2, test_date),
                lab_name = COALESCE($3, lab_name),
                remark = COALESCE($4, remark),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.test_date)
        .bind(&req.lab_name)
        .bind(&req.remark)
        .fetch_one(&mut *tx)
        .await?;

        // R30-16: items 修改改走 correct_item_with_reason 端點（append-only + 必填原因）。
        // 本 endpoint 僅修主表三欄（test_date / lab_name / remark）。

        // audit display：取得動物 IACUC + 耳號
        let animal_info = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT ear_tag, iacuc_no FROM animals WHERE id = $1",
        )
        .bind(after.animal_id)
        .fetch_optional(&mut *tx)
        .await?;

        let display = match animal_info {
            Some((ear_tag, iacuc_no)) => {
                let iacuc = iacuc_no.unwrap_or_else(|| "未指派".to_string());
                format!("[{}] {}", iacuc, ear_tag)
            }
            None => format!("血液檢查紀錄 #{}", id),
        };

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "BLOOD_TEST_UPDATE",
                entity: Some(AuditEntity::new(
                    "animal_blood_test",
                    after.animal_id,
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // 重新取得完整資料
        Self::get_blood_test_by_id(pool, id).await
    }

    /// R30-16: 反查 blood_test 對應 animal_id（用於 handler IDOR 檢查）。
    /// 僅從 active blood_test 反查；soft-deleted parent 直接 NotFound。
    pub async fn resolve_animal_id(pool: &PgPool, blood_test_id: Uuid) -> Result<Uuid> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT animal_id FROM animal_blood_tests \
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(blood_test_id)
        .fetch_optional(pool)
        .await?;
        row.map(|(a,)| a)
            .ok_or_else(|| AppError::NotFound("血液檢查紀錄不存在".to_string()))
    }

    /// R30-16: tx 內鎖 parent blood_test 並驗 soft-delete 狀態，回傳 animal_id。
    async fn lock_active_parent_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        blood_test_id: Uuid,
    ) -> Result<Uuid> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT animal_id FROM animal_blood_tests \
             WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(blood_test_id)
        .fetch_optional(&mut **tx)
        .await?;
        row.map(|(a,)| a).ok_or_else(|| {
            AppError::NotFound("血液檢查紀錄不存在或已刪除，無法修正項目".to_string())
        })
    }

    /// R30-16: tx 內鎖 current item row（必須 superseded_by_id IS NULL）。
    async fn lock_current_item_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item_id: Uuid,
        blood_test_id: Uuid,
    ) -> Result<AnimalBloodTestItem> {
        sqlx::query_as::<_, AnimalBloodTestItem>(
            "SELECT * FROM animal_blood_test_items \
             WHERE id = $1 AND blood_test_id = $2 AND superseded_by_id IS NULL \
             FOR UPDATE",
        )
        .bind(item_id)
        .bind(blood_test_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("血檢 item 不存在或已被修正過（非 current）".to_string()))
    }

    /// R30-16: tx 內 INSERT 新 row（修正後值；自動為 current）。
    async fn insert_corrected_item_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        blood_test_id: Uuid,
        req: &crate::models::CorrectBloodTestItemRequest,
    ) -> Result<AnimalBloodTestItem> {
        Ok(sqlx::query_as::<_, AnimalBloodTestItem>(
            r#"
            INSERT INTO animal_blood_test_items
                (blood_test_id, template_id, item_name, result_value, result_unit,
                 reference_range, is_abnormal, remark, sort_order, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            RETURNING *
            "#,
        )
        .bind(blood_test_id)
        .bind(req.template_id)
        .bind(&req.item_name)
        .bind(&req.result_value)
        .bind(&req.result_unit)
        .bind(&req.reference_range)
        .bind(req.is_abnormal)
        .bind(&req.remark)
        .bind(req.sort_order)
        .fetch_one(&mut **tx)
        .await?)
    }

    /// R30-16: tx 內 UPDATE 原 row 4 欄翻 superseded（trigger 強制一致性）。
    async fn mark_item_superseded_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        old_id: Uuid,
        new_id: Uuid,
        corrector_id: Uuid,
        correction_reason: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE animal_blood_test_items \
             SET superseded_by_id = $1, superseded_at = NOW(), \
                 corrected_by = $2, correction_reason = $3 \
             WHERE id = $4",
        )
        .bind(new_id)
        .bind(corrector_id)
        .bind(correction_reason)
        .bind(old_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// R30-16: tx 內組 audit display（IACUC + 耳號 + 原 item 名稱）。
    async fn build_correct_audit_display_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        blood_test_id: Uuid,
        item_id: Uuid,
        item_name: &str,
    ) -> Result<String> {
        let info = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT a.ear_tag, a.iacuc_no \
             FROM animals a \
             INNER JOIN animal_blood_tests bt ON bt.animal_id = a.id \
             WHERE bt.id = $1",
        )
        .bind(blood_test_id)
        .fetch_optional(&mut **tx)
        .await?;
        Ok(match info {
            Some((ear_tag, iacuc_no)) => {
                let iacuc = iacuc_no.unwrap_or_else(|| "未指派".to_string());
                format!("[{}] {} - 修正血檢項目「{}」", iacuc, ear_tag, item_name)
            }
            None => format!("修正血檢項目 #{}", item_id),
        })
    }

    /// R30-16: 修正單筆血檢 item（append-only supersede）— orchestration only。
    ///
    /// 流程：
    /// 1. 簽章鎖 (pool + tx 雙重 atomic)
    /// 2. lock parent blood_test 並驗 deleted_at IS NULL（防修正已刪除紀錄的 items）
    /// 3. lock current item row（必須 superseded_by_id IS NULL）
    /// 4. INSERT 新 row（自動為 current）
    /// 5. UPDATE 原 row 翻 superseded（trigger 嚴格驗證 4 欄一致）
    /// 6. 寫 BLOOD_TEST_ITEM_CORRECT audit（含 before/after 完整 snapshot）
    pub async fn correct_item_with_reason(
        pool: &PgPool,
        actor: &ActorContext,
        blood_test_id: Uuid,
        item_id: Uuid,
        req: &crate::models::CorrectBloodTestItemRequest,
    ) -> Result<AnimalBloodTestItem> {
        let user = actor.require_user()?;
        let corrector_id = user.id;

        SignatureService::ensure_not_locked_uuid(pool, "blood_test", blood_test_id).await?;
        let mut tx = pool.begin().await?;
        SignatureService::ensure_not_locked_uuid_tx(&mut tx, "blood_test", blood_test_id).await?;

        Self::lock_active_parent_tx(&mut tx, blood_test_id).await?;
        let before = Self::lock_current_item_tx(&mut tx, item_id, blood_test_id).await?;

        let new_item = Self::insert_corrected_item_tx(&mut tx, blood_test_id, req).await?;
        Self::mark_item_superseded_tx(
            &mut tx,
            item_id,
            new_item.id,
            corrector_id,
            &req.correction_reason,
        )
        .await?;

        let display = Self::build_correct_audit_display_tx(
            &mut tx,
            blood_test_id,
            item_id,
            &before.item_name,
        )
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "BLOOD_TEST_ITEM_CORRECT",
                entity: Some(AuditEntity::new(
                    "animal_blood_test_item",
                    item_id,
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&new_item))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(new_item)
    }

    /// R30-16: 取單筆血檢的完整 item 修正歷史（含已 superseded rows）
    ///
    /// 用於 audit / 詳情頁時間軸顯示。預設 `get_blood_test_by_id` 只回 current items。
    pub async fn get_blood_test_history_by_id(
        pool: &PgPool,
        blood_test_id: Uuid,
    ) -> Result<Vec<AnimalBloodTestItem>> {
        let items = sqlx::query_as::<_, AnimalBloodTestItem>(
            "SELECT * FROM animal_blood_test_items \
             WHERE blood_test_id = $1 \
             ORDER BY sort_order, created_at",
        )
        .bind(blood_test_id)
        .fetch_all(pool)
        .await?;
        Ok(items)
    }

    /// 軟刪除血液檢查 — Service-driven audit
    pub async fn soft_delete_blood_test(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let deleted_by = user.id;

        // C1 (GLP) fail-fast：簽章後鎖定的血液檢查拒絕刪除
        SignatureService::ensure_not_locked_uuid(pool, "blood_test", id).await?;

        let mut tx = pool.begin().await?;

        // C1 atomic：tx 內以 FOR UPDATE 再次驗證
        SignatureService::ensure_not_locked_uuid_tx(&mut tx, "blood_test", id).await?;

        // 取得 before（含 animal_id 用於 audit display）
        let before = sqlx::query_as::<_, AnimalBloodTest>(
            "SELECT * FROM animal_blood_tests WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("血液檢查紀錄不存在".to_string()))?;

        let after = sqlx::query_as::<_, AnimalBloodTest>(
            r#"
            UPDATE animal_blood_tests SET
                deleted_at = NOW(),
                deleted_by = $2,
                delete_reason = $3,
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(deleted_by)
        .bind(reason)
        .fetch_one(&mut *tx)
        .await?;

        // audit display：取得動物 IACUC + 耳號
        let animal_info = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT ear_tag, iacuc_no FROM animals WHERE id = $1",
        )
        .bind(before.animal_id)
        .fetch_optional(&mut *tx)
        .await?;

        let display = match animal_info {
            Some((ear_tag, iacuc_no)) => {
                let iacuc = iacuc_no.unwrap_or_else(|| "未指派".to_string());
                format!("[{}] {} (原因: {})", iacuc, ear_tag, reason)
            }
            None => format!("血液檢查紀錄 #{} (原因: {})", id, reason),
        };

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "BLOOD_TEST_DELETE",
                entity: Some(AuditEntity::new(
                    "animal_blood_test",
                    before.animal_id,
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // 血液檢查項目模板管理
    // ============================================

    /// 列出所有血液檢查項目模板
    pub async fn list_blood_test_templates(pool: &PgPool) -> Result<Vec<BloodTestTemplate>> {
        let templates = sqlx::query_as::<_, BloodTestTemplate>(
            "SELECT * FROM blood_test_templates WHERE is_active = true ORDER BY sort_order, code",
        )
        .fetch_all(pool)
        .await?;

        Ok(templates)
    }

    /// 列出所有模板（含停用）- 管理用
    pub async fn list_all_blood_test_templates(pool: &PgPool) -> Result<Vec<BloodTestTemplate>> {
        let templates = sqlx::query_as::<_, BloodTestTemplate>(
            "SELECT * FROM blood_test_templates ORDER BY sort_order, code",
        )
        .fetch_all(pool)
        .await?;

        Ok(templates)
    }

    /// 建立血液檢查項目模板 — Service-driven audit
    pub async fn create_blood_test_template(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateBloodTestTemplateRequest,
    ) -> Result<BloodTestTemplate> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let template = sqlx::query_as::<_, BloodTestTemplate>(
            r#"
            INSERT INTO blood_test_templates (code, name, default_unit, reference_range, default_price, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(&req.code)
        .bind(&req.name)
        .bind(&req.default_unit)
        .bind(&req.reference_range)
        .bind(req.default_price)
        .bind(req.sort_order)
        .fetch_one(&mut *tx)
        .await?;

        // 若有指定分類，寫入 panel_items 關聯
        if let Some(panel_id) = req.panel_id {
            sqlx::query(
                r#"
                INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order)
                VALUES ($1, $2, $3)
                ON CONFLICT (panel_id, template_id) DO NOTHING
                "#,
            )
            .bind(panel_id)
            .bind(template.id)
            .bind(req.sort_order)
            .execute(&mut *tx)
            .await?;
        }

        let display = format!("建立血檢模板: {}", template.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TEMPLATE_CREATE",
                entity: Some(AuditEntity::new(
                    "blood_test_template",
                    template.id,
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&template)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(template)
    }

    /// 更新血液檢查項目模板 — Service-driven audit
    pub async fn update_blood_test_template(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateBloodTestTemplateRequest,
    ) -> Result<BloodTestTemplate> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, BloodTestTemplate>(
            "SELECT * FROM blood_test_templates WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("模板不存在".to_string()))?;

        let after = sqlx::query_as::<_, BloodTestTemplate>(
            r#"
            UPDATE blood_test_templates SET
                name = COALESCE($2, name),
                default_unit = COALESCE($3, default_unit),
                reference_range = COALESCE($4, reference_range),
                default_price = COALESCE($5, default_price),
                sort_order = COALESCE($6, sort_order),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.default_unit)
        .bind(&req.reference_range)
        .bind(req.default_price)
        .bind(req.sort_order)
        .bind(req.is_active)
        .fetch_one(&mut *tx)
        .await?;

        // 若有指定分類，先刪除該 template 的所有 panel 關聯，再寫入新關聯
        // R30-41: 重設 join table 前後快照 panel_ids，發 PANEL_TEMPLATE_CHANGE audit
        if let Some(panel_id) = req.panel_id {
            let before_panel_ids: Vec<Uuid> = sqlx::query_scalar(
                "SELECT panel_id FROM blood_test_panel_items WHERE template_id = $1 ORDER BY panel_id",
            )
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

            sqlx::query("DELETE FROM blood_test_panel_items WHERE template_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            sqlx::query(
                r#"
                INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order)
                VALUES ($1, $2, $3)
                ON CONFLICT (panel_id, template_id) DO NOTHING
                "#,
            )
            .bind(panel_id)
            .bind(id)
            .bind(after.sort_order)
            .execute(&mut *tx)
            .await?;

            // R30-41 follow-up：成員未變動時不寫 audit（避免冗餘日誌）
            let after_panel_ids = vec![panel_id];
            if before_panel_ids != after_panel_ids {
                let before_snap = TemplateMembershipSnapshot {
                    template_id: id,
                    panel_ids: before_panel_ids,
                };
                let after_snap = TemplateMembershipSnapshot {
                    template_id: id,
                    panel_ids: after_panel_ids,
                };
                let change_display =
                    format!("血檢模板分類變更: {} → panel {}", after.name, panel_id);
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry {
                        event_category: "ANIMAL",
                        event_type: EVT_PANEL_TEMPLATE_CHANGE,
                        entity: Some(AuditEntity::new("blood_test_template", id, &change_display)),
                        data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                        request_context: None,
                    },
                )
                .await?;
            }
        }

        let display = format!("更新血檢模板: {}", after.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TEMPLATE_UPDATE",
                entity: Some(AuditEntity::new("blood_test_template", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// 刪除血液檢查項目模板（軟刪除，設為停用）— Service-driven audit
    pub async fn delete_blood_test_template(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, BloodTestTemplate>(
            "SELECT * FROM blood_test_templates WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("模板不存在".to_string()))?;

        let after = sqlx::query_as::<_, BloodTestTemplate>(
            "UPDATE blood_test_templates SET is_active = false, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("停用血檢模板: {}", before.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TEMPLATE_DELETE",
                entity: Some(AuditEntity::new("blood_test_template", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // 血液檢查組合 (Panel) 管理
    // ============================================

    /// N+1 修復：一次撈齊所有指定組合的模板項目再於記憶體分組，
    /// 取代原本「每個組合各查一次」（`list_blood_test_panels` 與
    /// `list_all_blood_test_panels` 兩處重複同一模式）。
    ///
    /// `only_active_items` = 前台用途只收啟用模板；管理用途連停用的一併列出。
    async fn attach_panel_items(
        pool: &PgPool,
        panels: Vec<BloodTestPanel>,
        only_active_items: bool,
    ) -> Result<Vec<BloodTestPanelWithItems>> {
        use std::collections::HashMap;

        if panels.is_empty() {
            return Ok(Vec::new());
        }
        let panel_ids: Vec<Uuid> = panels.iter().map(|p| p.id).collect();

        let rows = sqlx::query_as::<_, PanelItemRow>(
            r#"
            SELECT pi.panel_id, t.*
            FROM blood_test_templates t
            INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
            WHERE pi.panel_id = ANY($1::uuid[])
              AND (NOT $2::bool OR t.is_active = true)
            ORDER BY pi.panel_id, pi.sort_order, t.sort_order, t.code
            "#,
        )
        .bind(&panel_ids)
        .bind(only_active_items)
        .fetch_all(pool)
        .await?;

        let mut items_by_panel: HashMap<Uuid, Vec<BloodTestTemplate>> = HashMap::new();
        for r in rows {
            items_by_panel
                .entry(r.panel_id)
                .or_default()
                .push(r.template);
        }

        // 沒有任何項目的組合不會出現在 join 結果中，對應原本查回空 Vec 的行為。
        Ok(panels
            .into_iter()
            .map(|panel| {
                let items = items_by_panel.remove(&panel.id).unwrap_or_default();
                BloodTestPanelWithItems { panel, items }
            })
            .collect())
    }

    /// 列出所有啟用的組合（含其模板項目）
    pub async fn list_blood_test_panels(pool: &PgPool) -> Result<Vec<BloodTestPanelWithItems>> {
        let panels = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels WHERE is_active = true ORDER BY sort_order, key",
        )
        .fetch_all(pool)
        .await?;

        Self::attach_panel_items(pool, panels, true).await
    }

    /// 列出所有組合（含停用）- 管理用
    pub async fn list_all_blood_test_panels(pool: &PgPool) -> Result<Vec<BloodTestPanelWithItems>> {
        let panels = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels ORDER BY sort_order, key",
        )
        .fetch_all(pool)
        .await?;

        Self::attach_panel_items(pool, panels, false).await
    }

    /// 建立血液檢查組合 — Service-driven audit
    pub async fn create_blood_test_panel(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateBloodTestPanelRequest,
    ) -> Result<BloodTestPanelWithItems> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let panel = sqlx::query_as::<_, BloodTestPanel>(
            r#"
            INSERT INTO blood_test_panels (key, name, icon, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(&req.key)
        .bind(&req.name)
        .bind(&req.icon)
        .bind(req.sort_order)
        .fetch_one(&mut *tx)
        .await?;

        // 建立組合項目關聯
        for (idx, template_id) in req.template_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order) VALUES ($1, $2, $3)"
            )
            .bind(panel.id)
            .bind(template_id)
            .bind(idx as i32)
            .execute(&mut *tx)
            .await?;
        }

        let display = format!("建立血檢組合: {}", panel.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "PANEL_CREATE",
                entity: Some(AuditEntity::new("blood_test_panel", panel.id, &display)),
                data_diff: Some(DataDiff::create_only(&panel)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // 重新載入含 items
        let items = sqlx::query_as::<_, BloodTestTemplate>(
            r#"
            SELECT t.*
            FROM blood_test_templates t
            INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
            WHERE pi.panel_id = $1
            ORDER BY pi.sort_order, t.sort_order
            "#,
        )
        .bind(panel.id)
        .fetch_all(pool)
        .await?;

        Ok(BloodTestPanelWithItems { panel, items })
    }

    /// 更新血液檢查組合 — Service-driven audit
    pub async fn update_blood_test_panel(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateBloodTestPanelRequest,
    ) -> Result<BloodTestPanelWithItems> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("組合不存在".to_string()))?;

        let after = sqlx::query_as::<_, BloodTestPanel>(
            r#"
            UPDATE blood_test_panels SET
                name = COALESCE($2, name),
                icon = COALESCE($3, icon),
                sort_order = COALESCE($4, sort_order),
                is_active = COALESCE($5, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.icon)
        .bind(req.sort_order)
        .bind(req.is_active)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("更新血檢組合: {}", after.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "PANEL_UPDATE",
                entity: Some(AuditEntity::new("blood_test_panel", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        let items = sqlx::query_as::<_, BloodTestTemplate>(
            r#"
            SELECT t.*
            FROM blood_test_templates t
            INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
            WHERE pi.panel_id = $1
            ORDER BY pi.sort_order, t.sort_order
            "#,
        )
        .bind(after.id)
        .fetch_all(pool)
        .await?;

        Ok(BloodTestPanelWithItems {
            panel: after,
            items,
        })
    }

    /// 更新組合內的項目 — Service-driven audit
    pub async fn update_blood_test_panel_items(
        pool: &PgPool,
        actor: &ActorContext,
        panel_id: Uuid,
        req: &UpdateBloodTestPanelItemsRequest,
    ) -> Result<BloodTestPanelWithItems> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        // 檢查組合是否存在
        let panel = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels WHERE id = $1 FOR UPDATE",
        )
        .bind(panel_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("組合不存在".to_string()))?;

        // R30-41: 重設 join table 前後快照 template_ids，發 PANEL_TEMPLATE_CHANGE audit
        let before_template_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT template_id FROM blood_test_panel_items WHERE panel_id = $1 ORDER BY sort_order, template_id",
        )
        .bind(panel_id)
        .fetch_all(&mut *tx)
        .await?;

        // 清空舊關聯
        sqlx::query("DELETE FROM blood_test_panel_items WHERE panel_id = $1")
            .bind(panel_id)
            .execute(&mut *tx)
            .await?;

        // 插入新關聯
        for (idx, template_id) in req.template_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
            )
            .bind(panel_id)
            .bind(template_id)
            .bind(idx as i32)
            .execute(&mut *tx)
            .await?;
        }

        // R30-41 follow-up：req.template_ids 可能含重複，DB INSERT ON CONFLICT 會去重，
        // audit after 必須與 DB 實際狀態一致；保序去重後比對才不誤觸 / 漏觸
        let after_template_ids = dedup_preserving_order(&req.template_ids);
        if before_template_ids != after_template_ids {
            let before_snap = PanelMembershipSnapshot {
                panel_id,
                template_ids: before_template_ids,
            };
            let after_snap = PanelMembershipSnapshot {
                panel_id,
                template_ids: after_template_ids,
            };
            let display = format!("更新血檢組合項目: {}", panel.name);
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "ANIMAL",
                    event_type: EVT_PANEL_TEMPLATE_CHANGE,
                    entity: Some(AuditEntity::new("blood_test_panel", panel_id, &display)),
                    data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                    request_context: None,
                },
            )
            .await?;
        }

        tx.commit().await?;

        let items = sqlx::query_as::<_, BloodTestTemplate>(
            r#"
            SELECT t.*
            FROM blood_test_templates t
            INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
            WHERE pi.panel_id = $1
            ORDER BY pi.sort_order, t.sort_order
            "#,
        )
        .bind(panel.id)
        .fetch_all(pool)
        .await?;

        Ok(BloodTestPanelWithItems { panel, items })
    }

    /// 刪除血液檢查組合（軟刪除）— Service-driven audit
    pub async fn delete_blood_test_panel(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("組合不存在".to_string()))?;

        let after = sqlx::query_as::<_, BloodTestPanel>(
            "UPDATE blood_test_panels SET is_active = false, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("停用血檢組合: {}", before.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "PANEL_DELETE",
                entity: Some(AuditEntity::new("blood_test_panel", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // 血液檢查常用組合 (Preset) 管理
    // ============================================

    /// 列出啟用中的常用組合
    pub async fn list_blood_test_presets(pool: &PgPool) -> Result<Vec<BloodTestPreset>> {
        let presets = sqlx::query_as::<_, BloodTestPreset>(
            "SELECT * FROM blood_test_presets WHERE is_active = true ORDER BY sort_order, name",
        )
        .fetch_all(pool)
        .await?;
        Ok(presets)
    }

    /// 列出所有常用組合（含停用）- 管理用
    pub async fn list_all_blood_test_presets(pool: &PgPool) -> Result<Vec<BloodTestPreset>> {
        let presets = sqlx::query_as::<_, BloodTestPreset>(
            "SELECT * FROM blood_test_presets ORDER BY sort_order, name",
        )
        .fetch_all(pool)
        .await?;
        Ok(presets)
    }

    /// 建立常用組合 — Service-driven audit
    pub async fn create_blood_test_preset(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateBloodTestPresetRequest,
    ) -> Result<BloodTestPreset> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let preset = sqlx::query_as::<_, BloodTestPreset>(
            r#"
            INSERT INTO blood_test_presets (name, icon, panel_keys, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(&req.name)
        .bind(&req.icon)
        .bind(&req.panel_keys)
        .bind(req.sort_order)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("建立常用組合: {}", preset.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "PRESET_CREATE",
                entity: Some(AuditEntity::new("blood_test_preset", preset.id, &display)),
                data_diff: Some(DataDiff::create_only(&preset)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(preset)
    }

    /// 更新常用組合 — Service-driven audit
    pub async fn update_blood_test_preset(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateBloodTestPresetRequest,
    ) -> Result<BloodTestPreset> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, BloodTestPreset>(
            "SELECT * FROM blood_test_presets WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("常用組合不存在".to_string()))?;

        let after = sqlx::query_as::<_, BloodTestPreset>(
            r#"
            UPDATE blood_test_presets SET
                name = COALESCE($2, name),
                icon = COALESCE($3, icon),
                panel_keys = COALESCE($4, panel_keys),
                sort_order = COALESCE($5, sort_order),
                is_active = COALESCE($6, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.icon)
        .bind(&req.panel_keys)
        .bind(req.sort_order)
        .bind(req.is_active)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("更新常用組合: {}", after.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "PRESET_UPDATE",
                entity: Some(AuditEntity::new("blood_test_preset", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// 刪除常用組合（軟刪除）— Service-driven audit
    pub async fn delete_blood_test_preset(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, BloodTestPreset>(
            "SELECT * FROM blood_test_presets WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("常用組合不存在".to_string()))?;

        let after = sqlx::query_as::<_, BloodTestPreset>(
            "UPDATE blood_test_presets SET is_active = false, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("停用常用組合: {}", before.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "PRESET_DELETE",
                entity: Some(AuditEntity::new("blood_test_preset", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
