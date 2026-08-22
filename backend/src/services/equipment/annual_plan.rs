// 年度計畫（annual plans）與執行摘要

use rand::RngExt;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        AnnualPlanExecutionRow, AnnualPlanExecutionSummary, AnnualPlanQuery,
        AnnualPlanWithEquipment, CalibrationCycle, CreateAnnualPlanRequest, Equipment,
        ExecutionSummaryQuery, GenerateAnnualPlanRequest, MonthExecutionDetail,
        MonthExecutionStatus, UpdateAnnualPlanRequest,
    },
    Result,
};

use super::{check_view_permission, EquipmentService};

impl EquipmentService {
    // ========== Annual Plan (年度計畫) ==========

    pub async fn list_annual_plans(
        pool: &PgPool,
        query: &AnnualPlanQuery,
        current_user: &CurrentUser,
    ) -> Result<Vec<AnnualPlanWithEquipment>> {
        check_view_permission(current_user)?;

        let data = sqlx::query_as::<_, AnnualPlanWithEquipment>(
            r#"
            SELECT ap.id, ap.year, ap.equipment_id, e.name AS equipment_name,
                   e.serial_number AS equipment_serial_number,
                   ap.calibration_type, ap.cycle,
                   ap.month_1, ap.month_2, ap.month_3, ap.month_4,
                   ap.month_5, ap.month_6, ap.month_7, ap.month_8,
                   ap.month_9, ap.month_10, ap.month_11, ap.month_12,
                   ap.generated_at
            FROM equipment_annual_plans ap
            INNER JOIN equipment e ON ap.equipment_id = e.id
            WHERE ap.year = $1
              AND ($2::uuid IS NULL OR ap.equipment_id = $2)
              AND ($3::calibration_type IS NULL OR ap.calibration_type = $3)
            ORDER BY e.name, ap.calibration_type
            "#,
        )
        .bind(query.year)
        .bind(query.equipment_id)
        .bind(&query.calibration_type)
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn generate_annual_plan(
        pool: &PgPool,
        payload: &GenerateAnnualPlanRequest,
        current_user: &CurrentUser,
    ) -> Result<Vec<AnnualPlanWithEquipment>> {
        if !current_user.has_permission("equipment.plan.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理年度計畫".into()));
        }

        // 取得所有啟用的設備（有設定週期的）
        let equipment_list = sqlx::query_as::<_, Equipment>(
            "SELECT * FROM equipment WHERE status = 'active' AND (calibration_cycle IS NOT NULL OR inspection_cycle IS NOT NULL)",
        )
        .fetch_all(pool)
        .await?;

        // 查詢各設備最後一次校正月份，用於智慧月份推算
        #[derive(sqlx::FromRow)]
        struct LastCalMonth {
            equipment_id: Uuid,
            calibration_type: crate::models::CalibrationType,
            last_month: i32,
        }
        let eq_ids: Vec<Uuid> = equipment_list.iter().map(|e| e.id).collect();
        let last_cal_records = if eq_ids.is_empty() {
            vec![]
        } else {
            sqlx::query_as::<_, LastCalMonth>(
                r#"
                SELECT equipment_id, calibration_type,
                       EXTRACT(MONTH FROM MAX(calibrated_at))::int AS last_month
                FROM equipment_calibrations
                WHERE equipment_id = ANY($1)
                GROUP BY equipment_id, calibration_type
                "#,
            )
            .bind(&eq_ids)
            .fetch_all(pool)
            .await?
        };

        use std::collections::HashMap;
        let mut last_cal_map: HashMap<(Uuid, String), u32> = HashMap::new();
        for rec in last_cal_records {
            let type_key = format!("{:?}", rec.calibration_type).to_lowercase();
            last_cal_map.insert((rec.equipment_id, type_key), rec.last_month as u32);
        }

        // 預先產生所有月份（避免持有 rng 跨 await）
        let plans: Vec<_> = {
            let mut rng = rand::rng();
            equipment_list
                .iter()
                .flat_map(|eq| {
                    let mut items = Vec::new();
                    if let (Some(cal_type), Some(cycle)) =
                        (&eq.calibration_type, &eq.calibration_cycle)
                    {
                        let type_key = format!("{:?}", cal_type).to_lowercase();
                        let last_month = last_cal_map.get(&(eq.id, type_key)).copied();
                        let months = pick_smart_months(cycle, last_month, &mut rng);
                        items.push((eq.id, cal_type.clone(), cycle.clone(), months));
                    }
                    if let Some(cycle) = &eq.inspection_cycle {
                        let type_key = "inspection".to_string();
                        let last_month = last_cal_map.get(&(eq.id, type_key)).copied();
                        let months = pick_smart_months(cycle, last_month, &mut rng);
                        items.push((
                            eq.id,
                            crate::models::CalibrationType::Inspection,
                            cycle.clone(),
                            months,
                        ));
                    }
                    items
                })
                .collect()
        };

