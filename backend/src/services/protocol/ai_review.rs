//! R20-5: AI 預審 Service（Claude API 整合）
//!
//! 整合 Level 1 規則引擎 + Level 2 Claude API，
//! 支援客戶端預審 (client_pre_submit) 與執行秘書標註 (staff_pre_review)。

use std::time::Instant;

use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::models::ai_review::{AiReviewResponse, ProtocolAiReview, ValidationResult};
use crate::{AppError, Result};

use super::validation::validate_protocol;

/// 每日每位使用者 AI 預審上限
const DAILY_AI_REVIEW_LIMIT: i64 = 10;
/// 序列化給 AI 的最大字元數（約 8K tokens）
const MAX_AI_INPUT_CHARS: usize = 32000;

pub struct AiReviewService;

impl AiReviewService {
    /// 執行 AI 預審（Level 1 + Level 2）
    pub async fn review_protocol(
        db: &PgPool,
        config: &Config,
        protocol_id: Uuid,
        review_type: &str,
        triggered_by: Option<Uuid>,
    ) -> Result<AiReviewResponse> {
        let started = Instant::now();

        // 1. 讀取 protocol
        let protocol = sqlx::query_as::<_, crate::models::Protocol>(
            "SELECT * FROM protocols WHERE id = $1",
        )
        .bind(protocol_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        let content = protocol
            .working_content
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("Protocol has no content".to_string()))?;

        // 2. Level 1 規則檢查
        let rule_result = validate_protocol(content);

        // 3. 快取檢查：同 version + type 不重複
        let latest_version_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM protocol_versions WHERE protocol_id = $1 ORDER BY version_no DESC LIMIT 1",
        )
        .bind(protocol_id)
        .fetch_optional(db)
        .await?;

        if let Some(vid) = latest_version_id {
            let existing: Option<ProtocolAiReview> = sqlx::query_as(
                "SELECT * FROM protocol_ai_reviews WHERE protocol_version_id = $1 AND review_type = $2",
            )
            .bind(vid)
            .bind(review_type)
            .fetch_optional(db)
            .await?;

            if let Some(cached) = existing {
                return Ok(AiReviewResponse::from(cached));
            }
        }

        // 4. Level 2 Claude API（如果有 API key 且 enabled）
        let ai_result = if config.ai_review_enabled {
            if let Some(ref api_key) = config.anthropic_api_key {
                match call_claude_api(config, api_key, content, review_type).await {
                    Ok(result) => Some(result),
                    Err(e) => {
                        tracing::warn!("[AI Review] Claude API call failed: {}", e);
                        None
                    }
                }
            } else {
                tracing::debug!("[AI Review] No ANTHROPIC_API_KEY, skipping Level 2");
                None
            }
        } else {
            None
        };

        // 5. 計算統計
        let total_errors = rule_result.errors.len() as i32
            + ai_result
                .as_ref()
                .and_then(|r| r.get("issues"))
                .and_then(|i| i.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter(|i| i.get("severity").and_then(|s| s.as_str()) == Some("error"))
                        .count() as i32
                })
                .unwrap_or(0);

        let total_warnings = rule_result.warnings.len() as i32
            + ai_result
                .as_ref()
                .and_then(|r| r.get("issues"))
                .and_then(|i| i.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter(|i| {
                            i.get("severity").and_then(|s| s.as_str()) == Some("warning")
                        })
                        .count() as i32
                })
                .unwrap_or(0);

        let score = ai_result
            .as_ref()
            .and_then(|r| r.get("score"))
            .and_then(|s| s.as_i64())
            .map(|s| s as i32);

        let ai_model = ai_result.as_ref().map(|_| config.ai_review_model.clone());
        let ai_input_tokens = ai_result
            .as_ref()
            .and_then(|r| r.get("_input_tokens"))
            .and_then(|t| t.as_i64())
            .map(|t| t as i32);
        let ai_output_tokens = ai_result
            .as_ref()
            .and_then(|r| r.get("_output_tokens"))
            .and_then(|t| t.as_i64())
            .map(|t| t as i32);

        let duration_ms = started.elapsed().as_millis() as i32;

        let rule_json = serde_json::to_value(&rule_result)
            .map_err(|e| AppError::Internal(format!("serialize rule_result: {}", e)))?;

        // 6. 儲存到 DB
        let record = sqlx::query_as::<_, ProtocolAiReview>(
            r#"
            INSERT INTO protocol_ai_reviews (
                protocol_id, protocol_version_id, review_type,
                rule_result, ai_result, ai_model,
                ai_input_tokens, ai_output_tokens,
                total_errors, total_warnings, score,
                triggered_by, duration_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(protocol_id)
        .bind(latest_version_id)
        .bind(review_type)
        .bind(&rule_json)
        .bind(&ai_result)
        .bind(&ai_model)
        .bind(ai_input_tokens)
        .bind(ai_output_tokens)
        .bind(total_errors)
        .bind(total_warnings)
        .bind(score)
        .bind(triggered_by)
        .bind(duration_ms)
        .fetch_one(db)
        .await?;

        Ok(AiReviewResponse::from(record))
    }

    /// 取得最新一筆 AI review
    pub async fn get_latest(
        db: &PgPool,
        protocol_id: Uuid,
        review_type: &str,
    ) -> Result<Option<AiReviewResponse>> {
        let record: Option<ProtocolAiReview> = sqlx::query_as(
            r#"
            SELECT * FROM protocol_ai_reviews
            WHERE protocol_id = $1 AND review_type = $2
            ORDER BY created_at DESC LIMIT 1
            "#,
        )
        .bind(protocol_id)
        .bind(review_type)
        .fetch_optional(db)
        .await?;

        Ok(record.map(AiReviewResponse::from))
    }

    /// 檢查使用者今日剩餘 AI 預審次數
    pub async fn remaining_daily_count(
        db: &PgPool,
        user_id: Uuid,
    ) -> Result<i64> {
        let used: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM protocol_ai_reviews
            WHERE triggered_by = $1
              AND review_type = 'client_pre_submit'
              AND created_at > CURRENT_DATE
            "#,
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;

        Ok((DAILY_AI_REVIEW_LIMIT - used).max(0))
    }
}

