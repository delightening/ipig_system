use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    models::{
        BloodTestTemplate, BloodTestListItem, PigBloodTest, PigBloodTestItem, PigBloodTestWithItems,
        CreateBloodTestRequest, UpdateBloodTestRequest,
        CreateBloodTestTemplateRequest, UpdateBloodTestTemplateRequest,
        BloodTestPanel, BloodTestPanelWithItems,
        CreateBloodTestPanelRequest, UpdateBloodTestPanelRequest, UpdateBloodTestPanelItemsRequest,
    },
    AppError, Result,
};

impl AnimalService {

    // ============================================
    // 血液檢查管理
    // ============================================

    /// 列出血液檢查紀錄
    pub async fn list_blood_tests(pool: &PgPool, pig_id: Uuid) -> Result<Vec<BloodTestListItem>> {
        let tests = sqlx::query_as::<_, BloodTestListItem>(
            r#"
            SELECT 
                bt.id, bt.pig_id, bt.test_date, bt.lab_name, bt.status,
                bt.remark, bt.vet_read, bt.created_at,
                u.display_name as created_by_name,
                COUNT(bti.id) as item_count,
                COUNT(CASE WHEN bti.is_abnormal THEN 1 END) as abnormal_count
            FROM pig_blood_tests bt
            LEFT JOIN pig_blood_test_items bti ON bti.blood_test_id = bt.id
            LEFT JOIN users u ON u.id = bt.created_by
            WHERE bt.pig_id = $1 AND bt.is_deleted = false
            GROUP BY bt.id, bt.pig_id, bt.test_date, bt.lab_name, bt.status,
                     bt.remark, bt.vet_read, bt.created_at, u.display_name
            ORDER BY bt.test_date DESC, bt.created_at DESC
            "#
        )
        .bind(pig_id)
        .fetch_all(pool)
        .await?;

        Ok(tests)
    }

