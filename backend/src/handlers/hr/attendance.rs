// 出勤管理 Handlers

use axum::{
    body::Body,
    extract::{ConnectInfo, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Extension, Json,
};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::{extract_real_ip_with_trust, ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, AttendanceBackfillRequest, AttendanceCorrectionRequest,
        AttendanceQuery, AttendanceWithUser, ClockInRequest, ClockOutRequest,
        MonthlyAttendanceQuery, MonthlyAttendanceSummary, PaginatedResponse,
    },
    require_permission,
    services::{
        audit::{ActivityLogEntry, AuditEntity, RequestContext},
        AuditService, HrService,
    },
    AppState, Result,
};

// 權限碼一律以**字面字串**寫在檢查點上，不抽成 const：
// `backend/tests/permission_codes_exist.rs` 靠 regex 掃 `require_permission!(_, "…")` /
// `has_permission("…")` 的字面值來確認「被檢查的碼真的有定義」。抽成 const 之後
// 那支防呆看不到這些檢查點，等於自願退出保護——這是全庫一致的寫法，不是疏漏。
//
// 補卡用的碼是 `hr.attendance.correct`。此前這裡檢查的是 `hr.attendance.manage`，
// 但該碼**沒有授予任何角色**，而唯一被授予的 `hr.attendance.correct` **沒有任何
// handler 檢查**——兩碼互不相交，行政拿著更正權按不動，實際只有 admin 靠
// `has_permission` 短路做得到。2026-08-26 統一到 `correct`，`manage` 在
// `startup/permissions.rs` 保留為標示過的死碼（不刪 DB 列）。

/// 判斷 IP 是否屬於 Docker 內部網段 (172.16.0.0/12，即 172.16.x.x ~ 172.31.x.x)
fn is_docker_internal_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    let (first, second) = match (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
        (Ok(a), Ok(b)) => (a, b),
        _ => return false,
    };
    first == 172 && (16..=31).contains(&second)
}

/// 驗證打卡 IP 是否在允許範圍內，回傳是否通過
fn check_clock_ip(ip: &str, allowed_ranges: &[String]) -> bool {
    // Docker 內部網段 (172.16.0.0/12)，視為本機存取，直接放行
    if is_docker_internal_ip(ip) {
        return true;
    }
    // 白名單為空表示不限制 → 視為通過
    if allowed_ranges.is_empty() {
        return true;
    }
    HrService::is_ip_in_ranges(ip, allowed_ranges)
}

/// Haversine 公式計算兩點之間的距離（公尺）
fn haversine_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const R: f64 = 6_371_000.0; // 地球半徑（公尺）
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a =
        (d_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    R * c
}

/// 驗證 GPS 座標是否在辦公室允許範圍內，回傳是否通過
/// 若未設定辦公室座標 → 視為通過（不啟用 GPS 驗證）
/// 若使用者未提供 GPS → 視為不通過
fn check_clock_gps(
    user_lat: Option<f64>,
    user_lng: Option<f64>,
    office_lat: Option<f64>,
    office_lng: Option<f64>,
    radius_meters: f64,
) -> bool {
    // 未設定辦公室座標 → 不啟用 GPS 驗證 → 通過
    let (o_lat, o_lng) = match (office_lat, office_lng) {
        (Some(lat), Some(lng)) => (lat, lng),
        _ => return true,
    };

    // 使用者未提供 GPS → 不通過
    let (u_lat, u_lng) = match (user_lat, user_lng) {
        (Some(lat), Some(lng)) => (lat, lng),
        _ => return false,
    };

    let distance = haversine_distance(u_lat, u_lng, o_lat, o_lng);
    tracing::debug!(
        "GPS 距離計算：使用者 ({}, {}) → 辦公室 ({}, {}) = {:.0}m（允許 {:.0}m）",
        u_lat,
        u_lng,
        o_lat,
        o_lng,
        distance,
        radius_meters
    );
    distance <= radius_meters
}

