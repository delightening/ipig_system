use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{
        Amendment, AmendmentStatus, AmendmentType, AmendmentVersion,
        CreateAmendmentRequest, UpdateAmendmentRequest,
    },
    AppError, Result,
};

use super::AmendmentService;

impl AmendmentService {
    /// 產生變更編號
    /// 格式：{IACUC_NO}-R{序號:02}
    /// 例如：PIG-114001-R01, PIG-114001-R02
    pub async fn generate_amendment_no(pool: &PgPool, protocol_id: Uuid) -> Result<(String, i32)> {
        // 取得原計畫的 IACUC NO
        let protocol = sqlx::query!(
            r#"SELECT iacuc_no FROM protocols WHERE id = $1"#,
            protocol_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".into()))?;

        let iacuc_no = protocol.iacuc_no
            .ok_or_else(|| AppError::BadRequest("Protocol has no IACUC number".into()))?;

        // 取得目前最大的 revision_number
        let max_revision = sqlx::query_scalar!(
            r#"SELECT COALESCE(MAX(revision_number), 0) as "max!" FROM amendments WHERE protocol_id = $1"#,
            protocol_id
        )
        .fetch_one(pool)
        .await?;

        let new_revision = max_revision + 1;
        let amendment_no = format!("{}-R{:02}", iacuc_no, new_revision);

        Ok((amendment_no, new_revision))
    }

    /// 建立變更申請
    pub async fn create(
        pool: &PgPool,
        req: &CreateAmendmentRequest,
        created_by: Uuid,
    ) -> Result<Amendment> {
        req.validate()?;

        // 檢查計畫是否存在且為已核准狀態
        let protocol = sqlx::query!(
            r#"SELECT status::text as "status!" FROM protocols WHERE id = $1"#,
            req.protocol_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".into()))?;

        // 只有已核准的計畫才能提交變更申請
        if !["APPROVED", "APPROVED_WITH_CONDITIONS"].contains(&protocol.status.as_str()) {
            return Err(AppError::BadRequest(
                "Only approved protocols can have amendments".into(),
            ));
        }

        let (amendment_no, revision_number) = 
            Self::generate_amendment_no(pool, req.protocol_id).await?;

        let id = Uuid::new_v4();

        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            INSERT INTO amendments (
                id, protocol_id, amendment_no, revision_number,
                amendment_type, status, title, description, 
                change_items, changes_content, created_by
            )
            VALUES ($1, $2, $3, $4, 'PENDING', 'DRAFT', $5, $6, $7, $8, $9)
            RETURNING 
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id
            "#,
            id,
            req.protocol_id,
            amendment_no,
            revision_number,
            req.title,
            req.description,
            req.change_items.as_deref(),
            req.changes_content,
            created_by
        )
        .fetch_one(pool)
        .await?;

        // 記錄狀態歷程
        Self::record_status_change(
            pool,
            id,
            None,
            AmendmentStatus::Draft,
            created_by,
            Some("變更申請草稿建立".to_string()),
        )
        .await?;

        Ok(amendment)
    }

    /// 更新變更申請
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateAmendmentRequest,
    ) -> Result<Amendment> {
        req.validate()?;

        // 檢查是否為草稿狀態
        let current = Self::get_by_id_raw(pool, id).await?;
        if current.status != AmendmentStatus::Draft
            && current.status != AmendmentStatus::RevisionRequired {
            return Err(AppError::BadRequest(
                "Only draft or revision required amendments can be updated".into(),
            ));
        }

        // C2 (GLP §11.10(e)(1)) 防呆守衛：即使 status 守衛失效（例如未來重構誤放行），
        // 已有 approved/rejected 簽章的 amendment 一律拒絕修改。回 409 表示衝突
        // 而非請求格式問題。
        if current.approved_signature_id.is_some() || current.rejected_signature_id.is_some() {
            return Err(AppError::Conflict(
                "Amendment 已被簽章核准/否決，不可修改（GLP §11.10(e)(1)）".into(),
            ));
        }

        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments
            SET 
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                change_items = COALESCE($4, change_items),
                changes_content = COALESCE($5, changes_content),
                updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id
            "#,
            id,
            req.title,
            req.description,
            req.change_items.as_deref(),
            req.changes_content,
        )
        .fetch_one(pool)
        .await?;

        Ok(amendment)
    }


    /// 記錄狀態變更
    pub(crate) async fn record_status_change(
        pool: &PgPool,
        amendment_id: Uuid,
        from_status: Option<AmendmentStatus>,
        to_status: AmendmentStatus,
        changed_by: Uuid,
        remark: Option<String>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO amendment_status_history
                (id, amendment_id, from_status, to_status, changed_by, remark)
            VALUES ($1, $2, ($3::TEXT)::amendment_status, ($4::TEXT)::amendment_status, $5, $6)
            "#,
            Uuid::new_v4(),
            amendment_id,
            from_status.map(|s| s.as_str()),
            to_status.as_str(),
            changed_by,
            remark,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 記錄狀態變更（tx 版）— C2 用，將狀態歷程與決定簽章/UPDATE 寫入同一 tx。
    pub(crate) async fn record_status_change_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        amendment_id: Uuid,
        from_status: Option<AmendmentStatus>,
        to_status: AmendmentStatus,
        changed_by: Uuid,
        remark: Option<String>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO amendment_status_history
                (id, amendment_id, from_status, to_status, changed_by, remark)
            VALUES ($1, $2, ($3::TEXT)::amendment_status, ($4::TEXT)::amendment_status, $5, $6)
            "#,
            Uuid::new_v4(),
            amendment_id,
            from_status.map(|s| s.as_str()),
            to_status.as_str(),
            changed_by,
            remark,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 建立版本快照
    pub(crate) async fn create_version_snapshot(
        pool: &PgPool,
        amendment_id: Uuid,
        submitted_by: Uuid,
    ) -> Result<AmendmentVersion> {
        // 取得目前最大版本號
        let max_version = sqlx::query_scalar!(
            r#"SELECT COALESCE(MAX(version_no), 0) as "max!" FROM amendment_versions WHERE amendment_id = $1"#,
            amendment_id
        )
        .fetch_one(pool)
        .await?;

        let new_version = max_version + 1;

        // 取得目前變更申請內容作為快照
        let current = Self::get_by_id_raw(pool, amendment_id).await?;

        let snapshot = serde_json::json!({
            "title": current.title,
            "description": current.description,
            "change_items": current.change_items,
            "changes_content": current.changes_content,
        });

        let version = sqlx::query_as!(
            AmendmentVersion,
            r#"
            INSERT INTO amendment_versions (id, amendment_id, version_no, content_snapshot, submitted_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, amendment_id, version_no, content_snapshot, submitted_at, submitted_by
            "#,
            Uuid::new_v4(),
            amendment_id,
            new_version,
            snapshot,
            submitted_by,
        )
        .fetch_one(pool)
        .await?;

        Ok(version)
    }

    /// 取得單一變更申請（原始）
    pub(crate) async fn get_by_id_raw(pool: &PgPool, id: Uuid) -> Result<Amendment> {
        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            SELECT 
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id
            FROM amendments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Amendment not found".into()))?;

        Ok(amendment)
    }
}
