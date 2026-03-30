// 存取權限檢查（IDOR 防護）

use sqlx::PgPool;
use uuid::Uuid;

use crate::{middleware::CurrentUser, AppError, Result};

use super::SignatureService;

/// R16-10: 允許在動態 SQL 中使用的 table 名稱白名單
const ALLOWED_RECORD_TABLES: &[&str] = &[
    "animal_observations",
    "animal_sacrifices",
    "animal_surgeries",
    "animal_weights",
    "animal_vaccinations",
    "animal_transfers",
];

impl SignatureService {
    /// 檢查使用者是否有權存取安樂死單據（PI、VET、CHAIR 或管理員）
    pub async fn check_euthanasia_access(
        pool: &PgPool,
        order_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_permission("animal.euthanasia.arbitrate") || current_user.is_admin() {
            return Ok(());
        }
        let related: Option<(Uuid, Uuid)> = sqlx::query_as(
            "SELECT pi_user_id, vet_user_id FROM euthanasia_orders WHERE id = $1",
        )
        .bind(order_id)
        .fetch_optional(pool)
        .await?;

        match related {
            Some((pi_id, vet_id)) if pi_id == current_user.id || vet_id == current_user.id => {
                Ok(())
            }
            Some(_) => Err(AppError::Forbidden("無權存取此安樂死單據".into())),
            None => Err(AppError::NotFound("找不到安樂死單據".into())),
        }
    }

    /// 檢查使用者是否有權存取轉讓記錄（透過動物所屬計畫關聯）
    pub async fn check_transfer_access(
        pool: &PgPool,
        transfer_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
            return Ok(());
        }
        let has_access: Option<(i64,)> = sqlx::query_as(
            r#"SELECT 1 FROM animal_transfers t
               JOIN animals a ON t.animal_id = a.id
               LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
               WHERE t.id = $1 AND up.user_id = $2"#,
        )
        .bind(transfer_id)
        .bind(current_user.id)
        .fetch_optional(pool)
        .await?;

        if has_access.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此轉讓記錄".into()))
        }
    }

    /// 檢查使用者是否有權存取計畫書（PI、共同編輯者、審查委員或管理員）
    pub async fn check_protocol_access(
        pool: &PgPool,
        protocol_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
            return Ok(());
        }
        let has_access: Option<(i64,)> = sqlx::query_as(
            r#"SELECT 1 FROM user_protocols
               WHERE protocol_id = $1 AND user_id = $2"#,
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_optional(pool)
        .await?;

        if has_access.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此計畫書".into()))
        }
    }

    /// 檢查使用者是否有權存取動物記錄（透過動物所屬計畫關聯，記錄 ID 為 i32）
    pub async fn check_animal_record_access(
        pool: &PgPool,
        table: &str,
        record_id: i32,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if !ALLOWED_RECORD_TABLES.contains(&table) {
            return Err(AppError::Validation(format!("不允許的資料表名稱: {table}")));
        }
        if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
            return Ok(());
        }
        let query = format!(
            r#"SELECT 1 FROM {} r
               JOIN animals a ON r.animal_id = a.id
               LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
               WHERE r.id = $1 AND up.user_id = $2"#,
            table
        );
        let has_access: Option<(i64,)> = sqlx::query_as(&query)
            .bind(record_id)
            .bind(current_user.id)
            .fetch_optional(pool)
            .await?;

        if has_access.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此記錄".into()))
        }
    }

    /// 檢查使用者是否有權存取動物記錄（記錄 ID 為 UUID）
    pub async fn check_animal_record_access_uuid(
        pool: &PgPool,
        table: &str,
        record_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if !ALLOWED_RECORD_TABLES.contains(&table) {
            return Err(AppError::Validation(format!("不允許的資料表名稱: {table}")));
        }
        if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
            return Ok(());
        }
        let query = format!(
            r#"SELECT 1 FROM {} r
               JOIN animals a ON r.animal_id = a.id
               LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
               WHERE r.id = $1 AND up.user_id = $2"#,
            table
        );
        let has_access: Option<(i64,)> = sqlx::query_as(&query)
            .bind(record_id)
            .bind(current_user.id)
            .fetch_optional(pool)
            .await?;

        if has_access.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此記錄".into()))
        }
    }
}
