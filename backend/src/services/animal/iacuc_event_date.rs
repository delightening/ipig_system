//! IACUC No. 變更事件的「補登實際日期」（migration 027）。
//!
//! 較早寫入的 `IACUC_CHANGE` 事件沒有存實際變更日期，稽核表又不能更新，
//! 所以實際日期另存 `animal_event_actual_dates`：只新增、不覆蓋，讀取取同一事件最新一筆。
//! 每次補登在同一交易寫一筆稽核事件（`IACUC_EVENT_DATE_BACKFILL`）。
//!
//! 日期範圍（以該動物「全部」IACUC 事件依登錄時間排序計算）：
//! - 事件的「有效日期」＝有補登用補登日，否則用登錄日（登錄時間換算成台北時間的日曆日）。
//! - 下限：第一次分配＝進場日；之後的更換＝前一事件的有效日期。
//! - 上限：不晚於自己的登錄日；有下一事件則不晚於下一事件的有效日期；
//!   首次分配（事件的舊值沒有 IACUC No.；不是「時間上第一個事件」）另外不晚於
//!   「第一筆需計畫的試驗紀錄」的日期。批次分配與轉讓直接改 `animals.iacuc_no`、不寫事件，
//!   所以第一個被記錄的事件可能是「A → B」，它不套用這個上限。
//! - 「需計畫的試驗紀錄」＝手術、試驗性觀察、疼痛評估（疼痛評估沒有自己的日期，取其父紀錄的日期）。
//!   體重、疫苗、血液檢查是日常照護，不算；病理報告沒有臨床日期（只有建立時間），也不算。

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    middleware::ActorContext,
    models::audit_diff::{AuditRedact, DataDiff},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

/// 補登原因的長度上限（字元數），與 migration 027 的 CHECK 一致
const REASON_MAX_CHARS: usize = 500;

/// 補登實際日期的請求
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct BackfillIacucEventDateRequest {
    /// 實際變更日期（YYYY-MM-DD）
    pub actual_date: String,
    /// 補登原因（必填，最多 500 字）
    pub reason: String,
}

/// 補登資訊（時間軸與補登回應共用）
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct IacucEventBackfill {
    /// 補登人姓名
    pub by_name: Option<String>,
    /// 補登時間
    pub at: DateTime<Utc>,
    /// 補登原因
    pub reason: String,
}

/// 補登實際日期的回應
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct BackfillIacucEventDateResponse {
    pub event_id: Uuid,
    pub actual_date: NaiveDate,
    pub backfill: IacucEventBackfill,
}

/// 稽核事件的 before/after 快照：只放這次補登相關的欄位。
#[derive(serde::Serialize)]
struct BackfillSnapshot {
    event_id: Uuid,
    /// 補登的實際日期；補登前沒有補登紀錄時為 None
    actual_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

impl AuditRedact for BackfillSnapshot {}

/// 稽核表裡的一筆 IACUC 變更事件：範圍計算與判斷「是否為首次分配」需要的欄位。
#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    created_at: DateTime<Utc>,
    before_data: Option<serde_json::Value>,
    after_data: Option<serde_json::Value>,
    entity_display_name: Option<String>,
}

impl EventRow {
    fn is_initial_assignment(&self) -> bool {
        is_initial_assignment(
            self.before_data.as_ref(),
            self.after_data.as_ref(),
            self.entity_display_name.as_deref(),
        )
    }
}

