// HR 儀表板 + 員工列表 Handlers

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::Datelike;

use crate::{
    middleware::CurrentUser, models::DashboardCalendarData, repositories::hr as hr_repo,
    services::HrService, AppState, Result,
};

/// 工作人員出勤統計（儀表板用）
pub async fn get_attendance_stats(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.has_permission("hr.attendance.view_all") && !current_user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "無權查看出勤統計".to_string(),
        ));
    }
    // coderabbit-fix: 強型別 NaiveDate 直接傳給 repo（取代 String + $1::date cast）
    let parse_date = |raw: &str| {
        chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .map_err(|_| crate::error::AppError::BadRequest("日期格式應為 YYYY-MM-DD".into()))
    };
    let (start_date, end_date) = match (params.get("start_date"), params.get("end_date")) {
        (Some(s), Some(e)) => (parse_date(s)?, parse_date(e)?),
        _ => {
            let now = chrono::Utc::now();
            let start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .unwrap_or_else(|| now.date_naive());
            (start, now.date_naive())
        }
    };
    // R34-1: SQL 下沉 repositories/hr.rs（CLAUDE.md §4「handler 禁直接寫 SQL」）
    let stats =
        hr_repo::list_attendance_stats_by_date_range(&state.db, start_date, end_date).await?;
    let data: Vec<serde_json::Value> = stats
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "user_id": s.user_id.to_string(),
                "display_name": s.display_name,
                "attendance_days": s.attendance_days,
                "late_count": s.late_count,
                "leave_days": s.leave_days,
                // R34-5: overtime_hours 為 Option<f64>，None → JSON null（資料缺失可見）
                "overtime_hours": s.overtime_hours,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "data": data })))
}

