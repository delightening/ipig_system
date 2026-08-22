//! R28-5 Phase B: Backfill `hmac_version` for pre-migration-037 rows.
//!
//! For each `user_activity_logs` row where `hmac_version IS NULL AND
//! integrity_hash IS NOT NULL`, looks up the immediately preceding row's
//! integrity_hash (the chain link), recomputes HMAC with v2 then v1, and sets
//! `hmac_version` to whichever matches.
//!
//! The immutability trigger only protects content columns — `hmac_version` is
//! outside the guard, so UPDATE is safe without disabling the trigger.
//!
//! ## Usage
//! ```bash
//! cargo run --bin backfill_hmac_version
//! cargo run --bin backfill_hmac_version -- --dry-run   # preview only
//! ```

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct BackfillRow {
    id: Uuid,
    partition_date: chrono::NaiveDate,
    created_at: chrono::DateTime<chrono::Utc>,
    event_category: String,
    event_type: String,
    actor_user_id: Option<Uuid>,
    before_data: Option<serde_json::Value>,
    after_data: Option<serde_json::Value>,
    impersonated_by_user_id: Option<Uuid>,
    changed_fields: Option<Vec<String>>,
    integrity_hash: Option<String>,
    extra_input: Option<String>,
    entity_type: Option<String>,
    entity_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let dry_run = std::env::args().any(|a| a == "--dry-run");

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set in .env")?;
    let hmac_key = std::env::var("AUDIT_HMAC_KEY").context("AUDIT_HMAC_KEY must be set in .env")?;

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    let rows: Vec<BackfillRow> = sqlx::query_as(
        r#"
        SELECT id, partition_date, created_at, event_category, event_type,
               actor_user_id, before_data, after_data, impersonated_by_user_id,
               changed_fields, integrity_hash,
               extra_input, entity_type, entity_id::text AS entity_id
        FROM user_activity_logs
        WHERE hmac_version IS NULL
          AND integrity_hash IS NOT NULL
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .context("Failed to query NULL hmac_version rows")?;

    println!(
        "Found {} rows with hmac_version=NULL and integrity_hash IS NOT NULL",
        rows.len()
    );

    if rows.is_empty() {
        println!("Nothing to backfill.");
        return Ok(());
    }

    let mut v2_count = 0u32;
    let mut v1_count = 0u32;
    let mut mismatch_count = 0u32;
    let system_user_id = erp_backend::middleware::SYSTEM_USER_ID;

    for row in &rows {
        let stored_hash = match row.integrity_hash.as_deref() {
            Some(h) => h,
            None => continue,
        };

        let prev_hash: Option<String> = sqlx::query_scalar(
            r#"
            SELECT integrity_hash FROM user_activity_logs
            WHERE (created_at, id) < ($1, $2)
              AND partition_date <= $1::date
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(row.created_at)
        .bind(row.id)
        .fetch_optional(&pool)
        .await?
        .flatten();

        let changed_fields = row.changed_fields.as_deref().unwrap_or(&[]);
        let actor_user_id = row.actor_user_id.unwrap_or(system_user_id);

        let build_input = || erp_backend::services::audit::HmacInput {
            event_category: &row.event_category,
            event_type: &row.event_type,
            actor_user_id,
            before_data: &row.before_data,
            after_data: &row.after_data,
            impersonated_by: row.impersonated_by_user_id,
            changed_fields,
            entity_type: row.entity_type.as_deref(),
            entity_id: row.entity_id.as_deref(),
            extra_input: row.extra_input.as_deref(),
        };

        let v2_hash =
            erp_backend::services::audit::AuditService::compute_hmac_for_fields_versioned(
                &hmac_key,
                prev_hash.as_deref(),
                build_input(),
                2,
            )?;

        let version = if v2_hash == stored_hash {
            v2_count += 1;
            2i16
        } else {
            let v1_hash =
                erp_backend::services::audit::AuditService::compute_hmac_for_fields_versioned(
                    &hmac_key,
                    prev_hash.as_deref(),
                    build_input(),
                    1,
                )?;

            if v1_hash == stored_hash {
                v1_count += 1;
                1i16
            } else {
                mismatch_count += 1;
                eprintln!(
                    "MISMATCH: row {} (created_at={}) — neither v1 nor v2 match",
                    row.id, row.created_at
                );
                continue;
            }
        };

        if !dry_run {
            sqlx::query(
                "UPDATE user_activity_logs SET hmac_version = $1 \
                 WHERE id = $2 AND partition_date = $3",
            )
            .bind(version)
            .bind(row.id)
            .bind(row.partition_date)
            .execute(&pool)
            .await
            .with_context(|| format!("Failed to update row {}", row.id))?;
        }
    }

    println!("\n=== Backfill Summary ===");
    println!("v2 (canonical): {v2_count}");
    println!("v1 (legacy):    {v1_count}");
    println!("mismatch:       {mismatch_count}");
    if dry_run {
        println!("(dry-run mode — no rows updated)");
    } else {
        println!("Updated {} rows total.", v2_count + v1_count);
    }

    Ok(())
}
