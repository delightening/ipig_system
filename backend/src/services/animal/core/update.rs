use sqlx::PgPool;
use uuid::Uuid;

use super::super::utils::AnimalUtils;
use super::super::AnimalService;
use super::IacucChangeInfo;
use crate::{
    middleware::ActorContext,
    models::{audit_diff::DataDiff, Animal, AnimalStatus, UpdateAnimalRequest},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

impl AnimalService {
    /// 更新動物 — Service-driven audit
    ///
    /// 回傳 (Animal, Option<IacucChangeInfo>)；IACUC 變更時 service 內先寫一筆
    /// IACUC_CHANGE（含 before/after），再寫 ANIMAL_UPDATE（含完整 before/after
    /// diff）。同 tx：UPDATE animal + UPDATE pens.current_count + audit(es)。
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateAnimalRequest,
    ) -> Result<(Animal, Option<IacucChangeInfo>)> {
        let user = actor.require_user()?;
        let updated_by = user.id;

        // 驗證新 pen_id 對應的欄位是否可收容動物（tx 外預先驗，失敗即早退出）
        if let Some(new_pen_id) = req.pen_id {
            Self::validate_pen_for_assignment(pool, new_pen_id).await?;
        }

        let mut tx = pool.begin().await?;

        // Gemini PR #184 MED：完整 before snapshot 在 tx 內 + FOR UPDATE 鎖行
        // （與 optimistic lock 互補；FOR UPDATE 保證 diff 計算時的 before 與
        // commit 時的 animal 狀態一致）
        let before: Animal = sqlx::query_as(
            "SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("動物不存在".to_string()))?;

        let current_status = before.status;
        let existing_iacuc = before.iacuc_no.clone();

        if let Some(new_status) = req.status {
            if current_status != new_status && !current_status.can_transition_to(new_status) {
                return Err(AppError::BadRequest(format!(
                    "無法從「{}」轉換到「{}」",
                    current_status.display_name(),
                    new_status.display_name()
                )));
            }

            if new_status == AnimalStatus::InExperiment
                && req.iacuc_no.is_none()
                && existing_iacuc.is_none()
            {
                return Err(AppError::BadRequest(
                    "分配實驗需要指定 IACUC No.".to_string(),
                ));
            }

            if current_status == AnimalStatus::Completed && new_status == AnimalStatus::Transferred {
                return Err(AppError::BadRequest("動物轉讓請使用轉讓 API".to_string()));
            }
        }

        if current_status == AnimalStatus::InExperiment {
            if let Some(ref new_iacuc) = req.iacuc_no {
                if let Some(ref old) = existing_iacuc {
                    if old != new_iacuc {
                        return Err(AppError::BadRequest(
                            "實驗中的動物不可更改 IACUC No.".to_string(),
                        ));
                    }
                }
            }
        }

        let is_assigning_to_experiment = req.status == Some(AnimalStatus::InExperiment);

        let iacuc_change = if let Some(ref new_iacuc) = req.iacuc_no {
            let changed = match &existing_iacuc {
                Some(old) => old != new_iacuc,
                None => true,
            };
            if changed {
                Some(IacucChangeInfo {
                    old_iacuc_no: existing_iacuc.clone(),
                    new_iacuc_no: new_iacuc.clone(),
                })
            } else {
                None
            }
        } else {
            None
        };

        if current_status.is_terminal() {
            if let Some(ref loc) = req.pen_location {
                if !loc.trim().is_empty() {
                    return Err(AppError::BadRequest(
                        "已犧牲或猝死的動物無法移動到欄位".to_string(),
                    ));
                }
            }
        }

        let pen_location_bind = if current_status == AnimalStatus::Euthanized {
            req.pen_location
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .map(|s| AnimalUtils::format_pen_location(&s))
        } else {
            req.pen_location
                .as_ref()
                .map(|s| AnimalUtils::format_pen_location(s))
        };

        let old_pen_id = before.pen_id;

        let animal = sqlx::query_as::<_, Animal>(
            r#"
            UPDATE animals SET
                status = COALESCE($2, status),
                pen_location = CASE WHEN status = 'euthanized' THEN $3 ELSE COALESCE($3, pen_location) END,
                pen_id = COALESCE($10, pen_id),
                species_id = COALESCE($11, species_id),
                iacuc_no = COALESCE($4, iacuc_no),
                experiment_date = CASE WHEN $7 AND experiment_date IS NULL THEN CURRENT_DATE ELSE COALESCE($5, experiment_date) END,
                remark = COALESCE($6, remark),
                experiment_assigned_by = CASE WHEN $7 THEN $8 ELSE experiment_assigned_by END,
                version = version + 1,
                updated_at = NOW()
            WHERE id = $1
            AND deleted_at IS NULL
            AND ($9::INT IS NULL OR version = $9)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.status)
        .bind(&pen_location_bind)
        .bind(&req.iacuc_no)
        .bind(req.experiment_date)
        .bind(&req.remark)
        .bind(is_assigning_to_experiment)
        .bind(updated_by)
        .bind(req.version)
        .bind(req.pen_id)
        .bind(req.species_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::Conflict("此記錄已被其他人修改，請重新載入後再試。".to_string())
        })?;

        // 更新 pen current_count（同 tx；舊 pen 和新 pen）
        let new_pen_id = animal.pen_id;
        let pen_ids_to_update: Vec<Uuid> = [old_pen_id, new_pen_id]
            .iter()
            .filter_map(|p| *p)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        for pid in pen_ids_to_update {
            sqlx::query(
                "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 AND deleted_at IS NULL AND status NOT IN ('euthanized', 'sudden_death', 'transferred')) WHERE id = $1"
            )
            .bind(pid)
            .execute(&mut *tx)
            .await?;
        }

        // IACUC_CHANGE 先寫入（時間軸上可獨立查詢）
        // 完整 before/after diff 在 ANIMAL_UPDATE 事件內；此事件重點在顯示轉換軌跡
        if let Some(ref change) = iacuc_change {
            let old = change.old_iacuc_no.as_deref().unwrap_or("(無)");
            let display = format!("[{} → {}] {}", old, change.new_iacuc_no, animal.ear_tag);
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "ANIMAL",
                    event_type: "IACUC_CHANGE",
                    entity: Some(AuditEntity::new("animal", id, &display)),
                    data_diff: None,
                    request_context: None,
                },
            )
            .await?;
        }

        // 一般 ANIMAL_UPDATE（包含完整 before/after diff）
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "ANIMAL_UPDATE",
                entity: Some(AuditEntity::new("animal", id, &animal.ear_tag)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&animal))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok((animal, iacuc_change))
    }
}
