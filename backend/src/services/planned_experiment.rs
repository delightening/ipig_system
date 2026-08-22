use std::collections::HashMap;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        Animal, CreatePlannedExperimentRequest, PlannedExperiment, PlannedExperimentResponse,
        PlanningAnimalRow, ReservableAnimalRow, ReservableQuery, ReservationPlanningGroup,
        ReserveAnimalsRequest, UnreserveAnimalsRequest, UpdatePlannedExperimentRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

/// 預定試驗服務（動物預約與試驗規劃；執行秘書 IACUC_STAFF + admin 管理）。
pub struct PlannedExperimentService;

const EVENT_CATEGORY: &str = "PLANNED_EXPERIMENT";
const ENTITY_TYPE: &str = "planned_experiment";
const EVENT_CREATED: &str = "PLANNED_EXPERIMENT_CREATED";
const EVENT_UPDATED: &str = "PLANNED_EXPERIMENT_UPDATED";
const EVENT_DELETED: &str = "PLANNED_EXPERIMENT_DELETED";
const EVENT_RESERVED: &str = "ANIMAL_RESERVED";
const EVENT_UNRESERVED: &str = "ANIMAL_UNRESERVED";

/// 計算群組內 (已預約, 實驗中, 實驗完成) 數（Phase 5 三態；completed 亦計入缺口）。
fn relation_counts(animals: &[PlanningAnimalRow]) -> (i64, i64, i64) {
    let (mut reserved, mut in_experiment, mut completed) = (0i64, 0i64, 0i64);
    for a in animals {
        match a.relation.as_str() {
            "reserved" => reserved += 1,
            "in_experiment" => in_experiment += 1,
            "completed" => completed += 1,
            _ => {}
        }
    }
    (reserved, in_experiment, completed)
}

