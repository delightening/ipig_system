use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AnimalBloodTest, AnimalBloodTestItem, AnimalBloodTestWithItems,
        BloodTestListItem, BloodTestPanel, BloodTestPanelWithItems, BloodTestPreset,
        BloodTestTemplate, CreateBloodTestPanelRequest, CreateBloodTestPresetRequest,
        CreateBloodTestRequest, CreateBloodTestTemplateRequest, UpdateBloodTestPanelItemsRequest,
        UpdateBloodTestPanelRequest, UpdateBloodTestPresetRequest, UpdateBloodTestRequest,
        UpdateBloodTestTemplateRequest,
    },
    repositories,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, SignatureService,
    },
    AppError, Result,
};

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
            LEFT JOIN animal_blood_test_items bti ON bti.blood_test_id = bt.id
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

    /// 取得單筆血液檢查（含明細項目）
    pub async fn get_blood_test_by_id(pool: &PgPool, id: Uuid) -> Result<AnimalBloodTestWithItems> {
        let blood_test = sqlx::query_as::<_, AnimalBloodTest>(
            "SELECT * FROM animal_blood_tests WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("血液檢查紀錄不存在".to_string()))?;

        let items = sqlx::query_as::<_, AnimalBloodTestItem>(
            "SELECT * FROM animal_blood_test_items WHERE blood_test_id = $1 ORDER BY sort_order, created_at"
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

        // 如果提供了 items，覆蓋更新
        if let Some(ref new_items) = req.items {
            // 刪除舊項目
            sqlx::query("DELETE FROM animal_blood_test_items WHERE blood_test_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            // 插入新項目
            for item_input in new_items {
                sqlx::query(
                    r#"
                    INSERT INTO animal_blood_test_items
                        (blood_test_id, template_id, item_name, result_value, result_unit, reference_range, is_abnormal, remark, sort_order, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                    "#
                )
                .bind(id)
                .bind(item_input.template_id)
                .bind(&item_input.item_name)
                .bind(&item_input.result_value)
                .bind(&item_input.result_unit)
                .bind(&item_input.reference_range)
                .bind(item_input.is_abnormal)
                .bind(&item_input.remark)
                .bind(item_input.sort_order)
                .execute(&mut *tx)
                .await?;
            }
        }

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
                entity: Some(AuditEntity::new("blood_test_template", template.id, &display)),
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
        if let Some(panel_id) = req.panel_id {
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

    /// 列出所有啟用的組合（含其模板項目）
    pub async fn list_blood_test_panels(pool: &PgPool) -> Result<Vec<BloodTestPanelWithItems>> {
        let panels = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels WHERE is_active = true ORDER BY sort_order, key",
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for panel in panels {
            let items = sqlx::query_as::<_, BloodTestTemplate>(
                r#"
                SELECT t.*
                FROM blood_test_templates t
                INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
                WHERE pi.panel_id = $1 AND t.is_active = true
                ORDER BY pi.sort_order, t.sort_order, t.code
                "#,
            )
            .bind(panel.id)
            .fetch_all(pool)
            .await?;

            result.push(BloodTestPanelWithItems { panel, items });
        }

        Ok(result)
    }

    /// 列出所有組合（含停用）- 管理用
    pub async fn list_all_blood_test_panels(pool: &PgPool) -> Result<Vec<BloodTestPanelWithItems>> {
        let panels = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels ORDER BY sort_order, key",
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for panel in panels {
            let items = sqlx::query_as::<_, BloodTestTemplate>(
                r#"
                SELECT t.*
                FROM blood_test_templates t
                INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
                WHERE pi.panel_id = $1
                ORDER BY pi.sort_order, t.sort_order, t.code
                "#,
            )
            .bind(panel.id)
            .fetch_all(pool)
            .await?;

            result.push(BloodTestPanelWithItems { panel, items });
        }

        Ok(result)
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

        Ok(BloodTestPanelWithItems { panel: after, items })
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

        let display = format!("更新血檢組合項目: {}", panel.name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "PANEL_UPDATE",
                entity: Some(AuditEntity::new("blood_test_panel", panel_id, &display)),
                data_diff: Some(DataDiff::empty()),
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
