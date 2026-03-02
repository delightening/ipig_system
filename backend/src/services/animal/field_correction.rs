//! 動物欄位修正申請服務
//! 耳號、出生日期、性別、品種等欄位需經 admin 批准後才能修改

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use super::AnimalService;
use crate::{
    models::{
        AnimalBreed, AnimalGender,
        CreateAnimalFieldCorrectionRequest,
        ReviewAnimalFieldCorrectionRequest,
        AnimalFieldCorrectionRequestListItem,
        CORRECTABLE_FIELDS,
    },
    AppError, Result,
};

/// 動物欄位修正申請服務
pub struct AnimalFieldCorrectionService;

impl AnimalFieldCorrectionService {
    /// 建立修正申請（staff 可呼叫）
    pub async fn create_request(
        pool: &PgPool,
        animal_id: Uuid,
        req: &CreateAnimalFieldCorrectionRequest,
        requested_by: Uuid,
    ) -> Result<Uuid> {
        req.validate()?;

        let field = req.field_name.as_str();
        if !CORRECTABLE_FIELDS.contains(&field) {
            return Err(AppError::Validation(format!(
                "欄位 {} 不可申請修正，僅支援：{}",
                field,
                CORRECTABLE_FIELDS.join(", ")
            )));
        }

        // 取得動物現有資料
        let animal = AnimalService::get_by_id(pool, animal_id).await?;

        let old_value = match field {
            "ear_tag" => Some(animal.ear_tag.clone()),
            "birth_date" => animal.birth_date.map(|d| d.to_string()),
            "gender" => Some(format!("{:?}", animal.gender).to_lowercase()),
            "breed" => Some(Self::breed_to_db_value(&animal.breed)),
            _ => None,
        };

        // 驗證 new_value 格式
        Self::validate_new_value(field, &req.new_value)?;

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO animal_field_correction_requests
                (id, animal_id, field_name, old_value, new_value, reason, status, requested_by)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)
            "#,
        )
        .bind(id)
        .bind(animal_id)
        .bind(field)
        .bind(&old_value)
        .bind(&req.new_value)
        .bind(&req.reason)
        .bind(requested_by)
        .execute(pool)
        .await?;

        Ok(id)
    }

    fn breed_to_db_value(breed: &AnimalBreed) -> String {
        match breed {
            AnimalBreed::Minipig => "miniature".to_string(),
            AnimalBreed::White => "white".to_string(),
            AnimalBreed::LYD => "LYD".to_string(),
            AnimalBreed::Other => "other".to_string(),
        }
    }

    fn format_ear_tag(ear_tag: &str) -> String {
        if let Ok(num) = ear_tag.parse::<u32>() {
            if num < 100 {
                return format!("{:03}", num);
            }
        }
        ear_tag.to_string()
    }

    fn validate_new_value(field: &str, new_value: &str) -> Result<()> {
        match field {
            "ear_tag" => {
                let formatted = Self::format_ear_tag(new_value);
                if formatted.len() != 3 || !formatted.chars().all(|c| c.is_ascii_digit()) {
                    return Err(AppError::Validation("耳號必須為三位數".to_string()));
                }
            }
            "birth_date" => {
                NaiveDate::parse_from_str(new_value, "%Y-%m-%d")
                    .map_err(|_| AppError::Validation("出生日期格式須為 YYYY-MM-DD".to_string()))?;
            }
            "gender" => {
                if new_value != "male" && new_value != "female" {
                    return Err(AppError::Validation("性別須為 male 或 female".to_string()));
                }
            }
            "breed" => {
                let valid = ["miniature", "minipig", "white", "LYD", "lyd", "other"];
                if !valid.contains(&new_value) {
                    return Err(AppError::Validation(
                        "品種須為 minipig/miniature, white, LYD, other 之一".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 列出待審核的修正申請（admin 用）
    pub async fn list_pending(pool: &PgPool) -> Result<Vec<AnimalFieldCorrectionRequestListItem>> {
        let rows = sqlx::query_as::<_, AnimalFieldCorrectionRequestListItem>(
            r#"
            SELECT
                r.id, r.animal_id, r.field_name, r.old_value, r.new_value, r.reason, r.status,
                r.requested_by, u.display_name as requested_by_name,
                r.reviewed_by, r.reviewed_at, r.created_at,
                a.ear_tag as animal_ear_tag
            FROM animal_field_correction_requests r
            JOIN animals a ON a.id = r.animal_id AND a.deleted_at IS NULL
            LEFT JOIN users u ON u.id = r.requested_by
            WHERE r.status = 'pending'
            ORDER BY r.created_at ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// 審核修正申請（admin 用）
    pub async fn review(
        pool: &PgPool,
        request_id: Uuid,
        req: &ReviewAnimalFieldCorrectionRequest,
        reviewed_by: Uuid,
    ) -> Result<()> {
        let row: Option<(Uuid, String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT animal_id, field_name, new_value, old_value
            FROM animal_field_correction_requests
            WHERE id = $1 AND status = 'pending'
            "#,
        )
        .bind(request_id)
        .fetch_optional(pool)
        .await?;

        let (animal_id, field_name, new_value, _old_value) = row
            .ok_or_else(|| AppError::NotFound("找不到待審核的修正申請".to_string()))?;

        if req.approved {
            // 套用修正
            Self::apply_correction(pool, animal_id, &field_name, &new_value).await?;

            // 更新申請狀態
            sqlx::query(
                r#"
                UPDATE animal_field_correction_requests
                SET status = 'approved', reviewed_by = $2, reviewed_at = NOW(), updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(request_id)
            .bind(reviewed_by)
            .execute(pool)
            .await?;
        } else {
            let reject_reason = req
                .reject_reason
                .as_deref()
                .unwrap_or("未提供拒絕原因");

            sqlx::query(
                r#"
                UPDATE animal_field_correction_requests
                SET status = 'rejected', reviewed_by = $2, reviewed_at = NOW(),
                    reason = reason || E'\n[拒絕原因] ' || $3, updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(request_id)
            .bind(reviewed_by)
            .bind(reject_reason)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    async fn apply_correction(
        pool: &PgPool,
        animal_id: Uuid,
        field_name: &str,
        new_value: &str,
    ) -> Result<()> {
        match field_name {
            "ear_tag" => {
                let formatted = Self::format_ear_tag(new_value);
                sqlx::query("UPDATE animals SET ear_tag = $2, updated_at = NOW() WHERE id = $1")
                    .bind(animal_id)
                    .bind(&formatted)
                    .execute(pool)
                    .await?;
            }
            "birth_date" => {
                let d = NaiveDate::parse_from_str(new_value, "%Y-%m-%d")
                    .map_err(|_| AppError::Validation("日期格式錯誤".to_string()))?;
                sqlx::query("UPDATE animals SET birth_date = $2, updated_at = NOW() WHERE id = $1")
                    .bind(animal_id)
                    .bind(d)
                    .execute(pool)
                    .await?;
            }
            "gender" => {
                let gender: AnimalGender = match new_value {
                    "male" => AnimalGender::Male,
                    "female" => AnimalGender::Female,
                    _ => return Err(AppError::Validation("性別值無效".to_string())),
                };
                sqlx::query("UPDATE animals SET gender = $2::animal_gender, updated_at = NOW() WHERE id = $1")
                    .bind(animal_id)
                    .bind(gender)
                    .execute(pool)
                    .await?;
            }
            "breed" => {
                let breed_str = match new_value {
                    "minipig" | "miniature" => "miniature",
                    "white" => "white",
                    "lyd" | "LYD" => "LYD",
                    "other" => "other",
                    _ => return Err(AppError::Validation("品種值無效".to_string())),
                };
                sqlx::query("UPDATE animals SET breed = $2::animal_breed, updated_at = NOW() WHERE id = $1")
                    .bind(animal_id)
                    .bind(breed_str)
                    .execute(pool)
                    .await?;
            }
            _ => return Err(AppError::Validation("不支援的欄位".to_string())),
        }
        Ok(())
    }

    /// 取得某動物的修正申請列表
    pub async fn list_by_animal(
        pool: &PgPool,
        animal_id: Uuid,
    ) -> Result<Vec<AnimalFieldCorrectionRequestListItem>> {
        let rows = sqlx::query_as::<_, AnimalFieldCorrectionRequestListItem>(
            r#"
            SELECT
                r.id, r.animal_id, r.field_name, r.old_value, r.new_value, r.reason, r.status,
                r.requested_by, u.display_name as requested_by_name,
                r.reviewed_by, r.reviewed_at, r.created_at,
                a.ear_tag as animal_ear_tag
            FROM animal_field_correction_requests r
            JOIN animals a ON a.id = r.animal_id AND a.deleted_at IS NULL
            LEFT JOIN users u ON u.id = r.requested_by
            WHERE r.animal_id = $1
            ORDER BY r.created_at DESC
            "#,
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
