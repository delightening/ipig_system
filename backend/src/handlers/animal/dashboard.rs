// 儀表板 API Handlers

use axum::{
    extract::{Query, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{middleware::CurrentUser, require_permission, AppState, Result};

/// 取得最近的獸醫師評論（儀表板用）
pub async fn get_vet_comments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    // SEC-IDOR: v2 審計發現 — 儀表板獸醫評論需要 animal.record.view 權限
    // 此端點返回跨計畫聚合資料，需限制為有權查看動物記錄的使用者
    require_permission!(current_user, "animal.record.view");
    // #138 IDOR：非 view_all 使用者僅能看自己可存取計畫的 vet 評論；空集合 → 看不到任何跨計畫資料。
    let boundary: Option<Vec<uuid::Uuid>> =
        if crate::services::access::has_protocol_view_all(&current_user) {
            None
        } else {
            Some(
                crate::services::access::accessible_protocol_ids(&state.db, current_user.id)
                    .await?,
            )
        };
    let limit: i64 = params
        .get("per_page")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    // 查詢最近的獸醫師建議（單一來源 animal_vet_advice_records，巡場完成同步；vet_recommendations 已退役）。
    // 前端 widget 依「巡場報告（獸醫 + 巡場日期）」分組呈現，故此處：
    //   - 依 source_vet_patrol_entry_id 聚合（同一觀察掛的多隻豬併一列、去重——同步是每 entry×每豬
    //     各一筆、內容相同）；jsonb_agg 帶出各豬 id + 耳號供前端做「可點耳號 → 該豬病歷」。
    //   - 日期用 advice_date（= 巡場日期），非 created_at（同步/backfill 當下）。
    //   - IDOR 邊界先濾動物列（$2），故不在可見範圍的豬不進聚合、整條無可見豬則自然消失。
    let rows = sqlx::query_as::<_, (Option<Uuid>, chrono::NaiveDate, String, String, String, serde_json::Value)>(
        r#"
        SELECT
            ar.source_vet_patrol_entry_id AS entry_id,
            ar.advice_date,
            COALESCE(u.display_name, '—') AS author_name,
            ar.suggested_treatment,
            ar.follow_up,
            jsonb_agg(
                jsonb_build_object('id', p.id::text, 'ear_tag', p.ear_tag, 'pen_location', p.pen_location)
                ORDER BY p.ear_tag
            ) AS animals
        FROM animal_vet_advice_records ar
        INNER JOIN animals p ON ar.animal_id = p.id
        LEFT JOIN users u ON ar.created_by = u.id
        WHERE ar.deleted_at IS NULL
          AND (ar.suggested_treatment <> '' OR ar.follow_up <> '')
          AND ($2::uuid[] IS NULL OR p.iacuc_no IN (SELECT iacuc_no FROM protocols WHERE id = ANY($2)))
        GROUP BY ar.source_vet_patrol_entry_id, ar.advice_date, u.display_name,
                 ar.suggested_treatment, ar.follow_up
        ORDER BY ar.advice_date DESC, author_name, ar.source_vet_patrol_entry_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .bind(boundary)
    .fetch_all(&state.db)
    .await?;

    let data: Vec<serde_json::Value> = rows
        .into_iter()
        .map(
            |(entry_id, advice_date, author_name, suggested_treatment, follow_up, animals)| {
                serde_json::json!({
                    "entry_id": entry_id.map(|e| e.to_string()),
                    "advice_date": advice_date.to_string(),
                    "author_name": author_name,
                    "suggested_treatment": suggested_treatment,
                    "follow_up": follow_up,
                    "animals": animals
                })
            },
        )
        .collect();

    Ok(Json(serde_json::json!({ "data": data })))
}
