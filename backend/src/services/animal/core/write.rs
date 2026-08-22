use sqlx::PgPool;
use uuid::Uuid;

use super::super::species_link::SpeciesLink;
use super::super::utils::AnimalUtils;
use super::super::AnimalService;
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, Animal, AnimalBreed, AnimalStatus, BatchAssignRequest,
        CreateAnimalRequest, CreateWeightRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

/// 建立動物時，前置驗證與推導完成後的值。
///
/// 收成 struct 而非攤平成一長串 INSERT 參數（參數數量門檻 ≤5）。
struct ResolvedAnimalCreate<'a> {
    req: &'a CreateAnimalRequest,
    ear_tag: String,
    pen_location: Option<String>,
    species_id: Uuid,
    breed: AnimalBreed,
    created_by: Uuid,
}

impl AnimalService {
    /// 更新動物備註（Phase 5，規劃頁 inline 編輯）— Service-driven audit。
    /// last-write-wins（備註低競爭、不做樂觀鎖）；FOR UPDATE 取 before → UPDATE remark →
    /// `log_activity_tx` + before/after DataDiff（event `ANIMAL_REMARK_UPDATE`）。
    /// 拒絕 Anonymous 觸發（對齊 R28-4；System / User 皆可，供 scheduler / 測試）。
    /// 空白備註正規化為 NULL（與清單「—」顯示一致）。
    pub async fn update_remark(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        remark: Option<String>,
    ) -> Result<Animal> {
        if matches!(actor, ActorContext::Anonymous) {
            return Err(AppError::Forbidden("備註編輯不可由匿名觸發".to_string()));
        }
        let normalized: Option<&str> = remark.as_deref().map(str::trim).filter(|s| !s.is_empty());

        let mut tx = pool.begin().await?;

        // 完整 before snapshot（FOR UPDATE 鎖行，確保 diff 與 commit 狀態一致）
        let before: Animal =
            sqlx::query_as("SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("動物不存在".to_string()))?;

        let after: Animal = sqlx::query_as(
            "UPDATE animals SET remark = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(normalized)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("備註更新 [{}]", after.ear_tag);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "ANIMAL_REMARK_UPDATE",
                entity: Some(AuditEntity::new("animal", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// 建立動物 — Service-driven audit
    ///
    /// 本函數只負責流程編排，各步驟拆在下方私有 helper。
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateAnimalRequest,
    ) -> Result<Animal> {
        let user = actor.require_user()?;
        let ear_tag = Self::normalize_ear_tag(&req.ear_tag)?;
        Self::guard_duplicate_ear_tag(pool, &ear_tag, req).await?;

        // 驗證 pen_id 對應的欄位是否可收容動物
        if let Some(pen_id) = req.pen_id {
            Self::validate_pen_for_assignment(pool, pen_id).await?;
        }
        let pen_location = Self::require_pen_location(req)?;

        let mut tx = pool.begin().await?;

        // species_id 是動物種類的真相源，breed 由它推導；只帶 breed 的舊呼叫端則反查補上
        // species_id，讓新建的動物一律兩欄齊備（規則見 SpeciesLink）。
        // 在交易內解析並鎖住物種列，避免驗證通過後、INSERT 前該物種被停用。
        let (species_id, breed) = SpeciesLink::resolve(&mut tx, req.species_id, req.breed).await?;

        let animal = Self::insert_animal(
            &mut tx,
            &ResolvedAnimalCreate {
                req,
                ear_tag,
                pen_location,
                species_id,
                breed,
                created_by: user.id,
            },
        )
        .await?;

        Self::refresh_pen_count(&mut tx, animal.pen_id).await?;
        Self::log_animal_create(&mut tx, actor, &animal).await?;

        tx.commit().await?;

        Self::record_initial_weight(pool, &animal, req).await;

        Ok(animal)
    }

    /// 耳號正規化：純數字補零成三位，其餘原樣送驗；最終一律必須是三位數字。
    fn normalize_ear_tag(raw: &str) -> Result<String> {
        let formatted = if let Ok(num) = raw.parse::<u32>() {
            format!("{:03}", num)
        } else {
            raw.to_string()
        };
        if !formatted.chars().all(|c| c.is_ascii_digit()) || formatted.len() != 3 {
            return Err(AppError::Validation("耳號必須為三位數".to_string()));
        }
        Ok(formatted)
    }

    /// 耳號重複守衛。
    ///
    /// 同耳號**且**同出生日期一律擋（`force_create` 也跳不過）；僅耳號相同時回可確認的
    /// 警告，由呼叫端帶 `force_create` 再送一次。查詢僅含未刪除且未進終態的動物。
    async fn guard_duplicate_ear_tag(
        pool: &PgPool,
        ear_tag: &str,
        req: &CreateAnimalRequest,
    ) -> Result<()> {
        let existing: Vec<(Uuid, Option<chrono::NaiveDate>, String, Option<String>)> =
            sqlx::query_as(
                r#"
            SELECT id, birth_date, status::text, pen_location
            FROM animals
            WHERE ear_tag = $1 AND deleted_at IS NULL
            AND status NOT IN ('euthanized', 'sudden_death')
            "#,
            )
            .bind(ear_tag)
            .fetch_all(pool)
            .await?;

        if existing.is_empty() {
            return Ok(());
        }

        if existing.iter().any(|(_, bd, _, _)| *bd == req.birth_date) {
            return Err(AppError::Conflict(format!(
                "耳號 {} 已存在同出生日期的存活動物，無法重複建立",
                ear_tag
            )));
        }

        if req.force_create {
            return Ok(());
        }

        let animals_info: Vec<serde_json::Value> = existing
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
        Err(AppError::DuplicateWarning {
            message: format!("耳號 {} 已存在其他存活動物，請確認是否繼續建立", ear_tag),
            existing_animals: animals_info,
        })
    }

    /// 欄位為必填，並統一格式。
    fn require_pen_location(req: &CreateAnimalRequest) -> Result<Option<String>> {
        match &req.pen_location {
            Some(s) if !s.trim().is_empty() => Ok(Some(AnimalUtils::format_pen_location(s))),
            _ => Err(AppError::Validation("欄位為必填".to_string())),
        }
    }

    /// 寫入 animals，並把 DB 的三位數 check constraint 轉成可讀的驗證錯誤。
    async fn insert_animal(
        conn: &mut sqlx::PgConnection,
        resolved: &ResolvedAnimalCreate<'_>,
    ) -> Result<Animal> {
        let req = resolved.req;
        sqlx::query_as::<_, Animal>(
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
        .bind(&resolved.ear_tag)
        .bind(AnimalStatus::Unassigned)
        .bind(AnimalUtils::breed_to_db_value(&resolved.breed))
        .bind(&req.breed_other)
        .bind(req.source_id)
        .bind(req.gender)
        .bind(req.birth_date)
        .bind(req.entry_date)
        .bind(req.entry_weight)
        .bind(&resolved.pen_location)
        .bind(req.pen_id)
        .bind(resolved.species_id)
        .bind(&req.pre_experiment_code)
        .bind(&req.remark)
        .bind(resolved.created_by)
        .fetch_one(conn)
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
        })
    }

    /// ANIMAL_CREATE 稽核事件（同 tx，與動物寫入同生共死）。
    async fn log_animal_create(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        animal: &Animal,
    ) -> Result<()> {
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "ANIMAL_CREATE",
                entity: Some(AuditEntity::new("animal", animal.id, &animal.ear_tag)),
                data_diff: Some(DataDiff::create_only(animal)),
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    /// 重算欄位在欄頭數（同 tx，CRIT-02 修補）。
    async fn refresh_pen_count(conn: &mut sqlx::PgConnection, pen_id: Option<Uuid>) -> Result<()> {
        let Some(pid) = pen_id else {
            return Ok(());
        };
        sqlx::query(
            "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 AND deleted_at IS NULL AND status NOT IN ('euthanized', 'sudden_death', 'transferred')) WHERE id = $1"
        )
        .bind(pid)
        .execute(conn)
        .await?;
        Ok(())
    }

    /// 建立初始體重紀錄。
    ///
    /// 刻意排在動物交易 commit 之後、且失敗只記 warning：體重紀錄是附屬資料，
    /// 不該因為它失敗而讓已經建好的動物整筆回滾。
    async fn record_initial_weight(pool: &PgPool, animal: &Animal, req: &CreateAnimalRequest) {
        let weight_req = CreateWeightRequest {
            measure_date: req.entry_date,
            weight: req.entry_weight,
            // 初始體重：新建動物必為場內存活，無需強制檢查
            enforce_active: false,
        };
        // 使用 System actor（reason 標示為 animal_create_initial_weight），
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
            // Gemini PR #184 MED：加 FOR UPDATE 鎖行，避免快照 → UPDATE 之間
            // 動物被其他並發 tx 修改，DataDiff 計算失準
            let before = sqlx::query_as::<_, Animal>(
                "SELECT * FROM animals WHERE id = $1 AND status = $2 FOR UPDATE",
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
                    -- 正式分配進實驗時清空預約（動物預約規劃 Phase 2）
                    reserved_protocol_id = NULL,
                    reserved_planned_experiment_id = NULL,
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
        let summary = format!("批次分配 {} 隻至 {}", updated_animals.len(), req.iacuc_no);
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
