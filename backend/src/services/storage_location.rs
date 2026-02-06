use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CreateStorageLocationRequest, StorageLocation, StorageLocationQuery,
        StorageLocationWithWarehouse, UpdateStorageLayoutRequest,
        UpdateStorageLocationRequest,
    },
    AppError, Result,
};

pub struct StorageLocationService;

impl StorageLocationService {
    /// 建立儲位
    pub async fn create(
        pool: &PgPool,
        req: &CreateStorageLocationRequest,
    ) -> Result<StorageLocation> {
        // 若沒有提供 code，自動產生一個
        let code = match &req.code {
            Some(c) if !c.is_empty() => c.clone(),
            _ => Self::generate_code(pool, req.warehouse_id).await?,
        };

        // 檢查 code 在該倉庫內是否已存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM storage_locations WHERE warehouse_id = $1 AND code = $2)",
        )
        .bind(req.warehouse_id)
        .bind(&code)
        .fetch_one(pool)
        .await?;

        if exists {
            return Err(AppError::Conflict(
                "Storage location code already exists in this warehouse".to_string(),
            ));
        }

        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            INSERT INTO storage_locations 
                (id, warehouse_id, code, name, location_type, row_index, col_index, 
                 width, height, capacity, color, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(req.warehouse_id)
        .bind(&code)
        .bind(Some(&req.name))
        .bind(req.location_type.as_deref().unwrap_or("shelf"))
        .bind(req.row_index.unwrap_or(0))
        .bind(req.col_index.unwrap_or(0))
        .bind(req.width.unwrap_or(2))
        .bind(req.height.unwrap_or(2))
        .bind(req.capacity)
        .bind(&req.color)
        .bind(&req.config)
        .fetch_one(pool)
        .await?;

        Ok(location)
    }

    /// 列出儲位
    pub async fn list(
        pool: &PgPool,
        query: &StorageLocationQuery,
    ) -> Result<Vec<StorageLocationWithWarehouse>> {
        let mut sql = String::from(
            r#"
            SELECT 
                sl.id, sl.warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                sl.code, sl.name, sl.location_type, sl.row_index, sl.col_index,
                sl.width, sl.height, sl.capacity, sl.current_count, sl.color,
                sl.is_active, sl.config
            FROM storage_locations sl
            JOIN warehouses w ON sl.warehouse_id = w.id
            WHERE 1=1
            "#,
        );

        let mut param_idx = 1;

        if query.warehouse_id.is_some() {
            sql.push_str(&format!(" AND sl.warehouse_id = ${}", param_idx));
            param_idx += 1;
        }
        if query.location_type.is_some() {
            sql.push_str(&format!(" AND sl.location_type = ${}", param_idx));
            param_idx += 1;
        }
        if query.is_active.is_some() {
            sql.push_str(&format!(" AND sl.is_active = ${}", param_idx));
            param_idx += 1;
        }
        if query.keyword.is_some() {
            sql.push_str(&format!(
                " AND (sl.code ILIKE ${} OR sl.name ILIKE ${})",
                param_idx,
                param_idx
            ));
        }

        sql.push_str(" ORDER BY sl.row_index, sl.col_index, sl.code");

        // 動態綁定參數
        let mut query_builder = sqlx::query_as::<_, StorageLocationWithWarehouse>(&sql);

        if let Some(ref warehouse_id) = query.warehouse_id {
            query_builder = query_builder.bind(warehouse_id);
        }
        if let Some(ref location_type) = query.location_type {
            query_builder = query_builder.bind(location_type);
        }
        if let Some(is_active) = query.is_active {
            query_builder = query_builder.bind(is_active);
        }
        if let Some(ref keyword) = query.keyword {
            query_builder = query_builder.bind(format!("%{}%", keyword));
        }

        let locations = query_builder.fetch_all(pool).await?;
        Ok(locations)
    }