/// 取得儀表板日曆資料
pub async fn get_dashboard_calendar(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<DashboardCalendarData>> {
    if !current_user.has_permission("hr.attendance.view_all") && !current_user.is_admin() {
        return Err(crate::error::AppError::Forbidden("無權查看 HR 日曆".into()));
    }
    let data = HrService::get_dashboard_calendar(&state.db).await?;
    Ok(Json(data))
}

/// 工作人員簡易資訊
#[derive(Debug, serde::Serialize)]
pub struct StaffInfo {
    pub id: uuid::Uuid,
    pub display_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub entry_date: Option<chrono::NaiveDate>,
    pub position: Option<String>,
    pub aup_roles: Vec<String>,
    pub years_experience: i32,
    pub trainings: serde_json::Value,
    /// 是否為本場受僱人員。
    ///
    /// 預設查詢只回 `true` 的人，此欄位恆為 true；`include_external=true`
    /// 時清單會混入外部人員，UI 需要靠它加標籤區分（見部門成員管理）。
    pub is_internal: bool,
}

/// 最小選擇器 DTO：巡場報告陪同人員下拉用（不含 email / phone / trainings 等 PII）
#[derive(Debug, serde::Serialize)]
pub struct StaffSelectorInfo {
    pub id: uuid::Uuid,
    pub display_name: String,
}

/// active EXPERIMENT_STAFF 清單（排序為場內慣用順序）
///
/// `include_external=false`（預設）只回本場受僱人員，供**人事作業**用途
/// （請假代理人、出勤紀錄）——外部人員不適用人事作業，不該出現在那些清單裡。
/// `true` 則一併回外部人員，供**非人事**用途（巡場報告陪同人員、計畫書人員選擇）
/// ——外聘獸醫陪同巡場、外部委員掛在計畫書上都是正常情形。
async fn fetch_active_experiment_staff(
    pool: &sqlx::PgPool,
    include_external: bool,
) -> Result<Vec<StaffInfo>> {
    let staff = sqlx::query_as::<
        _,
        (
            uuid::Uuid,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<chrono::NaiveDate>,
            Option<String>,
            Vec<String>,
            i32,
            serde_json::Value,
            bool,
        ),
    >(
        r#"SELECT id, display_name, email, phone, organization,
               entry_date, position, aup_roles, years_experience, trainings, is_internal
        FROM (
            SELECT DISTINCT u.id, u.display_name, u.email, u.phone, u.organization,
                   u.entry_date, u.position, u.aup_roles, u.years_experience, u.trainings,
                   u.is_internal
            FROM users u
            INNER JOIN user_roles ur ON u.id = ur.user_id
            INNER JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND ($1 OR u.is_internal = true)
              AND r.code = 'EXPERIMENT_STAFF'
        ) s
        -- 依到職日（資深在前）排序；未填到職日者（例如實習生）以 NULLS LAST 排最後，
        -- 與舊版把實習生放在末位的意圖一致。
        -- 舊版是寫死 `display_name LIKE '%<姓名>%'` 的 CASE 清單：既把真實員工姓名
        -- 帶進原始碼，也要求每次人事異動都改 code（清單裡已有離職者的死分支）。
        ORDER BY
            entry_date ASC NULLS LAST,
            display_name ASC"#,
    )
    .bind(include_external)
    .fetch_all(pool)
    .await?;
    Ok(staff
        .into_iter()
        .map(
            |(
                id,
                display_name,
                email,
                phone,
                organization,
                entry_date,
                position,
                aup_roles,
                years_experience,
                trainings,
                is_internal,
            )| StaffInfo {
                id,
                display_name,
                email,
                phone,
                organization,
                entry_date,
                position,
                aup_roles,
                years_experience,
                trainings,
                is_internal,
            },
        )
        .collect())
}

/// `/hr/staff` 查詢參數
#[derive(Debug, serde::Deserialize)]
pub struct StaffForProxyQuery {
    /// 一併列出外部人員。
    ///
    /// 預設 `false`——**人事用途**（請假代理人、出勤紀錄）維持只看本場受僱人員。
    /// 非人事用途要自己帶 `true`：巡場報告陪同人員（外聘獸醫）、計畫書人員選擇
    /// （外聘獸醫 / 外部委員）本來就會有外部人員。
    ///
    /// 與同檔 `InternalUsersQuery::include_external` 同名同義同預設，刻意保持一致。
    #[serde(default)]
    pub include_external: bool,
}

/// 工作人員列表（供請假代理人 / 出勤紀錄 / 巡場報告陪同人員 / 計畫書人員選擇）
///
/// 回傳內容依權限分級（least-privilege）：
/// - HR 權限（hr.leave.create / hr.attendance.view_all）或 admin → 完整 `StaffInfo`
/// - 僅 animal.vet.recommend（獸醫選巡場報告陪同人員）→ 最小欄位 `StaffSelectorInfo`，
///   避免將 staff PII 擴大暴露給 VET 角色
///
/// ⚠️ **「誰該入列」由 `include_external` 決定，不由上面的權限分支決定**——那個分支
/// 回答的是「能看到多少欄位」（PII 曝光層級），與「這個用途該不該含外部人員」是
/// 兩個不同問題。用權限分支兼差判斷入列，admin 去編計畫書時就會看不到外部人員。
pub async fn list_staff_for_proxy(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<StaffForProxyQuery>,
) -> Result<Response> {
    let has_hr_view = current_user.has_permission("hr.leave.create")
        || current_user.has_permission("hr.attendance.view_all")
        || current_user.is_admin();
    if !has_hr_view && !current_user.has_permission("animal.vet.recommend") {
        return Err(crate::error::AppError::Forbidden(
            "無權查看工作人員列表".into(),
        ));
    }
    let staff = fetch_active_experiment_staff(&state.db, params.include_external).await?;
    if has_hr_view {
        return Ok(Json(staff).into_response());
    }
    let selector: Vec<StaffSelectorInfo> = staff
        .into_iter()
        .map(|s| StaffSelectorInfo {
            id: s.id,
            display_name: s.display_name,
        })
        .collect();
    Ok(Json(selector).into_response())
}

/// 內部員工列表查詢參數
#[derive(Debug, serde::Deserialize)]
pub struct InternalUsersQuery {
    /// 一併列出外部人員（部門成員指派用）。
    ///
    /// 預設 `false`，四個人事用途（特休、加班、訓練、QA）維持原行為不變。
    #[serde(default)]
    pub include_external: bool,
}

/// 內部員工列表（排除 admin；供特休管理、人員訓練等使用）
///
/// `include_external=true` 時一併列出外部人員，供**部門成員指派**使用——
/// 「屬於哪個部門」與「適不適用人事作業」是兩件正交的事：IACUC 是內部部門
/// 但聘用外部委員，那些委員需要被編入部門，卻不該出現在特休/加班清單裡。
/// 沿用同一個端點而非另開一支，是為了不讓兩份「誰可以被列出」的邏輯分岔。
pub async fn list_internal_users_for_balance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<InternalUsersQuery>,
) -> Result<Json<Vec<StaffInfo>>> {
    if !current_user.is_admin()
        && !current_user
            .roles
            .contains(&crate::constants::ROLE_ADMIN_STAFF.to_string())
        && !current_user.has_permission("hr.balance.manage")
        && !current_user.has_permission("training.view")
        && !current_user.has_permission("training.manage")
    {
        return Err(crate::error::AppError::Forbidden(
            "無權查看員工列表".to_string(),
        ));
    }
    // 放寬名單是「指派部門成員」的用途，故與該動作用同一個權限碼
    // （`admin.user.edit`，見 handlers/facility.rs 的 assign_department_member）。
    //
    // ⚠️ 這裡原本寫 `facility.manage`，與前端不一致：部門成員對話框是用
    // `admin.user.edit` 決定要不要顯示開關的，兩者不同——持有 admin.user.edit
    // 但無 facility.manage 的角色會看到開關、按下去卻吃 403。
    // 判準必須與「誰能指派成員」一致，否則就是開一個按了會失敗的按鈕。
    if params.include_external
        && !current_user.is_admin()
        && !current_user.has_permission("admin.user.edit")
    {
        return Err(crate::error::AppError::Forbidden(
            "無權查看外部人員列表".to_string(),
        ));
    }
    let staff = sqlx::query_as::<
        _,
        (
            uuid::Uuid,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<chrono::NaiveDate>,
            Option<String>,
            Vec<String>,
            i32,
            serde_json::Value,
            bool,
        ),
    >(
        r#"SELECT u.id, u.display_name, u.email, u.phone, u.organization,
               u.entry_date, u.position, u.aup_roles, u.years_experience, u.trainings,
               u.is_internal
        FROM users u
        WHERE u.is_active = true
        AND ($1 OR u.is_internal = true)
        -- 排除管理員帳號。原本這裡另有一條比對種子管理員信箱字面值的過濾，已移除。
        --
        -- 等價性不只是執行期觀察（雖然實測也確實是同一組 8 人、0 筆差異）：
        -- `startup/seed.rs::ensure_admin_user` 在**每次啟動**都會替該帳號補上
        -- SYSTEM_ADMIN / admin 角色（ON CONFLICT DO NOTHING），所以「種子管理員
        -- 帳號必定持有管理員角色」是由程式碼保證的不變量，下面這條判斷涵蓋得到。
        --
        -- 且「持有管理員角色」本來就是語意正確的判準——信箱可以改，
        -- 授權角色才是事實來源。
        AND NOT EXISTS (
            SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = u.id AND (r.code = 'SYSTEM_ADMIN' OR r.code = 'admin')
        )
        ORDER BY u.is_internal DESC, u.display_name"#,
    )
    .bind(params.include_external)
    .fetch_all(&state.db)
    .await?;
    let result: Vec<StaffInfo> = staff
        .into_iter()
        .map(
            |(
                id,
                display_name,
                email,
                phone,
                organization,
                entry_date,
                position,
                aup_roles,
                years_experience,
                trainings,
                is_internal,
            )| StaffInfo {
                id,
                display_name,
                email,
                phone,
                organization,
                entry_date,
                position,
                aup_roles,
                years_experience,
                trainings,
                is_internal,
            },
        )
        .collect();
    Ok(Json(result))
}
