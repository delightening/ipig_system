// 照護紀錄（疼痛評估）Service
// 評估欄位依據 TU-03-05-03B 試驗豬隻疼痛評估紀錄表

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::Result;

/// 照護給藥紀錄模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "care_record_mode", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CareRecordMode {
    Legacy,
    PainAssessment,
}

/// 獸醫紀錄類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "vet_record_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CareVetRecordType {
    Observation,
    Surgery,
}

/// 照護紀錄（DB 結構）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CareRecord {
    pub id: Uuid,
    pub record_type: CareVetRecordType,
    pub record_id: Uuid,
    pub record_mode: CareRecordMode,
    pub post_op_days: Option<i32>,
    pub time_period: Option<String>,
    /// 傷口狀況 (0–3)
    pub incision: Option<i16>,
    /// 態度/行為 (0–5)
    pub attitude_behavior: Option<i16>,
    /// 食慾 (0–2)
    pub appetite: Option<i16>,
    /// 排便 (0–3)
    pub feces: Option<i16>,
    /// 排尿 (0–3)
    pub urine: Option<i16>,
    /// 疼痛分數 (1–4)
    pub pain_score: Option<i16>,
    /// 術後給藥：注射 Ketorolac（保留向後相容）
    pub injection_ketorolac: bool,
    /// 術後給藥：注射 Meloxicam（保留向後相容）
    pub injection_meloxicam: bool,
    /// 術後給藥：口服 Meloxicam（保留向後相容）
    pub oral_meloxicam: bool,
    /// 術後給藥（自由格式，JSONB 陣列）
    #[sqlx(default)]
    pub post_medications: Option<serde_json::Value>,
    pub vet_read: bool,
    pub created_at: DateTime<Utc>,
}

// R26-9: 異種器官移植研究 + GLP 情境下，照護紀錄數值（疼痛分數、給藥情形）
// 是研究資料本身，必須完整保留於 audit log（21 CFR Part 11 §11.10 稽核軌跡）。
// 空 `redacted_fields()` 是**主動決策**而非遺漏；未來若判斷某欄位屬員工
// PII（例如自由註記含他人姓名），於此覆寫即可。
impl crate::models::audit_diff::AuditRedact for CareRecord {}

/// 建立照護紀錄請求
#[derive(Debug, Deserialize)]
pub struct CreateCareRecordRequest {
    pub record_type: CareVetRecordType,
    pub record_id: Uuid,
    #[serde(default = "default_pain_assessment")]
    pub record_mode: CareRecordMode,
    pub post_op_days: Option<i32>,
    pub time_period: Option<String>,
    pub incision: Option<i16>,
    pub attitude_behavior: Option<i16>,
    pub appetite: Option<i16>,
    pub feces: Option<i16>,
    pub urine: Option<i16>,
    pub pain_score: Option<i16>,
    #[serde(default)]
    pub injection_ketorolac: bool,
    #[serde(default)]
    pub injection_meloxicam: bool,
    #[serde(default)]
    pub oral_meloxicam: bool,
    pub post_medications: Option<serde_json::Value>,
}

fn default_pain_assessment() -> CareRecordMode {
    CareRecordMode::PainAssessment
}

/// 更新照護紀錄請求
#[derive(Debug, Deserialize)]
pub struct UpdateCareRecordRequest {
    pub post_op_days: Option<i32>,
    pub time_period: Option<String>,
    pub incision: Option<i16>,
    pub attitude_behavior: Option<i16>,
    pub appetite: Option<i16>,
    pub feces: Option<i16>,
    pub urine: Option<i16>,
    pub pain_score: Option<i16>,
    #[serde(default)]
    pub injection_ketorolac: bool,
    #[serde(default)]
    pub injection_meloxicam: bool,
    #[serde(default)]
    pub oral_meloxicam: bool,
    pub post_medications: Option<serde_json::Value>,
}

pub struct CareRecordService;

