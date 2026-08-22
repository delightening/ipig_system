use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        CreateProtocolTemplateVersionRequest, ProtocolTemplateVersion,
        ProtocolTemplateVersionResponse, TemplateVersionDocument,
        UpdateProtocolTemplateVersionRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, FileService,
    },
    AppError, Result,
};

/// 計畫書範本版本服務（院區層級，admin 管理）。
pub struct ProtocolTemplateVersionService;

const EVENT_CATEGORY: &str = "PROTOCOL_TEMPLATE";
/// 附件 entity_type（沿用 attachments 存放版本的 SOP / 表單文件）。
const ENTITY_TYPE: &str = "protocol_template_version";

impl ProtocolTemplateVersionService {
    pub async fn list(pool: &PgPool) -> Result<Vec<ProtocolTemplateVersionResponse>> {
        let rows = sqlx::query_as::<_, ProtocolTemplateVersionResponse>(
            r#"
            SELECT t.id, t.version_label, t.effective_date, t.is_current, t.notes,
                   t.created_by, u.display_name AS created_by_name, t.created_at, t.updated_at
            FROM protocol_template_versions t
            LEFT JOIN users u ON u.id = t.created_by
            ORDER BY t.is_current DESC, t.effective_date DESC NULLS LAST, t.created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<ProtocolTemplateVersion> {
        sqlx::query_as::<_, ProtocolTemplateVersion>(
            "SELECT * FROM protocol_template_versions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到計畫書範本版本".into()))
    }

    /// 取得操作者 id（拒絕 Anonymous）。
    fn actor_id(actor: &ActorContext) -> Result<Uuid> {
        actor
            .actor_user_id()
            .ok_or_else(|| AppError::Forbidden("計畫書範本版本須由已登入使用者管理".into()))
    }

    async fn audit_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        event_type: &'static str,
        v: &ProtocolTemplateVersion,
    ) -> Result<()> {
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: EVENT_CATEGORY,
                event_type,
                entity: Some(AuditEntity::new(
                    "protocol_template_version",
                    v.id,
                    &v.version_label,
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateProtocolTemplateVersionRequest,
    ) -> Result<ProtocolTemplateVersion> {
        let created_by = Self::actor_id(actor)?;
        let label = req.version_label.trim();
        if label.is_empty() {
            return Err(AppError::Validation("版本號不可為空白".into()));
        }
        let mut tx = pool.begin().await?;
        let v = sqlx::query_as::<_, ProtocolTemplateVersion>(
            r#"
            INSERT INTO protocol_template_versions
                (id, version_label, effective_date, is_current, notes, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, false, $4, $5, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(label)
        .bind(req.effective_date)
        .bind(req.notes.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;
        Self::audit_tx(&mut tx, actor, "PROTOCOL_TEMPLATE_VERSION_CREATED", &v).await?;
        tx.commit().await?;
        Ok(v)
    }

    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateProtocolTemplateVersionRequest,
    ) -> Result<ProtocolTemplateVersion> {
        Self::actor_id(actor)?;
        // version_label 若有提供，trim 後不可為空白（與 create 一致；避免 COALESCE 靜默不更新）
        let label = req.version_label.as_deref().map(str::trim);
        if let Some(l) = label {
            if l.is_empty() {
                return Err(AppError::Validation("版本號不可為空白".into()));
            }
        }
        let mut tx = pool.begin().await?;
        // 註：effective_date / notes 為全量取代語意（前端編輯表單恆送完整物件，
        // null 代表使用者明確清空）；version_label 未提供時以 COALESCE 保留。
        let v = sqlx::query_as::<_, ProtocolTemplateVersion>(
            r#"
            UPDATE protocol_template_versions SET
                version_label = COALESCE($2, version_label),
                effective_date = $3,
                notes = $4,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(label)
        .bind(req.effective_date)
        .bind(
            req.notes
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty()),
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到計畫書範本版本".into()))?;
        Self::audit_tx(&mut tx, actor, "PROTOCOL_TEMPLATE_VERSION_UPDATED", &v).await?;
        tx.commit().await?;
        Ok(v)
    }

    /// 設為現行版本（清除其他版本的 is_current，僅保留此版本）。
    pub async fn set_current(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<ProtocolTemplateVersion> {
        Self::actor_id(actor)?;
        let mut tx = pool.begin().await?;
        sqlx::query("UPDATE protocol_template_versions SET is_current = false, updated_at = NOW() WHERE is_current = true AND id <> $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        let v = sqlx::query_as::<_, ProtocolTemplateVersion>(
            "UPDATE protocol_template_versions SET is_current = true, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到計畫書範本版本".into()))?;
        Self::audit_tx(&mut tx, actor, "PROTOCOL_TEMPLATE_VERSION_SET_CURRENT", &v).await?;
        tx.commit().await?;
        Ok(v)
    }

    /// 刪除版本：交易內刪除其附件 DB 列 + 版本列 + audit，commit 後再 best-effort 刪實體檔。
    pub async fn delete(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        Self::actor_id(actor)?;
        let mut tx = pool.begin().await?;
        // 收集並刪除附件 DB 列（attachments 與此表無 FK cascade）
        let file_paths: Vec<String> = sqlx::query_scalar(
            "DELETE FROM attachments WHERE entity_type = $1 AND entity_id::text = $2 RETURNING file_path",
        )
        .bind(ENTITY_TYPE)
        .bind(id.to_string())
        .fetch_all(&mut *tx)
        .await?;
        let v = sqlx::query_as::<_, ProtocolTemplateVersion>(
            "DELETE FROM protocol_template_versions WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到計畫書範本版本".into()))?;
        Self::audit_tx(&mut tx, actor, "PROTOCOL_TEMPLATE_VERSION_DELETED", &v).await?;
        tx.commit().await?;
        // commit 後刪實體檔（缺檔不阻斷）
        for path in &file_paths {
            let _ = FileService::delete(path).await;
        }
        Ok(())
    }

    /// 列出版本的文件（沿用 attachments）。
    pub async fn list_documents(pool: &PgPool, id: Uuid) -> Result<Vec<TemplateVersionDocument>> {
        let docs = sqlx::query_as::<_, TemplateVersionDocument>(
            r#"SELECT id, file_name, file_path, file_size::bigint AS file_size, mime_type, created_at
               FROM attachments
               WHERE entity_type = $1 AND entity_id::text = $2
               ORDER BY created_at DESC"#,
        )
        .bind(ENTITY_TYPE)
        .bind(id.to_string())
        .fetch_all(pool)
        .await?;
        Ok(docs)
    }

    /// 刪除版本的某份文件（DB 列 + 實體檔）。
    pub async fn delete_document(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        doc_id: Uuid,
    ) -> Result<()> {
        Self::actor_id(actor)?;
        let file_path: Option<String> = sqlx::query_scalar(
            "DELETE FROM attachments WHERE id = $1 AND entity_type = $2 AND entity_id::text = $3 RETURNING file_path",
        )
        .bind(doc_id)
        .bind(ENTITY_TYPE)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;
        let Some(path) = file_path else {
            return Err(AppError::NotFound("找不到文件".into()));
        };
        FileService::delete(&path).await?;
        Ok(())
    }
}