        for (eq_id, cal_type, cycle, months) in &plans {
            insert_annual_plan(pool, payload.year, *eq_id, cal_type, cycle, months).await?;
        }

        // 回傳產生的計畫
        let query = AnnualPlanQuery {
            year: payload.year,
            equipment_id: None,
            calibration_type: None,
        };
        Self::list_annual_plans(pool, &query, current_user).await
    }

    pub async fn create_annual_plan(
        pool: &PgPool,
        payload: &CreateAnnualPlanRequest,
        current_user: &CurrentUser,
    ) -> Result<AnnualPlanWithEquipment> {
        if !current_user.has_permission("equipment.plan.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理年度計畫".into()));
        }

        let months = [
            payload.month_1,
            payload.month_2,
            payload.month_3,
            payload.month_4,
            payload.month_5,
            payload.month_6,
            payload.month_7,
            payload.month_8,
            payload.month_9,
            payload.month_10,
            payload.month_11,
            payload.month_12,
        ];
        insert_annual_plan(
            pool,
            payload.year,
            payload.equipment_id,
            &payload.calibration_type,
            &payload.cycle,
            &months,
        )
        .await?;

        let plan = sqlx::query_as::<_, AnnualPlanWithEquipment>(
            r#"
            SELECT ap.id, ap.year, ap.equipment_id, e.name AS equipment_name,
                   e.serial_number AS equipment_serial_number,
                   ap.calibration_type, ap.cycle,
                   ap.month_1, ap.month_2, ap.month_3, ap.month_4,
                   ap.month_5, ap.month_6, ap.month_7, ap.month_8,
                   ap.month_9, ap.month_10, ap.month_11, ap.month_12,
                   ap.generated_at
            FROM equipment_annual_plans ap
            INNER JOIN equipment e ON ap.equipment_id = e.id
            WHERE ap.year = $1 AND ap.equipment_id = $2 AND ap.calibration_type = $3
            "#,
        )
        .bind(payload.year)
        .bind(payload.equipment_id)
        .bind(&payload.calibration_type)
        .fetch_one(pool)
        .await?;

        Ok(plan)
    }

    pub async fn update_annual_plan(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateAnnualPlanRequest,
        current_user: &CurrentUser,
    ) -> Result<AnnualPlanWithEquipment> {
        if !current_user.has_permission("equipment.plan.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理年度計畫".into()));
        }

        sqlx::query(
            r#"
            UPDATE equipment_annual_plans SET
                calibration_type = COALESCE($2::calibration_type, calibration_type),
                cycle = COALESCE($3::calibration_cycle, cycle),
                month_1 = $4, month_2 = $5, month_3 = $6, month_4 = $7,
                month_5 = $8, month_6 = $9, month_7 = $10, month_8 = $11,
                month_9 = $12, month_10 = $13, month_11 = $14, month_12 = $15,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(payload.calibration_type.as_ref())
        .bind(payload.cycle.as_ref())
        .bind(payload.month_1)
        .bind(payload.month_2)
        .bind(payload.month_3)
        .bind(payload.month_4)
        .bind(payload.month_5)
        .bind(payload.month_6)
        .bind(payload.month_7)
        .bind(payload.month_8)
        .bind(payload.month_9)
        .bind(payload.month_10)
        .bind(payload.month_11)
        .bind(payload.month_12)
        .execute(pool)
        .await?;

        let plan = sqlx::query_as::<_, AnnualPlanWithEquipment>(
            r#"
            SELECT ap.id, ap.year, ap.equipment_id, e.name AS equipment_name,
                   e.serial_number AS equipment_serial_number,
                   ap.calibration_type, ap.cycle,
                   ap.month_1, ap.month_2, ap.month_3, ap.month_4,
                   ap.month_5, ap.month_6, ap.month_7, ap.month_8,
                   ap.month_9, ap.month_10, ap.month_11, ap.month_12,
                   ap.generated_at
            FROM equipment_annual_plans ap
            INNER JOIN equipment e ON ap.equipment_id = e.id
            WHERE ap.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(plan)
    }

    pub async fn delete_annual_plan(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if !current_user.has_permission("equipment.plan.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理年度計畫".into()));
        }

        let result = sqlx::query("DELETE FROM equipment_annual_plans WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("年度計畫項目不存在".into()));
        }

        Ok(())
    }

    pub async fn get_execution_summary(
        pool: &PgPool,
        query: &ExecutionSummaryQuery,
        current_user: &CurrentUser,
    ) -> Result<AnnualPlanExecutionSummary> {
        check_view_permission(current_user)?;

        // 查詢 1：取計畫列
        let plans = sqlx::query_as::<_, AnnualPlanWithEquipment>(
            r#"
            SELECT ap.id, ap.year, ap.equipment_id, e.name AS equipment_name,
                   e.serial_number AS equipment_serial_number,
                   ap.calibration_type, ap.cycle,
                   ap.month_1, ap.month_2, ap.month_3, ap.month_4,
                   ap.month_5, ap.month_6, ap.month_7, ap.month_8,
                   ap.month_9, ap.month_10, ap.month_11, ap.month_12,
                   ap.generated_at
            FROM equipment_annual_plans ap
            INNER JOIN equipment e ON ap.equipment_id = e.id
            WHERE ap.year = $1
              AND ($2::uuid IS NULL OR ap.equipment_id = $2)
              AND ($3::calibration_type IS NULL OR ap.calibration_type = $3)
            ORDER BY e.name, ap.calibration_type
            "#,
        )
        .bind(query.year)
        .bind(query.equipment_id)
        .bind(&query.calibration_type)
        .fetch_all(pool)
        .await?;

        // 查詢 2：取該年度實際校正記錄（每組 equipment+type+month 取最早一筆）
        #[derive(sqlx::FromRow)]
        struct CalRecord {
            equipment_id: Uuid,
            calibration_type: crate::models::CalibrationType,
            cal_month: i32,
            calibration_id: Uuid,
            calibrated_at: chrono::NaiveDate,
            result: Option<String>,
        }

        let cal_records = sqlx::query_as::<_, CalRecord>(
            r#"
            SELECT DISTINCT ON (ec.equipment_id, ec.calibration_type, EXTRACT(MONTH FROM ec.calibrated_at))
                ec.equipment_id,
                ec.calibration_type,
                EXTRACT(MONTH FROM ec.calibrated_at)::int AS cal_month,
                ec.id AS calibration_id,
                ec.calibrated_at,
                ec.result
            FROM equipment_calibrations ec
            WHERE EXTRACT(YEAR FROM ec.calibrated_at) = $1
              AND ($2::uuid IS NULL OR ec.equipment_id = $2)
              AND ($3::calibration_type IS NULL OR ec.calibration_type = $3)
            ORDER BY ec.equipment_id, ec.calibration_type, EXTRACT(MONTH FROM ec.calibrated_at), ec.calibrated_at ASC
            "#,
        )
        .bind(query.year)
        .bind(query.equipment_id)
        .bind(&query.calibration_type)
        .fetch_all(pool)
        .await?;

        // 建立 HashMap 加速查找
        use std::collections::HashMap;
        let mut cal_map: HashMap<(Uuid, String, i32), CalRecord> = HashMap::new();
        for rec in cal_records {
            let type_key = format!("{:?}", rec.calibration_type).to_lowercase();
            cal_map.insert((rec.equipment_id, type_key, rec.cal_month), rec);
        }

        let today = chrono::Local::now().date_naive();

        let mut rows: Vec<AnnualPlanExecutionRow> = Vec::new();
        let mut total_planned = 0i32;
        let mut total_completed = 0i32;
        let mut total_overdue = 0i32;

        for plan in &plans {
            let type_key = format!("{:?}", plan.calibration_type).to_lowercase();
            let plan_months = [
                plan.month_1,
                plan.month_2,
                plan.month_3,
                plan.month_4,
                plan.month_5,
                plan.month_6,
                plan.month_7,
                plan.month_8,
                plan.month_9,
                plan.month_10,
                plan.month_11,
                plan.month_12,
            ];

            let mut month_details: Vec<MonthExecutionDetail> = Vec::with_capacity(12);
            let mut planned_count = 0i32;
            let mut completed_count = 0i32;
            let mut overdue_count = 0i32;

            for m in 1i32..=12 {
                let planned = plan_months[(m - 1) as usize];
                let cal = cal_map.get(&(plan.equipment_id, type_key.clone(), m));

                let status = match (planned, cal) {
                    (false, _) => MonthExecutionStatus::Unplanned,
                    (true, Some(_)) => MonthExecutionStatus::Completed,
                    (true, None) => {
                        // 月末日期：m+1月1日前一天，12月用12/31
                        let month_end = if m < 12 {
                            chrono::NaiveDate::from_ymd_opt(query.year, (m + 1) as u32, 1)
                                .and_then(|d| d.pred_opt())
                                .unwrap_or(
                                    chrono::NaiveDate::from_ymd_opt(query.year, m as u32, 28)
                                        .expect("valid fallback date m/28"),
                                )
                        } else {
                            chrono::NaiveDate::from_ymd_opt(query.year, 12, 31)
                                .expect("valid date 12/31")
                        };
                        if month_end < today {
                            MonthExecutionStatus::Overdue
                        } else {
                            MonthExecutionStatus::PlannedPending
                        }
                    }
                };

                if planned {
                    planned_count += 1;
                }
                if status == MonthExecutionStatus::Completed {
                    completed_count += 1;
                }
                if status == MonthExecutionStatus::Overdue {
                    overdue_count += 1;
                }

                month_details.push(MonthExecutionDetail {
                    month: m,
                    planned,
                    status,
                    calibration_id: cal.map(|c| c.calibration_id),
                    calibrated_at: cal.map(|c| c.calibrated_at),
                    result: cal.and_then(|c| c.result.clone()),
                });
            }

            total_planned += planned_count;
            total_completed += completed_count;
            total_overdue += overdue_count;

            rows.push(AnnualPlanExecutionRow {
                plan_id: plan.id,
                year: plan.year,
                equipment_id: plan.equipment_id,
                equipment_name: plan.equipment_name.clone(),
                equipment_serial_number: plan.equipment_serial_number.clone(),
                calibration_type: plan.calibration_type.clone(),
                cycle: plan.cycle.clone(),
                months: month_details,
                planned_count,
                completed_count,
                overdue_count,
            });
        }

        let completion_rate = if total_planned > 0 {
            total_completed as f64 / total_planned as f64
        } else {
            0.0
        };

        Ok(AnnualPlanExecutionSummary {
            year: query.year,
            total_planned,
            total_completed,
            total_overdue,
            completion_rate,
            rows,
        })
    }
}

