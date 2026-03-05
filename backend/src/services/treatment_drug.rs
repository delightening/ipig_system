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
        let mut qb =
            sqlx::QueryBuilder::new("SELECT * FROM treatment_drug_options WHERE 1=1");

        if let Some(ref keyword) = query.keyword {
            let pattern = format!("%{}%", keyword);
            qb.push(" AND (name ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR display_name ILIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        if let Some(ref category) = query.category {
            qb.push(" AND category = ");
            qb.push_bind(category.clone());
        }

        if let Some(is_active) = query.is_active {
            qb.push(" AND is_active = ");
            qb.push_bind(is_active);
        }

        qb.push(" ORDER BY sort_order, name");

        let results = qb
            .build_query_as::<TreatmentDrugOption>()
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

    /// 依業務鍵查詢啟用中的藥物選項（name、category 正規化後比對）
    fn business_key(name: &str, category: Option<&String>) -> (String, String) {
        let n = name.trim().to_lowercase();
        let c = category.map(|s| s.as_str()).unwrap_or("").to_string();
        (n, c)
    }

    /// 查詢是否已有同業務鍵的啟用項目（排除指定 id，用於 update）
    pub async fn find_active_by_business_key(
        &self,
        name: &str,
        category: Option<&String>,
        exclude_id: Option<Uuid>,
    ) -> Result<Option<TreatmentDrugOption>, AppError> {
        let (n, c) = Self::business_key(name, category);
        let mut qb = sqlx::QueryBuilder::new(
            "SELECT * FROM treatment_drug_options WHERE is_active = true AND lower(trim(name)) = "
        );
        qb.push_bind(n).push(" AND COALESCE(category, '') = ").push_bind(c.clone());
        if let Some(id) = exclude_id {
            qb.push(" AND id != ").push_bind(id);
        }
        qb.push(" LIMIT 1");
        let row = qb
            .build_query_as::<TreatmentDrugOption>()
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("查詢藥物選項失敗: {}", e)))?;
        Ok(row)
    }

    /// 建立藥物選項（業務鍵防重：同名稱＋分類僅允許一筆啟用）
    pub async fn create(
        &self,
        request: CreateTreatmentDrugRequest,
        created_by: Option<Uuid>,
    ) -> Result<TreatmentDrugOption, AppError> {
        if self
            .find_active_by_business_key(&request.name, request.category.as_ref(), None)
            .await?
            .is_some()
        {
            let cat = request.category.as_deref().unwrap_or("（無）");
            return Err(AppError::Conflict(format!(
                "已存在相同「藥物名稱＋分類」的啟用項目：{} / {}，請改為編輯既有項目或使用不同名稱／分類",
                request.name, cat
            )));
        }

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

    /// 更新藥物選項（若變更名稱或分類，須通過業務鍵防重）
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdateTreatmentDrugRequest,
    ) -> Result<TreatmentDrugOption, AppError> {
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

        let new_name = request.name.as_deref().unwrap_or(&existing.name);
        let new_category = request.category.as_ref().or(existing.category.as_ref());
        let (existing_n, existing_c) = Self::business_key(&existing.name, existing.category.as_ref());
        let (new_n, new_c) = Self::business_key(new_name, new_category);
        if (new_n != existing_n || new_c != existing_c)
            && self
                .find_active_by_business_key(new_name, new_category, Some(id))
                .await?
                .is_some()
        {
            let cat = new_category.map(|s| s.as_str()).unwrap_or("（無）");
            return Err(AppError::Conflict(format!(
                "已存在相同「藥物名稱＋分類」的啟用項目：{} / {}，請使用不同名稱或分類",
                new_name, cat
            )));
        }

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

    /// 從 ERP 產品匯入（同 erp_product_id 不重複；同名稱＋分類則更新既有列並連結 ERP）
    pub async fn import_from_erp(
        &self,
        request: ImportFromErpRequest,
        created_by: Option<Uuid>,
    ) -> Result<Vec<TreatmentDrugOption>, AppError> {
        let category = request.category.as_deref().unwrap_or("其他");
        let cat_opt = request.category.as_ref();
        let mut imported = Vec::new();

        for product_id in &request.product_ids {
            let by_erp = sqlx::query_as::<_, TreatmentDrugOption>(
                "SELECT * FROM treatment_drug_options WHERE erp_product_id = $1",
            )
            .bind(product_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("查詢 ERP 藥物選項失敗: {}", e)))?;

            if by_erp.is_some() {
                continue;
            }

            let product = sqlx::query_as::<_, (String, String, Option<String>)>(
                "SELECT name, base_uom, spec FROM products WHERE id = $1 AND is_active = true",
            )
            .bind(product_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("查詢 ERP 產品失敗: {}", e)))?;

            if let Some((name, base_uom, _spec)) = product {
                if let Some(existing) = self
                    .find_active_by_business_key(&name, cat_opt, None)
                    .await?
                {
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

                    let updated = sqlx::query_as::<_, TreatmentDrugOption>(
                        r#"
                        UPDATE treatment_drug_options SET
                            default_dosage_unit = $2,
                            available_units = $3,
                            erp_product_id = $4,
                            updated_at = NOW()
                        WHERE id = $1
                        RETURNING *
                        "#,
                    )
                    .bind(existing.id)
                    .bind(&base_uom)
                    .bind(&available_units)
                    .bind(product_id)
                    .fetch_one(&self.db)
                    .await
                    .map_err(|e| AppError::Internal(format!("更新藥物選項 ERP 連結失敗: {}", e)))?;

                    imported.push(updated);
                    continue;
                }

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
                .bind(category)
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
