//! R20-1: Level 1 規則引擎 — 計劃書驗證
//!
//! 從 working_content JSON 解析欄位，執行 14 條格式/完整性檢查。
//! 所有欄位皆 Option 容忍缺失。

use crate::models::ai_review::{ValidationIssue, ValidationResult};

/// 無效填寫文字（視為未填）
const INVALID_TEXTS: [&str; 4] = ["略", "同上", "無", "N/A"];

/// 模糊用詞（人道終點）
const VAGUE_WORDS: [&str; 4] = ["明顯", "嚴重", "顯著", "很大"];

pub fn validate_protocol(content: &serde_json::Value) -> ValidationResult {
    let mut passed = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    check_significance(content, &mut passed, &mut errors);
    check_3rs_replacement(content, &mut passed, &mut errors);
    check_3rs_reduction(content, &mut passed, &mut errors);
    check_3rs_refinement(content, &mut passed, &mut errors);
    check_date_logic(content, &mut passed, &mut errors);
    check_animal_count(content, &mut passed, &mut errors);
    check_pain_anesthesia(content, &mut passed, &mut warnings);
    check_personnel_training(content, &mut passed, &mut warnings);
    check_alt_databases(content, &mut passed, &mut warnings);
    check_humane_endpoint(content, &mut passed, &mut warnings);
    check_postop_observation(content, &mut passed, &mut warnings);
    check_experiment_duration(content, &mut passed, &mut warnings);
    check_euthanasia_method(content, &mut passed, &mut warnings);
    check_attachments(content, &mut passed, &mut warnings);

    ValidationResult {
        passed,
        errors,
        warnings,
    }
}

fn get_text(val: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut current = val;
    for &key in path {
        current = current.get(key)?;
    }
    current.as_str().map(|s| s.to_string())
}

fn is_valid_text(text: &str, min_len: usize) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().count() < min_len {
        return false;
    }
    !INVALID_TEXTS
        .iter()
        .any(|&inv| trimmed.eq_ignore_ascii_case(inv))
}

// ── 14 條驗證規則 ──

fn check_significance(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    errors: &mut Vec<ValidationIssue>,
) {
    let text = get_text(content, &["purpose", "significance"]).unwrap_or_default();
    if is_valid_text(&text, 100) {
        passed.push("PURPOSE_SIGNIFICANCE".to_string());
    } else {
        errors.push(ValidationIssue {
            code: "PURPOSE_SIGNIFICANCE_SHORT".to_string(),
            category: "purpose".to_string(),
            section: "purpose".to_string(),
            message: "研究目的說明不足（需 100 字以上，且不可填寫「略」「同上」「無」）".to_string(),
            suggestion: "請詳細說明本計畫的研究目的與重要性。".to_string(),
        });
    }
}

