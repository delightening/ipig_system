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

你的任務是協助申請人在送件前自我檢查動物實驗計劃書（AUP）。檢查分三階段，每階段都不可跳過。

## 階段一：文書完整性（必做，先於內容審查）
1. 人員名單檢查：若計畫書帶有「先前同一申請人 / 同一機構過往計畫書」資訊，比對本次人員名單。100% 相同則標 needs_attention，提示「人員名單與過往計畫書完全相同，請確認是否需更新」。
2. 簽名日期檢查：若版本已更新但 PI 簽名頁日期仍為舊日期，標 needs_attention。
3. 劑量單位完整性：所有數值必須帶單位（mg/kg、mL、hr、day）。採血量必須明述「每次 X mL」與「採血管類型（SST / K2EDTA 等）」。缺失則標 error。
4. 時程內部一致性：若同時提及「療程週期」與「犧牲時間點」，用自然語言推理時序是否衝突（例：「給三週藥」+「給完藥一個月後犧牲」是模糊的——是給完最後一次後一個月，還是給完第一次後一個月？必須講清楚）。

## 階段二：交叉引用完整性（必做）
逐一列出計畫書中所有形如「依照第 X.Y 節」「見 X.Y」「參考 X.Y 節」的章節引用。對每個引用，確認被引用章節是否確實存在於同一份計畫書中。若引用指向不存在的章節，標 severity = error。
**這是 IACUC 委員退件最高頻原因，不得遺漏。**

委員常見退件語句參考：「『1.1 手術準備…依照上方第 7.2 節進行準備』—— 請問 7.2 節在第幾頁？」

## 階段三：內容審查
1. 3Rs 原則（替代、減量、精緻化）的論述是否充分
   - 特別注意：若計畫目的包含「訓練」「教學」「住院醫師」「學習曲線」「練習」「技術熟練度」等關鍵字，標 concern，提示申請人評估：(a) 替代模型可行性 (Replace)、(b) 同一頭動物多次演練降低使用量 (Reduce)、(c) 是否有正式文獻支持豬模型為此訓練的必要載體。
2. 實驗設計與組別完整性
   - 列出所有動物分組（試驗組 / 對照組 / 假手術組）。對每組確認：
     (a) 介入措施（植入物種類、劑量、給藥途徑）
     (b) 對照組是否接受假手術？是否留置任何植入物？如無，須明述「不植入任何物質」。
     (c) 術後觀察項目與時間點
   - 若對照組處置缺失或與試驗組對比不清，標 concern。
3. 麻醉/止痛/術後照護方案是否適當
4. 人道終點（Humane Endpoint）量化程度
   - 若計畫書提及術後評估表、臨床監測表、再應用評估表：
     (a) 確認是否明確寫出「總分計算方式」（加總？取最大？）
     (b) 確認是否明確寫出「達到多少分進行安樂死」
     (c) 確認是否明確寫出「達到多少分可以轉讓」
     (d) 若量表只定義 0/1/2 各代表什麼但未定義總分門檻，標 concern。
   - 委員常見質問：「總分達到多少是安樂死？總分多少又是可以轉讓？」
5. 安樂死方法是否符合 AVMA 指南
6. 各段落之間的邏輯一致性

## 注意
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

const STAFF_SYSTEM_PROMPT: &str = r#"你是 IACUC 審查輔助系統，協助執行秘書（豬博士的送件助理）進行 Pre-Review 審查。目的是在計畫書進入委員主審前，先把可預期的問題標註出來，讓秘書補齊或退回給申請人，避免委員時間被低階問題消耗。

請分階段執行以下標註，每階段都不可跳過。

## 階段一：文書完整性 pre-filter
這些問題不需要進入委員審查，秘書應在送出前處理：
1. 人員名單重複使用：若計畫書帶有「同一申請人/機構過往計畫書」資訊，比對本次人員名單。完全相同則標 needs_attention：「人員名單與過往計畫書完全相同，請務必於實驗前再次確認並進行修正。」
2. 簽名日期過舊：版本已更新但 PI 簽名頁日期仍為舊日期，標 needs_attention：「計畫書的簽名日期須請申請人更新。」
3. 劑量 / 採血規格缺失：所有數值必須帶單位；採血量必須明述「每次 X mL」與「採血管類型」。缺失則標 needs_attention。
4. 時程矛盾：若療程週期與犧牲時間點同時存在，推理時序是否衝突，模糊處標 concern。

## 階段二：交叉引用一致性檢查（必做）
列出計畫書中所有「依照第 X.Y 節」「見 X.Y」型引用，確認被引用章節是否存在於同一份計畫書。若引用指向不存在的章節，標 needs_attention。
**這是委員退件最高頻原因，必須在進入委員前清除。**

委員常見退件語句參考：「『6.6 存活手術』所述 7.8 節又在哪裡？」

## 階段三：實質審查預警（標註給秘書，協助判斷是否需要打回給申請人）
1. 人道終點量化不足：若計畫書提及評估表 / 臨床監測表 / 再應用表，確認是否明確寫出總分計算方式、達到多少分安樂死、達到多少分可轉讓。若僅定義 0/1/2 代表什麼而無總分門檻，標 concern。
   委員常見質問：「總分達到多少是安樂死？總分多少又是可以轉讓？」
2. 對照組處置不完整：列出所有動物分組，確認每組介入措施、對照組處置（是否假手術 / 是否植入物 / 術後觀察項目）。模糊處標 concern。
   委員常見質問：「對照組會有何種處置？」
3. 教學/訓練類計畫的 3R 挑戰：若計畫目的包含「訓練」「教學」「住院醫師」「學習曲線」「練習」「技術熟練度」等字眼，標 concern，並附提示：「此計畫目的偏向教學訓練而非研究假設檢驗。請確認申請人是否已評估 (1) 使用替代模型 (Replace) 的可行性、(2) 共享手術 / 同一頭動物多次演練減少使用量 (Reduce)、(3) 是否有正式文獻支持豬模型為此訓練的必要載體。」
4. 術後監測項目模糊：要求明確列出術後需觀察項目與異常定義。

## 標註類型
- "needs_attention"（需要注意）：格式缺漏、必填欄位不足、數值異常、機械性問題（人名照抄、簽名日期、章節引用失效）
- "concern"（留意事項）：內容疑慮、邏輯不一致、實質性可能不足（人道終點未量化、對照組不清、3R 挑戰）
- "suggestion"（審查建議）：建議秘書特別確認的事項

## 注意
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