/// 計算打卡位置是否被拒絕：IP 或 GPS 任一通過即放行。
/// 通過回 `None`；被拒回 `Some(原因)`——原因字串同時供 422 回應與 audit log 使用，
/// 故不在此直接組 `AppError`（見 [`clock_location_business_rule`]）。
fn clock_location_denial_reason(
    ip: &str,
    allowed_ip_ranges: &[String],
    user_lat: Option<f64>,
    user_lng: Option<f64>,
    office_lat: Option<f64>,
    office_lng: Option<f64>,
    gps_radius: f64,
) -> Option<String> {
    let ip_ok = check_clock_ip(ip, allowed_ip_ranges);
    let gps_ok = check_clock_gps(user_lat, user_lng, office_lat, office_lng, gps_radius);
    if ip_ok || gps_ok {
        return None;
    }

    let mut reasons = Vec::new();
    if !allowed_ip_ranges.is_empty() {
        reasons.push(format!("IP ({}) 不在允許範圍", ip));
    }
    if let (Some(o_lat), Some(o_lng)) = (office_lat, office_lng) {
        match (user_lat, user_lng) {
            (Some(lat), Some(lng)) => {
                let dist = haversine_distance(lat, lng, o_lat, o_lng);
                reasons.push(format!("GPS 距離 {:.0}m 超出允許範圍", dist));
            }
            _ => reasons.push("未提供 GPS 定位".to_string()),
        }
    }
    Some(reasons.join("；"))
}

/// 將打卡位置拒絕原因組成統一的 422 BusinessRule 錯誤。
///
/// 地理圍籬失敗是「業務規則拒絕」(422)，不是「權限不足」(403)。
/// 切勿改回 AppError::Forbidden：security_response_logger (R22-3/R22-6) 會把所有 403
/// 當成 IDOR 探測事件計數，正常員工重複打卡失敗會在 5 分內累積 20 次而觸發自動停權 +
/// 封鎖來源 IP 24h（封到辦公室共用對外 IP 時會導致全院無法登入）。詳見 PROGRESS.md。
fn clock_location_business_rule(reason: &str) -> AppError {
    AppError::BusinessRule(format!(
        "打卡位置驗證失敗：{reason}。請確認您在辦公室範圍內或連接辦公室 WiFi。"
    ))
}

/// 打卡位置驗證失敗時，寫入一筆 HR 業務 audit log（含原因 / IP / UA）。
///
/// 刻意用一般 [`AuditService::log_activity_oneshot`]（`event_category=HR`、
/// `is_suspicious=false`），**不可**改用 `log_security_event`——後者標
/// `is_suspicious=true`，正常員工反覆打卡失敗會被當可疑事件累積，可能觸發 R22
/// 自動停權 / 封 IP（與「地理圍籬失敗回 422 而非 403」同一動機）。
/// audit 寫入失敗只記 log、不阻擋打卡回應。
async fn audit_clock_denied(
    state: &AppState,
    current_user: &CurrentUser,
    ip: &str,
    headers: &HeaderMap,
    event_type: &str,
    reason: &str,
) {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let mut data_diff = DataDiff::empty();
    data_diff.after = Some(serde_json::json!({ "reason": reason, "ip": ip }));

    let actor = ActorContext::User(current_user.clone());
    let entry = ActivityLogEntry {
        event_category: "HR",
        event_type,
        entity: Some(AuditEntity::new(
            "user",
            current_user.id,
            &current_user.email,
        )),
        data_diff: Some(data_diff),
        request_context: Some(RequestContext {
            ip_address: Some(ip),
            user_agent,
        }),
    };
    if let Err(e) = AuditService::log_activity_oneshot(&state.db, &actor, entry).await {
        tracing::warn!("打卡失敗 audit 寫入失敗（不阻擋回應）: {e}");
    }
}

/// 依使用者權限與請求參數，解析出勤查詢的 user_id 篩選範圍。
///
/// 規則：
/// - 無 `hr.attendance.view_all` 權限 → 一律只能查自己（忽略請求帶的他人 user_id）
/// - 具權限 + 指定 user_id → 篩選該特定人員（「篩選人員」下拉）
/// - 具權限 + view_all=true 且未指定 user_id → 查看所有人（user_id = None）
/// - 具權限 + 未指定 user_id 且未要求 view_all → 預設只看自己
fn resolve_attendance_query_scope(query: &mut AttendanceQuery, current_user: &CurrentUser) {
    if !current_user.has_permission("hr.attendance.view_all") {
        // 無權限：強制只看自己
        query.user_id = Some(current_user.id);
        return;
    }
    // 具權限但未指定特定人員、也未要求查看所有人 → 預設只看自己。
    // 其餘情況保留原 user_id：指定 user_id → 篩選該人；view_all=true 且未指定 → None 查看全部。
    if query.user_id.is_none() && query.view_all != Some(true) {
        query.user_id = Some(current_user.id);
    }
}

