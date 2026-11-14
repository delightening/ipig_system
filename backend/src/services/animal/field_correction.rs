//! 動物欄位修正申請服務
//! 耳號、出生日期、進場日期、性別、品種等欄位需經 admin 批准後才能修改

use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use validator::Validate;

use super::AnimalService;
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, Animal, AnimalBreed, AnimalFieldCorrectionRequestListItem,
        AnimalGender, CreateAnimalFieldCorrectionRequest, ReviewAnimalFieldCorrectionRequest,
        CORRECTABLE_FIELDS,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
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
            "entry_date" => Some(animal.entry_date.to_string()),
            "gender" => Some(format!("{:?}", animal.gender).to_lowercase()),
            "breed" => Some(Self::breed_to_db_value(&animal.breed)),
            _ => None,
        };

        // 驗證 new_value 格式
        Self::validate_new_value(field, &req.new_value)?;

        // 進場日期另需對照動物現況（出生日期、已有紀錄的最早日期）
        if field == "entry_date" {
            let new_date = Self::parse_entry_date(&req.new_value)?;
            let mut conn = pool.acquire().await?;
            let earliest = Self::earliest_record_date(&mut conn, animal_id).await?;
            super::utils::AnimalUtils::ensure_entry_date_plausible(
                new_date,
                crate::time::today_taiwan_naive(),
                animal.birth_date,
                earliest,
            )?;
        }

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
                let parsed = NaiveDate::parse_from_str(new_value, "%Y-%m-%d")
                    .map_err(|_| AppError::Validation("出生日期格式須為 YYYY-MM-DD".to_string()))?;
                super::utils::AnimalUtils::ensure_birth_date_not_future(
                    parsed,
                    crate::time::today_taiwan_naive(),
                )?;
            }
            "entry_date" => {
                Self::parse_entry_date(new_value)?;
            }
            "gender" if new_value != "male" && new_value != "female" => {
                return Err(AppError::Validation("性別須為 male 或 female".to_string()));
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

    /// 審核修正申請（核准/拒絕）。
    ///
    /// R71-1：收歸單一交易（悲觀鎖）以補齊三軸：
    /// - `FOR UPDATE` 鎖申請列並重驗 pending（防兩個並發 approve 重複套用）
    /// - 核准時「改動物欄位 + 標記申請」同一 tx（原子，解狀態漂移）
    /// - `log_activity_tx` 記動物 before/after `DataDiff`（補稽核軌跡）
    pub async fn review(
        pool: &PgPool,
        actor: &ActorContext,
        request_id: Uuid,
        req: &ReviewAnimalFieldCorrectionRequest,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let reviewed_by = user.id;

        let mut tx = pool.begin().await?;

        // 鎖申請列並重驗 pending（同一 tx 可見性，防並發重複核准）
        let row: Option<(Uuid, String, String, Option<Uuid>)> = sqlx::query_as(
            r#"
            SELECT animal_id, field_name, new_value, requested_by
            FROM animal_field_correction_requests
            WHERE id = $1 AND status = 'pending'
            FOR UPDATE
            "#,
        )
        .bind(request_id)
        .fetch_optional(&mut *tx)
        .await?;

        let (animal_id, field_name, new_value, requested_by) =
            row.ok_or_else(|| AppError::NotFound("找不到待審核的修正申請".to_string()))?;

        // 職責分離：申請人不可核准自己的修正申請（動物身分欄位的更正需第二雙眼睛）。
        // 拒絕（等同撤回）自己的申請不受限，因為不會改動任何資料。
        if req.approved && requested_by == Some(reviewed_by) {
            return Err(AppError::Forbidden(
                "不可核准自己提出的修正申請，請由其他審核者處理".to_string(),
            ));
        }

        if req.approved {
            // 鎖動物列並取 before 快照
            let before: Animal = sqlx::query_as(
                "SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
            )
            .bind(animal_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("動物不存在".to_string()))?;

            // 進場日期：核准當下重驗。申請建立後動物可能又多了紀錄或被改過出生日期，
            // 建立時通過的申請不代表現在仍合理（與出生日期的核准重驗同一個道理）。
            if field_name == "entry_date" {
                let new_date = Self::parse_entry_date(&new_value)?;
                let earliest = Self::earliest_record_date(&mut tx, animal_id).await?;
                super::utils::AnimalUtils::ensure_entry_date_plausible(
                    new_date,
                    crate::time::today_taiwan_naive(),
                    before.birth_date,
                    earliest,
                )?;
            }

            // 套用修正（同一 tx）
            Self::apply_correction(&mut tx, animal_id, &field_name, &new_value).await?;

            // after 快照（同一 tx 可見已套用的變更）
            let after: Animal = sqlx::query_as("SELECT * FROM animals WHERE id = $1")
                .bind(animal_id)
                .fetch_one(&mut *tx)
                .await?;

            // 標記申請已核准
            sqlx::query(
                r#"
                UPDATE animal_field_correction_requests
                SET status = 'approved', reviewed_by = $2, reviewed_at = NOW(), updated_at = NOW()
                WHERE id = $1 AND status = 'pending'
                "#,
            )
            .bind(request_id)
            .bind(reviewed_by)
            .execute(&mut *tx)
            .await?;

            // 稽核：記動物身分欄位 before/after diff（同一 tx，與資料變更原子）
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "ANIMAL",
                    event_type: "FIELD_CORRECTION_APPROVE",
                    entity: Some(AuditEntity::new("animal", animal_id, &after.ear_tag)),
                    data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                    request_context: None,
                },
            )
            .await?;
        } else {
            let reject_reason = req.reject_reason.as_deref().unwrap_or("未提供拒絕原因");

            sqlx::query(
                r#"
                UPDATE animal_field_correction_requests
                SET status = 'rejected', reviewed_by = $2, reviewed_at = NOW(),
                    reason = COALESCE(reason, '') || E'\n[拒絕原因] ' || $3, updated_at = NOW()
                WHERE id = $1 AND status = 'pending'
                "#,
            )
            .bind(request_id)
            .bind(reviewed_by)
            .bind(reject_reason)
            .execute(&mut *tx)
            .await?;

            // 稽核：拒絕不改動物，記在申請 entity 上
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "ANIMAL",
                    event_type: "FIELD_CORRECTION_REJECT",
                    entity: Some(AuditEntity::new(
                        "animal_field_correction_request",
                        request_id,
                        &field_name,
                    )),
                    data_diff: None,
                    request_context: None,
                },
            )
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 核准時解析並重驗出生日期。
    ///
    /// 建立申請時已擋未來日期（`validate_new_value`），但核准另有一條寫入路徑：
    /// 「禁止未來日期」上線前建立、仍在待審的申請可能帶著未來日期，核准時若不重驗，
    /// 仍會把它寫進動物資料。`today` 由呼叫端傳入（台灣今日），好讓單元測試不依賴時鐘。
    fn parse_birth_date_for_apply(new_value: &str, today: NaiveDate) -> Result<NaiveDate> {
        let d = NaiveDate::parse_from_str(new_value, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("日期格式錯誤".to_string()))?;
        super::utils::AnimalUtils::ensure_birth_date_not_future(d, today)?;
        Ok(d)
    }

    /// 解析進場日期（YYYY-MM-DD），只驗格式。
    ///
    /// 合理性（不晚於今日、不早於出生日期、不晚於已有紀錄的最早日期）由
    /// `AnimalUtils::ensure_entry_date_plausible` 負責；建立申請與核准兩條路徑都呼叫，
    /// 避免規則漂移。建立「動物」本身的進場日期規則不在此變動。
    fn parse_entry_date(new_value: &str) -> Result<NaiveDate> {
        NaiveDate::parse_from_str(new_value, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("進場日期格式須為 YYYY-MM-DD".to_string()))
    }

    /// 該動物已有紀錄（體重、觀察、手術、疫苗驅蟲、血液檢查；不含已軟刪除）的最早日期。
    ///
    /// 沒有任何紀錄回 `None`。只有本檔兩處（建立申請、核准）使用，所以不抽到 repositories。
    async fn earliest_record_date(
        conn: &mut sqlx::PgConnection,
        animal_id: Uuid,
    ) -> Result<Option<NaiveDate>> {
        let earliest: Option<NaiveDate> = sqlx::query_scalar(
            r#"
            SELECT MIN(d) FROM (
                SELECT measure_date AS d FROM animal_weights
                    WHERE animal_id = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT event_date FROM animal_observations
                    WHERE animal_id = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT surgery_date FROM animal_surgeries
                    WHERE animal_id = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT administered_date FROM animal_vaccinations
                    WHERE animal_id = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT test_date FROM animal_blood_tests
                    WHERE animal_id = $1 AND deleted_at IS NULL
            ) records
            "#,
        )
        .bind(animal_id)
        .fetch_one(conn)
        .await?;
        Ok(earliest)
    }

    async fn apply_correction(
        tx: &mut Transaction<'_, Postgres>,
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
                    .execute(&mut **tx)
                    .await?;
            }
            "birth_date" => {
                let d =
                    Self::parse_birth_date_for_apply(new_value, crate::time::today_taiwan_naive())?;
                sqlx::query("UPDATE animals SET birth_date = $2, updated_at = NOW() WHERE id = $1")
                    .bind(animal_id)
                    .bind(d)
                    .execute(&mut **tx)
                    .await?;
            }
            "entry_date" => {
                // 只改 animals.entry_date。建立動物時自動寫入的初始體重紀錄（以進場日期當量測日）
                // 刻意不連動——使用者 2026-11-12 裁定：體重紀錄是獨立的量測事實，
                // 需要時另行更正體重紀錄。
                let d = Self::parse_entry_date(new_value)?;
                sqlx::query("UPDATE animals SET entry_date = $2, updated_at = NOW() WHERE id = $1")
                    .bind(animal_id)
                    .bind(d)
                    .execute(&mut **tx)
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
                    .execute(&mut **tx)
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
                sqlx::query(
                    "UPDATE animals SET breed = $2::animal_breed, updated_at = NOW() WHERE id = $1",
                )
                .bind(animal_id)
                .bind(breed_str)
                .execute(&mut **tx)
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_new_value: ear_tag ──

    #[test]
    fn test_ear_tag_valid_three_digits() {
        assert!(AnimalFieldCorrectionService::validate_new_value("ear_tag", "001").is_ok());
    }

    #[test]
    fn test_ear_tag_valid_single_digit_padded() {
        // "5" → format_ear_tag → "005" → 3 digits OK
        assert!(AnimalFieldCorrectionService::validate_new_value("ear_tag", "5").is_ok());
    }

    #[test]
    fn test_ear_tag_valid_99() {
        assert!(AnimalFieldCorrectionService::validate_new_value("ear_tag", "99").is_ok());
    }

    #[test]
    fn test_ear_tag_three_digit_100() {
        assert!(AnimalFieldCorrectionService::validate_new_value("ear_tag", "100").is_ok());
    }

    #[test]
    fn test_ear_tag_rejects_non_numeric() {
        assert!(AnimalFieldCorrectionService::validate_new_value("ear_tag", "abc").is_err());
    }

    #[test]
    fn test_ear_tag_rejects_four_digits() {
        assert!(AnimalFieldCorrectionService::validate_new_value("ear_tag", "1000").is_err());
    }

    // ── validate_new_value: birth_date ──

    #[test]
    fn test_birth_date_valid() {
        assert!(
            AnimalFieldCorrectionService::validate_new_value("birth_date", "2025-01-15").is_ok()
        );
    }

    #[test]
    fn test_birth_date_future_rejected() {
        let future = (crate::time::today_taiwan_naive() + chrono::Days::new(1)).to_string();
        assert!(AnimalFieldCorrectionService::validate_new_value("birth_date", &future).is_err());
    }

    #[test]
    fn test_birth_date_today_allowed() {
        let today = crate::time::today_taiwan_naive().to_string();
        assert!(AnimalFieldCorrectionService::validate_new_value("birth_date", &today).is_ok());
    }

    #[test]
    fn test_birth_date_invalid_format() {
        assert!(
            AnimalFieldCorrectionService::validate_new_value("birth_date", "01/15/2025").is_err()
        );
    }

    #[test]
    fn test_birth_date_invalid_date() {
        assert!(
            AnimalFieldCorrectionService::validate_new_value("birth_date", "2025-13-01").is_err()
        );
    }

    // ── parse_birth_date_for_apply：核准時重驗（回歸：舊的待審申請不可繞過未來日期限制）──

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("測試用日期必須合法")
    }

    #[test]
    fn apply_rejects_future_birth_date_from_older_pending_request() {
        let today = ymd(2026, 11, 2);
        let err = AnimalFieldCorrectionService::parse_birth_date_for_apply("2026-11-03", today)
            .expect_err("明天起的日期核准時也要擋下");
        assert!(err.to_string().contains("出生日期不可晚於今日"), "{err}");
    }

    #[test]
    fn apply_allows_today_and_past_birth_date() {
        let today = ymd(2026, 11, 2);
        assert_eq!(
            AnimalFieldCorrectionService::parse_birth_date_for_apply("2026-11-02", today)
                .expect("當天可核准"),
            today
        );
        assert!(
            AnimalFieldCorrectionService::parse_birth_date_for_apply("2025-01-15", today).is_ok()
        );
    }

    #[test]
    fn apply_rejects_unparseable_birth_date() {
        let today = ymd(2026, 11, 2);
        assert!(
            AnimalFieldCorrectionService::parse_birth_date_for_apply("01/15/2025", today).is_err()
        );
    }

    // ── validate_new_value: entry_date ──

    #[test]
    fn test_entry_date_valid() {
        assert!(
            AnimalFieldCorrectionService::validate_new_value("entry_date", "2026-01-05").is_ok()
        );
    }

    #[test]
    fn test_entry_date_invalid_format() {
        let err = AnimalFieldCorrectionService::validate_new_value("entry_date", "01/05/2026")
            .expect_err("非 YYYY-MM-DD 格式應被擋下");
        assert!(err.to_string().contains("進場日期格式"), "{err}");
    }

    #[test]
    fn test_entry_date_invalid_date() {
        assert!(
            AnimalFieldCorrectionService::validate_new_value("entry_date", "2026-02-30").is_err()
        );
    }

    #[test]
    fn test_entry_date_is_a_correctable_field() {
        assert!(CORRECTABLE_FIELDS.contains(&"entry_date"));
    }

    // ── validate_new_value: gender ──

    #[test]
    fn test_gender_valid_male() {
        assert!(AnimalFieldCorrectionService::validate_new_value("gender", "male").is_ok());
    }

    #[test]
    fn test_gender_valid_female() {
        assert!(AnimalFieldCorrectionService::validate_new_value("gender", "female").is_ok());
    }

    #[test]
    fn test_gender_rejects_other() {
        assert!(AnimalFieldCorrectionService::validate_new_value("gender", "other").is_err());
    }

    #[test]
    fn test_gender_rejects_uppercase() {
        // 嚴格比對，不接受大寫
        assert!(AnimalFieldCorrectionService::validate_new_value("gender", "Male").is_err());
    }

    // ── validate_new_value: breed ──

    #[test]
    fn test_breed_valid_miniature() {
        assert!(AnimalFieldCorrectionService::validate_new_value("breed", "miniature").is_ok());
    }

    #[test]
    fn test_breed_valid_lyd_uppercase() {
        assert!(AnimalFieldCorrectionService::validate_new_value("breed", "LYD").is_ok());
    }

    #[test]
    fn test_breed_valid_lyd_lowercase() {
        assert!(AnimalFieldCorrectionService::validate_new_value("breed", "lyd").is_ok());
    }

    #[test]
    fn test_breed_rejects_unknown() {
        assert!(AnimalFieldCorrectionService::validate_new_value("breed", "Duroc").is_err());
    }

    // ── validate_new_value: unknown field passes ──

    #[test]
    fn test_unknown_field_always_ok() {
        assert!(AnimalFieldCorrectionService::validate_new_value("remark", "anything").is_ok());
    }
}