    /// 取得單一儲位
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<StorageLocationWithWarehouse> {
        let location = sqlx::query_as::<_, StorageLocationWithWarehouse>(
            r#"
            SELECT 
                sl.id, sl.warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                sl.code, sl.name, sl.location_type, sl.row_index, sl.col_index,
                sl.width, sl.height, sl.capacity, sl.current_count, sl.color,
                sl.is_active, sl.config
            FROM storage_locations sl
            JOIN warehouses w ON sl.warehouse_id = w.id
            WHERE sl.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Storage location not found".to_string()))?;

        Ok(location)
    }

    /// 更新儲位
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateStorageLocationRequest,
    ) -> Result<StorageLocation> {
        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            UPDATE storage_locations SET
                code = COALESCE($1, code),
                name = COALESCE($2, name),
                location_type = COALESCE($3, location_type),
                row_index = COALESCE($4, row_index),
                col_index = COALESCE($5, col_index),
                width = COALESCE($6, width),
                height = COALESCE($7, height),
                capacity = COALESCE($8, capacity),
                color = COALESCE($9, color),
                is_active = COALESCE($10, is_active),
                config = COALESCE($11, config),
                updated_at = NOW()
            WHERE id = $12
            RETURNING *
            "#,
        )
        .bind(&req.code)
        .bind(&req.name)
        .bind(&req.location_type)
        .bind(req.row_index)
        .bind(req.col_index)
        .bind(req.width)
        .bind(req.height)
        .bind(req.capacity)
        .bind(&req.color)
        .bind(req.is_active)
        .bind(&req.config)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Storage location not found".to_string()))?;

