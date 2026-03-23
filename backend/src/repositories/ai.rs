//! AI 接口 Repository 層：封裝 AI 相關的 SQL 查詢

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::ai::AiApiKey;
use crate::AppError;

pub struct AiRepository;

impl AiRepository {
    // ── API Key 管理 ──

    pub async fn insert_api_key(
        pool: &PgPool,
        name: &str,
        key_hash: &str,
        key_prefix: &str,
        created_by: Uuid,
        scopes: &serde_json::Value,
        expires_at: Option<DateTime<Utc>>,
        rate_limit_per_minute: i32,
    ) -> Result<AiApiKey, AppError> {
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
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn list_api_keys(pool: &PgPool) -> Result<Vec<AiApiKey>, AppError> {
        let rows = sqlx::query_as::<_, AiApiKey>(
            "SELECT * FROM ai_api_keys ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_api_key_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<AiApiKey>, AppError> {
        let row = sqlx::query_as::<_, AiApiKey>(
            "SELECT * FROM ai_api_keys WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn update_api_key_active(
        pool: &PgPool,
        id: Uuid,
        is_active: bool,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE ai_api_keys SET is_active = $1, updated_at = now() WHERE id = $2")
            .bind(is_active)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
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
                (SELECT COUNT(*) FROM animals WHERE status = 'alive') as active_animals,
                (SELECT COUNT(*) FROM protocols) as total_protocols,
                (SELECT COUNT(*) FROM protocols WHERE status = 'approved') as active_protocols,
                (SELECT COUNT(*) FROM animal_observations WHERE created_at >= now() - INTERVAL '30 days') as observations_30d,
                (SELECT COUNT(*) FROM animal_surgeries WHERE created_at >= now() - INTERVAL '30 days') as surgeries_30d,
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

    pub async fn query_animals(
        pool: &PgPool,
        filters: &serde_json::Value,
        page: i64,
        per_page: i64,
        sort_by: Option<&str>,
        sort_order: &str,
    ) -> Result<(Vec<serde_json::Value>, i64), AppError> {
        let per_page = per_page.clamp(1, 100);
        let offset = (page.max(1) - 1) * per_page;
        let order = if sort_order == "asc" { "ASC" } else { "DESC" };
        let sort_col = validate_sort_field(
            sort_by.unwrap_or("created_at"),
            &["ear_tag", "name", "breed", "status", "sex", "created_at", "birth_date"],
            "created_at",
        );

        let status = filters.get("status").and_then(|v| v.as_str());
        let breed = filters.get("breed").and_then(|v| v.as_str());
        let keyword = filters.get("keyword").and_then(|v| v.as_str());

        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM animals
               WHERE ($1::text IS NULL OR status = $1)
                 AND ($2::text IS NULL OR breed = $2)
                 AND ($3::text IS NULL OR ear_tag ILIKE '%' || $3 || '%' OR name ILIKE '%' || $3 || '%')"#,
        )
        .bind(status)
        .bind(breed)
        .bind(keyword)
        .fetch_one(pool)
        .await?;

        let query_str = format!(
            r#"SELECT id, ear_tag, name, breed, sex, status, birth_date, source_type, iacuc_no, pen_id, created_at
               FROM animals
               WHERE ($1::text IS NULL OR status = $1)
                 AND ($2::text IS NULL OR breed = $2)
                 AND ($3::text IS NULL OR ear_tag ILIKE '%' || $3 || '%' OR name ILIKE '%' || $3 || '%')
               ORDER BY {} {}
               LIMIT $4 OFFSET $5"#,
            sort_col, order
        );

        let rows = sqlx::query_as::<_, (
            uuid::Uuid, Option<String>, Option<String>, Option<String>,
            Option<String>, String, Option<chrono::NaiveDate>, Option<String>,
            Option<String>, Option<uuid::Uuid>, DateTime<Utc>,
        )>(&query_str)
        .bind(status)
        .bind(breed)
        .bind(keyword)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let data: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.0,
                    "ear_tag": r.1,
                    "name": r.2,
                    "breed": r.3,
                    "sex": r.4,
                    "status": r.5,
                    "birth_date": r.6,
                    "source_type": r.7,
                    "iacuc_no": r.8,
                    "pen_id": r.9,
                    "created_at": r.10,
                })
            })
            .collect();

