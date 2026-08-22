//! 申請須知簽署（PR-B）：申請人（PI/SD）對當前生效須知的手寫電子簽署。

use sqlx::PgPool;

use super::ProtocolService;
use crate::middleware::ActorContext;
use crate::models::{
    NewNoticeAcknowledgement, NoticeAcknowledgementStatus, Protocol, ProtocolNoticeAcknowledgement,
    ProtocolStatus,
};
use crate::repositories::application_notice::{
    ApplicationNoticeRepository, NoticeAcknowledgementRepository,
};
use crate::services::access;
use crate::services::signature::{SignatureMeaning, SignatureService, SignatureType};
use crate::{AppError, Result};

impl ProtocolService {
    /// 計畫的須知簽署狀態：當前生效須知 + 是否已簽。供前端填表/送審顯示。
    pub async fn get_notice_status(
        pool: &PgPool,
        scope: access::Scoped<access::ProtocolId>,
    ) -> Result<NoticeAcknowledgementStatus> {
        let active = ApplicationNoticeRepository::find_active(pool).await?;
        let ack = NoticeAcknowledgementRepository::find_by_protocol(pool, scope.id()).await?;
        let acknowledged = matches!((&active, &ack), (Some(a), Some(k)) if k.notice_id == a.id);
        let acknowledged_at = ack
            .as_ref()
            .filter(|_| acknowledged)
            .map(|k| k.acknowledged_at);
        Ok(NoticeAcknowledgementStatus {
            active_notice: active,
            acknowledged,
            acknowledged_at,
        })
    }

    /// 申請人（PI 或 SD）簽署當前生效的動物試驗申請須知。
    ///
    /// 守衛：限該計畫 PI / SD；計畫須為 DRAFT。手寫電子簽章（meaning=ACKNOWLEDGE）+
    /// upsert 簽署紀錄於**同一 tx 原子**完成（含 SIGNATURE_CREATE audit chain）。
    pub async fn acknowledge_notice(
        pool: &PgPool,
        actor: &ActorContext,
        scope: access::Scoped<access::NoticeSign>,
        handwriting_svg: &str,
        stroke_data: Option<&serde_json::Value>,
    ) -> Result<ProtocolNoticeAcknowledgement> {
        let user = actor.require_user()?;
        // 簽署授權已由 `Scoped<NoticeSign>` 在 handler 層證明（限該計畫 PI / SD）。
        let protocol_id = scope.id();

        // 須有生效須知版本可簽
        let active = ApplicationNoticeRepository::find_active(pool)
            .await?
            .ok_or_else(|| AppError::BusinessRule("目前無生效的申請須知版本".to_string()))?;

        let mut tx = pool.begin().await?;
        let protocol: Protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(protocol_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;
        if protocol.status != ProtocolStatus::Draft {
            return Err(AppError::BusinessRule(
                "僅草稿計畫可簽署申請須知".to_string(),
            ));
        }

        // 手寫電子簽章（meaning=ACKNOWLEDGE）+ chain entry，同 tx
        let content = format!(
            "protocol_notice_ack:{}:{}:{}",
            protocol_id, active.version_label, user.id
        );
        let signature = SignatureService::sign_with_handwriting_tx(
            &mut tx,
            actor,
            "protocol_notice_ack",
            &protocol_id.to_string(),
            user.id,
            SignatureType::Confirm,
            &content,
            handwriting_svg,
            stroke_data,
            SignatureMeaning::Acknowledge,
        )
        .await?;

        let ack = NoticeAcknowledgementRepository::upsert_tx(
            &mut tx,
            &NewNoticeAcknowledgement {
                protocol_id,
                notice_id: active.id,
                signer_id: user.id,
                signature_id: Some(signature.id),
                notice_attachment_id: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(ack)
    }
}