fn pick_smart_months(
    cycle: &CalibrationCycle,
    last_month: Option<u32>,
    rng: &mut impl RngExt,
) -> [bool; 12] {
    let Some(lm) = last_month else {
        return pick_random_months(cycle, rng);
    };
    let lm = lm.clamp(1, 12) as usize - 1; // 0-indexed
    let mut months = [false; 12];
    match cycle {
        CalibrationCycle::Monthly => {
            months = [true; 12];
        }
        CalibrationCycle::Quarterly => {
            // 保持與歷史相同的季內偏移（0/1/2）
            let offset = lm % 3;
            for q in 0..4 {
                months[q * 3 + offset] = true;
            }
        }
        CalibrationCycle::SemiAnnual => {
            // 上半年取歷史月份，下半年加 6
            let first = lm % 6;
            let second = (first + 6) % 12;
            months[first] = true;
            months[second] = true;
        }
        CalibrationCycle::Annual => {
            months[lm] = true;
        }
    }
    months
}

fn pick_random_months(cycle: &CalibrationCycle, rng: &mut impl RngExt) -> [bool; 12] {
    let mut months = [false; 12];
    match cycle {
        CalibrationCycle::Monthly => {
            months = [true; 12];
        }
        CalibrationCycle::Quarterly => {
            // 每季隨機選一個月
            for quarter in 0..4 {
                let start = quarter * 3;
                let pick = start + rng.random_range(0..3);
                months[pick] = true;
            }
        }
        CalibrationCycle::SemiAnnual => {
            // 每半年隨機選一個月
            let first = rng.random_range(0..6);
            let second = 6 + rng.random_range(0..6);
            months[first] = true;
            months[second] = true;
        }
        CalibrationCycle::Annual => {
            // 一年隨機選一個月
            let pick = rng.random_range(0..12);
            months[pick] = true;
        }
    }
    months
}