impl CareRecordService {
    /// 列出某動物的照護紀錄（排除已刪除）
    pub async fn list_by_animal(pool: &PgPool, animal_id: Uuid) -> Result<Vec<CareRecord>> {
        let records = sqlx::query_as::<_, CareRecord>(
            r#"
            SELECT c.id, c.record_type, c.record_id, c.record_mode, c.post_op_days,
                   c.time_period, c.incision, c.attitude_behavior, c.appetite,
                   c.feces, c.urine, c.pain_score,
                   c.injection_ketorolac, c.injection_meloxicam, c.oral_meloxicam,
                   c.post_medications, c.vet_read, c.created_at
            FROM care_medication_records c
            WHERE c.deleted_at IS NULL
              AND (
                (c.record_type = 'observation' AND c.record_id IN (
                    SELECT id FROM animal_observations WHERE animal_id = $1 AND deleted_at IS NULL
                ))
                OR
                (c.record_type = 'surgery' AND c.record_id IN (
                    SELECT id FROM animal_surgeries WHERE animal_id = $1 AND deleted_at IS NULL
                ))
            )
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// 列出指定紀錄的照護紀錄（按 record_type + record_id）
    pub async fn list_by_record(
        pool: &PgPool,
        record_type: CareVetRecordType,
        record_id: Uuid,
    ) -> Result<Vec<CareRecord>> {
        let records = sqlx::query_as::<_, CareRecord>(
            r#"
            SELECT c.id, c.record_type, c.record_id, c.record_mode, c.post_op_days,
                   c.time_period, c.incision, c.attitude_behavior, c.appetite,
                   c.feces, c.urine, c.pain_score,
                   c.injection_ketorolac, c.injection_meloxicam, c.oral_meloxicam,
                   c.post_medications, c.vet_read, c.created_at
            FROM care_medication_records c
            WHERE c.deleted_at IS NULL
              AND c.record_type = $1
              AND c.record_id = $2
            ORDER BY c.post_op_days ASC NULLS LAST, c.time_period ASC, c.created_at ASC
            "#,
        )
        .bind(record_type)
        .bind(record_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// 建立照護紀錄
    pub async fn create(pool: &PgPool, req: &CreateCareRecordRequest) -> Result<CareRecord> {
        let record = sqlx::query_as::<_, CareRecord>(
            r#"
            INSERT INTO care_medication_records
                (record_type, record_id, record_mode, post_op_days, time_period,
                 incision, attitude_behavior, appetite, feces, urine, pain_score,
                 injection_ketorolac, injection_meloxicam, oral_meloxicam, post_medications)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, record_type, record_id, record_mode, post_op_days, time_period,
                      incision, attitude_behavior, appetite, feces, urine, pain_score,
                      injection_ketorolac, injection_meloxicam, oral_meloxicam,
                      post_medications, vet_read, created_at
            "#,
        )
        .bind(req.record_type)
        .bind(req.record_id)
        .bind(req.record_mode)
        .bind(req.post_op_days)
        .bind(&req.time_period)
        .bind(req.incision)
        .bind(req.attitude_behavior)
        .bind(req.appetite)
        .bind(req.feces)
        .bind(req.urine)
        .bind(req.pain_score)
        .bind(req.injection_ketorolac)
        .bind(req.injection_meloxicam)
        .bind(req.oral_meloxicam)
        .bind(&req.post_medications)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 更新照護紀錄
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateCareRecordRequest,
    ) -> Result<CareRecord> {
        let record = sqlx::query_as::<_, CareRecord>(
            r#"
            UPDATE care_medication_records
            SET post_op_days        = $2,
                time_period         = $3,
                incision            = $4,
                attitude_behavior   = $5,
                appetite            = $6,
                feces               = $7,
                urine               = $8,
                pain_score          = $9,
                injection_ketorolac = $10,
                injection_meloxicam = $11,
                oral_meloxicam      = $12,
                post_medications    = $13
            WHERE id = $1
            RETURNING id, record_type, record_id, record_mode, post_op_days, time_period,
                      incision, attitude_behavior, appetite, feces, urine, pain_score,
                      injection_ketorolac, injection_meloxicam, oral_meloxicam,
                      post_medications, vet_read, created_at
            "#,
        )
        .bind(id)
        .bind(req.post_op_days)
        .bind(&req.time_period)
        .bind(req.incision)
        .bind(req.attitude_behavior)
        .bind(req.appetite)
        .bind(req.feces)
        .bind(req.urine)
        .bind(req.pain_score)
        .bind(req.injection_ketorolac)
        .bind(req.injection_meloxicam)
        .bind(req.oral_meloxicam)
        .bind(&req.post_medications)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 軟刪除照護紀錄（含刪除原因）- GLP 合規
    pub async fn soft_delete_with_reason(
        pool: &PgPool,
        id: Uuid,
        reason: &str,
        deleted_by: Uuid,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE care_medication_records SET
                deleted_at = NOW(),
                deletion_reason = $2,
                deleted_by = $3
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::AppError::NotFound("照護紀錄不存在或已刪除".into()));
        }

        Ok(())
    }
}