/// 列出出勤記錄
/// - 預設只顯示自己的紀錄；具備 hr.attendance.view_all 且傳 view_all=true 時可查看所有人
#[utoipa::path(get, path = "/api/v1/hr/attendance", responses((status = 200)), tag = "HR 出勤", security(("bearer" = [])))]
pub async fn list_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Json<PaginatedResponse<AttendanceWithUser>>> {
    let mut query = params;
    resolve_attendance_query_scope(&mut query, &current_user);
    let result = HrService::list_attendance(&state.db, &query).await?;
    Ok(Json(result))
}

/// 打卡上班
#[utoipa::path(post, path = "/api/v1/hr/attendance/clock-in", request_body = ClockInRequest, responses((status = 200)), tag = "HR 出勤", security(("bearer" = [])))]
pub async fn clock_in(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<ClockInRequest>,
) -> Result<Json<serde_json::Value>> {
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);

    // 驗證位置（IP 或 GPS 任一通過即可）；失敗則寫 audit log + 回 422
    if let Some(reason) = clock_location_denial_reason(
        &ip,
        &state.config.allowed_clock_ip_ranges,
        payload.latitude,
        payload.longitude,
        state.config.clock_office_latitude,
        state.config.clock_office_longitude,
        state.config.clock_gps_radius_meters,
    ) {
        tracing::warn!("打卡位置驗證失敗：{reason}");
        audit_clock_denied(
            &state,
            &current_user,
            &ip,
            &headers,
            "ATTENDANCE_CLOCK_IN_DENIED",
            &reason,
        )
        .await;
        return Err(clock_location_business_rule(&reason));
    }

    let actor = ActorContext::User(current_user.clone());
    let record = HrService::clock_in(
        &state.db,
        &actor,
        payload.source.as_deref(),
        Some(&ip),
        payload.latitude,
        payload.longitude,
    )
    .await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "clock_in_time": record.clock_in_time,
        "message": "打卡成功"
    })))
}

/// 打卡下班
pub async fn clock_out(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<ClockOutRequest>,
) -> Result<Json<serde_json::Value>> {
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);

    // 驗證位置（IP 或 GPS 任一通過即可）；失敗則寫 audit log + 回 422
    if let Some(reason) = clock_location_denial_reason(
        &ip,
        &state.config.allowed_clock_ip_ranges,
        payload.latitude,
        payload.longitude,
        state.config.clock_office_latitude,
        state.config.clock_office_longitude,
        state.config.clock_gps_radius_meters,
    ) {
        tracing::warn!("打卡位置驗證失敗：{reason}");
        audit_clock_denied(
            &state,
            &current_user,
            &ip,
            &headers,
            "ATTENDANCE_CLOCK_OUT_DENIED",
            &reason,
        )
        .await;
        return Err(clock_location_business_rule(&reason));
    }

    let actor = ActorContext::User(current_user.clone());
    let record = HrService::clock_out(
        &state.db,
        &actor,
        payload.source.as_deref(),
        Some(&ip),
        payload.latitude,
        payload.longitude,
    )
    .await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "clock_out_time": record.clock_out_time,
        "regular_hours": record.regular_hours,
        "message": "打卡成功"
    })))
}

/// 匯出出勤記錄為 Excel
#[utoipa::path(get, path = "/api/v1/hr/attendance/export", responses((status = 200)), tag = "HR 出勤", security(("bearer" = [])))]
pub async fn export_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Response> {
    let mut query = params;
    resolve_attendance_query_scope(&mut query, &current_user);

    let data = HrService::export_attendance_to_excel(&state.db, &query).await?;
    let date_str = chrono::Utc::now().format("%Y%m%d").to_string();
    let filename = format!("attendance_records_{}.xlsx", date_str);
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(&filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

/// 更正出勤記錄（既有紀錄改時間）。
///
/// 整天沒打卡的日子沒有 row，這條會 404——那種情況走 `POST /hr/attendance` 補登。
#[utoipa::path(put, path = "/api/v1/hr/attendance/{id}", request_body = AttendanceCorrectionRequest, responses((status = 200)), tag = "HR 出勤", security(("bearer" = [])))]
pub async fn correct_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AttendanceCorrectionRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "hr.attendance.correct");
    let actor = ActorContext::User(current_user.clone());
    HrService::correct_attendance(&state.db, &actor, id, &payload).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "已更正出勤記錄"
    })))
}

