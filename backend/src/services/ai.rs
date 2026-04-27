//! AI 接口 Service 層：核心業務邏輯

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::ai_auth::{generate_api_key, hash_api_key};
use crate::middleware::ActorContext;
use crate::models::ai::*;
use crate::models::audit_diff::DataDiff;
use crate::repositories::AiRepository;
use crate::services::{
    audit::{ActivityLogEntry, AuditEntity},
    AuditService,
};
use crate::{AppError, Result};

pub struct AiService;

impl AiService {
    /// 建立新的 AI API key（僅管理員）— Service-driven audit (SECURITY)
    ///
    /// Raw key 只回傳給呼叫端一次（post-create response），絕不寫入 audit log；
    /// `key_hash` 欄位由 `AuditRedact::redacted_fields` 遮蔽（見 `AiApiKey` 實作）。
    pub async fn create_api_key(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateAiApiKeyRequest,
    ) -> Result<CreateAiApiKeyResponse> {
        let user = actor.require_user()?;
        let created_by = user.id;

        if req.name.trim().is_empty() {
            return Err(AppError::Validation("API key 名稱不可為空".to_string()));
        }
        if req.name.len() > 100 {
            return Err(AppError::Validation("API key 名稱不可超過 100 字元".to_string()));
        }
        if req.rate_limit_per_minute < 1 || req.rate_limit_per_minute > 1000 {
            return Err(AppError::Validation(
                "rate_limit_per_minute 須介於 1-1000".to_string(),
            ));
        }

        let raw_key = generate_api_key();
        let key_hash = hash_api_key(&raw_key);
        let key_prefix = &raw_key[..12.min(raw_key.len())];
        let scopes_json = serde_json::to_value(&req.scopes)
            .map_err(|e| AppError::Internal(format!("scopes serialize error: {}", e)))?;

        // Repository 的 insert_api_key 接 pool 不接 tx — 為保持 Service-driven
        // audit 原子性，service 層自行 pool.begin 包住 INSERT + audit。
        let mut tx = pool.begin().await?;

        let record = sqlx::query_as::<_, AiApiKey>(
            r#"
            INSERT INTO ai_api_keys (name, key_hash, key_prefix, created_by, scopes,
                                     is_active, expires_at, rate_limit_per_minute)
            VALUES ($1, $2, $3, $4, $5, true, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&key_hash)
        .bind(key_prefix)
        .bind(created_by)
        .bind(&scopes_json)
        .bind(req.expires_at)
        .bind(req.rate_limit_per_minute)
        .fetch_one(&mut *tx)
        .await?;

        // SECURITY audit：AI API key 建立屬權限擴張，稽核重點事件。
        // AuditRedact 自動遮蔽 key_hash；raw_key 從未進入 record，無洩漏風險。
        let display = format!("{} ({})", record.name, record.key_prefix);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "AI_API_KEY_CREATE",
                entity: Some(AuditEntity::new("ai_api_key", record.id, &display)),
                data_diff: Some(DataDiff::create_only(&record)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(CreateAiApiKeyResponse {
            id: record.id,
            name: record.name,
            api_key: raw_key,
            scopes: req.scopes.clone(),
            expires_at: record.expires_at,
            rate_limit_per_minute: record.rate_limit_per_minute,
            created_at: record.created_at,
        })
    }

    /// 列出所有 API keys（管理員用）
    pub async fn list_api_keys(pool: &PgPool) -> Result<Vec<AiApiKeyInfo>> {
        let keys = AiRepository::list_api_keys(pool).await?;
        Ok(keys.into_iter().map(Self::key_to_info).collect())
    }

    /// 停用 / 啟用 API key — Service-driven audit (SECURITY)
    pub async fn toggle_api_key(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        is_active: bool,
    ) -> Result<()> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, AiApiKey>(
            "SELECT * FROM ai_api_keys WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("API key not found".to_string()))?;

        if before.is_active == is_active {
            // no-op；仍 commit（空 tx）
            tx.commit().await?;
            return Ok(());
        }

        let after = sqlx::query_as::<_, AiApiKey>(
            "UPDATE ai_api_keys SET is_active = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(is_active)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} ({})", after.name, after.key_prefix);
        let event_type = if is_active {
            "AI_API_KEY_ENABLE"
        } else {
            "AI_API_KEY_DISABLE"
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type,
                entity: Some(AuditEntity::new("ai_api_key", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 刪除 API key — Service-driven audit (SECURITY)
    ///
    /// 硬刪除；audit 保留 `delete_only` snapshot（key_hash 經 AuditRedact 遮蔽），
    /// 日後稽核可從 audit log 查誰何時刪了哪個 key。
    pub async fn delete_api_key(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, AiApiKey>(
            "SELECT * FROM ai_api_keys WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("API key not found".to_string()))?;

        sqlx::query("DELETE FROM ai_api_keys WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let display = format!("{} ({})", before.name, before.key_prefix);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "AI_API_KEY_DELETE",
                entity: Some(AuditEntity::new("ai_api_key", before.id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 取得系統概覽
    pub async fn get_system_overview(pool: &PgPool) -> Result<AiSystemOverview> {
        let stats = AiRepository::query_system_overview(pool).await?;
        Ok(AiSystemOverview {
            system_name: "iPig System".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            total_animals: stats["total_animals"].as_i64().unwrap_or(0),
            active_animals: stats["active_animals"].as_i64().unwrap_or(0),
            total_protocols: stats["total_protocols"].as_i64().unwrap_or(0),
            active_protocols: stats["active_protocols"].as_i64().unwrap_or(0),
            total_observations_30d: stats["total_observations_30d"].as_i64().unwrap_or(0),
            total_surgeries_30d: stats["total_surgeries_30d"].as_i64().unwrap_or(0),
            facilities_count: stats["facilities_count"].as_i64().unwrap_or(0),
            generated_at: Utc::now(),
        })
    }

    /// 執行 AI 資料查詢
    pub async fn execute_query(
        pool: &PgPool,
        req: &AiQueryRequest,
    ) -> Result<AiQueryResponse> {
        let per_page = req.per_page.clamp(1, 100);
        let (data, total) = match &req.domain {
            AiQueryDomain::Animals => {
                AiRepository::query_animals(
                    pool,
                    &req.filters,
                    req.page,
                    per_page,
                    req.sort_by.as_deref(),
                    &req.sort_order,
                )
                .await?
            }
            AiQueryDomain::Observations => {
                AiRepository::query_observations(pool, &req.filters, req.page, per_page).await?
            }
            AiQueryDomain::Surgeries => {
                AiRepository::query_surgeries(pool, &req.filters, req.page, per_page).await?
            }
            AiQueryDomain::Weights => {
                AiRepository::query_weights(pool, &req.filters, req.page, per_page).await?
            }
            AiQueryDomain::Protocols => {
                AiRepository::query_protocols(pool, &req.filters, req.page, per_page).await?
            }
            AiQueryDomain::Facilities => {
                AiRepository::query_facilities(pool, req.page, per_page).await?
            }
            AiQueryDomain::Stock | AiQueryDomain::HrSummary => {
                return Err(AppError::BadRequest(format!(
                    "Domain {:?} is not yet supported",
                    req.domain
                )));
            }
        };

        let total_pages = if per_page > 0 {
            (total as f64 / per_page as f64).ceil() as i64
        } else {
            0
        };

        let summary = Self::generate_summary(&req.domain, total, &data);

        Ok(AiQueryResponse {
            domain: req.domain.clone(),
            data: serde_json::Value::Array(data),
            total,
            page: req.page,
            per_page,
            total_pages,
            summary: Some(summary),
        })
    }

    /// 取得 API schema（讓 AI 知道可以查什麼）
    pub fn get_schema() -> AiSchemaResponse {
        AiSchemaResponse {
            version: "1.0".to_string(),
            domains: vec![
                AiDomainSchema {
                    name: "animals".to_string(),
                    description: "實驗動物基本資料，包含耳標、品種、狀態、所屬計畫等".to_string(),
                    available_filters: vec![
                        filter_field("status", "string", "動物狀態", Some("alive")),
                        filter_field("breed", "string", "品種", Some("Lanyu")),
                        filter_field("keyword", "string", "搜尋耳標或名稱", None),
                    ],
                    available_sort_fields: vec![
                        "ear_tag", "name", "breed", "status", "sex", "created_at", "birth_date",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                },
                AiDomainSchema {
                    name: "observations".to_string(),
                    description: "動物觀察紀錄（每日健康觀察、異常紀錄等）".to_string(),
                    available_filters: vec![
                        filter_field("animal_id", "uuid", "動物 ID", None),
                        filter_field("days", "integer", "查詢最近 N 天", Some("30")),
                    ],
                    available_sort_fields: vec!["created_at".to_string()],
                },
                AiDomainSchema {
                    name: "surgeries".to_string(),
                    description: "動物手術紀錄".to_string(),
                    available_filters: vec![
                        filter_field("animal_id", "uuid", "動物 ID", None),
                    ],
                    available_sort_fields: vec!["created_at".to_string()],
                },
                AiDomainSchema {
                    name: "weights".to_string(),
                    description: "動物體重量測紀錄".to_string(),
                    available_filters: vec![
                        filter_field("animal_id", "uuid", "動物 ID", None),
                    ],
                    available_sort_fields: vec!["measured_date".to_string()],
                },
                AiDomainSchema {
                    name: "protocols".to_string(),
                    description: "動物使用計畫 (AUP)，含 IACUC 編號、PI 姓名、審核狀態".to_string(),
                    available_filters: vec![
                        filter_field("status", "string", "計畫狀態", Some("approved")),
                    ],
                    available_sort_fields: vec!["created_at".to_string()],
                },
                AiDomainSchema {
                    name: "facilities".to_string(),
                    description: "設施與欄位資訊".to_string(),
                    available_filters: vec![],
                    available_sort_fields: vec!["name".to_string()],
                },
            ],
        }
    }

    /// 記錄 AI 查詢日誌
    pub async fn log_query(
        pool: &PgPool,
        api_key_id: Uuid,
        endpoint: &str,
        method: &str,
        query_summary: Option<&serde_json::Value>,
        response_status: i16,
        duration_ms: i32,
        source_ip: Option<&str>,
    ) {
        if let Err(e) = AiRepository::insert_query_log(
            pool,
            api_key_id,
            endpoint,
            method,
            query_summary,
            response_status,
            duration_ms,
            source_ip,
        )
        .await
        {
            tracing::warn!("[AI] Failed to log query: {}", e);
        }
    }

    fn key_to_info(key: crate::models::ai::AiApiKey) -> AiApiKeyInfo {
        let scopes: Vec<String> =
            serde_json::from_value(key.scopes).unwrap_or_default();
        AiApiKeyInfo {
            id: key.id,
            name: key.name,
            key_prefix: key.key_prefix,
            scopes,
            is_active: key.is_active,
            expires_at: key.expires_at,
            last_used_at: key.last_used_at,
            usage_count: key.usage_count,
            rate_limit_per_minute: key.rate_limit_per_minute,
            created_at: key.created_at,
        }
    }

    fn generate_summary(
        domain: &AiQueryDomain,
        total: i64,
        data: &[serde_json::Value],
    ) -> String {
        match domain {
            AiQueryDomain::Animals => {
                let alive = data
                    .iter()
                    .filter(|d| d["status"].as_str() == Some("alive"))
                    .count();
                format!("共 {} 筆動物資料，本頁顯示 {} 筆（其中 {} 筆狀態為 alive）", total, data.len(), alive)
            }
            AiQueryDomain::Observations => {
                format!("共 {} 筆觀察紀錄，本頁顯示 {} 筆", total, data.len())
            }
            AiQueryDomain::Protocols => {
                let approved = data
                    .iter()
                    .filter(|d| d["status"].as_str() == Some("approved"))
                    .count();
                format!("共 {} 筆計畫，本頁顯示 {} 筆（其中 {} 筆已核准）", total, data.len(), approved)
            }
            _ => format!("共 {} 筆資料，本頁顯示 {} 筆", total, data.len()),
        }
    }
}

fn filter_field(
    field: &str,
    field_type: &str,
    desc: &str,
    example: Option<&str>,
) -> AiFilterField {
    AiFilterField {
        field: field.to_string(),
        field_type: field_type.to_string(),
        description: desc.to_string(),
        example: example.map(String::from),
    }
}
