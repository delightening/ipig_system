use sqlx::PgPool;
use uuid::Uuid;

use super::super::utils::AnimalUtils;
use super::super::AnimalService;
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, Animal, AnimalStatus, BatchAssignRequest, CreateAnimalRequest,
        CreateWeightRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

impl AnimalService {
    /// 建立動物 — Service-driven audit
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateAnimalRequest,
    ) -> Result<Animal> {
        let user = actor.require_user()?;
        let created_by = user.id;

        let formatted_ear_tag = if let Ok(num) = req.ear_tag.parse::<u32>() {
            format!("{:03}", num)
        } else {
            req.ear_tag.clone()
        };

        if !formatted_ear_tag.chars().all(|c| c.is_ascii_digit()) || formatted_ear_tag.len() != 3 {
            return Err(AppError::Validation("耳號必須為三位數".to_string()));
        }

        // 檢查耳號是否已存在（僅查未刪除且存活的動物，排除終態：euthanized/sudden_death）
        let existing_animals: Vec<(Uuid, Option<chrono::NaiveDate>, String, Option<String>)> =
            sqlx::query_as(
                r#"
            SELECT id, birth_date, status::text, pen_location
            FROM animals
            WHERE ear_tag = $1 AND deleted_at IS NULL
            AND status NOT IN ('euthanized', 'sudden_death')
            "#,
            )
            .bind(&formatted_ear_tag)
            .fetch_all(pool)
            .await?;

        if !existing_animals.is_empty() {
            let same_birthday = existing_animals
                .iter()
                .any(|(_, bd, _, _)| *bd == req.birth_date);
            if same_birthday {
                return Err(AppError::Conflict(format!(
                    "耳號 {} 已存在同出生日期的存活動物，無法重複建立",
                    formatted_ear_tag
                )));
            }

            if !req.force_create {
                let animals_info: Vec<serde_json::Value> = existing_animals
                    .iter()
                    .map(|(id, bd, status, pen)| {
                        serde_json::json!({
                            "id": id,
                            "birth_date": bd.map(|d| d.to_string()),
                            "status": status,
                            "pen_location": pen,
                        })
                    })
                    .collect();
                return Err(AppError::DuplicateWarning {
                    message: format!(
                        "耳號 {} 已存在其他存活動物，請確認是否繼續建立",
                        formatted_ear_tag
                    ),
                    existing_animals: animals_info,
                });
            }
        }

        // 驗證 pen_id 對應的欄位是否可收容動物
        if let Some(pen_id) = req.pen_id {
            Self::validate_pen_for_assignment(pool, pen_id).await?;
        }

        let pen_location = match &req.pen_location {
            Some(s) if !s.trim().is_empty() => Some(AnimalUtils::format_pen_location(s)),
            _ => return Err(AppError::Validation("欄位為必填".to_string())),
        };

        let breed_str = AnimalUtils::breed_to_db_value(&req.breed);

        let mut tx = pool.begin().await?;

        let animal = sqlx::query_as::<_, Animal>(
            r#"
            INSERT INTO animals (
                ear_tag, status, breed, breed_other, source_id, gender, birth_date,
                entry_date, entry_weight, pen_location, pen_id, species_id,
                pre_experiment_code, remark, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3::animal_breed, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(&formatted_ear_tag)
        .bind(AnimalStatus::Unassigned)
        .bind(breed_str)
        .bind(&req.breed_other)
        .bind(req.source_id)
        .bind(req.gender)
        .bind(req.birth_date)
        .bind(req.entry_date)
        .bind(req.entry_weight)
        .bind(&pen_location)
        .bind(req.pen_id)
        .bind(req.species_id)
        .bind(&req.pre_experiment_code)
        .bind(&req.remark)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if let Some(constraint) = db_err.constraint() {
                    if constraint == "check_ear_tag_three_digits" {
                        return AppError::Validation("耳號必須為三位數".to_string());
                    }
                }
                let msg = db_err.message();
                if msg.contains("check_ear_tag_three_digits") || msg.contains("three digits") {
                    return AppError::Validation("耳號必須為三位數".to_string());
                }
            }
            AppError::Database(e)
        })?;

        // 更新 pen 的 current_count（同 tx，CRIT-02 修補）
        if let Some(pid) = animal.pen_id {
            sqlx::query(
                "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 AND deleted_at IS NULL AND status NOT IN ('euthanized', 'sudden_death', 'transferred')) WHERE id = $1"
            )
            .bind(pid)
            .execute(&mut *tx)
            .await?;
        }

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "ANIMAL_CREATE",
                entity: Some(AuditEntity::new("animal", animal.id, &animal.ear_tag)),
                data_diff: Some(DataDiff::create_only(&animal)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        let weight_req = CreateWeightRequest {
            measure_date: req.entry_date,
            weight: req.entry_weight,
        };
        // 初始體重：使用 System actor（reason 標示為 animal_create_initial_weight），
        // service 內部會用 SYSTEM_USER_ID 作 created_by。
        let actor = crate::middleware::ActorContext::System {
            reason: "animal_create_initial_weight",
        };
        if let Err(e) =
            super::super::weight::AnimalWeightService::create(pool, &actor, animal.id, &weight_req)
                .await
        {
            tracing::warn!("建立初始體重紀錄失敗: {e}");
        }

        Ok(animal)
    }

    /// 批次分配動物至計劃 — Service-driven audit (N+1：per-row + summary, D-12)
    ///
    /// 分配後直接進入實驗中狀態（跳過已分配狀態）。同 tx：
    /// - N 筆 ANIMAL_ASSIGN per-row audit（便於稽核單一動物分配軌跡）
    /// - 1 筆 ANIMAL_BATCH_ASSIGN summary audit
    pub async fn batch_assign(
        pool: &PgPool,
        actor: &ActorContext,
        req: &BatchAssignRequest,
    ) -> Result<Vec<Animal>> {
        let user = actor.require_user()?;
        let assigned_by = user.id;

        let mut tx = pool.begin().await?;
        let mut updated_animals: Vec<Animal> = Vec::new();

        for animal_id in &req.animal_ids {
            // before snapshot（尚未 update 前）
            let before = sqlx::query_as::<_, Animal>(
                "SELECT * FROM animals WHERE id = $1 AND status = $2",
            )
            .bind(animal_id)
            .bind(AnimalStatus::Unassigned)
            .fetch_optional(&mut *tx)
            .await?;

            let Some(before) = before else { continue };

            let animal = sqlx::query_as::<_, Animal>(
                r#"
                UPDATE animals SET
                    iacuc_no = $2,
                    status = $3,
                    experiment_date = CURRENT_DATE,
                    experiment_assigned_by = $5,
                    updated_at = NOW()
                WHERE id = $1 AND status = $4
                RETURNING *
                "#,
            )
            .bind(animal_id)
            .bind(&req.iacuc_no)
            .bind(AnimalStatus::InExperiment)
            .bind(AnimalStatus::Unassigned)
            .bind(assigned_by)
            .fetch_optional(&mut *tx)
            .await?;

            if let Some(a) = animal {
                let display = format!("[{}] {}", req.iacuc_no, a.ear_tag);
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry {
                        event_category: "ANIMAL",
                        event_type: "ANIMAL_ASSIGN",
                        entity: Some(AuditEntity::new("animal", a.id, &display)),
                        data_diff: Some(DataDiff::compute(Some(&before), Some(&a))),
                        request_context: None,
                    },
                )
                .await?;
                updated_animals.push(a);
            }
        }

        // summary audit
        let summary = format!(
            "批次分配 {} 隻至 {}",
            updated_animals.len(),
            &req.iacuc_no
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "ANIMAL_BATCH_ASSIGN",
                entity: Some(AuditEntity::new("animal", Uuid::nil(), &summary)),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(updated_animals)
    }
}