/// 補登出勤記錄（補卡）——為缺漏日建立紀錄。
///
/// 「不得補自己的卡」由 service 判定（`HrService::backfill_attendance`），
/// 不放這裡：那是業務規則，且更正路徑也要用同一條，集中在 service 才不會兩邊分歧。
#[utoipa::path(post, path = "/api/v1/hr/attendance", request_body = AttendanceBackfillRequest, responses((status = 200)), tag = "HR 出勤", security(("bearer" = [])))]
pub async fn backfill_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AttendanceBackfillRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "hr.attendance.correct");
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::backfill_attendance(&state.db, &actor, &payload).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "id": record.id,
        "work_date": record.work_date,
        "regular_hours": record.regular_hours,
        "message": "已補登出勤記錄"
    })))
}

/// 收斂工時月報的查詢範圍。
///
/// 無 `hr.attendance.view_all` → 一律只看自己（忽略請求帶的他人 user_id）；
/// 具權限且未指定人員 → 看全體。與出勤列表不同，月報預設是管理視角，不預設收斂成自己。
fn resolve_monthly_report_scope(query: &mut MonthlyAttendanceQuery, current_user: &CurrentUser) {
    if !current_user.has_permission("hr.attendance.view_all") {
        query.user_id = Some(current_user.id);
    }
}

/// 工時月報：某年月每人一列的工時合計
#[utoipa::path(
    get,
    path = "/api/v1/hr/attendance/monthly-report",
    params(MonthlyAttendanceQuery),
    responses((status = 200, description = "工時月報", body = Vec<MonthlyAttendanceSummary>)),
    tag = "HR 出勤",
    security(("bearer" = []))
)]
pub async fn get_monthly_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<MonthlyAttendanceQuery>,
) -> Result<Json<Vec<MonthlyAttendanceSummary>>> {
    // 🔴 CodeRabbit PR #35 指出：`resolve_monthly_report_scope` 只在**沒有** `view_all`
    // 時把 `user_id` 收斂到自己，從未拒絕過請求——8/14 個角色（PI、VET、CLIENT 等
    // 不屬於「內部員工」的角色）根本沒有 `hr.attendance.view`，但呼叫本端點只會拿到
    // 自己（不存在）的紀錄，不會被 403。補上基準門檻，跟 `hr.attendance.correct`
    // 在 backfill/correct 兩支的作法一致（同檔 :375/:394）。
    require_permission!(current_user, "hr.attendance.view");
    let mut query = params;
    resolve_monthly_report_scope(&mut query, &current_user);
    let rows =
        HrService::monthly_attendance_report(&state.db, query.year, query.month, query.user_id)
            .await?;
    Ok(Json(rows))
}