        Ok(location)
    }

    /// 批次更新儲位佈局（拖拽後儲存）
    pub async fn update_layout(
        pool: &PgPool,
        warehouse_id: Uuid,
        req: &UpdateStorageLayoutRequest,
    ) -> Result<Vec<StorageLocation>> {
        let mut tx = pool.begin().await?;

        for item in &req.items {
            sqlx::query(
                r#"
                UPDATE storage_locations SET
                    row_index = $1,
                    col_index = $2,
                    width = $3,
                    height = $4,
                    updated_at = NOW()
                WHERE id = $5 AND warehouse_id = $6
                "#,
            )
            .bind(item.row_index)
            .bind(item.col_index)
            .bind(item.width)
            .bind(item.height)
            .bind(item.id)
            .bind(warehouse_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // 回傳更新後的儲位列表
        let locations = sqlx::query_as::<_, StorageLocation>(
            "SELECT * FROM storage_locations WHERE warehouse_id = $1 ORDER BY row_index, col_index, code",
        )
        .bind(warehouse_id)
        .fetch_all(pool)
        .await?;

        Ok(locations)
    }

    /// 刪除儲位
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM storage_locations WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Storage location not found".to_string()));
        }

        Ok(())
    }

    /// 自動產生儲位代碼
    pub async fn generate_code(pool: &PgPool, warehouse_id: Uuid) -> Result<String> {
        // 取得該倉庫最大的儲位編號
        let max_code: Option<String> = sqlx::query_scalar(
            r#"
            SELECT code FROM storage_locations 
            WHERE warehouse_id = $1 AND code ~ '^[A-Z][0-9]+$'
            ORDER BY code DESC LIMIT 1
            "#,
        )
        .bind(warehouse_id)
        .fetch_optional(pool)
        .await?;

        let next_code = if let Some(code) = max_code {
            // 解析現有代碼，例如 A01 -> A02
            let prefix = code.chars().next().unwrap_or('A');
            let num: i32 = code[1..].parse().unwrap_or(0);
            if num >= 99 {
                // 超過 99，換下一個字母
                let next_prefix = ((prefix as u8) + 1) as char;
                format!("{}01", next_prefix)
            } else {
                format!("{}{:02}", prefix, num + 1)
            }
        } else {
            "A01".to_string()
        };

        Ok(next_code)
    }

    /// 取得儲位庫存明細
    pub async fn get_inventory(
        pool: &PgPool,
        storage_location_id: Uuid,
    ) -> Result<Vec<crate::models::StorageLocationInventoryItem>> {
        let items = sqlx::query_as::<_, crate::models::StorageLocationInventoryItem>(
            r#"
            SELECT 
                sli.id,
                sli.storage_location_id,
                sli.product_id,
                p.sku as product_sku,
                p.name as product_name,
                sli.on_hand_qty,
                p.base_uom,
                sli.batch_no,
                sli.expiry_date,
                sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.storage_location_id = $1 AND sli.on_hand_qty > 0
            ORDER BY p.name, sli.batch_no
            "#,
        )
        .bind(storage_location_id)
        .fetch_all(pool)
        .await?;

        Ok(items)
    }

    /// 更新儲位庫存項目數量
    pub async fn update_inventory_item(
        pool: &PgPool,
        item_id: Uuid,
        req: &crate::models::UpdateStorageLocationInventoryItemRequest,
    ) -> Result<crate::models::StorageLocationInventoryItem> {
        // Validate non-negative quantity (since validator crate doesn't support rust_decimal)
        if req.on_hand_qty < rust_decimal::Decimal::ZERO {
            return Err(AppError::Validation("on_hand_qty must be non-negative".to_string()));
        }

        // 先取得項目資訊
        let storage_location_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT storage_location_id FROM storage_location_inventory WHERE id = $1"
        )
        .bind(item_id)
        .fetch_optional(pool)
        .await?;

        let storage_location_id = storage_location_id
            .ok_or_else(|| AppError::NotFound("Inventory item not found".to_string()))?;

        // 更新庫存數量
        sqlx::query(
            r#"
            UPDATE storage_location_inventory 
            SET on_hand_qty = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(&req.on_hand_qty)
        .bind(item_id)
        .execute(pool)
        .await?;

        // 更新儲位的 current_count
        sqlx::query(
            r#"
            UPDATE storage_locations 
            SET current_count = (
                SELECT COUNT(*) FROM storage_location_inventory 
                WHERE storage_location_id = $1 AND on_hand_qty > 0
            ),
            updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(storage_location_id)
        .execute(pool)
        .await?;

        // 回傳更新後的項目
        let item = sqlx::query_as::<_, crate::models::StorageLocationInventoryItem>(
            r#"
            SELECT 
                sli.id,
                sli.storage_location_id,
                sli.product_id,
                p.sku as product_sku,
                p.name as product_name,
                sli.on_hand_qty,
                p.base_uom,
                sli.batch_no,
                sli.expiry_date,
                sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.id = $1
            "#,
        )
        .bind(item_id)
        .fetch_one(pool)
        .await?;

        Ok(item)
    }

    /// 新增儲位庫存項目
    pub async fn create_inventory_item(
        pool: &PgPool,
        storage_location_id: Uuid,
        req: &crate::models::CreateStorageLocationInventoryItemRequest,
    ) -> Result<crate::models::StorageLocationInventoryItem> {
        // 驗證數量
        if req.on_hand_qty < rust_decimal::Decimal::ZERO {
            return Err(crate::AppError::Validation("Quantity must be non-negative".to_string()));
        }

        // 驗證儲位存在
        let _location = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM storage_locations WHERE id = $1"
        )
        .bind(storage_location_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::AppError::NotFound("Storage location not found".to_string()))?;

        let item_id = Uuid::new_v4();

        // 插入或更新庫存記錄
        sqlx::query(
            r#"
            INSERT INTO storage_location_inventory (
                id, storage_location_id, product_id, on_hand_qty, batch_no, expiry_date, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (storage_location_id, product_id, COALESCE(batch_no, ''), COALESCE(expiry_date, '1970-01-01'::date))
            DO UPDATE SET 
                on_hand_qty = storage_location_inventory.on_hand_qty + EXCLUDED.on_hand_qty,
                updated_at = NOW()
            "#,
        )
        .bind(item_id)
        .bind(storage_location_id)
        .bind(req.product_id)
        .bind(req.on_hand_qty)
        .bind(&req.batch_no)
        .bind(req.expiry_date)
        .execute(pool)
        .await?;

        // 更新儲位的 current_count
        sqlx::query(
            r#"
            UPDATE storage_locations 
            SET current_count = (
                SELECT COUNT(*) FROM storage_location_inventory 
                WHERE storage_location_id = $1 AND on_hand_qty > 0
            ),
            updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(storage_location_id)
        .execute(pool)
        .await?;

        // 查詢並返回新增的項目
        let item = sqlx::query_as::<_, crate::models::StorageLocationInventoryItem>(
            r#"
            SELECT 
                sli.id,
                sli.storage_location_id,
                sli.product_id,
                p.sku as product_sku,
                p.name as product_name,
                sli.on_hand_qty,
                p.base_uom,
                sli.batch_no,
                sli.expiry_date,
                sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.storage_location_id = $1 AND sli.product_id = $2
            ORDER BY sli.updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(storage_location_id)
        .bind(req.product_id)
        .fetch_one(pool)
        .await?;

        Ok(item)
    }

    /// 調撥儲位庫存 (同倉庫內不需單據)
    pub async fn transfer_inventory(
        pool: &PgPool,
        item_id: Uuid,
        req: &crate::models::TransferStorageLocationInventoryRequest,
    ) -> Result<crate::models::StorageLocationInventoryItem> {
        // 驗證數量
        if req.qty <= rust_decimal::Decimal::ZERO {
            return Err(crate::AppError::Validation("Transfer quantity must be positive".to_string()));
        }

        // 取得來源庫存項目
        let source_item = sqlx::query_as::<_, crate::models::StorageLocationInventoryItem>(
            r#"
            SELECT 
                sli.id,
                sli.storage_location_id,
                sli.product_id,
                p.sku as product_sku,
                p.name as product_name,
                sli.on_hand_qty,
                p.base_uom,
                sli.batch_no,
                sli.expiry_date,
                sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.id = $1
            "#,
        )
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::AppError::NotFound("Inventory item not found".to_string()))?;

        // 驗證數量足夠
        if source_item.on_hand_qty < req.qty {
            return Err(crate::AppError::BusinessRule(format!(
                "Insufficient quantity. Available: {}, Requested: {}",
                source_item.on_hand_qty, req.qty
            )));
        }

        // 驗證來源和目標儲位在同一倉庫
        let same_warehouse: Option<bool> = sqlx::query_scalar(
            r#"
            SELECT (SELECT warehouse_id FROM storage_locations WHERE id = $1) = 
                   (SELECT warehouse_id FROM storage_locations WHERE id = $2)
            "#,
        )
        .bind(source_item.storage_location_id)
        .bind(req.to_storage_location_id)
        .fetch_one(pool)
        .await?;

        if same_warehouse != Some(true) {
            return Err(crate::AppError::BusinessRule(
                "Internal transfer must be within the same warehouse".to_string()
            ));
        }

        let mut tx = pool.begin().await?;

        // 從來源扣減
        sqlx::query(
            r#"
            UPDATE storage_location_inventory 
            SET on_hand_qty = on_hand_qty - $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(req.qty)
        .bind(item_id)
        .execute(&mut *tx)
        .await?;

        // 增加到目標儲位
        let target_item_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO storage_location_inventory (
                id, storage_location_id, product_id, on_hand_qty, batch_no, expiry_date, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (storage_location_id, product_id, COALESCE(batch_no, ''), COALESCE(expiry_date, '1970-01-01'::date))
            DO UPDATE SET 
                on_hand_qty = storage_location_inventory.on_hand_qty + EXCLUDED.on_hand_qty,
                updated_at = NOW()
            "#,
        )
        .bind(target_item_id)
        .bind(req.to_storage_location_id)
        .bind(source_item.product_id)
        .bind(req.qty)
        .bind(&source_item.batch_no)
        .bind(source_item.expiry_date)
        .execute(&mut *tx)
        .await?;

        // 更新來源儲位的 current_count
        sqlx::query(
            r#"
            UPDATE storage_locations 
            SET current_count = (
                SELECT COUNT(*) FROM storage_location_inventory 
                WHERE storage_location_id = $1 AND on_hand_qty > 0
            ),
            updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(source_item.storage_location_id)
        .execute(&mut *tx)
        .await?;

        // 更新目標儲位的 current_count
        sqlx::query(
            r#"
            UPDATE storage_locations 
            SET current_count = (
                SELECT COUNT(*) FROM storage_location_inventory 
                WHERE storage_location_id = $1 AND on_hand_qty > 0
            ),
            updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(req.to_storage_location_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // 回傳更新後的來源項目
        let updated_item = sqlx::query_as::<_, crate::models::StorageLocationInventoryItem>(
            r#"
            SELECT 
                sli.id,
                sli.storage_location_id,
                sli.product_id,
                p.sku as product_sku,
                p.name as product_name,
                sli.on_hand_qty,
                p.base_uom,
                sli.batch_no,
                sli.expiry_date,
                sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.id = $1
            "#,
        )
        .bind(item_id)
        .fetch_one(pool)
        .await?;

        Ok(updated_item)
    }
}