/// 事件是否為「首次分配」：變更前沒有 IACUC No.（舊值為空）。
///
/// 判斷依據沿用 `IacucChangeSnapshot::pair` 的規則：`before_data.iacuc_no` 為 null 或缺席。
/// 修正前寫入的歷史事件 before/after 都是空的，退回解析顯示文字 `[(無) → X] 耳號`。
/// 兩種來源都判斷不出來時回 `false`（不套用「第一筆試驗紀錄」上限）：
/// 寧可放寬，也不誤擋合法的補登。
fn is_initial_assignment(
    before: Option<&serde_json::Value>,
    after: Option<&serde_json::Value>,
    display: Option<&str>,
) -> bool {
    let before_value: Option<serde_json::Value> = match (before, after) {
        (None, None) => display
            .and_then(AnimalService::iacuc_change_data_from_display)
            .map(|(b, _)| b),
        (b, _) => b.cloned(),
    };
    before_value.is_some_and(|b| b.get("iacuc_no").is_none_or(serde_json::Value::is_null))
}

/// 時間軸上的一個 IACUC 變更事件（只留範圍計算需要的日期）
#[derive(Debug, Clone, Copy)]
struct TimelineEvent {
    /// 登錄日：事件寫入時間換算成台北時間的日曆日
    registered: NaiveDate,
    /// 最新一筆補登的實際日期
    backfilled: Option<NaiveDate>,
}

impl TimelineEvent {
    /// 有效日期：有補登用補登日，否則用登錄日
    fn effective(&self) -> NaiveDate {
        self.backfilled.unwrap_or(self.registered)
    }
}

/// 允許的實際日期範圍（含兩端）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DateRange {
    from: NaiveDate,
    to: NaiveDate,
}

/// 計算第 `index` 個事件（依登錄時間由舊到新排序）允許的實際日期範圍。
///
/// `first_plan_record` 只套用在第一次分配（`index == 0`）：之後的更換事件發生時，
/// 試驗紀錄本來就可能已經存在，再套用只會把上限無謂地拉前。
fn allowed_range(
    events: &[TimelineEvent],
    index: usize,
    entry_date: NaiveDate,
    first_plan_record: Option<NaiveDate>,
) -> DateRange {
    let event = events[index];
    let from = if index == 0 {
        entry_date
    } else {
        events[index - 1].effective()
    };

    // 事件不可能發生在它被登錄之後
    let mut to = event.registered;
    if let Some(next) = events.get(index + 1) {
        to = to.min(next.effective());
    }
    if index == 0 {
        if let Some(first) = first_plan_record {
            to = to.min(first);
        }
    }
    DateRange { from, to }
}

fn ensure_within(range: DateRange, date: NaiveDate) -> Result<()> {
    if range.from > range.to {
        return Err(AppError::Validation(
            "此事件目前沒有可選的日期範圍（進場日、前後事件或試驗紀錄的日期互相衝突），請先確認其他事件的日期"
                .to_string(),
        ));
    }
    if date < range.from || date > range.to {
        return Err(AppError::Validation(format!(
            "實際日期須介於 {} 與 {} 之間（含）",
            range.from, range.to
        )));
    }
    Ok(())
}

fn parse_actual_date(raw: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::Validation("實際日期格式須為 YYYY-MM-DD".to_string()))
}

fn clean_reason(raw: &str) -> Result<String> {
    let reason = raw.trim();
    if reason.is_empty() {
        return Err(AppError::Validation("請填寫補登原因".to_string()));
    }
    if reason.chars().count() > REASON_MAX_CHARS {
        return Err(AppError::Validation(format!(
            "補登原因最多 {REASON_MAX_CHARS} 字"
        )));
    }
    Ok(reason.to_string())
}