/// 工時月報匯出 Excel
#[utoipa::path(
    get,
    path = "/api/v1/hr/attendance/monthly-report/export",
    params(MonthlyAttendanceQuery),
    responses((
        status = 200,
        description = "工時月報 Excel 檔",
        content_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        body = Vec<u8>
    )),
    tag = "HR 出勤",
    security(("bearer" = []))
)]
pub async fn export_monthly_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<MonthlyAttendanceQuery>,
) -> Result<Response> {
    // 理由同 `get_monthly_report`——兩支端點是同一個查詢的 JSON／Excel 兩種輸出，
    // 授權門檻要一致。
    require_permission!(current_user, "hr.attendance.view");
    let mut query = params;
    resolve_monthly_report_scope(&mut query, &current_user);

    let data = HrService::export_monthly_report_to_excel(
        &state.db,
        query.year,
        query.month,
        query.user_id,
    )
    .await?;
    let filename = format!("attendance_monthly_{}_{:02}.xlsx", query.year, query.month);
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(&filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    const OFFICE_LAT: f64 = 24.654053;
    const OFFICE_LNG: f64 = 120.784923;
    const RADIUS: f64 = 200.0;

    /// 迴歸測試（核心）：地理圍籬失敗必須回 422 (BusinessRule)，**不可**是 403。
    /// 403 會被 security_response_logger 當成 IDOR 探測 → 自動停權 + 封 IP。
    #[test]
    fn geofence_failure_maps_to_422_not_403() {
        // 非辦公室 IP（行動網路）+ GPS 距辦公室極遠 → 兩條都不過
        let reason = clock_location_denial_reason(
            "203.0.113.42",
            &["10.0.4.0/24".to_string()],
            Some(25.047), // 台北車站附近，距苗栗辦公室 > 200m
            Some(121.517),
            Some(OFFICE_LAT),
            Some(OFFICE_LNG),
            RADIUS,
        )
        .expect("距辦公室過遠應驗證失敗");

        let status = clock_location_business_rule(&reason)
            .into_response()
            .status();
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "地理圍籬失敗必須回 422；若回 403 會觸發 IDOR 自動封鎖造成全院鎖死"
        );
        assert_ne!(status, StatusCode::FORBIDDEN);
    }

    /// 拒絕原因須含 GPS 距離數字 —— audit log（ATTENDANCE_CLOCK_*_DENIED）靠它記錄失敗主因。
    #[test]
    fn denial_reason_reports_gps_distance() {
        let reason = clock_location_denial_reason(
            "203.0.113.42",
            &["10.0.4.0/24".to_string()],
            Some(25.047),
            Some(121.517),
            Some(OFFICE_LAT),
            Some(OFFICE_LNG),
            RADIUS,
        )
        .expect("距辦公室過遠應驗證失敗");
        assert!(
            reason.contains("GPS 距離"),
            "原因應含 GPS 距離供 audit 記錄，實際為：{reason}"
        );
    }

    /// GPS 在辦公室半徑內 → 通過（即使 IP 不在白名單，行動網路使用者靠 GPS 打卡）
    #[test]
    fn gps_within_radius_passes_even_when_ip_not_allowed() {
        assert!(
            clock_location_denial_reason(
                "203.0.113.42",
                &["10.0.4.0/24".to_string()],
                Some(OFFICE_LAT),
                Some(OFFICE_LNG),
                Some(OFFICE_LAT),
                Some(OFFICE_LNG),
                RADIUS,
            )
            .is_none(),
            "GPS 在辦公室範圍內應通過"
        );
    }

    /// 辦公室網段 IP → 通過（桌機在辦公室 WiFi，不需 GPS）
    #[test]
    fn office_ip_passes_without_gps() {
        assert!(
            clock_location_denial_reason(
                "10.0.4.17",
                &["10.0.4.0/24".to_string()],
                None,
                None,
                Some(OFFICE_LAT),
                Some(OFFICE_LNG),
                RADIUS,
            )
            .is_none(),
            "辦公室網段 IP 應通過，毋須 GPS"
        );
    }

    fn make_user(permissions: &[&str]) -> CurrentUser {
        CurrentUser {
            id: Uuid::new_v4(),
            email: "u@example.com".to_string(),
            roles: vec![],
            permissions: permissions.iter().map(|s| s.to_string()).collect(),
            jti: "jti".to_string(),
            exp: 0,
            impersonated_by: None,
        }
    }

    fn make_query(user_id: Option<Uuid>, view_all: Option<bool>) -> AttendanceQuery {
        AttendanceQuery {
            user_id,
            view_all,
            from: None,
            to: None,
            status: None,
            page: None,
            per_page: None,
        }
    }

    /// 迴歸測試（核心 bug）：具權限 + view_all=true + 指定 user_id 時，
    /// 必須保留 user_id 篩選該人員，**不可**被清成 None（否則「篩選人員」下拉失效，顯示所有人）。
    #[test]
    fn view_all_with_filter_user_id_keeps_filter() {
        let user = make_user(&["hr.attendance.view_all"]);
        let target = Uuid::new_v4();
        let mut query = make_query(Some(target), Some(true));
        resolve_attendance_query_scope(&mut query, &user);
        assert_eq!(
            query.user_id,
            Some(target),
            "指定篩選人員時應保留該 user_id"
        );
    }

    /// 具權限 + view_all=true 且未指定 user_id → 查看所有人（user_id = None）
    #[test]
    fn view_all_without_filter_shows_everyone() {
        let user = make_user(&["hr.attendance.view_all"]);
        let mut query = make_query(None, Some(true));
        resolve_attendance_query_scope(&mut query, &user);
        assert_eq!(query.user_id, None, "未指定篩選人員時應查看所有人");
    }

    /// 具權限但未要求 view_all 且未指定 user_id → 預設只看自己
    #[test]
    fn no_view_all_defaults_to_self() {
        let user = make_user(&["hr.attendance.view_all"]);
        let mut query = make_query(None, None);
        resolve_attendance_query_scope(&mut query, &user);
        assert_eq!(
            query.user_id,
            Some(user.id),
            "未要求查看所有人時預設只看自己"
        );
    }

    /// 無權限即使帶他人 user_id 也強制只看自己（不得越權查看他人）
    #[test]
    fn without_permission_forced_to_self() {
        let user = make_user(&[]);
        let other = Uuid::new_v4();
        let mut query = make_query(Some(other), Some(true));
        resolve_attendance_query_scope(&mut query, &user);
        assert_eq!(query.user_id, Some(user.id), "無權限應強制只看自己");
    }
}
