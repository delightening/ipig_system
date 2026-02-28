use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    models::{
        Animal, AnimalListItem, AnimalQuery, AnimalStatus, AnimalsByPen, BatchAssignRequest,
        CreateAnimalRequest, CreateWeightRequest, UpdateAnimalRequest,
    },
    AppError, Result,
};

/// IACUC No. 變更資訊（供 handler 記錄審計日誌用）
pub struct IacucChangeInfo {
    pub old_iacuc_no: Option<String>,
    pub new_iacuc_no: String,
}

impl AnimalService {
    // ============================================
    // 動物管理
    // ============================================
    /// 取得動物列表
    pub async fn list(pool: &PgPool, query: &AnimalQuery) -> Result<Vec<AnimalListItem>> {
        // Build query with proper parameterized queries
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                p.id, p.animal_no, p.ear_tag, p.status, p.breed, p.breed_other, p.gender, p.pen_location,
                p.iacuc_no, p.entry_date, s.name as source_name,
                p.vet_last_viewed_at, p.created_at,
                -- Computed fields for frontend
                EXISTS(
                    SELECT 1 FROM animal_observations po 
                    WHERE po.animal_id = p.id 
                    AND po.record_type = 'abnormal'::record_type
                ) as has_abnormal_record,
                -- 檢查是否正在用藥：
                -- 只要觀察試驗紀錄或手術紀錄中有任何一筆 no_medication_needed = false，則為正在用藥
                (
                    EXISTS(
                        SELECT 1 FROM animal_observations po 
                        WHERE po.animal_id = p.id 
                        AND po.no_medication_needed = false
                    )
                    OR
                    EXISTS(
                        SELECT 1 FROM animal_surgeries ps 
                        WHERE ps.animal_id = p.id 
                        AND ps.no_medication_needed = false
                    )
                ) as is_on_medication,
                (
                    SELECT MAX(vr.created_at) 
                    FROM vet_recommendations vr
                    WHERE (vr.record_type = 'observation'::vet_record_type AND vr.record_id IN (
                        SELECT id FROM animal_observations WHERE animal_id = p.id
                    ))
                    OR (vr.record_type = 'surgery'::vet_record_type AND vr.record_id IN (
                        SELECT id FROM animal_surgeries WHERE animal_id = p.id
                    ))
                ) as vet_recommendation_date,
                -- 最新體重
                (
                    SELECT pw.weight 
                    FROM animal_weights pw 
                    WHERE pw.animal_id = p.id 
                    ORDER BY pw.measure_date DESC 
                    LIMIT 1
                ) as latest_weight,
                (
                    SELECT pw.measure_date 
                    FROM animal_weights pw 
                    WHERE pw.animal_id = p.id 
                    ORDER BY pw.measure_date DESC 
                    LIMIT 1
                ) as latest_weight_date
            FROM animals p
            LEFT JOIN animal_sources s ON p.source_id = s.id
            WHERE p.deleted_at IS NULL
            "#,
        );

        // Add filters with proper parameterization
        if let Some(status) = &query.status {
            query_builder.push(" AND p.status = ");
            query_builder.push_bind(status);
        }
        if let Some(breed) = &query.breed {
            // 轉換 breed enum 為資料庫期望的字串值
            let breed_str = match breed {
                crate::models::AnimalBreed::Minipig => "miniature",
                crate::models::AnimalBreed::White => "white",
                crate::models::AnimalBreed::LYD => "LYD",
                crate::models::AnimalBreed::Other => "other",
            };
            query_builder.push(" AND p.breed = ");
            query_builder.push_bind(breed_str);
        }
        if let Some(iacuc_no) = &query.iacuc_no {
            query_builder.push(" AND p.iacuc_no = ");
            query_builder.push_bind(iacuc_no);
        }
        if let Some(keyword) = &query.keyword {
            let keyword_pattern = format!("%{}%", keyword);
            query_builder.push(" AND (p.ear_tag ILIKE ");
            query_builder.push_bind(keyword_pattern.clone());
            query_builder.push(" OR p.pen_location ILIKE ");
            query_builder.push_bind(keyword_pattern);
            query_builder.push(")");
        }

        // 過濾正在用藥的動物
        if let Some(true) = query.is_on_medication {
            query_builder.push(
                r#"
                AND (
                    EXISTS(
                        SELECT 1 FROM animal_observations po 
                        WHERE po.animal_id = p.id 
                        AND po.no_medication_needed = false
                    )
                    OR
                    EXISTS(
                        SELECT 1 FROM animal_surgeries ps 
                        WHERE ps.animal_id = p.id 
                        AND ps.no_medication_needed = false
                    )
                )
            "#,
            );
        }

        query_builder.push(" ORDER BY p.id DESC");

        let mut animals = query_builder
            .build_query_as::<AnimalListItem>()
            .fetch_all(pool)
            .await?;

        // 格式化所有耳號與欄位編號
        for animal in &mut animals {
            animal.ear_tag = Self::format_ear_tag(&animal.ear_tag);
            if let Some(pen) = &animal.pen_location {
                animal.pen_location = Some(Self::format_pen_location(pen));
            }
        }