impl AnimalService {
    /// 為某個 IACUC 變更事件補登實際日期。
    ///
    /// 全程在單一交易內，並先鎖住動物列：同一隻動物的並行補登會排隊，
    /// 前後事件的有效日期不會在驗證與寫入之間被別人改掉。
    pub async fn backfill_iacuc_event_date(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        event_id: Uuid,
        req: &BackfillIacucEventDateRequest,
    ) -> Result<BackfillIacucEventDateResponse> {
        let user = actor.require_user()?;
        let actual_date = parse_actual_date(&req.actual_date)?;
        let reason = clean_reason(&req.reason)?;

        let mut tx = pool.begin().await?;

        let animal: Option<(NaiveDate, String)> = sqlx::query_as(
            "SELECT entry_date, ear_tag FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(animal_id)
        .fetch_optional(&mut *tx)
        .await?;
        let (entry_date, ear_tag) =
            animal.ok_or_else(|| AppError::NotFound("動物不存在".to_string()))?;

        // 該動物全部 IACUC 變更事件（不受時間軸 50 筆顯示上限影響），由舊到新
        let event_rows: Vec<EventRow> = sqlx::query_as(
            r#"
            SELECT id, created_at, before_data, after_data, entity_display_name
            FROM user_activity_logs
            WHERE entity_type = 'animal'
              AND entity_id = $1
              AND event_type = 'IACUC_CHANGE'
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(animal_id)
        .fetch_all(&mut *tx)
        .await?;

        let index = event_rows
            .iter()
            .position(|row| row.id == event_id)
            .ok_or_else(|| AppError::NotFound("找不到該動物的 IACUC 變更事件".to_string()))?;

        // 每個事件最新一筆補登
        let latest: Vec<(Uuid, NaiveDate)> = sqlx::query_as(
            r#"
            SELECT DISTINCT ON (event_id) event_id, actual_date
            FROM animal_event_actual_dates
            WHERE animal_id = $1
            ORDER BY event_id, created_at DESC, id DESC
            "#,
        )
        .bind(animal_id)
        .fetch_all(&mut *tx)
        .await?;
        let latest_of = |id: Uuid| latest.iter().find(|(e, _)| *e == id).map(|(_, d)| *d);

        let timeline: Vec<TimelineEvent> = event_rows
            .iter()
            .map(|row| TimelineEvent {
                registered: row
                    .created_at
                    .with_timezone(&crate::time::taiwan_offset())
                    .date_naive(),
                backfilled: latest_of(row.id),
            })
            .collect();

        // 「第一筆試驗紀錄」上限只套用在真正的首次分配：事件的舊值沒有 IACUC No.。
        // 不能只看「時間上第一個 IACUC_CHANGE」：批次分配與轉讓直接改 animals.iacuc_no、
        // 不寫這個事件，之後在更新頁換計畫時，第一個被記錄的事件是「A → B」，
        // 試驗紀錄是在 A 底下建立的，套用上限會誤擋合法的補登。
        let is_initial = index == 0 && event_rows[index].is_initial_assignment();
        let first_plan_record = if is_initial {
            Self::first_plan_required_record_date(&mut tx, animal_id).await?
        } else {
            None
        };
        ensure_within(
            allowed_range(&timeline, index, entry_date, first_plan_record),
            actual_date,
        )?;

        let previous = latest_of(event_id);

        let created_at: DateTime<Utc> = sqlx::query_scalar(
            r#"
            INSERT INTO animal_event_actual_dates (animal_id, event_id, actual_date, reason, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING created_at
            "#,
        )
        .bind(animal_id)
        .bind(event_id)
        .bind(actual_date)
        .bind(&reason)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        let by_name: Option<String> =
            sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
                .bind(user.id)
                .fetch_optional(&mut *tx)
                .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "IACUC_EVENT_DATE_BACKFILL",
                entity: Some(AuditEntity::new("animal", animal_id, &ear_tag)),
                data_diff: Some(DataDiff::compute(
                    Some(&BackfillSnapshot {
                        event_id,
                        actual_date: previous,
                        reason: None,
                    }),
                    Some(&BackfillSnapshot {
                        event_id,
                        actual_date: Some(actual_date),
                        reason: Some(reason.clone()),
                    }),
                )),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(BackfillIacucEventDateResponse {
            event_id,
            actual_date,
            backfill: IacucEventBackfill {
                by_name,
                at: created_at,
                reason,
            },
        })
    }

    /// 該動物「需計畫才能建立」的試驗紀錄中最早的日期；沒有則回 `None`。
    ///
    /// 與進場日期修正用的 `earliest_record_date` 不同：這裡不算體重、疫苗、血液檢查
    /// （日常照護，不需計畫），也不算病理報告（沒有臨床日期）。疼痛評估沒有自己的日期，
    /// 取它所掛的手術或觀察的日期。不含已軟刪除。
    async fn first_plan_required_record_date(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        animal_id: Uuid,
    ) -> Result<Option<NaiveDate>> {
        let first: Option<NaiveDate> = sqlx::query_scalar(
            r#"
            SELECT MIN(d) FROM (
                SELECT surgery_date AS d FROM animal_surgeries
                    WHERE animal_id = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT event_date FROM animal_observations
                    WHERE animal_id = $1 AND deleted_at IS NULL AND record_type = 'experiment'
                UNION ALL
                SELECT o.event_date
                    FROM care_medication_records c
                    JOIN animal_observations o ON c.record_type = 'observation' AND o.id = c.record_id
                    WHERE o.animal_id = $1 AND o.deleted_at IS NULL
                      AND c.record_mode = 'pain_assessment' AND c.deleted_at IS NULL
                UNION ALL
                SELECT s.surgery_date
                    FROM care_medication_records c
                    JOIN animal_surgeries s ON c.record_type = 'surgery' AND s.id = c.record_id
                    WHERE s.animal_id = $1 AND s.deleted_at IS NULL
                      AND c.record_mode = 'pain_assessment' AND c.deleted_at IS NULL
            ) records
            "#,
        )
        .bind(animal_id)
        .fetch_one(&mut **tx)
        .await?;
        Ok(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("valid date")
    }

    fn ev(registered: &str, backfilled: Option<&str>) -> TimelineEvent {
        TimelineEvent {
            registered: d(registered),
            backfilled: backfilled.map(d),
        }
    }

    #[test]
    fn first_assignment_is_bounded_by_entry_date_and_first_plan_record() {
        let events = [ev("2026-09-10", None)];
        let range = allowed_range(&events, 0, d("2026-07-13"), Some(d("2026-08-20")));
        assert_eq!(
            range,
            DateRange {
                from: d("2026-07-13"),
                to: d("2026-08-20")
            }
        );
    }

    #[test]
    fn first_assignment_without_plan_records_is_bounded_by_registered_date() {
        let events = [ev("2026-09-10", None)];
        let range = allowed_range(&events, 0, d("2026-07-13"), None);
        assert_eq!(range.to, d("2026-09-10"));
    }

    #[test]
    fn first_plan_record_later_than_registration_does_not_extend_the_upper_bound() {
        // 事件不可能發生在它被登錄之後
        let events = [ev("2026-09-10", None)];
        let range = allowed_range(&events, 0, d("2026-07-13"), Some(d("2026-10-01")));
        assert_eq!(range.to, d("2026-09-10"));
    }

    #[test]
    fn later_change_is_sandwiched_by_neighbouring_effective_dates() {
        let events = [
            ev("2026-09-01", Some("2026-08-01")),
            ev("2026-09-05", None),
            ev("2026-09-20", Some("2026-09-15")),
        ];
        // 中間事件：下限＝前一事件的補登日；上限＝自己的登錄日與下一事件補登日取較早者
        let range = allowed_range(&events, 1, d("2026-07-13"), Some(d("2026-07-20")));
        assert_eq!(
            range,
            DateRange {
                from: d("2026-08-01"),
                to: d("2026-09-05")
            }
        );
    }

    #[test]
    fn later_change_ignores_the_first_plan_record() {
        // 第一筆試驗紀錄上限只套用在第一次分配
        let events = [ev("2026-09-01", None), ev("2026-09-20", None)];
        let range = allowed_range(&events, 1, d("2026-07-13"), Some(d("2026-07-20")));
        assert_eq!(
            range,
            DateRange {
                from: d("2026-09-01"),
                to: d("2026-09-20")
            }
        );
    }

    #[test]
    fn first_assignment_is_also_bounded_by_the_next_event() {
        let events = [ev("2026-09-10", None), ev("2026-09-12", Some("2026-08-05"))];
        let range = allowed_range(&events, 0, d("2026-07-13"), None);
        assert_eq!(range.to, d("2026-08-05"));
    }

    #[test]
    fn range_is_inclusive_on_both_ends() {
        let range = DateRange {
            from: d("2026-07-13"),
            to: d("2026-08-20"),
        };
        assert!(ensure_within(range, d("2026-07-13")).is_ok());
        assert!(ensure_within(range, d("2026-08-20")).is_ok());
        assert!(ensure_within(range, d("2026-07-12")).is_err());
        assert!(ensure_within(range, d("2026-08-21")).is_err());
    }

    #[test]
    fn conflicting_neighbours_give_an_explicit_error() {
        let range = DateRange {
            from: d("2026-09-02"),
            to: d("2026-09-01"),
        };
        let err = ensure_within(range, d("2026-09-01")).expect_err("empty range");
        assert!(err.to_string().contains("沒有可選的日期範圍"), "{err}");
    }

    #[test]
    fn out_of_range_error_names_the_allowed_range() {
        let range = DateRange {
            from: d("2026-07-13"),
            to: d("2026-08-20"),
        };
        let err = ensure_within(range, d("2026-09-01")).expect_err("out of range");
        let msg = err.to_string();
        assert!(
            msg.contains("2026-07-13") && msg.contains("2026-08-20"),
            "{msg}"
        );
    }

    #[test]
    fn date_and_reason_validation() {
        assert!(parse_actual_date(" 2026-03-15 ").is_ok());
        assert!(parse_actual_date("2026/03/15").is_err());
        assert!(parse_actual_date("").is_err());
        assert_eq!(
            clean_reason("  依紙本申請單補登 ").expect("valid reason"),
            "依紙本申請單補登"
        );
        assert!(clean_reason("   ").is_err());
        assert!(clean_reason(&"字".repeat(501)).is_err());
        assert!(clean_reason(&"字".repeat(500)).is_ok());
    }

    // ── 是否為首次分配（決定要不要套用「第一筆試驗紀錄」上限）──

    #[test]
    fn event_without_a_previous_iacuc_is_an_initial_assignment() {
        let before = serde_json::json!({ "iacuc_no": null });
        let after = serde_json::json!({ "iacuc_no": "PIG-114005" });
        assert!(is_initial_assignment(Some(&before), Some(&after), None));
        // before_data 沒有 iacuc_no 這個鍵，也視為舊值為空
        let empty = serde_json::json!({});
        assert!(is_initial_assignment(Some(&empty), Some(&after), None));
    }

    #[test]
    fn event_with_a_previous_iacuc_is_not_an_initial_assignment() {
        // 批次分配或轉讓不寫事件，之後換計畫時第一個被記錄的事件是「A → B」
        let before = serde_json::json!({ "iacuc_no": "PIG-114004" });
        let after = serde_json::json!({ "iacuc_no": "PIG-114005" });
        assert!(!is_initial_assignment(Some(&before), Some(&after), None));
    }

    #[test]
    fn legacy_event_falls_back_to_the_display_text() {
        // 修正前寫入的歷史事件：before/after 都是空，只有顯示文字
        assert!(is_initial_assignment(
            None,
            None,
            Some("[(無) → PIG-114005] 243")
        ));
        assert!(!is_initial_assignment(
            None,
            None,
            Some("[PIG-114004 → PIG-114005] 243")
        ));
    }

    #[test]
    fn unknown_previous_value_does_not_apply_the_cap() {
        // 兩種來源都判斷不出來：寧可放寬，也不誤擋合法的補登
        assert!(!is_initial_assignment(None, None, None));
        assert!(!is_initial_assignment(None, None, Some("看不懂的文字")));
    }
}