    /// 取得單筆血液檢查（含明細項目）
    pub async fn get_blood_test_by_id(pool: &PgPool, id: Uuid) -> Result<PigBloodTestWithItems> {
        let blood_test = sqlx::query_as::<_, PigBloodTest>(
            "SELECT * FROM pig_blood_tests WHERE id = $1 AND is_deleted = false"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("血液檢查紀錄不存在".to_string()))?;

        let items = sqlx::query_as::<_, PigBloodTestItem>(
            "SELECT * FROM pig_blood_test_items WHERE blood_test_id = $1 ORDER BY sort_order, created_at"
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let created_by_name = if let Some(created_by) = blood_test.created_by {
            sqlx::query_scalar::<_, String>("SELECT display_name FROM users WHERE id = $1")
                .bind(created_by)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        Ok(PigBloodTestWithItems {
            blood_test,
            items,
            created_by_name,
        })
    }

    /// 建立血液檢查
    pub async fn create_blood_test(
        pool: &PgPool,
        pig_id: Uuid,
        req: &CreateBloodTestRequest,
        created_by: Uuid,
    ) -> Result<PigBloodTestWithItems> {
        // 建立主表
        let blood_test = sqlx::query_as::<_, PigBloodTest>(
            r#"
            INSERT INTO pig_blood_tests (pig_id, test_date, lab_name, remark, status, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'completed', $5, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(pig_id)
        .bind(req.test_date)
        .bind(&req.lab_name)
        .bind(&req.remark)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        // 建立明細項目
        let mut items = Vec::new();
        for item_input in &req.items {
            let item = sqlx::query_as::<_, PigBloodTestItem>(
                r#"
                INSERT INTO pig_blood_test_items 
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
            .fetch_one(pool)
            .await?;
            items.push(item);
        }

        let created_by_name = sqlx::query_scalar::<_, String>("SELECT display_name FROM users WHERE id = $1")
            .bind(created_by)
            .fetch_optional(pool)
            .await?;

        Ok(PigBloodTestWithItems {
            blood_test,
            items,
            created_by_name,
        })
    }

    /// 更新血液檢查
    pub async fn update_blood_test(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateBloodTestRequest,
    ) -> Result<PigBloodTestWithItems> {
        let _blood_test = sqlx::query_as::<_, PigBloodTest>(
            r#"
            UPDATE pig_blood_tests SET
                test_date = COALESCE($2, test_date),
                lab_name = COALESCE($3, lab_name),
                remark = COALESCE($4, remark),
                updated_at = NOW()
            WHERE id = $1 AND is_deleted = false
            RETURNING *
            "#
        )
        .bind(id)
        .bind(req.test_date)
        .bind(&req.lab_name)
        .bind(&req.remark)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("血液檢查紀錄不存在".to_string()))?;

        // 如果提供了 items，覆蓋更新
        if let Some(ref new_items) = req.items {
            // 刪除舊項目
            sqlx::query("DELETE FROM pig_blood_test_items WHERE blood_test_id = $1")
                .bind(id)
                .execute(pool)
                .await?;

            // 插入新項目
            for item_input in new_items {
                sqlx::query(
                    r#"
                    INSERT INTO pig_blood_test_items 
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
                .execute(pool)
                .await?;
            }
        }

        // 重新取得完整資料
        Self::get_blood_test_by_id(pool, id).await
    }

    /// 軟刪除血液檢查
    pub async fn soft_delete_blood_test(
        pool: &PgPool,
        id: Uuid,
        reason: &str,
        deleted_by: Uuid,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE pig_blood_tests SET
                is_deleted = true,
                deleted_at = NOW(),
                deleted_by = $2,
                delete_reason = $3,
                updated_at = NOW()
            WHERE id = $1 AND is_deleted = false
            "#
        )
        .bind(id)
        .bind(deleted_by)
        .bind(reason)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("血液檢查紀錄不存在".to_string()));
        }

        Ok(())
    }

    // ============================================
    // 血液檢查項目模板管理
    // ============================================

    /// 列出所有血液檢查項目模板
    pub async fn list_blood_test_templates(pool: &PgPool) -> Result<Vec<BloodTestTemplate>> {
        let templates = sqlx::query_as::<_, BloodTestTemplate>(
            "SELECT * FROM blood_test_templates WHERE is_active = true ORDER BY sort_order, code"
        )
        .fetch_all(pool)
        .await?;

        Ok(templates)
    }

    /// 列出所有模板（含停用）- 管理用
    pub async fn list_all_blood_test_templates(pool: &PgPool) -> Result<Vec<BloodTestTemplate>> {
        let templates = sqlx::query_as::<_, BloodTestTemplate>(
            "SELECT * FROM blood_test_templates ORDER BY sort_order, code"
        )
        .fetch_all(pool)
        .await?;

        Ok(templates)
    }

    /// 建立血液檢查項目模板
    pub async fn create_blood_test_template(
        pool: &PgPool,
        req: &CreateBloodTestTemplateRequest,
    ) -> Result<BloodTestTemplate> {
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
        .fetch_one(pool)
        .await?;

        // 若有指定分類，寫入 panel_items 關聯
        if let Some(panel_id) = req.panel_id {
            sqlx::query(
                r#"
                INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order)
                VALUES ($1, $2, $3)
                ON CONFLICT (panel_id, template_id) DO NOTHING
                "#
            )
            .bind(panel_id)
            .bind(template.id)
            .bind(req.sort_order)
            .execute(pool)
            .await?;
        }

        Ok(template)
    }

    /// 更新血液檢查項目模板
    pub async fn update_blood_test_template(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateBloodTestTemplateRequest,
    ) -> Result<BloodTestTemplate> {
        let template = sqlx::query_as::<_, BloodTestTemplate>(
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
            "#
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.default_unit)
        .bind(&req.reference_range)
        .bind(req.default_price)
        .bind(req.sort_order)
        .bind(req.is_active)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("模板不存在".to_string()))?;

        // 若有指定分類，先刪除該 template 的所有 panel 關聯，再寫入新關聯
        if let Some(panel_id) = req.panel_id {
            sqlx::query("DELETE FROM blood_test_panel_items WHERE template_id = $1")
                .bind(id)
                .execute(pool)
                .await?;

            sqlx::query(
                r#"
                INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order)
                VALUES ($1, $2, $3)
                ON CONFLICT (panel_id, template_id) DO NOTHING
                "#
            )
            .bind(panel_id)
            .bind(id)
            .bind(template.sort_order)
            .execute(pool)
            .await?;
        }