        Ok(animals)
    }

    /// 取得使用者關聯計畫的 IACUC 編號清單
    /// 透過 user_protocols 表查詢使用者作為 PI/CO_EDITOR/CLIENT 的計畫
    pub async fn get_user_iacuc_nos(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>> {
        let iacuc_nos: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT p.iacuc_no
            FROM protocols p
            INNER JOIN user_protocols up ON up.protocol_id = p.id
            WHERE up.user_id = $1 AND p.iacuc_no IS NOT NULL
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(iacuc_nos)
    }

    /// 依欄位分組取得動物
    pub async fn list_by_pen(pool: &PgPool) -> Result<Vec<AnimalsByPen>> {
        let mut animals = sqlx::query_as::<_, AnimalListItem>(
            r#"
            SELECT 
                p.id, p.animal_no, p.ear_tag, p.status, p.breed, p.breed_other, p.gender, p.pen_location,
                p.iacuc_no, p.entry_date, s.name as source_name,
                p.vet_last_viewed_at, p.created_at
            FROM animals p
            LEFT JOIN animal_sources s ON p.source_id = s.id
            WHERE p.pen_location IS NOT NULL
            AND p.deleted_at IS NULL
            ORDER BY p.pen_location, p.id
            "#
        )
        .fetch_all(pool)
        .await?;

        // 格式化所有耳號與欄位編號
        for animal in &mut animals {
            animal.ear_tag = Self::format_ear_tag(&animal.ear_tag);
            if let Some(pen) = &animal.pen_location {
                animal.pen_location = Some(Self::format_pen_location(pen));
            }
        }

        // 依欄位分組
        let mut grouped: std::collections::HashMap<String, Vec<AnimalListItem>> =
            std::collections::HashMap::new();
        for animal in animals {
            if let Some(pen) = &animal.pen_location {
                grouped.entry(pen.clone()).or_default().push(animal);
            }
        }

        let result: Vec<AnimalsByPen> = grouped
            .into_iter()
            .map(|(pen, animals)| AnimalsByPen {
                pen_location: pen,
                animals,
            })
            .collect();

        Ok(result)
    }

    /// 取得單一動物
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Animal> {
        let mut animal = sqlx::query_as::<_, Animal>(
            r#"
            SELECT p.*,
                (SELECT u.display_name FROM users u WHERE u.id = p.experiment_assigned_by) AS experiment_assigned_by_name
            FROM animals p
            WHERE p.id = $1 AND p.deleted_at IS NULL
            "#
        )
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Animal not found".to_string()))?;

        // 格式化耳號與欄位編號
        animal.ear_tag = Self::format_ear_tag(&animal.ear_tag);
        if let Some(pen) = &animal.pen_location {
            animal.pen_location = Some(Self::format_pen_location(pen));
        }

        Ok(animal)
    }

    /// 建立動物
    pub async fn create(
        pool: &PgPool,
        req: &CreateAnimalRequest,
        created_by: Uuid,
    ) -> Result<Animal> {
        // 格式化耳號：如果是數字則補零至三位數
        let formatted_ear_tag = if let Ok(num) = req.ear_tag.parse::<u32>() {
            format!("{:03}", num)
        } else {
            req.ear_tag.clone()
        };

        // 驗證耳號必須為三位數
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
            // 檢查是否有同出生日期的 → 完全阻擋
            let same_birthday = existing_animals
                .iter()
                .any(|(_, bd, _, _)| *bd == req.birth_date);
            if same_birthday {
                return Err(AppError::Conflict(format!(
                    "耳號 {} 已存在同出生日期的存活動物，無法重複建立",
                    formatted_ear_tag
                )));
            }

            // 不同出生日期，且未 force_create → 回傳警告
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

        // 驗證欄位必須填寫並格式化
        let pen_location = match &req.pen_location {
            Some(s) if !s.trim().is_empty() => Some(Self::format_pen_location(s)),
            _ => return Err(AppError::Validation("欄位為必填".to_string())),
        };

        // 將 breed enum 轉換為資料庫期望的字串值
        let breed_str = match req.breed {
            crate::models::AnimalBreed::Minipig => "miniature",
            crate::models::AnimalBreed::White => "white",
            crate::models::AnimalBreed::LYD => "LYD",
            crate::models::AnimalBreed::Other => "other",
        };

        let animal = sqlx::query_as::<_, Animal>(
            r#"
            INSERT INTO animals (
                ear_tag, status, breed, breed_other, source_id, gender, birth_date,
                entry_date, entry_weight, pen_location, pre_experiment_code,
                remark, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3::animal_breed, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())
            RETURNING *
            "#
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
        .bind(&req.pre_experiment_code)
        .bind(&req.remark)
        .bind(created_by)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            // 檢查是否為資料庫約束違規錯誤
            if let sqlx::Error::Database(db_err) = &e {
                if let Some(constraint) = db_err.constraint() {
                    if constraint == "check_ear_tag_three_digits" {
                        return AppError::Validation("耳號必須為三位數".to_string());
                    }
                }
                // 檢查錯誤訊息是否包含約束相關資訊
                let msg = db_err.message();
                if msg.contains("check_ear_tag_three_digits") || msg.contains("three digits") {
                    return AppError::Validation("耳號必須為三位數".to_string());
                }
            }
            AppError::Database(e)
        })?;

        // 自動建立第一筆體重紀錄
        let weight_req = CreateWeightRequest {
            measure_date: req.entry_date,
            weight: req.entry_weight,
        };
        // 忽略錯誤，避免影響動物建立
        if let Err(e) = Self::create_weight(pool, animal.id, &weight_req, created_by).await {
            tracing::warn!("建立初始體重紀錄失敗: {e}");
        }

        Ok(animal)
    }

    /// 更新動物
    /// 回傳 (Animal, Option<IacucChangeInfo>)，第二個元素不為 None 時表示 IACUC No. 有變更
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateAnimalRequest,
        updated_by: Uuid,
    ) -> Result<(Animal, Option<IacucChangeInfo>)> {
        // 查詢現有狀態與 IACUC No.（合併查詢以減少 I/O）
        let current: (AnimalStatus, Option<String>) = sqlx::query_as(
            "SELECT status as \"status: AnimalStatus\", iacuc_no FROM animals WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("動物不存在".to_string()))?;

        let current_status = current.0;
        let existing_iacuc = current.1;

        // 狀態轉換驗證
        if let Some(new_status) = req.status {
            if current_status != new_status && !current_status.can_transition_to(new_status) {
                return Err(AppError::BadRequest(format!(
                    "無法從「{}」轉換到「{}」",
                    current_status.display_name(),
                    new_status.display_name()
                )));
            }

            // 轉到 InExperiment 時必須有 iacuc_no
            if new_status == AnimalStatus::InExperiment
                && req.iacuc_no.is_none()
                && existing_iacuc.is_none()
            {
                return Err(AppError::BadRequest(
                    "分配實驗需要指定 IACUC No.".to_string(),
                ));
            }

            // 轉讓（completed → transferred）需透過 transfer API
            if current_status == AnimalStatus::Completed && new_status == AnimalStatus::Transferred
            {
                return Err(AppError::BadRequest("動物轉讓請使用轉讓 API".to_string()));
            }
        }

        // 實驗中的動物不可更改 IACUC No.
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

        // 以下欄位於建立後不可更改，不會在更新時修改：
        // - ear_tag (耳號)
        // - breed (品種)
        // - gender (性別)
        // - source_id (來源)
        // - birth_date (出生日期)
        // - entry_date (進場日期)
        // - entry_weight (進場體重)
        // - pre_experiment_code (實驗前代號)

        // 當狀態設為 InExperiment 時，自動記錄分配者與分配日期
        let is_assigning_to_experiment = req.status == Some(AnimalStatus::InExperiment);

        // 偵測 IACUC No. 變更
        let iacuc_change = if let Some(ref new_iacuc) = req.iacuc_no {
            let changed = match &existing_iacuc {
                Some(old) => old != new_iacuc,
                None => true, // 從無到有也算變更
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

        let animal = sqlx::query_as::<_, Animal>(
            r#"
            UPDATE animals SET
                status = COALESCE($2, status),
                pen_location = COALESCE($3, pen_location),
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
            "#
        )
        .bind(id)
        .bind(req.status)
        .bind(req.pen_location.as_ref().map(|s| Self::format_pen_location(s)))
        .bind(&req.iacuc_no)
        .bind(req.experiment_date)
        .bind(&req.remark)
        .bind(is_assigning_to_experiment)
        .bind(updated_by)
        .bind(req.version)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Conflict(
            "此記錄已被其他人修改，請重新載入後再試。".to_string()
        ))?;

        Ok((animal, iacuc_change))
    }

    /// 軟刪除動物
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE animals SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 軟刪除動物（含刪除原因）- GLP 合規
    pub async fn delete_with_reason(
        pool: &PgPool,
        id: Uuid,
        reason: &str,
        deleted_by: Uuid,
    ) -> Result<()> {
        // 記錄到 change_reasons 表
        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('animal', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        // 執行軟刪除
        sqlx::query(
            r#"
            UPDATE animals SET 
                deleted_at = NOW(), 
                deletion_reason = $2,
                deleted_by = $3,
                updated_at = NOW() 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 批次分配動物至計劃
    /// 分配後直接進入實驗中狀態（跳過已分配狀態）
    pub async fn batch_assign(
        pool: &PgPool,
        req: &BatchAssignRequest,
        assigned_by: Uuid,
    ) -> Result<Vec<Animal>> {
        let mut updated_animals = Vec::new();

        for animal_id in &req.animal_ids {
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
            .fetch_optional(pool)
            .await?;

            if let Some(a) = animal {
                updated_animals.push(a);
            }
        }

        Ok(updated_animals)
    }
}