/// 只做 Level 1 驗證（無 AI）
pub fn validate_only(content: &serde_json::Value) -> ValidationResult {
    validate_protocol(content)
}

/// 呼叫 Claude API
async fn call_claude_api(
    config: &Config,
    api_key: &str,
    content: &serde_json::Value,
    review_type: &str,
) -> Result<serde_json::Value> {
    let system_prompt = build_system_prompt(review_type);
    let user_content = serialize_for_ai(content);

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(config.ai_review_timeout_secs))
        .json(&serde_json::json!({
            "model": config.ai_review_model,
            "max_tokens": 2048,
            "system": system_prompt,
            "messages": [{
                "role": "user",
                "content": user_content
            }]
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Claude API request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(AppError::Internal(format!(
            "Claude API returned {}: {}",
            status, body
        )));
    }

    let api_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Claude API response parse failed: {}", e)))?;

    // 提取 content 和 usage
    let text = api_response
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("{}");

    let input_tokens = api_response
        .get("usage")
        .and_then(|u| u.get("input_tokens"))
        .and_then(|t| t.as_i64())
        .unwrap_or(0);
    let output_tokens = api_response
        .get("usage")
        .and_then(|u| u.get("output_tokens"))
        .and_then(|t| t.as_i64())
        .unwrap_or(0);

    // 嘗試解析 JSON（AI 回應可能包含 markdown code fence）
    let cleaned = text
        .trim()
        .strip_prefix("```json")
        .unwrap_or(text.trim())
        .strip_prefix("```")
        .unwrap_or(text.trim())
        .strip_suffix("```")
        .unwrap_or(text.trim())
        .trim();

    let mut result: serde_json::Value = serde_json::from_str(cleaned)
        .unwrap_or_else(|_| serde_json::json!({ "summary": text, "issues": [], "passed": [] }));

    // 附加 token 統計
    result["_input_tokens"] = serde_json::json!(input_tokens);
    result["_output_tokens"] = serde_json::json!(output_tokens);

    Ok(result)
}

fn build_system_prompt(review_type: &str) -> String {
    match review_type {
        "client_pre_submit" => CLIENT_SYSTEM_PROMPT.to_string(),
        "staff_pre_review" => STAFF_SYSTEM_PROMPT.to_string(),
        _ => CLIENT_SYSTEM_PROMPT.to_string(),
    }
}

fn serialize_for_ai(content: &serde_json::Value) -> String {
    let formatted = serde_json::to_string_pretty(content).unwrap_or_default();
    if formatted.len() > MAX_AI_INPUT_CHARS {
        format!(
            "{}...\n[內容已截斷，僅顯示前 {} 字元]",
            &formatted[..MAX_AI_INPUT_CHARS],
            MAX_AI_INPUT_CHARS
        )
    } else {
        formatted
    }
}

const CLIENT_SYSTEM_PROMPT: &str = r#"你是一位資深的 IACUC（動物實驗倫理委員會）審查委員，擁有實驗動物科學與獸醫學背景。

你的任務是預審動物實驗計劃書（AUP），檢查以下面向：
1. 3Rs 原則（替代、減量、精緻化）的論述是否充分
2. 實驗設計是否合理
3. 麻醉/止痛/術後照護方案是否適當
4. 人道終點是否明確可執行
5. 安樂死方法是否符合 AVMA 指南
6. 各段落之間的邏輯一致性

注意：
- 你是「預審助手」，不是最終決策者
- 區分「必須修正」(severity: "error") 和「建議改善」(severity: "warning")
- 回覆使用繁體中文
- 以 JSON 格式回傳結果，格式如下：
{
    "summary": "整體評語（1-2 句）",
    "score": 0-100,
    "issues": [
        {
            "severity": "error" | "warning",
            "category": "問題類別",
            "section": "所在段落",
            "message": "問題描述",
            "suggestion": "建議修正方式"
        }
    ],
    "passed": ["已通過的檢查項目代碼"]
}

只回傳 JSON，不要加其他文字。"#;

const STAFF_SYSTEM_PROMPT: &str = r#"你是 IACUC 審查輔助系統，你的任務是協助執行秘書進行 Pre-Review 審查。

請分析以下動物實驗計劃書內容，標註執行秘書應該特別注意的地方。

標註分為三類：
1. "needs_attention"（需要注意）：格式缺漏、必填欄位不足、數值異常
2. "concern"（留意事項）：內容疑慮、邏輯不一致、方案可能不足
3. "suggestion"（審查建議）：建議執行秘書特別確認的事項

注意：
- 你是「審查輔助」，不是決策者
- 以執行秘書的視角提供建議（「建議確認...」「請注意...」）
- 回覆使用繁體中文
- 以 JSON 格式回傳結果：
{
    "summary": "計劃書概況摘要（1-2 句）",
    "flags": [
        {
            "flag_type": "needs_attention" | "concern" | "suggestion",
            "section": "所在段落",
            "message": "標註內容",
            "suggestion": "建議行動"
        }
    ]
}

只回傳 JSON，不要加其他文字。"#;