async fn insert_annual_plan(
    pool: &PgPool,
    year: i32,
    equipment_id: Uuid,
    calibration_type: &crate::models::CalibrationType,
    cycle: &CalibrationCycle,
    months: &[bool; 12],
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO equipment_annual_plans
            (year, equipment_id, calibration_type, cycle,
             month_1, month_2, month_3, month_4, month_5, month_6,
             month_7, month_8, month_9, month_10, month_11, month_12)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        ON CONFLICT (year, equipment_id, calibration_type)
        DO UPDATE SET
            cycle = EXCLUDED.cycle,
            month_1 = EXCLUDED.month_1, month_2 = EXCLUDED.month_2,
            month_3 = EXCLUDED.month_3, month_4 = EXCLUDED.month_4,
            month_5 = EXCLUDED.month_5, month_6 = EXCLUDED.month_6,
            month_7 = EXCLUDED.month_7, month_8 = EXCLUDED.month_8,
            month_9 = EXCLUDED.month_9, month_10 = EXCLUDED.month_10,
            month_11 = EXCLUDED.month_11, month_12 = EXCLUDED.month_12,
            generated_at = NOW(), updated_at = NOW()
        "#,
    )
    .bind(year)
    .bind(equipment_id)
    .bind(calibration_type)
    .bind(cycle)
    .bind(months[0])
    .bind(months[1])
    .bind(months[2])
    .bind(months[3])
    .bind(months[4])
    .bind(months[5])
    .bind(months[6])
    .bind(months[7])
    .bind(months[8])
    .bind(months[9])
    .bind(months[10])
    .bind(months[11])
    .execute(pool)
    .await?;

    Ok(())
}