fn check_3rs_replacement(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    errors: &mut Vec<ValidationIssue>,
) {
    let text = content
        .get("purpose")
        .and_then(|p| p.get("replacement"))
        .and_then(|r| r.get("rationale"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if is_valid_text(text, 50) {
        passed.push("3RS_REPLACEMENT".to_string());
    } else {
        errors.push(ValidationIssue {
            code: "3RS_REPLACEMENT_MISSING".to_string(),
            category: "3Rs".to_string(),
            section: "purpose".to_string(),
            message: "Replacement（取代）說明不足（需 50 字以上）".to_string(),
            suggestion: "請說明為何無法以非動物實驗方法取代。".to_string(),
        });
    }
}

fn check_3rs_reduction(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    errors: &mut Vec<ValidationIssue>,
) {
    let text = content
        .get("purpose")
        .and_then(|p| p.get("reduction"))
        .and_then(|r| r.get("design"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if is_valid_text(text, 50) {
        passed.push("3RS_REDUCTION".to_string());
    } else {
        errors.push(ValidationIssue {
            code: "3RS_REDUCTION_MISSING".to_string(),
            category: "3Rs".to_string(),
            section: "purpose".to_string(),
            message: "Reduction（減量）說明不足（需 50 字以上）".to_string(),
            suggestion: "請說明動物數量的統計依據或減量策略。".to_string(),
        });
    }
}

fn check_3rs_refinement(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    errors: &mut Vec<ValidationIssue>,
) {
    // Refinement 可能在 purpose 或 design 中
    // 嘗試從 design 的 procedures 或 pain management 取得
    let procedures = get_text(content, &["design", "procedures"]).unwrap_or_default();
    let pain_plan = content
        .get("design")
        .and_then(|d| d.get("pain"))
        .and_then(|p| p.get("management_plan"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let combined = format!("{}{}", procedures, pain_plan);
    if is_valid_text(&combined, 50) {
        passed.push("3RS_REFINEMENT".to_string());
    } else {
        errors.push(ValidationIssue {
            code: "3RS_REFINEMENT_MISSING".to_string(),
            category: "3Rs".to_string(),
            section: "design".to_string(),
            message: "Refinement（精緻化）說明不足（需 50 字以上）".to_string(),
            suggestion: "請說明如何最小化動物痛苦（如麻醉、止痛、環境豐富化）。".to_string(),
        });
    }
}

fn check_date_logic(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    errors: &mut Vec<ValidationIssue>,
) {
    let start = get_text(content, &["basic", "start_date"]).unwrap_or_default();
    let end = get_text(content, &["basic", "end_date"]).unwrap_or_default();

    if start.is_empty() || end.is_empty() {
        errors.push(ValidationIssue {
            code: "DATE_MISSING".to_string(),
            category: "date".to_string(),
            section: "basic".to_string(),
            message: "計畫開始日或結束日未填寫".to_string(),
            suggestion: "請填寫計畫開始日期與結束日期。".to_string(),
        });
        return;
    }

    let start_date = chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d");
    let end_date = chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d");

    match (start_date, end_date) {
        (Ok(s), Ok(e)) => {
            if e <= s {
                errors.push(ValidationIssue {
                    code: "DATE_END_BEFORE_START".to_string(),
                    category: "date".to_string(),
                    section: "basic".to_string(),
                    message: "結束日期必須晚於開始日期".to_string(),
                    suggestion: "請確認日期設定是否正確。".to_string(),
                });
            } else if (e - s).num_days() > 365 * 3 {
                errors.push(ValidationIssue {
                    code: "DATE_EXCEEDS_3_YEARS".to_string(),
                    category: "date".to_string(),
                    section: "basic".to_string(),
                    message: "計畫期限超過 3 年".to_string(),
                    suggestion: "IACUC 計畫期限通常不超過 3 年，如有特殊需求請說明。".to_string(),
                });
            } else {
                passed.push("DATE_LOGIC".to_string());
            }
        }
        _ => {
            errors.push(ValidationIssue {
                code: "DATE_INVALID_FORMAT".to_string(),
                category: "date".to_string(),
                section: "basic".to_string(),
                message: "日期格式不正確".to_string(),
                suggestion: "請使用 YYYY-MM-DD 格式。".to_string(),
            });
        }
    }
}

fn check_animal_count(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    errors: &mut Vec<ValidationIssue>,
) {
    let total = content
        .get("animals")
        .and_then(|a| a.get("total_animals"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    if total > 0 {
        passed.push("ANIMAL_COUNT".to_string());
    } else {
        errors.push(ValidationIssue {
            code: "ANIMAL_COUNT_ZERO".to_string(),
            category: "animals".to_string(),
            section: "animals".to_string(),
            message: "動物總數必須大於 0".to_string(),
            suggestion: "請填寫本計畫使用的動物總數。".to_string(),
        });
    }
}

fn check_pain_anesthesia(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let category = content
        .get("design")
        .and_then(|d| d.get("pain"))
        .and_then(|p| p.get("category"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let high_pain = ["C", "D", "E"]
        .iter()
        .any(|c| category.eq_ignore_ascii_case(c));

    if !high_pain {
        passed.push("PAIN_ANESTHESIA".to_string());
        return;
    }

    let has_anesthesia = content
        .get("design")
        .and_then(|d| d.get("anesthesia"))
        .and_then(|a| a.get("is_under_anesthesia"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let has_pain_plan = content
        .get("design")
        .and_then(|d| d.get("pain"))
        .and_then(|p| p.get("management_plan"))
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    if has_anesthesia || has_pain_plan {
        passed.push("PAIN_ANESTHESIA".to_string());
    } else {
        warnings.push(ValidationIssue {
            code: "PAIN_NO_ANESTHESIA".to_string(),
            category: "anesthesia".to_string(),
            section: "design".to_string(),
            message: format!("疼痛分類為 {} 類，但未說明麻醉或止痛方案", category),
            suggestion: "C/D/E 類疼痛建議提供麻醉及止痛計畫。".to_string(),
        });
    }
}

fn check_personnel_training(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let personnel = content
        .get("personnel")
        .and_then(|v| v.as_array());

    match personnel {
        Some(list) if !list.is_empty() => {
            let has_training = list.iter().any(|p| {
                p.get("training_certificates")
                    .and_then(|t| t.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false)
                    || p.get("trainings")
                        .and_then(|t| t.as_array())
                        .map(|a| !a.is_empty())
                        .unwrap_or(false)
            });
            if has_training {
                passed.push("PERSONNEL_TRAINING".to_string());
            } else {
                warnings.push(ValidationIssue {
                    code: "PERSONNEL_NO_TRAINING".to_string(),
                    category: "personnel".to_string(),
                    section: "personnel".to_string(),
                    message: "工作人員未填寫訓練證照資料".to_string(),
                    suggestion: "建議填寫相關動物實驗訓練證照資訊。".to_string(),
                });
            }
        }
        _ => {
            warnings.push(ValidationIssue {
                code: "PERSONNEL_EMPTY".to_string(),
                category: "personnel".to_string(),
                section: "personnel".to_string(),
                message: "尚未填寫工作人員資料".to_string(),
                suggestion: "請新增計畫參與人員。".to_string(),
            });
        }
    }
}

fn check_alt_databases(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    warnings: &mut Vec<ValidationIssue>,
) {
    // 從 guidelines 的 databases 中找已勾選的
    let checked_count = content
        .get("guidelines")
        .and_then(|g| g.get("databases"))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|db| {
                    db.get("checked")
                        .and_then(|c| c.as_bool())
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0);

    // 也從 purpose.replacement.alt_search.platforms 取得
    let platform_count = content
        .get("purpose")
        .and_then(|p| p.get("replacement"))
        .and_then(|r| r.get("alt_search"))
        .and_then(|a| a.get("platforms"))
        .and_then(|p| p.as_array())
        .map(|arr| arr.iter().filter(|v| {
            v.as_str().map(|s| !s.is_empty()).unwrap_or(false)
        }).count())
        .unwrap_or(0);

    let total = checked_count.max(platform_count);
    if total >= 2 {
        passed.push("ALT_DATABASES".to_string());
    } else {
        warnings.push(ValidationIssue {
            code: "ALT_DATABASES_FEW".to_string(),
            category: "alternatives".to_string(),
            section: "guidelines".to_string(),
            message: format!("替代方案搜尋平台不足（目前 {} 個，建議 2 個以上）", total),
            suggestion: "建議搜尋至少 2 個替代方案資料庫（如 PubMed、ATLA 等）。".to_string(),
        });
    }
}

fn check_humane_endpoint(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let endpoint = content
        .get("design")
        .and_then(|d| d.get("endpoints"))
        .and_then(|e| e.get("humane_endpoint"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if endpoint.trim().is_empty() {
        warnings.push(ValidationIssue {
            code: "HUMANE_ENDPOINT_EMPTY".to_string(),
            category: "endpoints".to_string(),
            section: "design".to_string(),
            message: "人道終點未填寫".to_string(),
            suggestion: "請填寫具體的人道終點標準。".to_string(),
        });
        return;
    }

    let has_vague = VAGUE_WORDS.iter().any(|w| endpoint.contains(w));
    if has_vague {
        warnings.push(ValidationIssue {
            code: "HUMANE_ENDPOINT_VAGUE".to_string(),
            category: "endpoints".to_string(),
            section: "design".to_string(),
            message: "人道終點描述含有模糊用詞（如「明顯」「嚴重」）".to_string(),
            suggestion: "建議使用具體可量化指標（如「體重減輕 20%」或「BCS < 2」）。".to_string(),
        });
    } else {
        passed.push("HUMANE_ENDPOINT".to_string());
    }
}

fn check_postop_observation(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let has_surgery = content
        .get("design")
        .and_then(|d| d.get("anesthesia"))
        .and_then(|a| a.get("anesthesia_type"))
        .and_then(|v| v.as_str())
        .map(|s| s.contains("surgery"))
        .unwrap_or(false);

    if !has_surgery {
        passed.push("POSTOP_OBSERVATION".to_string());
        return;
    }

    let postop = content
        .get("surgery")
        .and_then(|s| s.get("postop_care"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let has_timepoint = postop.contains("小時")
        || postop.contains("hr")
        || postop.contains("天")
        || postop.contains("day")
        || postop.contains("24")
        || postop.contains("48")
        || postop.contains("72");

    if has_timepoint {
        passed.push("POSTOP_OBSERVATION".to_string());
    } else {
        warnings.push(ValidationIssue {
            code: "POSTOP_NO_TIMEPOINT".to_string(),
            category: "surgery".to_string(),
            section: "surgery".to_string(),
            message: "術後照護未提及觀察時間點".to_string(),
            suggestion: "建議明確術後 24/48/72 小時的觀察時間點與項目。".to_string(),
        });
    }
}

fn check_experiment_duration(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let start = get_text(content, &["basic", "start_date"]).unwrap_or_default();
    let end = get_text(content, &["basic", "end_date"]).unwrap_or_default();

    let start_date = chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d");
    let end_date = chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d");

    if let (Ok(s), Ok(e)) = (start_date, end_date) {
        let days = (e - s).num_days();
        if days > 365 * 2 {
            warnings.push(ValidationIssue {
                code: "DURATION_LONG".to_string(),
                category: "date".to_string(),
                section: "basic".to_string(),
                message: format!("實驗期程超過 2 年（{}天）", days),
                suggestion: "超過 2 年的實驗期程，請確認是否有充分理由。".to_string(),
            });
        } else {
            passed.push("EXPERIMENT_DURATION".to_string());
        }
    } else {
        passed.push("EXPERIMENT_DURATION".to_string());
    }
}

fn check_euthanasia_method(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let method = content
        .get("design")
        .and_then(|d| d.get("final_handling"))
        .and_then(|f| f.get("method"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if method == "euthanasia" {
        let euthanasia_type = content
            .get("design")
            .and_then(|d| d.get("final_handling"))
            .and_then(|f| f.get("euthanasia_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if euthanasia_type.is_empty() {
            warnings.push(ValidationIssue {
                code: "EUTHANASIA_NO_METHOD".to_string(),
                category: "euthanasia".to_string(),
                section: "design".to_string(),
                message: "安樂死方法未填寫具體方式".to_string(),
                suggestion: "請填寫具體的安樂死方法。".to_string(),
            });
        } else {
            passed.push("EUTHANASIA_METHOD".to_string());
        }
    } else {
        passed.push("EUTHANASIA_METHOD".to_string());
    }
}

fn check_attachments(
    content: &serde_json::Value,
    passed: &mut Vec<String>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let has_attachments = content
        .get("attachments")
        .and_then(|a| a.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    if has_attachments {
        passed.push("ATTACHMENTS".to_string());
    } else {
        warnings.push(ValidationIssue {
            code: "ATTACHMENTS_EMPTY".to_string(),
            category: "attachments".to_string(),
            section: "attachments".to_string(),
            message: "尚未上傳任何附件".to_string(),
            suggestion: "建議上傳相關研究計畫文件或支持文件。".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_content() {
        let content = json!({});
        let result = validate_protocol(&content);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_valid_significance() {
        let long_text = "a".repeat(100);
        let content = json!({
            "purpose": { "significance": long_text }
        });
        let result = validate_protocol(&content);
        assert!(result.passed.contains(&"PURPOSE_SIGNIFICANCE".to_string()));
    }

    #[test]
    fn test_invalid_significance_too_short() {
        let content = json!({
            "purpose": { "significance": "太短" }
        });
        let result = validate_protocol(&content);
        assert!(result.errors.iter().any(|e| e.code == "PURPOSE_SIGNIFICANCE_SHORT"));
    }

    #[test]
    fn test_invalid_significance_placeholder() {
        let content = json!({
            "purpose": { "significance": "略" }
        });
        let result = validate_protocol(&content);
        assert!(result.errors.iter().any(|e| e.code == "PURPOSE_SIGNIFICANCE_SHORT"));
    }

    #[test]
    fn test_date_logic_ok() {
        let content = json!({
            "basic": { "start_date": "2025-01-01", "end_date": "2026-01-01" }
        });
        let result = validate_protocol(&content);
        assert!(result.passed.contains(&"DATE_LOGIC".to_string()));
    }

    #[test]
    fn test_date_end_before_start() {
        let content = json!({
            "basic": { "start_date": "2026-01-01", "end_date": "2025-01-01" }
        });
        let result = validate_protocol(&content);
        assert!(result.errors.iter().any(|e| e.code == "DATE_END_BEFORE_START"));
    }

    #[test]
    fn test_animal_count_ok() {
        let content = json!({
            "animals": { "total_animals": 10 }
        });
        let result = validate_protocol(&content);
        assert!(result.passed.contains(&"ANIMAL_COUNT".to_string()));
    }

    #[test]
    fn test_animal_count_zero() {
        let content = json!({
            "animals": { "total_animals": 0 }
        });
        let result = validate_protocol(&content);
        assert!(result.errors.iter().any(|e| e.code == "ANIMAL_COUNT_ZERO"));
    }
}
