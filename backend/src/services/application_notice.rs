//! 動物試驗申請須知版本服務（院區層級，admin 管理）。

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        ApplicationNotice, ApplicationNoticeResponse, CreateApplicationNoticeRequest,
        NewApplicationNotice,
    },
    repositories::application_notice::{
        ApplicationNoticeRepository, NoticeAcknowledgementRepository,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

pub struct ApplicationNoticeService;

const EVENT_CATEGORY: &str = "AUP";

impl ApplicationNoticeService {
    pub async fn list(pool: &PgPool) -> Result<Vec<ApplicationNoticeResponse>> {
        ApplicationNoticeRepository::list(pool).await
    }

    /// 當前生效須知（供申請人填表/簽署時讀取；無生效版本時回 None）。
    pub async fn get_active(pool: &PgPool) -> Result<Option<ApplicationNotice>> {
        ApplicationNoticeRepository::find_active(pool).await
    }

    fn actor_id(actor: &ActorContext) -> Result<Uuid> {
        actor
            .actor_user_id()
            .ok_or_else(|| AppError::Forbidden("申請須知版本須由已登入使用者管理".into()))
    }

    async fn audit_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        event_type: &'static str,
        n: &ApplicationNotice,
    ) -> Result<()> {
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: EVENT_CATEGORY,
                event_type,
                entity: Some(AuditEntity::new(
                    "application_notice",
                    n.id,
                    &n.version_label,
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    /// 新增須知版本（預設未生效；以 activate 切換）。
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateApplicationNoticeRequest,
    ) -> Result<ApplicationNotice> {
        let created_by = Self::actor_id(actor)?;
        let label = req.version_label.trim();
        if ApplicationNoticeRepository::exists_version_label(pool, label).await? {
            return Err(AppError::BusinessRule(format!("版本號「{label}」已存在")));
        }

        let mut tx = pool.begin().await?;
        let notice = ApplicationNoticeRepository::insert_tx(
            &mut tx,
            &NewApplicationNotice {
                version_label: label.to_string(),
                title: req.title.trim().to_string(),
                content: req.content.clone(),
                effective_from: req.effective_from,
                attachment_id: req.attachment_id,
                created_by,
            },
        )
        .await?;
        Self::audit_tx(&mut tx, actor, "APPLICATION_NOTICE_CREATED", &notice).await?;
        tx.commit().await?;
        Ok(notice)
    }

    /// 更新版本正文（限尚無任何計劃簽署引用——已簽署版本內容不可變，維持受控文件完整性）。
    pub async fn update_content(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        content: &str,
    ) -> Result<ApplicationNotice> {
        Self::actor_id(actor)?;

        let mut tx = pool.begin().await?;
        // 守衛與更新置於同一交易，避免 check→update 之間插入簽署的 TOCTOU。
        let signed = NoticeAcknowledgementRepository::count_by_notice_tx(&mut tx, id).await?;
        if signed > 0 {
            return Err(AppError::BusinessRule(format!(
                "此須知版本已有 {signed} 筆簽署，內容不可變更（請改建新版次）"
            )));
        }
        let notice = ApplicationNoticeRepository::update_content_tx(&mut tx, id, content).await?;
        Self::audit_tx(
            &mut tx,
            actor,
            "APPLICATION_NOTICE_CONTENT_UPDATED",
            &notice,
        )
        .await?;
        tx.commit().await?;
        Ok(notice)
    }

    /// 設為當前生效版本（停用其他）。
    pub async fn activate(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<ApplicationNotice> {
        Self::actor_id(actor)?;
        let mut tx = pool.begin().await?;
        let notice = ApplicationNoticeRepository::activate_tx(&mut tx, id).await?;
        Self::audit_tx(&mut tx, actor, "APPLICATION_NOTICE_ACTIVATED", &notice).await?;
        tx.commit().await?;
        Ok(notice)
    }
}
