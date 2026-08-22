//! 申請須知（`application_notices`）與簽署紀錄（`protocol_notice_acknowledgements`）
//! 的 SQL primitives。Service 層做策略/權限，SQL 集中於此。
//!
//! 範圍：PR-A 僅資料存取。簽署流程（建電子簽章 + 送審驗證）於後續 PR 的 service 層。

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ApplicationNotice, ApplicationNoticeResponse, NewApplicationNotice, NewNoticeAcknowledgement,
    ProtocolNoticeAcknowledgement,
};
use crate::{AppError, Result};

pub struct ApplicationNoticeRepository;

impl ApplicationNoticeRepository {
    /// 新增一個須知版本（預設未生效；以 `activate` 切換生效版本）。
    pub async fn insert(pool: &PgPool, input: &NewApplicationNotice) -> Result<ApplicationNotice> {
        let mut tx = pool.begin().await?;
        let notice = Self::insert_tx(&mut tx, input).await?;
        tx.commit().await?;
        Ok(notice)
    }

    /// insert 的 tx 版本（供 service 與 audit 原子寫入）。
    pub async fn insert_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        input: &NewApplicationNotice,
    ) -> Result<ApplicationNotice> {
        let notice = sqlx::query_as::<_, ApplicationNotice>(
            r#"
            INSERT INTO application_notices
                (version_label, title, content, effective_from, attachment_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&input.version_label)
        .bind(&input.title)
        .bind(&input.content)
        .bind(input.effective_from)
        .bind(input.attachment_id)
        .bind(input.created_by)
        .fetch_one(&mut **tx)
        .await?;
        Ok(notice)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ApplicationNotice>> {
        let notice = sqlx::query_as::<_, ApplicationNotice>(
            "SELECT * FROM application_notices WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(notice)
    }

    /// 依版次標籤查須知（version_label UNIQUE，至多一筆）。供匯入承接歷史版次。
    pub async fn find_by_version_label(
        pool: &PgPool,
        version_label: &str,
    ) -> Result<Option<ApplicationNotice>> {
        let notice = sqlx::query_as::<_, ApplicationNotice>(
            "SELECT * FROM application_notices WHERE version_label = $1",
        )
        .bind(version_label)
        .fetch_optional(pool)
        .await?;
        Ok(notice)
    }

    /// 取當前生效須知（至多一筆，由 partial unique index 保證）。
    pub async fn find_active(pool: &PgPool) -> Result<Option<ApplicationNotice>> {
        let notice = sqlx::query_as::<_, ApplicationNotice>(
            "SELECT * FROM application_notices WHERE is_active = true",
        )
        .fetch_optional(pool)
        .await?;
        Ok(notice)
    }

    /// 版本列表（含建立者顯示名稱），新生效日在前。
    pub async fn list(pool: &PgPool) -> Result<Vec<ApplicationNoticeResponse>> {
        let rows = sqlx::query_as::<_, ApplicationNoticeResponse>(
            r#"
            SELECT n.id, n.version_label, n.title, n.content, n.attachment_id,
                   n.effective_from, n.is_active, n.created_by,
                   u.display_name AS created_by_name, n.created_at
            FROM application_notices n
            LEFT JOIN users u ON n.created_by = u.id
            ORDER BY n.effective_from DESC, n.created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn exists_version_label(pool: &PgPool, version_label: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM application_notices WHERE version_label = $1)",
        )
        .bind(version_label)
        .fetch_one(pool)
        .await?;
        Ok(exists)
    }

    /// 設為生效版本：停用其他、啟用指定（同 tx，維持「至多一個生效」不變量）。
    pub async fn activate(pool: &PgPool, id: Uuid) -> Result<ApplicationNotice> {
        let mut tx = pool.begin().await?;
        let notice = Self::activate_tx(&mut tx, id).await?;
        tx.commit().await?;
        Ok(notice)
    }

    /// activate 的 tx 版本：回傳啟用後的版本（供 service audit）。
    pub async fn activate_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
    ) -> Result<ApplicationNotice> {
        sqlx::query("UPDATE application_notices SET is_active = false WHERE is_active = true")
            .execute(&mut **tx)
            .await?;
        let notice = sqlx::query_as::<_, ApplicationNotice>(
            "UPDATE application_notices SET is_active = true WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Application notice not found".to_string()))?;
        Ok(notice)
    }

    /// 更新某版本的正文（限尚無簽署引用時，由 service 層守衛）。回傳更新後版本（供 audit）。
    pub async fn update_content_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
        content: &str,
    ) -> Result<ApplicationNotice> {
        let notice = sqlx::query_as::<_, ApplicationNotice>(
            "UPDATE application_notices SET content = $2 WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(content)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Application notice not found".to_string()))?;
        Ok(notice)
    }
}

