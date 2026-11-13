//! 批次新增動物（批量進豬）。
//!
//! 設計重點：**不另立一套建立規則**。每一隻都直接呼叫 [`AnimalService::create`]，
//! 所以驗證、欄位容量檢查、耳號重複規則、per-row `ANIMAL_CREATE` 稽核與單隻新增完全一致。
//!
//! 部分成功語意（使用者 2026-11-12 裁定）：每隻各自一個交易，一隻失敗不影響其他隻；
//! 結果逐隻回報，讓呼叫端知道哪幾隻沒進去、為什麼。整批結束後另記一筆
//! `ANIMAL_BATCH_CREATE` 摘要稽核。

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use super::super::AnimalService;
use crate::{
    middleware::ActorContext,
    models::{
        BatchCreateAnimalResult, BatchCreateAnimalsRequest, BatchCreateAnimalsResponse,
        BatchCreateOutcome, CreateAnimalRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

/// 單次批次新增的上限。前端有同值的常數（`BATCH_CREATE_MAX_ANIMALS`），改這裡要一起改。
pub const BATCH_CREATE_MAX_ANIMALS: usize = 50;

/// 非預期錯誤（資料庫、內部錯誤）對使用者的統一訊息；細節只進伺服器日誌，不回給前端。
const GENERIC_FAILURE_MESSAGE: &str = "建立失敗，請稍後重試或聯絡管理員";

impl AnimalService {
    /// 批次新增動物 — 部分成功，逐隻回報結果。
    ///
    /// 請求本身不合法（空陣列、超過上限、匿名）才整個回錯；單隻的問題一律記在該隻的結果裡。
    pub async fn batch_create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &BatchCreateAnimalsRequest,
    ) -> Result<BatchCreateAnimalsResponse> {
        // 拒絕 Anonymous（create 內也會擋，這裡先擋可避免空跑一輪才失敗）
        actor.require_user()?;

        if req.animals.is_empty() {
            return Err(AppError::Validation("請至少提供一隻動物".to_string()));
        }
        if req.animals.len() > BATCH_CREATE_MAX_ANIMALS {
            return Err(AppError::Validation(format!(
                "單次最多批次新增 {} 隻",
                BATCH_CREATE_MAX_ANIMALS
            )));
        }

        let mut results = Vec::with_capacity(req.animals.len());
        for (index, item) in req.animals.iter().enumerate() {
            results.push(Self::create_one_in_batch(pool, actor, index, item).await);
        }

        let created_count = results
            .iter()
            .filter(|r| r.outcome == BatchCreateOutcome::Created)
            .count();
        let not_created_count = results.len() - created_count;

        Self::log_batch_create_summary(pool, actor, created_count, not_created_count).await;

        Ok(BatchCreateAnimalsResponse {
            results,
            created_count,
            not_created_count,
        })
    }

    /// 建立批次中的一隻，並把結果（含失敗原因）轉成可回報的形狀。
    async fn create_one_in_batch(
        pool: &PgPool,
        actor: &ActorContext,
        index: usize,
        item: &CreateAnimalRequest,
    ) -> BatchCreateAnimalResult {
        let outcome = |outcome, animal_id, message, existing_animals| BatchCreateAnimalResult {
            index,
            ear_tag: item.ear_tag.clone(),
            outcome,
            animal_id,
            message,
            existing_animals,
        };

        if let Err(errors) = item.validate() {
            return outcome(
                BatchCreateOutcome::Failed,
                None,
                Some(validation_message(&errors)),
                Vec::new(),
            );
        }

        match Self::create(pool, actor, item).await {
            Ok(animal) => outcome(
                BatchCreateOutcome::Created,
                Some(animal.id),
                None,
                Vec::new(),
            ),
            Err(AppError::DuplicateWarning {
                message,
                existing_animals,
            }) => outcome(
                BatchCreateOutcome::DuplicateWarning,
                None,
                Some(message),
                existing_animals,
            ),
            Err(e) => outcome(
                BatchCreateOutcome::Failed,
                None,
                Some(failure_message(&e)),
                Vec::new(),
            ),
        }
    }

    /// 整批摘要稽核（`ANIMAL_BATCH_CREATE`）。
    ///
    /// 每隻動物的 `ANIMAL_CREATE` 已在各自的交易內寫入；摘要只是「這一批做了什麼」的索引，
    /// 失敗只記 warning，不能讓已建好的動物回報成失敗。
    async fn log_batch_create_summary(
        pool: &PgPool,
        actor: &ActorContext,
        created_count: usize,
        not_created_count: usize,
    ) {
        let display = format!(
            "批次新增動物: 成功 {} 隻, 未建立 {} 隻",
            created_count, not_created_count
        );
        if let Err(e) = AuditService::log_activity_oneshot(
            pool,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "ANIMAL_BATCH_CREATE",
                entity: Some(AuditEntity::new("animal", Uuid::nil(), &display)),
                data_diff: None,
                request_context: None,
            },
        )
        .await
        {
            tracing::warn!("批次新增動物摘要稽核寫入失敗: {e}");
        }
    }
}

/// 把 `validator` 的欄位錯誤整理成一行可讀文字（依字母序，輸出穩定）。
fn validation_message(errors: &validator::ValidationErrors) -> String {
    let mut messages: Vec<String> = errors
        .field_errors()
        .values()
        .flat_map(|errs| errs.iter())
        .map(|e| {
            e.message
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| e.code.to_string())
        })
        .collect();
    messages.sort();
    messages.join("; ")
}

/// 單隻建立失敗時給使用者的訊息。
///
/// 業務／驗證類錯誤的訊息本來就是寫給使用者看的，原樣回報；
/// 其餘（資料庫、內部錯誤）只回統一文字，細節留在伺服器日誌。
fn failure_message(err: &AppError) -> String {
    match err {
        AppError::Validation(m)
        | AppError::BadRequest(m)
        | AppError::Conflict(m)
        | AppError::BusinessRule(m)
        | AppError::NotFound(m)
        | AppError::Forbidden(m) => m.clone(),
        AppError::ValidationWithCode { message, .. } => message.clone(),
        other => {
            tracing::error!("批次新增動物：單隻建立發生非預期錯誤: {other}");
            GENERIC_FAILURE_MESSAGE.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_message_passes_through_user_facing_errors() {
        assert_eq!(
            failure_message(&AppError::Validation("耳號必須為三位數".to_string())),
            "耳號必須為三位數"
        );
        assert_eq!(
            failure_message(&AppError::Conflict("耳號 021 已存在".to_string())),
            "耳號 021 已存在"
        );
        assert_eq!(
            failure_message(&AppError::BusinessRule("欄位已滿".to_string())),
            "欄位已滿"
        );
    }

    #[test]
    fn failure_message_hides_internal_details() {
        let msg = failure_message(&AppError::Internal(
            "connection to db-secret-host refused".to_string(),
        ));
        assert_eq!(msg, GENERIC_FAILURE_MESSAGE);
        assert!(!msg.contains("db-secret-host"));
    }
}
