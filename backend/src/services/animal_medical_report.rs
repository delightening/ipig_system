// R53-7/8/10b: 豬隻病歷彙整報表 Service
//
// 統一 timeline：animal_observations + animal_surgeries + animal_blood_tests
// 三表分次查詢合併。轉移不納入（非實驗操作、不計費）。
// Filter 為 AND 邏輯：耳號 ∩ 計畫案 ∩ 時間區間。
//
// **不含 byproduct_sample**：對齊 R53-6 audit blacklist。

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::Result;

/// 週報 filter — AND 邏輯。
#[derive(Debug, Deserialize)]
pub struct WeeklyMedicalReportFilter {
    /// 耳號清單（OR within filter，AND 與其他 filter 之間）。None / 空 vec = 不過濾。
    pub animal_ear_tags: Option<Vec<String>>,
    /// 計畫案 UUID 清單。None / 空 vec = 不過濾。
    pub protocol_ids: Option<Vec<Uuid>>,
    /// event_date 起始（含）。None = 不過濾。
    pub start_date: Option<NaiveDate>,
    /// event_date 終止（含）。None = 不過濾。
    pub end_date: Option<NaiveDate>,
}

/// 統一 timeline event 結構 — 跨多表 UNION 後對齊。
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MedicalTimelineEvent {
    /// 動物 id
    pub animal_id: Uuid,
    /// 動物耳號（顯示用）
    pub ear_tag: String,
    /// 計畫案 IACUC no.（如有）— 顯示用，方便週報依計畫案分組
    pub iacuc_no: Option<String>,
    /// 事件日期
    pub event_date: NaiveDate,
    /// 事件類別 — `OBSERVATION` / 未來 `SURGERY` / `VET_PATROL` / ...
    pub event_type: String,
    /// 一句話摘要（如 record_type 中文化字串）
    pub summary: String,
    /// 細節（content 全文，可為空）
    pub details: Option<String>,
    /// 執行人員顯示名稱（如有）
    pub actor_name: Option<String>,
    /// 來源 row id — debug / 跳轉用
    pub source_id: Uuid,
    /// 該 row 的 created_at — 同日多筆穩定排序
    pub created_at: DateTime<Utc>,
    pub birth_date: Option<NaiveDate>,
    pub latest_weight: Option<rust_decimal::Decimal>,
    /// 委託單位名稱（protocol working_content.basic.sponsor.name）— 試驗單位欄顯示用
    pub sponsor_name: Option<String>,
    pub equipment_used: Option<String>,
    pub anesthesia_start: Option<DateTime<Utc>>,
    pub anesthesia_end: Option<DateTime<Utc>>,
}

pub struct AnimalMedicalReportService;