        Ok(template)
    }

    /// 刪除血液檢查項目模板（軟刪除，設為停用）
    pub async fn delete_blood_test_template(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "UPDATE blood_test_templates SET is_active = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("模板不存在".to_string()));
        }

        Ok(())
    }

    // ============================================
    // 血液檢查組合 (Panel) 管理
    // ============================================

    /// 列出所有啟用的組合（含其模板項目）
    pub async fn list_blood_test_panels(pool: &PgPool) -> Result<Vec<BloodTestPanelWithItems>> {
        let panels = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels WHERE is_active = true ORDER BY sort_order, key"
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
                "#
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
            "SELECT * FROM blood_test_panels ORDER BY sort_order, key"
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
                "#
            )
            .bind(panel.id)
            .fetch_all(pool)
            .await?;

            result.push(BloodTestPanelWithItems { panel, items });
        }

        Ok(result)
    }

    /// 建立血液檢查組合
    pub async fn create_blood_test_panel(
        pool: &PgPool,
        req: &CreateBloodTestPanelRequest,
    ) -> Result<BloodTestPanelWithItems> {
        let panel = sqlx::query_as::<_, BloodTestPanel>(
            r#"
            INSERT INTO blood_test_panels (key, name, icon, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(&req.key)
        .bind(&req.name)
        .bind(&req.icon)
        .bind(req.sort_order)
        .fetch_one(pool)
        .await?;

        // 建立組合項目關聯
        for (idx, template_id) in req.template_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order) VALUES ($1, $2, $3)"
            )
            .bind(panel.id)
            .bind(template_id)
            .bind(idx as i32)
            .execute(pool)
            .await?;
        }

        // 重新載入含 items
        let items = sqlx::query_as::<_, BloodTestTemplate>(
            r#"
            SELECT t.*
            FROM blood_test_templates t
            INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
            WHERE pi.panel_id = $1
            ORDER BY pi.sort_order, t.sort_order
            "#
        )
        .bind(panel.id)
        .fetch_all(pool)
        .await?;

        Ok(BloodTestPanelWithItems { panel, items })
    }

    /// 更新血液檢查組合
    pub async fn update_blood_test_panel(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateBloodTestPanelRequest,
    ) -> Result<BloodTestPanelWithItems> {
        let panel = sqlx::query_as::<_, BloodTestPanel>(
            r#"
            UPDATE blood_test_panels SET
                name = COALESCE($2, name),
                icon = COALESCE($3, icon),
                sort_order = COALESCE($4, sort_order),
                is_active = COALESCE($5, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.icon)
        .bind(req.sort_order)
        .bind(req.is_active)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("組合不存在".to_string()))?;

        let items = sqlx::query_as::<_, BloodTestTemplate>(
            r#"
            SELECT t.*
            FROM blood_test_templates t
            INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
            WHERE pi.panel_id = $1
            ORDER BY pi.sort_order, t.sort_order
            "#
        )
        .bind(panel.id)
        .fetch_all(pool)
        .await?;

        Ok(BloodTestPanelWithItems { panel, items })
    }

    /// 更新組合內的項目
    pub async fn update_blood_test_panel_items(
        pool: &PgPool,
        panel_id: Uuid,
        req: &UpdateBloodTestPanelItemsRequest,
    ) -> Result<BloodTestPanelWithItems> {
        // 檢查組合是否存在
        let panel = sqlx::query_as::<_, BloodTestPanel>(
            "SELECT * FROM blood_test_panels WHERE id = $1"
        )
        .bind(panel_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("組合不存在".to_string()))?;

        // 清空舊關聯
        sqlx::query("DELETE FROM blood_test_panel_items WHERE panel_id = $1")
            .bind(panel_id)
            .execute(pool)
            .await?;

        // 插入新關聯
        for (idx, template_id) in req.template_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
            )
            .bind(panel_id)
            .bind(template_id)
            .bind(idx as i32)
            .execute(pool)
            .await?;
        }

        let items = sqlx::query_as::<_, BloodTestTemplate>(
            r#"
            SELECT t.*
            FROM blood_test_templates t
            INNER JOIN blood_test_panel_items pi ON pi.template_id = t.id
            WHERE pi.panel_id = $1
            ORDER BY pi.sort_order, t.sort_order
            "#
        )
        .bind(panel.id)
        .fetch_all(pool)
        .await?;

        Ok(BloodTestPanelWithItems { panel, items })
    }

    /// 刪除血液檢查組合（軟刪除）
    pub async fn delete_blood_test_panel(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "UPDATE blood_test_panels SET is_active = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("組合不存在".to_string()));
        }

        Ok(())
    }
}
