use sqlx::PgPool;
use uuid::Uuid;

use super::super::species_link::SpeciesLink;
use super::super::utils::AnimalUtils;
use super::super::AnimalService;
use super::IacucChangeInfo;
use crate::{
    middleware::ActorContext,
    models::{audit_diff::DataDiff, Animal, AnimalBreed, AnimalStatus, UpdateAnimalRequest},
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
        let before: Animal =
            sqlx::query_as("SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE")
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

            if current_status == AnimalStatus::Completed && new_status == AnimalStatus::Transferred
            {
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

        // 本次更新後的有效狀態（SQL COALESCE($2, status) 的 Rust 對應）。
        // #999 follow-up High 修復：舊 SQL `CASE WHEN status = 'euthanized'` 讀到的是
        // **更新前**的舊值，「正要轉入終態」的那次更新反而落入 ELSE 保留舊 pen_location，
        // 殘留資料持續生成（migration 135 只做過一次歷史清理）。改以有效（新）狀態在
        // Rust 端算出離場旗標（$12，Rust 為單一事實來源，SQL 不寫死狀態字串）：
        // 離場（安樂死/猝死/轉讓）一律強制清空 pen_location，與 medical/euthanasia
        // 專用流程的清除行為一致。
        let effective_status = req.status.unwrap_or(current_status);
        let leaving_facility = !effective_status.is_active_in_facility();

        if leaving_facility {
            if let Some(ref loc) = req.pen_location {
                if !loc.trim().is_empty() {
                    return Err(AppError::BadRequest(
                        "已犧牲、猝死或轉讓的動物無法移動到欄位".to_string(),
                    ));
                }
            }
            // pen_id 同樣拒絕指派（CodeRabbit #1006 Major）：SQL 走 COALESCE($10, pen_id)，
            // 放行會讓離場動物被指到新欄位。**既有** pen_id 刻意保留不清——與
            // medical/euthanasia 專用流程一致（留「死亡時所在欄」FK 供追溯），
            // 欄位計數皆以 status 過濾、不受殘留 pen_id 影響。
            if req.pen_id.is_some() {
                return Err(AppError::BadRequest(
                    "已犧牲、猝死或轉讓的動物無法指派欄位".to_string(),
                ));
            }
        }

        let pen_location_bind = if leaving_facility {
            // 離場終態：SQL 端無條件清空（$3 不會被讀取），Rust 端綁 None 保持語意一致。
            None
        } else {
            req.pen_location
                .as_ref()
                .map(|s| AnimalUtils::format_pen_location(s))
        };

        let old_pen_id = before.pen_id;

        // R86-5：species_id 是動物種類的真相源，不可讓 COALESCE 直接寫入未經檢查的值
        // （不存在／已停用／非葉節點）。驗證比照建立流程 SpeciesLink::resolve_from_species，
        // 並同步重推 breed——否則會產生 species_id 與 breed 矛盾的列。
        //
        // 只在「真的要換」時才驗證（Qodo review 於 #125 指出）：resolve_from_species 會
        // 拒絕已停用的物種，但合法存在的舊資料可能就是停用物種（例如 LYD 透過
        // resolve_from_breed 的相容路徑建立，刻意不做 is_active 檢查）。若呼叫端只是把
        // 動物目前的 species_id 原樣回填（常見於「整份表單送出」型的編輯流程，即使
        // 沒改這格也會帶原值），不應該讓這次更新因為物種本身停用而 400——那不是使用者
        // 想改的東西。
        let new_breed: Option<AnimalBreed> = match req.species_id {
            Some(species_id) if Some(species_id) != before.species_id => {
                let (_, breed) = SpeciesLink::resolve_from_species(&mut tx, species_id).await?;
                Some(breed)
            }
            _ => None,
        };

        let animal = sqlx::query_as::<_, Animal>(
            r#"
            UPDATE animals SET
                status = COALESCE($2, status),
                pen_location = CASE WHEN $12 THEN NULL ELSE COALESCE($3, pen_location) END,
                pen_id = COALESCE($10, pen_id),
                species_id = COALESCE($11, species_id),
                breed = COALESCE($13, breed),
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
        .bind(leaving_facility)
        .bind(new_breed)
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
