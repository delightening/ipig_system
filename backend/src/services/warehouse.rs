use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{CreateWarehouseRequest, UpdateWarehouseRequest, Warehouse, WarehouseQuery},
    AppError, Result,
};

pub struct WarehouseService;

impl WarehouseService {
    /// 自動生成倉庫代碼（流水號格式：WH001, WH002, ...）
    async fn generate_code(pool: &PgPool) -> Result<String> {
        let prefix = "WH";
        
        // 取得所有以 WH 開頭的代碼
        let codes: Vec<String> = sqlx::query_scalar(
            "SELECT code FROM warehouses WHERE code LIKE $1 ORDER BY code DESC"
        )
        .bind(format!("{}%", prefix))
        .fetch_all(pool)
        .await?;

        // 解析序號並找出最大值
        let max_seq = codes
            .iter()
            .filter_map(|code| {
                if code.starts_with(prefix) && code.len() > prefix.len() {
                    let num_str = &code[prefix.len()..];
                    num_str.parse::<i32>().ok()
                } else {
                    None
                }
            })
            .max();

        let seq = max_seq.map(|s| s + 1).unwrap_or(1);
        Ok(format!("{}{:03}", prefix, seq))
    }

    /// 建立倉庫
    pub async fn create(pool: &PgPool, req: &CreateWarehouseRequest) -> Result<Warehouse> {
        // 如果 code 為空或未提供，則自動生成
        let code = if req.code.is_none() || req.code.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()).is_none() {
            Self::generate_code(pool).await?
        } else {
            let provided_code = req.code.as_ref().unwrap().trim().to_string();
            
            // 檢查 code 是否已存在
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM warehouses WHERE code = $1)"
            )
            .bind(&provided_code)
            .fetch_one(pool)
            .await?;

            if exists {
                return Err(AppError::Conflict("Warehouse code already exists".to_string()));
            }
            
            provided_code
        };

        let warehouse = sqlx::query_as::<_, Warehouse>(
            r#"
            INSERT INTO warehouses (id, code, name, address, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, NULL, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&code)
        .bind(&req.name)
        .fetch_one(pool)
        .await?;

        Ok(warehouse)
    }

    /// 取得倉庫列表
    pub async fn list(pool: &PgPool, query: &WarehouseQuery) -> Result<Vec<Warehouse>> {
        let mut sql = String::from("SELECT * FROM warehouses WHERE 1=1");
        
        if query.keyword.is_some() {
            sql.push_str(" AND (code ILIKE $1 OR name ILIKE $1)");
        }
        if query.is_active.is_some() {
            sql.push_str(" AND is_active = $2");
        }
        sql.push_str(" ORDER BY code");

        let warehouses = if let Some(ref kw) = query.keyword {
            let pattern = format!("%{}%", kw);
            if let Some(is_active) = query.is_active {
                sqlx::query_as::<_, Warehouse>(&sql)
                    .bind(&pattern)
                    .bind(is_active)
                    .fetch_all(pool)
                    .await?
            } else {
                sqlx::query_as::<_, Warehouse>(&sql)
                    .bind(&pattern)
                    .fetch_all(pool)
                    .await?
            }
        } else if let Some(is_active) = query.is_active {
            sqlx::query_as::<_, Warehouse>(
                "SELECT * FROM warehouses WHERE is_active = $1 ORDER BY code"
            )
            .bind(is_active)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Warehouse>(
                "SELECT * FROM warehouses ORDER BY code"
            )
            .fetch_all(pool)
            .await?
        };

        Ok(warehouses)
    }

    /// 取得單一倉庫
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Warehouse> {
        let warehouse = sqlx::query_as::<_, Warehouse>(
            "SELECT * FROM warehouses WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

        Ok(warehouse)
    }

    /// 更新倉庫
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateWarehouseRequest) -> Result<Warehouse> {
        let warehouse = sqlx::query_as::<_, Warehouse>(
            r#"
            UPDATE warehouses SET
                name = COALESCE($1, name),
                address = COALESCE($2, address),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#
        )
        .bind(&req.name)
        .bind(&req.address)
        .bind(req.is_active)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

        Ok(warehouse)
    }

    /// 刪除倉庫（軟刪除）
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "UPDATE warehouses SET is_active = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Warehouse not found".to_string()));
        }

        Ok(())
    }
}
