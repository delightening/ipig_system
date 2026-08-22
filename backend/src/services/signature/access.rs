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
        let related: Option<(Uuid, Uuid)> =
            sqlx::query_as("SELECT pi_user_id, vet_user_id FROM euthanasia_orders WHERE id = $1")
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

    /// CSO #1：計劃「審查核准」簽章的簽署權責檢查（非僅成員資格）。
    ///
    /// 核准簽章屬審查方：僅 IACUC 主席 / 執行秘書 / 系統管理員可簽。
    /// 申請方（PI / CLIENT / CO_EDITOR）即使是 user_protocols 成員亦不得簽自己的核准
    /// （21 CFR §11.50 non-repudiation + separation-of-duties）。操作者身分由簽章 signer_id
    /// + audit 記錄。view/status 仍走 check_protocol_access（成員可看）。
    pub fn check_protocol_signing_authority(current_user: &CurrentUser) -> Result<()> {
        let allowed = current_user.is_admin()
            || current_user.has_role(crate::constants::ROLE_IACUC_CHAIR)
            || current_user.has_role(crate::constants::ROLE_IACUC_STAFF);
        if allowed {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "僅 IACUC 主席或執行秘書可簽署計劃審查核准".into(),
            ))
        }
    }

    /// CSO #2：動物轉讓簽章的簽署權責檢查（非僅計畫成員資格）。
    ///
    /// 轉讓需三方：獸醫師確認動物狀況 + 轉出計劃 PI + 轉入計劃 PI。
    /// 簽署者須為 VET 角色，或為轉出 / 轉入計劃（依 from_iacuc_no / to_iacuc_no）的 PI。
    /// 單純 co-editor / 其他成員不得簽。無 admin escape（依使用者決策）。
    pub async fn check_transfer_signing_authority(
        pool: &PgPool,
        transfer_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_role(crate::constants::ROLE_VET) {
            return Ok(());
        }
        let is_party_pi: Option<(i64,)> = sqlx::query_as(
            r#"SELECT 1 FROM animal_transfers t
               JOIN protocols p
                 ON p.iacuc_no = t.from_iacuc_no
                 OR p.iacuc_no = t.to_iacuc_no
               WHERE t.id = $1 AND p.pi_user_id = $2"#,
        )
        .bind(transfer_id)
        .bind(current_user.id)
        .fetch_optional(pool)
        .await?;

        if is_party_pi.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "僅獸醫師或轉出 / 轉入計劃主持人可簽署此轉讓".into(),
            ))
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
        let has_access: Option<(i64,)> = sqlx::query_as(sqlx::AssertSqlSafe(query))
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
        let has_access: Option<(i64,)> = sqlx::query_as(sqlx::AssertSqlSafe(query))
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
