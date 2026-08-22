//! AI 接口 Repository 層：封裝 AI 相關的 SQL 查詢

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::ai::AiApiKey;
use crate::AppError;

pub struct AiRepository;

impl AiRepository {
    // ── API Key 管理 ──

    /// 建立 AI API key。executor 泛型（接受 `&PgPool` 或 `&mut *tx`），
    /// 讓 service 能在「INSERT + audit」同一 tx 內呼叫，維持 Service-driven audit 原子性。
    /// `is_active` 走 DB default（true），不在 SQL 顯式帶。
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_api_key<'e, E>(
        executor: E,
        name: &str,
        key_hash: &str,
        key_prefix: &str,
        created_by: Uuid,
        scopes: &serde_json::Value,
        expires_at: Option<DateTime<Utc>>,
        rate_limit_per_minute: i32,
    ) -> Result<AiApiKey, AppError>
    where
        E: sqlx::PgExecutor<'e>,
    {
        let row = sqlx::query_as::<_, AiApiKey>(
            r#"INSERT INTO ai_api_keys (name, key_hash, key_prefix, created_by, scopes, expires_at, rate_limit_per_minute)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(name)
        .bind(key_hash)
        .bind(key_prefix)
        .bind(created_by)
        .bind(scopes)
        .bind(expires_at)
        .bind(rate_limit_per_minute)
        .fetch_one(executor)
        .await?;
        Ok(row)
    }

    pub async fn list_api_keys(pool: &PgPool) -> Result<Vec<AiApiKey>, AppError> {
        let rows =
            sqlx::query_as::<_, AiApiKey>("SELECT * FROM ai_api_keys ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?;
        Ok(rows)
    }

    pub async fn delete_api_key(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM ai_api_keys WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    // ── 查詢日誌 ──

    pub async fn insert_query_log(
        pool: &PgPool,
        api_key_id: Uuid,
        endpoint: &str,
        method: &str,
        query_summary: Option<&serde_json::Value>,
        response_status: i16,
        duration_ms: i32,
        source_ip: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO ai_query_logs (api_key_id, endpoint, method, query_summary, response_status, duration_ms, source_ip)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(api_key_id)
        .bind(endpoint)
        .bind(method)
        .bind(query_summary)
        .bind(response_status)
        .bind(duration_ms)
        .bind(source_ip)
        .execute(pool)
        .await?;
        Ok(())
    }

    // ── 資料查詢（AI 唯讀） ──

    pub async fn query_system_overview(pool: &PgPool) -> Result<serde_json::Value, AppError> {
        let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64, i64)>(
            r#"SELECT
                (SELECT COUNT(*) FROM animals) as total_animals,
                (SELECT COUNT(*) FROM animals WHERE deleted_at IS NULL AND status::text NOT IN ('euthanized','sudden_death','transferred')) as active_animals,
                (SELECT COUNT(*) FROM protocols) as total_protocols,
                (SELECT COUNT(*) FROM protocols WHERE status::text = 'approved') as active_protocols,
                (SELECT COUNT(*) FROM animal_observations WHERE created_at >= now() - INTERVAL '30 days' AND animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)) as observations_30d,
                (SELECT COUNT(*) FROM animal_surgeries WHERE created_at >= now() - INTERVAL '30 days' AND animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)) as surgeries_30d,
                (SELECT COUNT(*) FROM facilities WHERE is_active = true) as facilities_count"#,
        )
        .fetch_one(pool)
        .await?;

        Ok(serde_json::json!({
            "total_animals": row.0,
            "active_animals": row.1,
            "total_protocols": row.2,
            "active_protocols": row.3,
            "total_observations_30d": row.4,
            "total_surgeries_30d": row.5,
            "facilities_count": row.6
        }))
    }

    /// AI 唯讀動物列表。未指定 `status` 篩選時**預設排除離場終態**
    /// （euthanized / sudden_death / transferred），與 `query_system_overview` 的
    /// active_animals 定義一致——避免殘留 pen_location 的終態動物混入
    /// 「某欄位有哪些豬」類查詢（#999 follow-up）。要查終態動物請明確帶 status。
    pub async fn query_animals(
        pool: &PgPool,
        filters: &serde_json::Value,
        page: i64,
        per_page: i64,
        sort_by: Option<&str>,
        sort_order: &str,
    ) -> Result<(Vec<serde_json::Value>, i64), AppError> {
        let per_page = per_page.clamp(1, 500);
        let offset = (page.max(1) - 1) * per_page;
        let order = if sort_order == "asc" { "ASC" } else { "DESC" };
        let sort_col = validate_sort_field(
            sort_by.unwrap_or("created_at"),
            &["ear_tag", "status", "gender", "created_at", "entry_date"],
            "created_at",
        );

        let status = filters.get("status").and_then(|v| v.as_str());
        let keyword = filters.get("keyword").and_then(|v| v.as_str());

        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM animals
               WHERE deleted_at IS NULL
                 AND ($1::text IS NULL OR status::text = $1)
                 AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred'))
                 AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%')"#,
        )
        .bind(status)
        .bind(keyword)
        .fetch_one(pool)
        .await?;

        // SEC-AUDIT-007: 使用靜態 SQL 查詢取代 format!() 動態建構，
        // 透過 match 分派到預定義的 ORDER BY 子句，消除 SQL injection 風險
        let query_str = match (sort_col, order) {
            ("ear_tag", "ASC") => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY ear_tag ASC LIMIT $3 OFFSET $4"#
            }
            ("ear_tag", _) => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY ear_tag DESC LIMIT $3 OFFSET $4"#
            }
            ("status", "ASC") => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY status ASC LIMIT $3 OFFSET $4"#
            }
            ("status", _) => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY status DESC LIMIT $3 OFFSET $4"#
            }
            ("gender", "ASC") => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY gender ASC LIMIT $3 OFFSET $4"#
            }
            ("gender", _) => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY gender DESC LIMIT $3 OFFSET $4"#
            }
            ("entry_date", "ASC") => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY entry_date ASC LIMIT $3 OFFSET $4"#
            }
            ("entry_date", _) => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY entry_date DESC LIMIT $3 OFFSET $4"#
            }
            (_, "ASC") => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY created_at ASC LIMIT $3 OFFSET $4"#
            }
            _ => {
                r#"SELECT id, ear_tag, animal_no, breed::text, gender::text, status::text, birth_date, entry_date, pen_location, iacuc_no, created_at FROM animals WHERE deleted_at IS NULL AND ($1::text IS NULL OR status::text = $1) AND ($1::text IS NOT NULL OR status::text NOT IN ('euthanized','sudden_death','transferred')) AND ($2::text IS NULL OR ear_tag ILIKE '%' || $2 || '%') ORDER BY created_at DESC LIMIT $3 OFFSET $4"#
            }
        };

        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<chrono::NaiveDate>,
                chrono::NaiveDate,
                Option<String>,
                Option<String>,
                DateTime<Utc>,
            ),
        >(query_str)
        .bind(status)
        .bind(keyword)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let data: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.0, "ear_tag": r.1, "animal_no": r.2,
                    "breed": r.3, "gender": r.4, "status": r.5,
                    "birth_date": r.6, "entry_date": r.7,
                    "pen_location": r.8, "iacuc_no": r.9, "created_at": r.10,
                })
            })
            .collect();

        Ok((data, count.0))
    }

    /// R86-10：`query_animals` 已排除軟刪豬，但這裡的觀察紀錄若不比照補上
    /// `animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)`，AI 仍能查到
    /// 已軟刪動物的觀察紀錄——單一真相源必須在每一條 animal-linked 查詢上各自成立，
    /// 不能只靠「動物清單本身有濾」去覆蓋其他表的查詢。
    pub async fn query_observations(
        pool: &PgPool,
        filters: &serde_json::Value,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<serde_json::Value>, i64), AppError> {
        let per_page = per_page.clamp(1, 500);
        let offset = (page.max(1) - 1) * per_page;
        let animal_id = filters
            .get("animal_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());
        let days = filters.get("days").and_then(|v| v.as_i64()).unwrap_or(30);

        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM animal_observations
               WHERE ($1::uuid IS NULL OR animal_id = $1)
                 AND created_at >= now() - make_interval(days => $2::int)
                 AND animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)"#,
        )
        .bind(animal_id)
        .bind(days as i32)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                uuid::Uuid,
                String,
                chrono::NaiveDate,
                Option<String>,
                DateTime<Utc>,
            ),
        >(
            r#"SELECT o.id, o.animal_id, o.record_type::text, o.event_date, o.content, o.created_at
               FROM animal_observations o
               WHERE ($1::uuid IS NULL OR o.animal_id = $1)
                 AND o.created_at >= now() - make_interval(days => $2::int)
                 AND o.animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)
               ORDER BY o.created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(animal_id)
        .bind(days as i32)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let data: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.0, "animal_id": r.1, "record_type": r.2,
                    "event_date": r.3, "content": r.4, "created_at": r.5,
                })
            })
            .collect();

        Ok((data, count.0))
    }

    pub async fn query_protocols(
        pool: &PgPool,
        filters: &serde_json::Value,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<serde_json::Value>, i64), AppError> {
        let per_page = per_page.clamp(1, 500);
        let offset = (page.max(1) - 1) * per_page;
        let status = filters.get("status").and_then(|v| v.as_str());

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM protocols WHERE ($1::text IS NULL OR status::text = $1)",
        )
        .bind(status)
        .fetch_one(pool)
        .await?;

        let list_sql = format!(
            r#"SELECT p.id, p.iacuc_no, p.title, {pi_name} as pi_name,
                      p.status::text, p.created_at
               FROM protocols p
               LEFT JOIN users u ON p.pi_user_id = u.id
               WHERE ($1::text IS NULL OR p.status::text = $1)
               ORDER BY p.created_at DESC
               LIMIT $2 OFFSET $3"#,
            pi_name = crate::utils::pi_sql::pi_display_name("u.display_name"),
        );
        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                String,
                String,
                Option<String>,
                String,
                DateTime<Utc>,
            ),
        >(sqlx::AssertSqlSafe(list_sql))
        .bind(status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let data: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.0,
                    "iacuc_no": r.1,
                    "title": r.2,
                    "pi_name": r.3,
                    "status": r.4,
                    "created_at": r.5,
                })
            })
            .collect();

        Ok((data, count.0))
    }

    pub async fn query_facilities(
        pool: &PgPool,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<serde_json::Value>, i64), AppError> {
        let per_page = per_page.clamp(1, 500);
        let offset = (page.max(1) - 1) * per_page;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM facilities")
            .fetch_one(pool)
            .await?;

        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                String,
                String,
                Option<String>,
                bool,
                DateTime<Utc>,
            ),
        >(
            r#"SELECT id, code, name, address, is_active, created_at
               FROM facilities
               ORDER BY code ASC
               LIMIT $1 OFFSET $2"#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let data: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.0, "code": r.1, "name": r.2,
                    "address": r.3, "is_active": r.4, "created_at": r.5,
                })
            })
            .collect();

        Ok((data, count.0))
    }

    pub async fn query_weights(
        pool: &PgPool,
        filters: &serde_json::Value,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<serde_json::Value>, i64), AppError> {
        let per_page = per_page.clamp(1, 500);
        let offset = (page.max(1) - 1) * per_page;
        let animal_id = filters
            .get("animal_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM animal_weights WHERE deleted_at IS NULL AND ($1::uuid IS NULL OR animal_id = $1) AND animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)",
        )
        .bind(animal_id)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                uuid::Uuid,
                rust_decimal::Decimal,
                chrono::NaiveDate,
                DateTime<Utc>,
            ),
        >(
            r#"SELECT id, animal_id, weight, measure_date, created_at
               FROM animal_weights
               WHERE deleted_at IS NULL AND ($1::uuid IS NULL OR animal_id = $1)
                 AND animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)
               ORDER BY measure_date DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(animal_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let data: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.0,
                    "animal_id": r.1,
                    "weight": r.2,
                    "measure_date": r.3,
                    "created_at": r.4,
                })
            })
            .collect();

        Ok((data, count.0))
    }

    pub async fn query_surgeries(
        pool: &PgPool,
        filters: &serde_json::Value,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<serde_json::Value>, i64), AppError> {
        let per_page = per_page.clamp(1, 500);
        let offset = (page.max(1) - 1) * per_page;
        let animal_id = filters
            .get("animal_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM animal_surgeries WHERE ($1::uuid IS NULL OR animal_id = $1) AND animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)",
        )
        .bind(animal_id)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                uuid::Uuid,
                chrono::NaiveDate,
                String,
                Option<String>,
                DateTime<Utc>,
            ),
        >(
            r#"SELECT id, animal_id, surgery_date, surgery_site, positioning, created_at
               FROM animal_surgeries
               WHERE ($1::uuid IS NULL OR animal_id = $1)
                 AND animal_id IN (SELECT id FROM animals WHERE deleted_at IS NULL)
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(animal_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let data: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.0,
                    "animal_id": r.1,
                    "surgery_date": r.2, "surgery_site": r.3,
                    "positioning": r.4, "created_at": r.5,
                })
            })
            .collect();

        Ok((data, count.0))
    }
}

/// 驗證排序欄位（防止 SQL injection），僅允許白名單欄位
fn validate_sort_field<'a>(input: &str, allowed: &[&'a str], default: &'a str) -> &'a str {
    if allowed.contains(&input) {
        allowed
            .iter()
            .find(|&&f| f == input)
            .copied()
            .unwrap_or(default)
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sort_field_allowed() {
        let result =
            validate_sort_field("ear_tag", &["ear_tag", "name", "created_at"], "created_at");
        assert_eq!(result, "ear_tag");
    }

    #[test]
    fn test_validate_sort_field_disallowed() {
        let result = validate_sort_field("DROP TABLE", &["ear_tag", "name"], "created_at");
        assert_eq!(result, "created_at");
    }
}