pub struct NoticeAcknowledgementRepository;

impl NoticeAcknowledgementRepository {
    /// 建立或覆寫某計劃的簽署紀錄（一計劃一筆；重簽走 upsert）。
    pub async fn upsert(
        pool: &PgPool,
        input: &NewNoticeAcknowledgement,
    ) -> Result<ProtocolNoticeAcknowledgement> {
        let mut tx = pool.begin().await?;
        let ack = Self::upsert_tx(&mut tx, input).await?;
        tx.commit().await?;
        Ok(ack)
    }

    /// upsert 的 tx 版本（供 acknowledge service 與簽章原子寫入）。
    pub async fn upsert_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        input: &NewNoticeAcknowledgement,
    ) -> Result<ProtocolNoticeAcknowledgement> {
        let ack = sqlx::query_as::<_, ProtocolNoticeAcknowledgement>(
            r#"
            INSERT INTO protocol_notice_acknowledgements
                (protocol_id, notice_id, signer_id, signature_id, notice_attachment_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (protocol_id) DO UPDATE SET
                notice_id            = EXCLUDED.notice_id,
                signer_id            = EXCLUDED.signer_id,
                signature_id         = EXCLUDED.signature_id,
                notice_attachment_id = EXCLUDED.notice_attachment_id,
                acknowledged_at      = NOW()
            RETURNING *
            "#,
        )
        .bind(input.protocol_id)
        .bind(input.notice_id)
        .bind(input.signer_id)
        .bind(input.signature_id)
        .bind(input.notice_attachment_id)
        .fetch_one(&mut **tx)
        .await?;
        Ok(ack)
    }

    /// 匯入舊計劃的歷史須知簽署（方案 A）：無電子簽章（signature_id NULL）+ 紙本掃描
    /// attachment + 歷史簽署日期（未提供時用 NOW()）。一計劃一筆（重匯 upsert）。
    pub async fn insert_legacy_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        protocol_id: Uuid,
        notice_id: Uuid,
        signer_id: Uuid,
        notice_attachment_id: Option<Uuid>,
        acknowledged_at: Option<DateTime<Utc>>,
    ) -> Result<ProtocolNoticeAcknowledgement> {
        let ack = sqlx::query_as::<_, ProtocolNoticeAcknowledgement>(
            r#"
            INSERT INTO protocol_notice_acknowledgements
                (protocol_id, notice_id, signer_id, signature_id, notice_attachment_id, acknowledged_at)
            VALUES ($1, $2, $3, NULL, $4, COALESCE($5, NOW()))
            ON CONFLICT (protocol_id) DO UPDATE SET
                notice_id            = EXCLUDED.notice_id,
                signer_id            = EXCLUDED.signer_id,
                signature_id         = NULL,
                notice_attachment_id = EXCLUDED.notice_attachment_id,
                acknowledged_at      = EXCLUDED.acknowledged_at
            RETURNING *
            "#,
        )
        .bind(protocol_id)
        .bind(notice_id)
        .bind(signer_id)
        .bind(notice_attachment_id)
        .bind(acknowledged_at)
        .fetch_one(&mut **tx)
        .await?;
        Ok(ack)
    }

    pub async fn find_by_protocol(
        pool: &PgPool,
        protocol_id: Uuid,
    ) -> Result<Option<ProtocolNoticeAcknowledgement>> {
        let ack = sqlx::query_as::<_, ProtocolNoticeAcknowledgement>(
            "SELECT * FROM protocol_notice_acknowledgements WHERE protocol_id = $1",
        )
        .bind(protocol_id)
        .fetch_optional(pool)
        .await?;
        Ok(ack)
    }

    /// 某須知版本被多少計劃簽署引用（用於守衛：已被簽署的版本不可改內容）。
    /// tx 版本：與 update 在同一交易內檢查，避免 check→update 之間插入簽署的 TOCTOU。
    pub async fn count_by_notice_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        notice_id: Uuid,
    ) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM protocol_notice_acknowledgements WHERE notice_id = $1",
        )
        .bind(notice_id)
        .fetch_one(&mut **tx)
        .await?;
        Ok(count)
    }
}
