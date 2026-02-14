// 儀表板 API Handlers

use axum::{
    extract::{Query, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    AppState, Result,
};

/// 取得最近的獸醫師評論（儀表板用）
pub async fn get_vet_comments(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    let limit: i64 = params
        .get("per_page")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    
    // 查詢最近的獸醫建議
    let comments = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, String, chrono::DateTime<chrono::Utc>, String)>(
        r#"
        SELECT 
            vr.id,
            p.id as pig_id,
            p.ear_tag,
            p.pen_location,
            vr.content,
            vr.created_at,
            u.display_name as created_by_name
        FROM vet_recommendations vr
        INNER JOIN pig_observations po ON vr.record_type = 'observation'::vet_record_type AND vr.record_id = po.id
        INNER JOIN pigs p ON po.pig_id = p.id
        INNER JOIN users u ON vr.created_by = u.id
        ORDER BY vr.created_at DESC
        LIMIT $1
        "#
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    
    let data: Vec<serde_json::Value> = comments
        .into_iter()
        .map(|(id, pig_id, ear_tag, pen_location, content, created_at, created_by_name)| {
            serde_json::json!({
                "id": id.to_string(),
                "pig_id": pig_id.to_string(),
                "pig_ear_tag": ear_tag.clone(),
                "pig_num": ear_tag,
                "pen_location": pen_location,
                "content": content,
                "created_at": created_at.to_rfc3339(),
                "author_name": created_by_name
            })
        })
        .collect();
    
    Ok(Json(serde_json::json!({ "data": data })))
}
