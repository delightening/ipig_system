//! AI 接口 Service 層：核心業務邏輯

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::ai_auth::{generate_api_key, hash_api_key};
use crate::models::ai::*;
use crate::repositories::AiRepository;
use crate::{AppError, Result};

pub struct AiService;

impl AiService {
    /// 建立新的 AI API key（僅管理員）
    pub async fn create_api_key(
        pool: &PgPool,
        req: &CreateAiApiKeyRequest,
        created_by: Uuid,
    ) -> Result<CreateAiApiKeyResponse> {
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

        let record = AiRepository::insert_api_key(
            pool,
            &req.name,
            &key_hash,
            key_prefix,
            created_by,
            &scopes_json,
            req.expires_at,
            req.rate_limit_per_minute,
        )
        .await?;

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

    /// 停用 / 啟用 API key
    pub async fn toggle_api_key(
        pool: &PgPool,
        id: Uuid,
        is_active: bool,
    ) -> Result<()> {
        let key = AiRepository::find_api_key_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("API key not found".to_string()))?;

        if key.is_active == is_active {
            return Ok(());
        }

        AiRepository::update_api_key_active(pool, id, is_active).await
    }

    /// 刪除 API key
    pub async fn delete_api_key(pool: &PgPool, id: Uuid) -> Result<()> {
        AiRepository::find_api_key_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("API key not found".to_string()))?;

        AiRepository::delete_api_key(pool, id).await
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