impl PlannedExperimentService {
    pub async fn list(pool: &PgPool) -> Result<Vec<PlannedExperimentResponse>> {
        let rows = sqlx::query_as::<_, PlannedExperimentResponse>(
            r#"
            SELECT pe.id, pe.unit, pe.description, pe.demand_count, pe.protocol_id,
                   p.iacuc_no AS protocol_iacuc_no,
                   pe.created_by, u.display_name AS created_by_name,
                   pe.created_at, pe.updated_at
            FROM planned_experiments pe
            LEFT JOIN users u ON u.id = pe.created_by
            LEFT JOIN protocols p ON p.id = pe.protocol_id
            ORDER BY pe.created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<PlannedExperiment> {
        sqlx::query_as::<_, PlannedExperiment>("SELECT * FROM planned_experiments WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("找不到預定試驗".into()))
    }

    /// 取得操作者 id（拒絕 Anonymous）。
    fn actor_id(actor: &ActorContext) -> Result<Uuid> {
        actor
            .actor_user_id()
            .ok_or_else(|| AppError::Forbidden("預定試驗須由已登入使用者管理".into()))
    }

    async fn audit_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        event_type: &'static str,
        v: &PlannedExperiment,
    ) -> Result<()> {
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: EVENT_CATEGORY,
                event_type,
                entity: Some(AuditEntity::new(ENTITY_TYPE, v.id, &v.unit)),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreatePlannedExperimentRequest,
    ) -> Result<PlannedExperiment> {
        let created_by = Self::actor_id(actor)?;
        let unit = req.unit.trim();
        if unit.is_empty() {
            return Err(AppError::Validation("委託單位不可為空白".into()));
        }
        let mut tx = pool.begin().await?;
        let v = sqlx::query_as::<_, PlannedExperiment>(
            r#"
            INSERT INTO planned_experiments
                (id, unit, description, demand_count, protocol_id, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(unit)
        .bind(req.description.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        .bind(req.demand_count)
        .bind(req.protocol_id)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;
        Self::audit_tx(&mut tx, actor, EVENT_CREATED, &v).await?;
        tx.commit().await?;
        Ok(v)
    }

    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdatePlannedExperimentRequest,
    ) -> Result<PlannedExperiment> {
        Self::actor_id(actor)?;
        // unit 若有提供，trim 後不可為空白（與 create 一致；避免 COALESCE 靜默不更新）
        let unit = req.unit.as_deref().map(str::trim);
        if let Some(u) = unit {
            if u.is_empty() {
                return Err(AppError::Validation("委託單位不可為空白".into()));
            }
        }
        let mut tx = pool.begin().await?;
        // description 為全量取代語意（前端編輯表單恆送完整物件，null = 明確清空）；
        // unit / demand_count 未提供時以 COALESCE 保留。protocol_id 不在此更新
        // （核准連結由核准流程管理，避免手動編輯靜默清空）。
        let v = sqlx::query_as::<_, PlannedExperiment>(
            r#"
            UPDATE planned_experiments SET
                unit = COALESCE($2, unit),
                description = $3,
                demand_count = COALESCE($4, demand_count),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(unit)
        .bind(
            req.description
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty()),
        )
        .bind(req.demand_count)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到預定試驗".into()))?;
        Self::audit_tx(&mut tx, actor, EVENT_UPDATED, &v).await?;
        tx.commit().await?;
        Ok(v)
    }

    /// 刪除預定試驗。已預約至此試驗的動物由 FK `ON DELETE SET NULL` 自動解除預約
    /// （migration 117：`animals.reserved_planned_experiment_id`）。
    pub async fn delete(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        Self::actor_id(actor)?;
        let mut tx = pool.begin().await?;
        let v = sqlx::query_as::<_, PlannedExperiment>(
            "DELETE FROM planned_experiments WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到預定試驗".into()))?;
        Self::audit_tx(&mut tx, actor, EVENT_DELETED, &v).await?;
        tx.commit().await?;
        Ok(())
    }

    /// 批次操作 summary audit（nil entity id + 摘要）。
    async fn audit_batch(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        event_type: &'static str,
        summary: &str,
    ) -> Result<()> {
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: EVENT_CATEGORY,
                event_type,
                entity: Some(AuditEntity::new("animal", Uuid::nil(), summary)),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    /// 批次預約動物到試驗（planned_experiment 或 protocol 二擇一；僅未分配、未預約動物）。
    /// 回傳實際被預約的動物（不符條件者靜默略過，不在 RETURNING 內）。
    pub async fn reserve(
        pool: &PgPool,
        actor: &ActorContext,
        req: &ReserveAnimalsRequest,
    ) -> Result<Vec<Animal>> {
        Self::actor_id(actor)?;
        // 目標須恰好二擇一，並驗證存在
        let target_desc = match (req.planned_experiment_id, req.protocol_id) {
            (Some(pe), None) => {
                Self::get(pool, pe).await?;
                format!("規劃試驗 {pe}")
            }
            (None, Some(p)) => {
                // protocols 無軟刪欄（僅 status 生命週期）；不加 deleted_at 過濾
                let exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM protocols WHERE id = $1)")
                        .bind(p)
                        .fetch_one(pool)
                        .await?;
                if !exists {
                    return Err(AppError::NotFound("計畫不存在".into()));
                }
                format!("計畫 {p}")
            }
            _ => {
                return Err(AppError::Validation(
                    "須指定 planned_experiment_id 或 protocol_id 其中恰好一個".into(),
                ))
            }
        };
        let mut tx = pool.begin().await?;
        let animals = if let Some(pe) = req.planned_experiment_id {
            sqlx::query_as::<_, Animal>(
                r#"UPDATE animals SET reserved_planned_experiment_id = $1, updated_at = NOW()
                   WHERE id = ANY($2::uuid[]) AND status = 'unassigned'::animal_status
                     AND deleted_at IS NULL
                     AND reserved_protocol_id IS NULL AND reserved_planned_experiment_id IS NULL
                   RETURNING *"#,
            )
            .bind(pe)
            .bind(&req.animal_ids)
            .fetch_all(&mut *tx)
            .await?
        } else if let Some(p) = req.protocol_id {
            sqlx::query_as::<_, Animal>(
                r#"UPDATE animals SET reserved_protocol_id = $1, updated_at = NOW()
                   WHERE id = ANY($2::uuid[]) AND status = 'unassigned'::animal_status
                     AND deleted_at IS NULL
                     AND reserved_protocol_id IS NULL AND reserved_planned_experiment_id IS NULL
                   RETURNING *"#,
            )
            .bind(p)
            .bind(&req.animal_ids)
            .fetch_all(&mut *tx)
            .await?
        } else {
            return Err(AppError::Validation("無預約目標".into()));
        };
        let summary = format!("批次預約 {} 隻至{}", animals.len(), target_desc);
        Self::audit_batch(&mut tx, actor, EVENT_RESERVED, &summary).await?;
        tx.commit().await?;
        Ok(animals)
    }

    /// 批次解除預約（清空兩個 reserved 欄）。回傳實際解除的動物。
    pub async fn unreserve(
        pool: &PgPool,
        actor: &ActorContext,
        req: &UnreserveAnimalsRequest,
    ) -> Result<Vec<Animal>> {
        Self::actor_id(actor)?;
        let mut tx = pool.begin().await?;
        let animals = sqlx::query_as::<_, Animal>(
            r#"UPDATE animals
               SET reserved_protocol_id = NULL, reserved_planned_experiment_id = NULL, updated_at = NOW()
               WHERE id = ANY($1::uuid[])
                 AND (reserved_protocol_id IS NOT NULL OR reserved_planned_experiment_id IS NOT NULL)
               RETURNING *"#,
        )
        .bind(&req.animal_ids)
        .fetch_all(&mut *tx)
        .await?;
        let summary = format!("批次解除預約 {} 隻", animals.len());
        Self::audit_batch(&mut tx, actor, EVENT_UNRESERVED, &summary).await?;
        tx.commit().await?;
        Ok(animals)
    }

    /// 搜尋可預約動物（未分配、未預約）依性別/月齡/最新體重篩選（None 不過濾）。
    pub async fn search_reservable(
        pool: &PgPool,
        query: &ReservableQuery,
    ) -> Result<Vec<ReservableAnimalRow>> {
        let rows = sqlx::query_as::<_, ReservableAnimalRow>(
            r#"
            WITH latest_weight AS (
                SELECT DISTINCT ON (animal_id) animal_id, weight, measure_date
                FROM animal_weights
                WHERE deleted_at IS NULL
                ORDER BY animal_id, measure_date DESC
            )
            SELECT a.id, a.ear_tag, a.animal_no, a.gender, a.birth_date,
                   CASE WHEN a.birth_date IS NULL THEN NULL
                        ELSE (EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                              + EXTRACT(MONTH FROM AGE(a.birth_date))::int) END AS age_months,
                   a.pen_location,
                   lw.weight AS latest_weight_kg,
                   lw.measure_date AS weight_measured_at,
                   a.remark
            FROM animals a
            LEFT JOIN latest_weight lw ON lw.animal_id = a.id
            WHERE a.deleted_at IS NULL
              AND a.status = 'unassigned'::animal_status
              AND a.reserved_protocol_id IS NULL
              AND a.reserved_planned_experiment_id IS NULL
              AND ($1::animal_gender IS NULL OR a.gender = $1)
              AND ($2::int IS NULL OR (a.birth_date IS NOT NULL
                    AND (EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                         + EXTRACT(MONTH FROM AGE(a.birth_date))::int) >= $2))
              AND ($3::int IS NULL OR (a.birth_date IS NOT NULL
                    AND (EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                         + EXTRACT(MONTH FROM AGE(a.birth_date))::int) <= $3))
              AND ($4::numeric IS NULL OR (lw.weight IS NOT NULL AND lw.weight >= $4))
              AND ($5::numeric IS NULL OR (lw.weight IS NOT NULL AND lw.weight <= $5))
            ORDER BY a.ear_tag
            "#,
        )
        .bind(query.gender)
        .bind(query.age_months_min)
        .bind(query.age_months_max)
        .bind(query.weight_min)
        .bind(query.weight_max)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 規劃分組查詢（Phase 3）：回「試驗群組」清單——已核准 protocol（有預約/分配動物者）
    /// + 所有 planned_experiment，各帶 需求 / 已預約 / 已分配 + 該組動物列。
    pub async fn get_reservation_planning(pool: &PgPool) -> Result<Vec<ReservationPlanningGroup>> {
        // 1) 一次撈出所有相關動物（預約 or 已分配至已核准計畫），帶 group_key + relation
        let animals = sqlx::query_as::<_, PlanningAnimalRow>(
            r#"
            WITH latest_weight AS (
                SELECT DISTINCT ON (animal_id) animal_id, weight, measure_date
                FROM animal_weights WHERE deleted_at IS NULL
                ORDER BY animal_id, measure_date DESC
            )
            -- (1) 已預約至規劃中 planned_experiment（一律顯示）
            SELECT 'planned:' || a.reserved_planned_experiment_id::text AS group_key,
                   a.id, a.ear_tag, a.animal_no, a.gender, a.birth_date,
                   lw.weight AS latest_weight_kg, lw.measure_date AS weight_measured_at,
                   a.remark, a.status::text AS status, 'reserved' AS relation
            FROM animals a LEFT JOIN latest_weight lw ON lw.animal_id = a.id
            WHERE a.reserved_planned_experiment_id IS NOT NULL AND a.deleted_at IS NULL
            UNION ALL
            -- (2) 已預約至「已核准」protocol（預約至非核准 protocol 者落 orphan）
            SELECT 'protocol:' || a.reserved_protocol_id::text,
                   a.id, a.ear_tag, a.animal_no, a.gender, a.birth_date,
                   lw.weight, lw.measure_date, a.remark, a.status::text, 'reserved'
            FROM animals a LEFT JOIN latest_weight lw ON lw.animal_id = a.id
            WHERE a.reserved_protocol_id IS NOT NULL AND a.deleted_at IS NULL
              AND EXISTS (SELECT 1 FROM protocols p WHERE p.id = a.reserved_protocol_id
                          AND p.status IN ('APPROVED', 'APPROVED_WITH_CONDITIONS'))
            UNION ALL
            -- (3) 實驗中，掛在已核准 protocol
            SELECT 'protocol:' || p.id::text,
                   a.id, a.ear_tag, a.animal_no, a.gender, a.birth_date,
                   lw.weight, lw.measure_date, a.remark, a.status::text, 'in_experiment'
            FROM animals a
            JOIN protocols p ON p.iacuc_no = a.iacuc_no
                AND p.status IN ('APPROVED', 'APPROVED_WITH_CONDITIONS')
            LEFT JOIN latest_weight lw ON lw.animal_id = a.id
            WHERE a.status = 'in_experiment'::animal_status AND a.deleted_at IS NULL
            UNION ALL
            -- (4) 實驗完成（待淘汰），掛在已核准 protocol；仍計入缺口
            SELECT 'protocol:' || p.id::text,
                   a.id, a.ear_tag, a.animal_no, a.gender, a.birth_date,
                   lw.weight, lw.measure_date, a.remark, a.status::text, 'completed'
            FROM animals a
            JOIN protocols p ON p.iacuc_no = a.iacuc_no
                AND p.status IN ('APPROVED', 'APPROVED_WITH_CONDITIONS')
            LEFT JOIN latest_weight lw ON lw.animal_id = a.id
            WHERE a.status = 'completed'::animal_status AND a.deleted_at IS NULL
            UNION ALL
            -- (5) orphan catch-all：存活但掛在非顯示計畫（已結案/暫停/非核准）下的動物，
            --     確保「全場活豬都顯示」。排除備用池（未分配未預約，由 /animals/reservable 供）。
            SELECT 'orphan',
                   a.id, a.ear_tag, a.animal_no, a.gender, a.birth_date,
                   lw.weight, lw.measure_date, a.remark, a.status::text,
                   CASE a.status
                       WHEN 'in_experiment'::animal_status THEN 'in_experiment'
                       WHEN 'completed'::animal_status THEN 'completed'
                       ELSE 'reserved' END
            FROM animals a LEFT JOIN latest_weight lw ON lw.animal_id = a.id
            WHERE a.deleted_at IS NULL
              AND a.status IN ('unassigned'::animal_status, 'in_experiment'::animal_status,
                               'completed'::animal_status)
              -- 非備用池（備用池 = 未分配且未預約）
              AND NOT (a.status = 'unassigned'::animal_status
                       AND a.reserved_protocol_id IS NULL
                       AND a.reserved_planned_experiment_id IS NULL)
              -- 未預約至 planned_experiment（那些一律顯示）
              AND a.reserved_planned_experiment_id IS NULL
              -- 未預約至已核准 protocol
              AND NOT EXISTS (SELECT 1 FROM protocols p WHERE p.id = a.reserved_protocol_id
                              AND p.status IN ('APPROVED', 'APPROVED_WITH_CONDITIONS'))
              -- in_experiment / completed 未掛在已核准 protocol
              AND NOT (a.status IN ('in_experiment'::animal_status, 'completed'::animal_status)
                       AND EXISTS (SELECT 1 FROM protocols p WHERE p.iacuc_no = a.iacuc_no
                                   AND p.status IN ('APPROVED', 'APPROVED_WITH_CONDITIONS')))
            ORDER BY 1, 3
            "#,
        )
        .fetch_all(pool)
        .await?;

        let mut by_group: HashMap<String, Vec<PlanningAnimalRow>> = HashMap::new();
        for a in animals {
            by_group.entry(a.group_key.clone()).or_default().push(a);
        }

        let mut groups: Vec<ReservationPlanningGroup> = Vec::new();

        // 2) 規劃中 planned_experiment 群組（全部）
        let planned = sqlx::query_as::<_, (Uuid, String, Option<String>, i32, Option<String>)>(
            r#"SELECT pe.id, pe.unit, pe.description, pe.demand_count, p.iacuc_no
               FROM planned_experiments pe
               LEFT JOIN protocols p ON p.id = pe.protocol_id
               ORDER BY pe.created_at DESC"#,
        )
        .fetch_all(pool)
        .await?;
        for (id, unit, description, demand_count, iacuc_no) in planned {
            let animals = by_group
                .remove(&format!("planned:{id}"))
                .unwrap_or_default();
            let (reserved_count, in_experiment_count, completed_count) = relation_counts(&animals);
            groups.push(ReservationPlanningGroup {
                group_type: "planned".to_string(),
                id,
                iacuc_no,
                unit: Some(unit),
                pi_name: None,
                description,
                demand: demand_count as i64,
                reserved_count,
                in_experiment_count,
                completed_count,
                animals,
            });
        }

        // 3) 已核准 protocol 群組（Phase 5：列出全部 APPROVED / APPROVED_WITH_CONDITIONS，
        //    含尚無動物的空計劃以露出缺口；Closed / Suspended / 未核准由狀態過濾自動排除）。
        //    demand 取 animals[].number 前導整數加總。
        let protocols = sqlx::query_as::<_, (Uuid, Option<String>, Option<String>, Option<String>, i64)>(
            r#"SELECT p.id, p.iacuc_no,
                      NULLIF(p.working_content->'basic'->'sponsor'->>'name', '') AS unit,
                      NULLIF(p.working_content->'basic'->'pi'->>'name', '') AS pi_name,
                      CASE WHEN jsonb_typeof(p.working_content->'animals'->'animals') = 'array' THEN
                          COALESCE((SELECT SUM(NULLIF(substring(e->>'number' FROM '^\d+'), '')::int)
                                    FROM jsonb_array_elements(p.working_content->'animals'->'animals') e), 0)::bigint
                          ELSE 0::bigint END AS demand
               FROM protocols p
               WHERE p.status IN ('APPROVED', 'APPROVED_WITH_CONDITIONS') AND p.iacuc_no IS NOT NULL
               ORDER BY p.iacuc_no"#,
        )
        .fetch_all(pool)
        .await?;
        for (id, iacuc_no, unit, pi_name, demand) in protocols {
            let animals = by_group
                .remove(&format!("protocol:{id}"))
                .unwrap_or_default();
            let (reserved_count, in_experiment_count, completed_count) = relation_counts(&animals);
            groups.push(ReservationPlanningGroup {
                group_type: "approved".to_string(),
                id,
                iacuc_no,
                unit,
                pi_name,
                description: None,
                demand,
                reserved_count,
                in_experiment_count,
                completed_count,
                animals,
            });
        }

        // 4) orphan catch-all：掛在非顯示計畫（已結案/暫停/非核准）下的存活動物，置底。
        //    確保「全場活豬都顯示」——這些豬不屬於任何顯示中的計畫組、也非備用池。
        //    demand 0；空時不輸出（前端亦有才顯示）。
        let orphans = by_group.remove("orphan").unwrap_or_default();
        if !orphans.is_empty() {
            let (reserved_count, in_experiment_count, completed_count) = relation_counts(&orphans);
            groups.push(ReservationPlanningGroup {
                group_type: "orphan".to_string(),
                id: Uuid::nil(),
                iacuc_no: None,
                unit: Some("計畫已結案 / 暫停・待處理".to_string()),
                pi_name: None,
                description: Some("掛在非現行計畫下的存活動物（計畫已結案或暫停）".to_string()),
                demand: 0,
                reserved_count,
                in_experiment_count,
                completed_count,
                animals: orphans,
            });
        }

        Ok(groups)
    }
}