impl AnimalMedicalReportService {
    /// 豬隻週報 — 多表 UNION timeline（observations + surgeries + blood_tests + transfers）。
    /// `accessible_protocol_ids`：資料邊界。`None` = 不限（view_all 角色）；
    /// `Some(&[])` = 不關聯任何計畫 → 回空；`Some(ids)` = 僅限該集合內計畫之豬隻。
    /// 與呼叫端傳入的 `filter.protocol_ids` 為 AND 關係（雙重收斂）。
    pub async fn weekly_report(
        pool: &PgPool,
        filter: &WeeklyMedicalReportFilter,
        accessible_protocol_ids: Option<&[Uuid]>,
    ) -> Result<Vec<MedicalTimelineEvent>> {
        let ear_tags: Option<Vec<String>> = filter
            .animal_ear_tags
            .as_ref()
            .filter(|v| !v.is_empty())
            .cloned();
        let protocol_ids: Option<Vec<Uuid>> = filter
            .protocol_ids
            .as_ref()
            .filter(|v| !v.is_empty())
            .cloned();

        // 分 4 次查詢再合併，避免 UNION ALL + ARRAY() 子查詢造成 PostgreSQL
        // stack depth limit exceeded（$1::text[] 在 4 個 UNION 分支重複引用
        // 加上 CASE/ARRAY 子查詢，optimizer 規劃時遞迴層太深）。
        let et = ear_tags.as_deref();
        let pi = protocol_ids.as_deref();
        let sd = filter.start_date;
        let ed = filter.end_date;
        // $5：使用者可存取計畫邊界。None → NULL → 不限；Some 空集合 → 空陣列 → 無 row。
        let acc = accessible_protocol_ids;
        // 邊界為空集合（使用者不關聯任何計畫）→ 結果必然為空，省去 3 次無意義 DB 查詢。
        if acc.is_some_and(<[Uuid]>::is_empty) {
            return Ok(Vec::new());
        }

        let mut rows: Vec<MedicalTimelineEvent> = Vec::new();

        // 1) Observations
        let obs = sqlx::query_as::<_, MedicalTimelineEvent>(
            r#"
            WITH latest_weights AS (
                SELECT DISTINCT ON (animal_id) animal_id, weight
                FROM animal_weights WHERE deleted_at IS NULL
                ORDER BY animal_id, measure_date DESC
            )
            SELECT
                a.id AS animal_id, a.ear_tag, a.iacuc_no,
                obs.event_date,
                'OBSERVATION'::text AS event_type,
                COALESCE(obs.record_type::varchar, '觀察') AS summary,
                obs.content AS details,
                u.display_name AS actor_name,
                obs.id AS source_id,
                obs.created_at,
                a.birth_date,
                lw.weight AS latest_weight,
                pr.working_content->'basic'->'sponsor'->>'name' AS sponsor_name,
                CASE
                    WHEN obs.equipment_used IS NOT NULL
                         AND obs.equipment_used != 'null'::jsonb
                         AND jsonb_typeof(obs.equipment_used) = 'array'
                    THEN array_to_string(ARRAY(
                        SELECT jsonb_array_elements_text(obs.equipment_used)
                    ), '、')
                END AS equipment_used,
                obs.anesthesia_start,
                obs.anesthesia_end
            FROM animal_observations obs
            JOIN animals a ON obs.animal_id = a.id AND a.deleted_at IS NULL
            LEFT JOIN users u ON obs.created_by = u.id
            LEFT JOIN protocols pr ON a.iacuc_no IS NOT NULL AND a.iacuc_no = pr.iacuc_no
            LEFT JOIN latest_weights lw ON lw.animal_id = a.id
            WHERE obs.deleted_at IS NULL
              AND ($1::text[] IS NULL OR a.ear_tag = ANY($1::text[]))
              AND ($2::uuid[] IS NULL OR pr.id = ANY($2::uuid[]))
              AND ($3::date IS NULL OR obs.event_date >= $3)
              AND ($4::date IS NULL OR obs.event_date <= $4)
              AND ($5::uuid[] IS NULL OR pr.id = ANY($5::uuid[]))
            "#,
        )
        .bind(et)
        .bind(pi)
        .bind(sd)
        .bind(ed)
        .bind(acc)
        .fetch_all(pool)
        .await?;
        rows.extend(obs);

        // 2) Surgeries
        let surg = sqlx::query_as::<_, MedicalTimelineEvent>(
            r#"
            WITH latest_weights AS (
                SELECT DISTINCT ON (animal_id) animal_id, weight
                FROM animal_weights WHERE deleted_at IS NULL
                ORDER BY animal_id, measure_date DESC
            )
            SELECT
                a.id AS animal_id, a.ear_tag, a.iacuc_no,
                s.surgery_date AS event_date,
                'SURGERY'::text AS event_type,
                COALESCE(s.surgery_site, '手術') AS summary,
                s.remark AS details,
                u.display_name AS actor_name,
                s.id AS source_id,
                s.created_at,
                a.birth_date,
                lw.weight AS latest_weight,
                pr.working_content->'basic'->'sponsor'->>'name' AS sponsor_name,
                NULL::text AS equipment_used,
                NULL::timestamptz AS anesthesia_start,
                NULL::timestamptz AS anesthesia_end
            FROM animal_surgeries s
            JOIN animals a ON s.animal_id = a.id AND a.deleted_at IS NULL
            LEFT JOIN users u ON s.created_by = u.id
            LEFT JOIN protocols pr ON a.iacuc_no IS NOT NULL AND a.iacuc_no = pr.iacuc_no
            LEFT JOIN latest_weights lw ON lw.animal_id = a.id
            WHERE s.deleted_at IS NULL
              AND ($1::text[] IS NULL OR a.ear_tag = ANY($1::text[]))
              AND ($2::uuid[] IS NULL OR pr.id = ANY($2::uuid[]))
              AND ($3::date IS NULL OR s.surgery_date >= $3)
              AND ($4::date IS NULL OR s.surgery_date <= $4)
              AND ($5::uuid[] IS NULL OR pr.id = ANY($5::uuid[]))
            "#,
        )
        .bind(et)
        .bind(pi)
        .bind(sd)
        .bind(ed)
        .bind(acc)
        .fetch_all(pool)
        .await?;
        rows.extend(surg);

        // 3) Blood tests
        let bt = sqlx::query_as::<_, MedicalTimelineEvent>(
            r#"
            WITH latest_weights AS (
                SELECT DISTINCT ON (animal_id) animal_id, weight
                FROM animal_weights WHERE deleted_at IS NULL
                ORDER BY animal_id, measure_date DESC
            )
            SELECT
                a.id AS animal_id, a.ear_tag, a.iacuc_no,
                bt.test_date AS event_date,
                'BLOOD_TEST'::text AS event_type,
                COALESCE(bt.lab_name, '血檢') AS summary,
                bt.remark AS details,
                u.display_name AS actor_name,
                bt.id AS source_id,
                bt.created_at,
                a.birth_date,
                lw.weight AS latest_weight,
                pr.working_content->'basic'->'sponsor'->>'name' AS sponsor_name,
                NULL::text AS equipment_used,
                NULL::timestamptz AS anesthesia_start,
                NULL::timestamptz AS anesthesia_end
            FROM animal_blood_tests bt
            JOIN animals a ON bt.animal_id = a.id AND a.deleted_at IS NULL
            LEFT JOIN users u ON bt.created_by = u.id
            LEFT JOIN protocols pr ON a.iacuc_no IS NOT NULL AND a.iacuc_no = pr.iacuc_no
            LEFT JOIN latest_weights lw ON lw.animal_id = a.id
            WHERE bt.deleted_at IS NULL
              AND ($1::text[] IS NULL OR a.ear_tag = ANY($1::text[]))
              AND ($2::uuid[] IS NULL OR pr.id = ANY($2::uuid[]))
              AND ($3::date IS NULL OR bt.test_date >= $3)
              AND ($4::date IS NULL OR bt.test_date <= $4)
              AND ($5::uuid[] IS NULL OR pr.id = ANY($5::uuid[]))
            "#,
        )
        .bind(et)
        .bind(pi)
        .bind(sd)
        .bind(ed)
        .bind(acc)
        .fetch_all(pool)
        .await?;
        rows.extend(bt);

        rows.sort_by(|a, b| {
            a.event_date
                .cmp(&b.event_date)
                .then(a.created_at.cmp(&b.created_at))
        });
        // #447：升序排序後若超過上限，保留「最新」5000 筆（移除最舊的），而非 truncate 砍掉
        // 最新筆。輸出維持升序，僅丟最舊的尾段。
        const MAX_ROWS: usize = 5000;
        if rows.len() > MAX_ROWS {
            let drop = rows.len() - MAX_ROWS;
            rows.drain(0..drop);
        }

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_serde_roundtrip_basic() {
        let f = WeeklyMedicalReportFilter {
            animal_ear_tags: Some(vec!["A001".into()]),
            protocol_ids: None,
            start_date: NaiveDate::from_ymd_opt(2026, 5, 10),
            end_date: NaiveDate::from_ymd_opt(2026, 5, 17),
        };
        let json = serde_json::to_string(&serde_json::json!({
            "animal_ear_tags": f.animal_ear_tags,
            "protocol_ids": f.protocol_ids,
            "start_date": f.start_date,
            "end_date": f.end_date,
        }))
        .expect("serialize");
        let back: WeeklyMedicalReportFilter = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            back.animal_ear_tags.as_deref(),
            Some(&["A001".to_string()][..])
        );
        assert_eq!(back.start_date, NaiveDate::from_ymd_opt(2026, 5, 10));
    }
}
