// Gotenberg PDF 匯出 Handlers（審核結果、審查意見回覆表、AUP 計畫書）

use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    Extension,
};
use uuid::Uuid;

use std::collections::HashMap;

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{access, ProtocolService},
    AppState, Result,
};

/// 建構 PDF 下載回應
fn pdf_response(
    pdf_bytes: Vec<u8>,
    filename: &str,
) -> Result<impl IntoResponse> {
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                crate::utils::http::content_disposition_header(filename),
            ),
        ],
        pdf_bytes,
    ))
}

/// 檢查使用者是否有權限查看指定計畫
async fn check_protocol_view_access(
    state: &AppState,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<()> {
    access::require_protocol_related_access(&state.db, current_user, protocol_id).await
}

/// 匯出審核結果 PDF (AD-04-01-10B)
#[utoipa::path(
    get,
    path = "/api/v1/protocols/{id}/export-review-result",
    params(("id" = Uuid, Path, description = "計畫 ID")),
    responses((status = 200, description = "PDF 檔案")),
    tag = "計畫書管理",
    security(("bearer" = []))
)]
pub async fn export_review_result(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    require_permission!(current_user, "aup.protocol.view_own");
    check_protocol_view_access(&state, &current_user, id).await?;

    let protocol = ProtocolService::get_by_id(&state.db, id).await?;

    // 查詢審查指派
    let assignments: Vec<(Uuid, String, Option<String>, Option<String>)> = sqlx::query_as(
        r#"SELECT ra.reviewer_id, u.display_name, ra.decision, ra.decided_at::text
           FROM review_assignments ra
           JOIN users u ON ra.reviewer_id = u.id
           WHERE ra.protocol_id = $1
           ORDER BY ra.assigned_at"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    // 召集人名稱
    let convener: Option<(String,)> = sqlx::query_as(
        r#"SELECT u.display_name FROM users u
           JOIN user_roles ur ON u.id = ur.user_id
           JOIN roles r ON ur.role_id = r.id
           WHERE r.name = 'IACUC_CHAIR' LIMIT 1"#,
    )
    .fetch_optional(&state.db)
    .await?;
    let convener_name = convener.map(|c| c.0).unwrap_or_default();

    // 建構模板資料
    let mut reviewers = Vec::new();
    for (_, name, decision, decided_at) in &assignments {
        let initial_result = match decision.as_deref() {
            Some("APPROVED") => "approved",
            Some("REVISION_REQUIRED") => "revision_required",
            Some("REJECTED") => "rejected",
            _ => "",
        };
        reviewers.push(serde_json::json!({
            "reviewer_name": name,
            "initial_result": initial_result,
            "initial_comment": "",
            "initial_date": decided_at.as_deref().unwrap_or(""),
            "final_result": "",
            "final_comment": "",
            "final_date": "",
        }));
    }

    // 若無委員，建立空白單頁
    if reviewers.is_empty() {
        reviewers.push(serde_json::json!({
            "reviewer_name": "",
            "initial_result": "",
            "initial_comment": "",
            "initial_date": "",
            "final_result": "",
            "final_comment": "",
            "final_date": "",
        }));
    }

    let mut ctx = tera::Context::new();
    ctx.insert("reviewers", &reviewers);
    ctx.insert("convener_name", &convener_name);

    let html = state.templates.render("review_result.html", &ctx)?;
    let pdf_bytes = state.gotenberg.html_to_pdf(&html).await?;

    let filename = format!(
        "{}_審核結果.pdf",
        protocol
            .protocol
            .iacuc_no
            .as_deref()
            .unwrap_or(&protocol.protocol.protocol_no)
    );
    pdf_response(pdf_bytes, &filename)
}

/// 匯出審查意見回覆表 PDF (AD-04-01-04C)
#[utoipa::path(
    get,
    path = "/api/v1/protocols/{id}/export-review-comments",
    params(("id" = Uuid, Path, description = "計畫 ID")),
    responses((status = 200, description = "PDF 檔案")),
    tag = "計畫書管理",
    security(("bearer" = []))
)]
pub async fn export_review_comments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    require_permission!(current_user, "aup.protocol.view_own");
    check_protocol_view_access(&state, &current_user, id).await?;

    let protocol = ProtocolService::get_by_id(&state.db, id).await?;

    // 直接查詢包含 review_stage 的意見
    #[derive(sqlx::FromRow)]
    struct CommentWithStage {
        id: Uuid,
        reviewer_id: Uuid,
        content: String,
        parent_comment_id: Option<Uuid>,
        review_stage: Option<String>,
    }

    let comments: Vec<CommentWithStage> = sqlx::query_as(
        r#"SELECT id, reviewer_id, content, parent_comment_id, review_stage
           FROM review_comments
           WHERE protocol_id = $1
           ORDER BY COALESCE(parent_comment_id, id), created_at"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let basic = protocol
        .protocol
        .working_content
        .as_ref()
        .and_then(|c| c.get("basic"));

    let iacuc_no = protocol
        .protocol
        .iacuc_no
        .as_deref()
        .or_else(|| {
            basic.and_then(|b| {
                b.get("apply_study_number")
                    .and_then(|v| v.as_str())
            })
        })
        .unwrap_or("尚未指派");

    let pi_name = basic
        .and_then(|b| b.get("pi"))
        .and_then(|pi| pi.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("未指定");

    // 初審意見（PRE_REVIEW 階段，非回覆）
    let pre_review: Vec<serde_json::Value> = comments
        .iter()
        .filter(|c| {
            c.parent_comment_id.is_none()
                && c.review_stage.as_deref() == Some("PRE_REVIEW")
        })
        .map(|c| {
            let replies: String = comments
                .iter()
                .filter(|r| r.parent_comment_id == Some(c.id))
                .map(|r| r.content.clone())
                .collect::<Vec<_>>()
                .join("\n");
            serde_json::json!({
                "content": c.content,
                "replies": replies,
            })
        })
        .collect();

    // 獸醫師審查項目
    let default_items = vec![
        "計畫基本資料",
        "簡述研究目的",
        "說明動物實驗必要性",
        "動物實驗試驗設計",
        "實驗預期結束時機及人道終點",
        "實驗結束動物處置方式",
        "有無進行危害性物質實驗",
        "動物麻醉用藥及方法合理性",
        "手術操作及術中動物觀察",
        "手術後照護及術後給藥方式",
        "實驗動物資料",
        "動物實驗相關人員資料",
    ];

    let vet_review = &protocol.vet_review;
    let vet_items: Vec<serde_json::Value> = if let Some(vr) = vet_review {
        if let Some(form_value) = &vr.review_form {
            match serde_json::from_value::<crate::models::VetReviewForm>(form_value.clone()) {
                Ok(form) => form
                    .items
                    .iter()
                    .map(|item| {
                        serde_json::json!({
                            "item_name": item.item_name,
                            "compliance": item.compliance,
                            "comment": item.comment,
                            "pi_reply": item.pi_reply,
                        })
                    })
                    .collect(),
                Err(_) => default_items
                    .iter()
                    .map(|name| {
                        serde_json::json!({
                            "item_name": name,
                            "compliance": "",
                            "comment": "",
                            "pi_reply": "",
                        })
                    })
                    .collect(),
            }
        } else {
            default_items
                .iter()
                .map(|name| {
                    serde_json::json!({
                        "item_name": name,
                        "compliance": "",
                        "comment": "",
                        "pi_reply": "",
                    })
                })
                .collect()
        }
    } else {
        default_items
            .iter()
            .map(|name| {
                serde_json::json!({
                    "item_name": name,
                    "compliance": "",
                    "comment": "",
                    "pi_reply": "",
                })
            })
            .collect()
    };

    let vet_signature = vet_review
        .as_ref()
        .and_then(|vr| vr.review_form.as_ref())
        .and_then(|fv| serde_json::from_value::<crate::models::VetReviewForm>(fv.clone()).ok())
        .and_then(|f| f.vet_signature.filter(|s| !s.is_empty()))
        .map(|_| "(已簽章)")
        .unwrap_or("");

    let vet_signed_date = vet_review
        .as_ref()
        .and_then(|vr| vr.review_form.as_ref())
        .and_then(|fv| serde_json::from_value::<crate::models::VetReviewForm>(fv.clone()).ok())
        .and_then(|f| f.signed_at.map(|d| d.format("%Y-%m-%d").to_string()))
        .unwrap_or_default();

    // 委員審查意見（按 reviewer 分組）
    let under_review_comments: Vec<&CommentWithStage> = comments
        .iter()
        .filter(|c| {
            c.review_stage.as_deref() == Some("UNDER_REVIEW")
                && c.parent_comment_id.is_none()
        })
        .collect();

    // 預建 reply 索引：parent_comment_id → 回覆內容（O(n) 建立）
    let mut reply_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for c in &comments {
        if let Some(parent_id) = c.parent_comment_id {
            reply_map
                .entry(parent_id)
                .or_default()
                .push(c.content.clone());
        }
    }

    // 預建 reviewer 分組索引（O(n) 建立，保留插入順序）
    let mut reviewer_order: Vec<Uuid> = Vec::new();
    let mut comments_by_reviewer: HashMap<Uuid, Vec<&CommentWithStage>> = HashMap::new();
    for c in &under_review_comments {
        comments_by_reviewer
            .entry(c.reviewer_id)
            .or_default()
            .push(c);
        if !comments_by_reviewer[&c.reviewer_id].len() > 1 {
            // first insert for this reviewer
        }
    }
    // 保留原始順序（第一次出現的 reviewer 在前）
    {
        let mut seen = std::collections::HashSet::new();
        for c in &under_review_comments {
            if seen.insert(c.reviewer_id) {
                reviewer_order.push(c.reviewer_id);
            }
        }
    }

    // O(n) 查找組裝（取代原本 O(n²×m) 的巢狀 filter）
    let reviewer_groups: Vec<serde_json::Value> = reviewer_order
        .iter()
        .map(|rid| {
            let group_comments: Vec<serde_json::Value> = comments_by_reviewer
                .get(rid)
                .map(|cmts| {
                    cmts.iter()
                        .map(|c| {
                            let replies = reply_map
                                .get(&c.id)
                                .map(|r| r.join("\n"))
                                .unwrap_or_default();
                            serde_json::json!({
                                "first_review": c.content,
                                "first_reply": replies,
                                "second_review": "",
                                "second_reply": "",
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            serde_json::json!({ "comments": group_comments })
        })
        .collect();

    let mut ctx = tera::Context::new();
    ctx.insert("iacuc_no", iacuc_no);
    ctx.insert("protocol_title", &protocol.protocol.title);
    ctx.insert("pi_name", pi_name);
    ctx.insert("pre_review_comments", &pre_review);
    ctx.insert("vet_review_items", &vet_items);
    ctx.insert("vet_signature", vet_signature);
    ctx.insert("vet_signed_date", &vet_signed_date);
    ctx.insert("reviewer_groups", &reviewer_groups);

    let html = state.templates.render("review_comments.html", &ctx)?;
    let pdf_bytes = state.gotenberg.html_to_pdf(&html).await?;

    let filename = format!("{}_審查意見回覆表.pdf", iacuc_no);
    pdf_response(pdf_bytes, &filename)
}

/// 匯出 AUP 計畫書 PDF（Gotenberg 版）
#[utoipa::path(
    get,
    path = "/api/v1/protocols/{id}/export-pdf-v2",
    params(("id" = Uuid, Path, description = "計畫 ID")),
    responses((status = 200, description = "PDF 檔案")),
    tag = "計畫書管理",
    security(("bearer" = []))
)]
pub async fn export_protocol_pdf_v2(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    require_permission!(current_user, "aup.protocol.view_own");
    check_protocol_view_access(&state, &current_user, id).await?;

    let protocol = ProtocolService::get_by_id(&state.db, id).await?;
    let content = protocol
        .protocol
        .working_content
        .as_ref()
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let basic = content.get("basic").cloned().unwrap_or(serde_json::json!({}));

    // 翻譯標籤
    let project_type_label = translate_project_type(
        basic.get("project_type").and_then(|v| v.as_str()),
        basic.get("project_type_other").and_then(|v| v.as_str()),
    );
    let project_category_label = translate_project_category(
        basic.get("project_category").and_then(|v| v.as_str()),
        basic.get("project_category_other").and_then(|v| v.as_str()),
    );
    let funding_label = translate_funding_sources(
        basic.get("funding_sources").and_then(|v| v.as_array()),
        basic.get("funding_other").and_then(|v| v.as_str()),
    );

    // 將翻譯後的標籤注入 basic 物件
    let mut basic_ctx = basic.as_object().cloned().unwrap_or_default();
    basic_ctx.insert(
        "project_type_label".to_string(),
        serde_json::Value::String(project_type_label),
    );
    basic_ctx.insert(
        "project_category_label".to_string(),
        serde_json::Value::String(project_category_label),
    );
    basic_ctx.insert(
        "funding_sources_label".to_string(),
        serde_json::Value::String(funding_label),
    );

    // 人員資料：預計算 roles_label / trainings_label / trainings_detail
    let personnel: Vec<serde_json::Value> = content
        .get("personnel")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|p| {
                    let mut p = p.clone();
                    if let Some(obj) = p.as_object_mut() {
                        // roles_label: 只顯示代碼，例如 "a, b, c"
                        let roles_label = obj
                            .get("roles")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|r| r.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                            .unwrap_or_default();
                        // trainings_label: 只顯示代碼，例如 "A, B"
                        let trainings_label = obj
                            .get("trainings")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|t| t.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                            .unwrap_or_default();
                        // trainings_detail: 代碼 + 證書編號，例如 "A（(112)動訓字第001號）；B"
                        let certs = obj
                            .get("training_certificates")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        let trainings_detail = obj
                            .get("trainings")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|t| t.as_str())
                                    .map(|code| {
                                        let cert_nos: Vec<&str> = certs
                                            .iter()
                                            .filter(|c| {
                                                c.get("training_code")
                                                    .and_then(|v| v.as_str())
                                                    == Some(code)
                                            })
                                            .filter_map(|c| {
                                                c.get("certificate_no")
                                                    .and_then(|v| v.as_str())
                                                    .filter(|s| !s.is_empty())
                                            })
                                            .collect();
                                        if cert_nos.is_empty() {
                                            code.to_string()
                                        } else {
                                            format!("{} ({})", code, cert_nos.join("; "))
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join("<br>")
                            })
                            .unwrap_or_default();
                        obj.insert(
                            "roles_label".to_string(),
                            serde_json::Value::String(roles_label),
                        );
                        obj.insert(
                            "trainings_label".to_string(),
                            serde_json::Value::String(trainings_label),
                        );
                        obj.insert(
                            "trainings_detail".to_string(),
                            serde_json::Value::String(trainings_detail),
                        );
                    }
                    p
                })
                .collect()
        })
        .unwrap_or_default();

    // 判斷是否需要手術
    let needs_surgery = content
        .get("design")
        .and_then(|d| d.get("anesthesia"))
        .map(|a| {
            let is_under = a
                .get("is_under_anesthesia")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let a_type = a
                .get("anesthesia_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            is_under
                && (a_type == "survival_surgery" || a_type == "non_survival_surgery")
        })
        .unwrap_or(false);

    let iacuc_no = protocol
        .protocol
        .iacuc_no
        .as_deref()
        .unwrap_or("尚未指派");

    // 取得委託單位名稱（供封面使用）
    let sponsor_name = basic_ctx
        .get("sponsor")
        .and_then(|s| s.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let mut ctx = tera::Context::new();
    ctx.insert("protocol_title", &protocol.protocol.title);
    ctx.insert("iacuc_no", iacuc_no);
    ctx.insert("pi_name", &protocol.pi_name.as_deref().unwrap_or(""));
    ctx.insert("sponsor_name", sponsor_name);
    ctx.insert(
        "start_date",
        &protocol
            .protocol
            .start_date
            .map(|d| d.to_string())
            .unwrap_or_default(),
    );
    ctx.insert(
        "end_date",
        &protocol
            .protocol
            .end_date
            .map(|d| d.to_string())
            .unwrap_or_default(),
    );
    ctx.insert("basic", &serde_json::Value::Object(basic_ctx));
    ctx.insert(
        "purpose",
        &content.get("purpose").unwrap_or(&serde_json::json!(null)),
    );
    ctx.insert(
        "items",
        &content.get("items").unwrap_or(&serde_json::json!(null)),
    );
    ctx.insert(
        "design",
        &content.get("design").unwrap_or(&serde_json::json!(null)),
    );
    ctx.insert(
        "guidelines",
        &content
            .get("guidelines")
            .unwrap_or(&serde_json::json!(null)),
    );
    ctx.insert(
        "surgery",
        &content.get("surgery").unwrap_or(&serde_json::json!(null)),
    );
    ctx.insert("needs_surgery", &needs_surgery);
    ctx.insert(
        "animals",
        &content.get("animals").unwrap_or(&serde_json::json!(null)),
    );
    ctx.insert("personnel", &personnel);
    ctx.insert(
        "attachments",
        &content
            .get("attachments")
            .unwrap_or(&serde_json::json!(null)),
    );

    // 渲染頁首頁尾模板（含動態頁碼）
    let mut hf_ctx = tera::Context::new();
    hf_ctx.insert("doc_id", "AD-04-01-01E");
    let header_html = state.templates.render("partials/header.html", &hf_ctx)?;
    let footer_html = state.templates.render("partials/footer.html", &hf_ctx)?;

    // ── 兩階段渲染：第一階段產出 PDF，解析章節頁碼 ──
    let first_html = state.templates.render("protocol.html", &ctx)?;
    let first_pdf = state
        .gotenberg
        .html_to_pdf_with_headers(&first_html, &header_html, &footer_html)
        .await?;

    let section_pages = crate::utils::pdf_pages::find_section_pages(&first_pdf);

    // ── 第二階段：將頁碼注入 TOC，重新渲染 ──
    for i in 1..=8 {
        let marker = format!("§SEC{}§", i);
        if let Some(page) = section_pages.get(&marker) {
            ctx.insert(format!("toc_page_{}", i), page);
        }
    }

    let final_html = state.templates.render("protocol.html", &ctx)?;
    let pdf_bytes = state
        .gotenberg
        .html_to_pdf_with_headers(&final_html, &header_html, &footer_html)
        .await?;

    let filename = format!("{}_AUP計畫書.pdf", protocol.protocol.title);
    pdf_response(pdf_bytes, &filename)
}

// ─── 翻譯輔助函式 ───

fn translate_project_type(key: Option<&str>, other: Option<&str>) -> String {
    let label = match key {
        Some("1_basic_research" | "basic_research") => "1. 基礎研究",
        Some("2_applied_research" | "applied_research") => "2. 應用研究",
        Some("3_pre_market_testing" | "pre_market_testing") => "3. 產品上市前測試",
        Some("4_educational" | "educational" | "4_teaching_training" | "teaching_training") => {
            "4. 教學訓練"
        }
        Some("5_biologics_manufacturing" | "biologics_manufacturing") => "5. 製造生物製劑",
        Some("6_other" | "other") => "6. 其他",
        Some(k) => return k.to_string(),
        None => return String::new(),
    };
    match other {
        Some(o) if !o.is_empty() => format!("{} ({})", label, o),
        _ => label.to_string(),
    }
}

fn translate_project_category(key: Option<&str>, other: Option<&str>) -> String {
    let label = match key {
        Some("1_medical" | "medical") => "1. 醫學研究",
        Some("2_agricultural" | "agricultural") => "2. 農業研究",
        Some("3_drugs_vaccines" | "drugs_vaccines" | "3_drug_herbal" | "drug_herbal") => {
            "3. 藥物及疫苗"
        }
        Some("4_supplements" | "supplements" | "4_health_food" | "health_food") => "4. 健康食品",
        Some("5_food" | "food") => "5. 食品",
        Some(
            "6_toxics_chemicals" | "toxics_chemicals" | "6_toxic_chemical" | "toxic_chemical",
        ) => "6. 毒、化學品",
        Some(
            "7_medical_materials" | "medical_materials" | "7_medical_device" | "medical_device",
        ) => "7. 醫療器材",
        Some("8_pesticide" | "pesticide") => "8. 農藥",
        Some("9_animal_drugs_vaccines" | "animal_drugs_vaccines") => "9. 動物用藥及疫苗",
        Some("10_animal_supplements_feed" | "animal_supplements_feed") => {
            "10. 動物保健品、飼料添加物"
        }
        Some("11_cosmetics" | "cosmetics") => "11. (含藥)化妝品",
        Some("12_other" | "other") => "12. 其他",
        Some(k) => return k.to_string(),
        None => return String::new(),
    };
    match other {
        Some(o) if !o.is_empty() => format!("{} ({})", label, o),
        _ => label.to_string(),
    }
}

fn translate_funding_sources(
    sources: Option<&Vec<serde_json::Value>>,
    other: Option<&str>,
) -> String {
    let sources = match sources {
        Some(s) if !s.is_empty() => s,
        _ => return String::new(),
    };
    let labels: Vec<&str> = sources
        .iter()
        .filter_map(|s| {
            s.as_str().map(|s| match s {
                "moa" => "農業部",
                "mohw" => "衛生福利部",
                "nstc" => "國家科學及技術委員會",
                "moe" => "教育部",
                "env" => "環境部",
                "other" => "其他",
                _ => s,
            })
        })
        .collect();
    let mut result = labels.join(", ");
    if let Some(o) = other {
        if !o.is_empty()
            && sources
                .iter()
                .any(|s| s.as_str() == Some("other"))
        {
            result = format!("{} ({})", result, o);
        }
    }
    result
}
