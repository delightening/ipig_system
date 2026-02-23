// 治療方式藥物選項 Service

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    CreateTreatmentDrugRequest, ImportFromErpRequest, TreatmentDrugOption, TreatmentDrugQuery,
    UpdateTreatmentDrugRequest,
};

pub struct TreatmentDrugService {
    db: PgPool,
}

impl TreatmentDrugService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// 列表查詢（支援 keyword / category / is_active 篩選）
    pub async fn list(
        &self,
        query: TreatmentDrugQuery,
    ) -> Result<Vec<TreatmentDrugOption>, AppError> {
        let mut sql = String::from("SELECT * FROM treatment_drug_options WHERE 1=1");
        let mut args: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref keyword) = query.keyword {
            sql.push_str(&format!(
                " AND (name ILIKE ${0} OR display_name ILIKE ${0})",
                param_idx
            ));
            args.push(format!("%{}%", keyword));
            param_idx += 1;
        }

        if let Some(ref category) = query.category {
            sql.push_str(&format!(" AND category = ${}", param_idx));
            args.push(category.clone());
            param_idx += 1;
        }

        if let Some(is_active) = query.is_active {
            sql.push_str(&format!(" AND is_active = ${}", param_idx));
            args.push(is_active.to_string());
            // param_idx += 1;  // 最後一個不需要遞增
        }

        sql.push_str(" ORDER BY sort_order, name");

        // 使用 sqlx::query_as 動態綁定
        let mut q = sqlx::query_as::<_, TreatmentDrugOption>(&sql);

        if let Some(ref keyword) = query.keyword {
            q = q.bind(format!("%{}%", keyword));
        }
        if let Some(ref category) = query.category {
            q = q.bind(category);
        }
        if let Some(is_active) = query.is_active {
            q = q.bind(is_active);
        }

        let results = q
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("查詢藥物選項失敗: {}", e)))?;

        Ok(results)
    }

    /// 列表查詢（僅啟用項目，供一般使用者）
    pub async fn list_active(&self) -> Result<Vec<TreatmentDrugOption>, AppError> {
        let results = sqlx::query_as::<_, TreatmentDrugOption>(
            "SELECT * FROM treatment_drug_options WHERE is_active = true ORDER BY sort_order, name",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(format!("查詢啟用藥物選項失敗: {}", e)))?;

        Ok(results)
    }

    /// 建立藥物選項
    pub async fn create(
        &self,
        request: CreateTreatmentDrugRequest,
        created_by: Option<Uuid>,
    ) -> Result<TreatmentDrugOption, AppError> {
        let result = sqlx::query_as::<_, TreatmentDrugOption>(
            r#"
            INSERT INTO treatment_drug_options 
                (name, display_name, default_dosage_unit, available_units, 
                 default_dosage_value, erp_product_id, category, sort_order, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.display_name)
        .bind(&request.default_dosage_unit)
        .bind(&request.available_units)
        .bind(&request.default_dosage_value)
        .bind(request.erp_product_id)
        .bind(&request.category)
        .bind(request.sort_order)
        .bind(created_by)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(format!("建立藥物選項失敗: {}", e)))?;

        Ok(result)
    }

    /// 更新藥物選項
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdateTreatmentDrugRequest,
    ) -> Result<TreatmentDrugOption, AppError> {
        // 先確認存在
        let existing = sqlx::query_as::<_, TreatmentDrugOption>(
            "SELECT * FROM treatment_drug_options WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::Internal(format!("查詢藥物選項失敗: {}", e)))?;

        let existing = match existing {
            Some(e) => e,
            None => return Err(AppError::NotFound(format!("找不到藥物選項 {}", id))),
        };

        let result = sqlx::query_as::<_, TreatmentDrugOption>(
            r#"
            UPDATE treatment_drug_options SET
                name = $2,
                display_name = $3,
                default_dosage_unit = $4,
                available_units = $5,
                default_dosage_value = $6,
                erp_product_id = $7,
                category = $8,
                sort_order = $9,
                is_active = $10,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(request.name.unwrap_or(existing.name))
        .bind(request.display_name.or(existing.display_name))
        .bind(request.default_dosage_unit.or(existing.default_dosage_unit))
        .bind(request.available_units.or(existing.available_units))
        .bind(
            request
                .default_dosage_value
                .or(existing.default_dosage_value),
        )
        .bind(request.erp_product_id.or(existing.erp_product_id))
        .bind(request.category.or(existing.category))
        .bind(request.sort_order.unwrap_or(existing.sort_order))
        .bind(request.is_active.unwrap_or(existing.is_active))
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(format!("更新藥物選項失敗: {}", e)))?;

        Ok(result)
    }

    /// 刪除藥物選項（軟刪除）
    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let rows = sqlx::query(
            "UPDATE treatment_drug_options SET is_active = false, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(format!("刪除藥物選項失敗: {}", e)))?;

        if rows.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("找不到藥物選項 {}", id)));
        }

        Ok(())
    }

    /// 從 ERP 產品匯入
    pub async fn import_from_erp(
        &self,
        request: ImportFromErpRequest,
        created_by: Option<Uuid>,
    ) -> Result<Vec<TreatmentDrugOption>, AppError> {
        let mut imported = Vec::new();

        for product_id in &request.product_ids {
            // 檢查是否已匯入
            let existing = sqlx::query_as::<_, TreatmentDrugOption>(
                "SELECT * FROM treatment_drug_options WHERE erp_product_id = $1",
            )
            .bind(product_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("查詢 ERP 藥物選項失敗: {}", e)))?;

            if existing.is_some() {
                continue; // 已匯入，跳過
            }

            // 取得 ERP 產品資料
            let product = sqlx::query_as::<_, (String, String, Option<String>)>(
                "SELECT name, base_uom, spec FROM products WHERE id = $1 AND is_active = true",
            )
            .bind(product_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("查詢 ERP 產品失敗: {}", e)))?;

            if let Some((name, base_uom, _spec)) = product {
                // 取得產品的所有 UOM
                let uom_conversions = sqlx::query_as::<_, (String,)>(
                    "SELECT uom FROM product_uom_conversions WHERE product_id = $1",
                )
                .bind(product_id)
                .fetch_all(&self.db)
                .await
                .map_err(|e| AppError::Internal(format!("查詢 UOM 失敗: {}", e)))?;

                let mut available_units = vec![base_uom.clone()];
                for (uom,) in &uom_conversions {
                    if !available_units.contains(uom) {
                        available_units.push(uom.clone());
                    }
                }

                let result = sqlx::query_as::<_, TreatmentDrugOption>(
                    r#"
                    INSERT INTO treatment_drug_options 
                        (name, default_dosage_unit, available_units, erp_product_id, 
                         category, created_by)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    RETURNING *
                    "#,
                )
                .bind(&name)
                .bind(&base_uom)
                .bind(&available_units)
                .bind(product_id)
                .bind(request.category.as_deref().unwrap_or("其他"))
                .bind(created_by)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::Internal(format!("匯入藥物選項失敗: {}", e)))?;

                imported.push(result);
            }
        }

        Ok(imported)
    }
}