        Ok((data, count.0))
    }

    pub async fn query_observations(
        pool: &PgPool,
        filters: &serde_json::Value,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<serde_json::Value>, i64), AppError> {
        let per_page = per_page.clamp(1, 100);
        let offset = (page.max(1) - 1) * per_page;
        let animal_id = filters
            .get("animal_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());
        let days = filters.get("days").and_then(|v| v.as_i64()).unwrap_or(30);

        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM animal_observations
               WHERE ($1::uuid IS NULL OR animal_id = $1)
                 AND created_at >= now() - make_interval(days => $2::int)"#,
        )
        .bind(animal_id)
        .bind(days as i32)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, String, Option<String>, DateTime<Utc>)>(
            r#"SELECT o.id, o.animal_id, o.observation_type, o.notes, o.created_at
               FROM animal_observations o
               WHERE ($1::uuid IS NULL OR o.animal_id = $1)
                 AND o.created_at >= now() - make_interval(days => $2::int)
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
                    "id": r.0,
                    "animal_id": r.1,
                    "observation_type": r.2,
                    "notes": r.3,
                    "created_at": r.4,
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
        let per_page = per_page.clamp(1, 100);
        let offset = (page.max(1) - 1) * per_page;
        let status = filters.get("status").and_then(|v| v.as_str());

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM protocols WHERE ($1::text IS NULL OR status = $1)",
        )
        .bind(status)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, (uuid::Uuid, String, String, Option<String>, String, DateTime<Utc>)>(
            r#"SELECT id, iacuc_no, title, pi_name, status, created_at
               FROM protocols
               WHERE ($1::text IS NULL OR status = $1)
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
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
        let per_page = per_page.clamp(1, 100);
        let offset = (page.max(1) - 1) * per_page;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM facilities")
            .fetch_one(pool)
            .await?;

        let rows = sqlx::query_as::<_, (uuid::Uuid, String, Option<String>, bool, DateTime<Utc>)>(
            r#"SELECT id, name, description, is_active, created_at
               FROM facilities
               ORDER BY name ASC
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
                    "id": r.0,
                    "name": r.1,
                    "description": r.2,
                    "is_active": r.3,
                    "created_at": r.4,
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
        let per_page = per_page.clamp(1, 100);
        let offset = (page.max(1) - 1) * per_page;
        let animal_id = filters
            .get("animal_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM animal_weights WHERE ($1::uuid IS NULL OR animal_id = $1)",
        )
        .bind(animal_id)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, rust_decimal::Decimal, chrono::NaiveDate, DateTime<Utc>)>(
            r#"SELECT id, animal_id, weight_kg, measured_date, created_at
               FROM animal_weights
               WHERE ($1::uuid IS NULL OR animal_id = $1)
               ORDER BY measured_date DESC
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
                    "weight_kg": r.2,
                    "measured_date": r.3,
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
        let per_page = per_page.clamp(1, 100);
        let offset = (page.max(1) - 1) * per_page;
        let animal_id = filters
            .get("animal_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM animal_surgeries WHERE ($1::uuid IS NULL OR animal_id = $1)",
        )
        .bind(animal_id)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, String, Option<String>, DateTime<Utc>)>(
            r#"SELECT id, animal_id, surgery_type, notes, created_at
               FROM animal_surgeries
               WHERE ($1::uuid IS NULL OR animal_id = $1)
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
                    "surgery_type": r.2,
                    "notes": r.3,
                    "created_at": r.4,
                })
            })
            .collect();

        Ok((data, count.0))
    }
}

/// 驗證排序欄位（防止 SQL injection），僅允許白名單欄位
fn validate_sort_field<'a>(input: &str, allowed: &[&'a str], default: &'a str) -> &'a str {
    if allowed.contains(&input) {
        allowed.iter().find(|&&f| f == input).copied().unwrap_or(default)
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sort_field_allowed() {
        let result = validate_sort_field("ear_tag", &["ear_tag", "name", "created_at"], "created_at");
        assert_eq!(result, "ear_tag");
    }

    #[test]
    fn test_validate_sort_field_disallowed() {
        let result = validate_sort_field("DROP TABLE", &["ear_tag", "name"], "created_at");
        assert_eq!(result, "created_at");
    }
}
