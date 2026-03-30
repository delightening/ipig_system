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
    middleware::{extract_real_ip_with_trust, CurrentUser},
    models::{
        AttendanceCorrectionRequest, AttendanceQuery, AttendanceWithUser, ClockInRequest,
        ClockOutRequest, PaginatedResponse,
    },
    services::HrService,
    AppState, Result,
};

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

/// 統一驗證打卡位置：IP 或 GPS 任一通過即可
fn validate_clock_location(
    ip: &str,
    allowed_ip_ranges: &[String],
    user_lat: Option<f64>,
    user_lng: Option<f64>,
    office_lat: Option<f64>,
    office_lng: Option<f64>,
    gps_radius: f64,
) -> Result<()> {
    let ip_ok = check_clock_ip(ip, allowed_ip_ranges);
    let gps_ok = check_clock_gps(user_lat, user_lng, office_lat, office_lng, gps_radius);

    if ip_ok || gps_ok {
        Ok(())
    } else {
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
        tracing::warn!("打卡位置驗證失敗：{}", reasons.join("；"));
        Err(AppError::Forbidden(format!(
            "打卡位置驗證失敗：{}。請確認您在辦公室範圍內或連接辦公室 WiFi。",
            reasons.join("；")
        )))
    }
}

/// 列出出勤記錄
/// - 預設只顯示自己的紀錄；具備 hr.attendance.view_all 且傳 view_all=true 時可查看所有人
#[utoipa::path(get, path = "/api/hr/attendance", responses((status = 200)), tag = "HR 出勤", security(("bearer" = [])))]
pub async fn list_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Json<PaginatedResponse<AttendanceWithUser>>> {
    let mut query = params;
    // 預設只顯示自己的紀錄；只有明確傳 view_all=true 且具權限時才顯示所有人
    let show_all = query.view_all == Some(true) && current_user.has_permission("hr.attendance.view_all");
    if !show_all && query.user_id.is_none() {
        query.user_id = Some(current_user.id);
    } else if show_all {
        query.user_id = None; // 允許查看所有人
    } else if let Some(uid) = query.user_id {
        // 查詢他人時需 view_all 權限
        if uid != current_user.id && !current_user.has_permission("hr.attendance.view_all") {
            query.user_id = Some(current_user.id);
        }
    }
    let result = HrService::list_attendance(&state.db, &query).await?;
    Ok(Json(result))
}

/// 打卡上班
#[utoipa::path(post, path = "/api/hr/attendance/clock-in", request_body = ClockInRequest, responses((status = 200)), tag = "HR 出勤", security(("bearer" = [])))]
pub async fn clock_in(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<ClockInRequest>,
) -> Result<Json<serde_json::Value>> {
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);

    // 驗證位置（IP 或 GPS 任一通過即可）
    validate_clock_location(
        &ip,
        &state.config.allowed_clock_ip_ranges,
        payload.latitude,
        payload.longitude,
        state.config.clock_office_latitude,
        state.config.clock_office_longitude,
        state.config.clock_gps_radius_meters,
    )?;

    let record = HrService::clock_in(
        &state.db,
        current_user.id,
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

    // 驗證位置（IP 或 GPS 任一通過即可）
    validate_clock_location(
        &ip,
        &state.config.allowed_clock_ip_ranges,
        payload.latitude,
        payload.longitude,
        state.config.clock_office_latitude,
        state.config.clock_office_longitude,
        state.config.clock_gps_radius_meters,
    )?;

    let record = HrService::clock_out(
        &state.db,
        current_user.id,
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
#[utoipa::path(get, path = "/api/hr/attendance/export", responses((status = 200)), tag = "HR 出勤", security(("bearer" = [])))]
pub async fn export_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Response> {
    let mut query = params;
    let show_all = query.view_all == Some(true) && current_user.has_permission("hr.attendance.view_all");
    if !show_all && query.user_id.is_none() {
        query.user_id = Some(current_user.id);
    } else if show_all {
        query.user_id = None;
    } else if let Some(uid) = query.user_id {
        if uid != current_user.id && !current_user.has_permission("hr.attendance.view_all") {
            query.user_id = Some(current_user.id);
        }
    }

    let data = HrService::export_attendance_to_excel(&state.db, &query).await?;
    let date_str = chrono::Utc::now().format("%Y%m%d").to_string();
    let filename = format!("attendance_records_{}.xlsx", date_str);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(&filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

/// 更正出勤記錄
pub async fn correct_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AttendanceCorrectionRequest>,
) -> Result<Json<serde_json::Value>> {
    HrService::correct_attendance(&state.db, id, current_user.id, &payload).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "已更正出勤記錄"
    })))
}
