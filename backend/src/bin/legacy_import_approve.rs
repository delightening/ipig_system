use std::str::FromStr;

use clap::Parser;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "legacy_import_approve",
    about = "Approve legacy import batch: write staged records into formal tables. Dry-run prints report only."
)]
struct Args {
    /// Database URL (e.g. postgres://user:pass@localhost:5432/db)
    #[arg(env = "DATABASE_URL")]
    database_url: String,
    /// Batch ID to approve
    batch_id: String,
    /// Import user email whose id will be used as created_by when recorder is unmatched/ambiguous (required)
    #[arg(long)]
    import_user_email: String,
    /// Actor email for audit trail and permission gate; defaults to ADMIN_EMAIL env or admin@ipigsystem.asia
    #[arg(long)]
    actor_email: Option<String>,
    /// Dry-run: do not write formal tables; prints summary JSON
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    let batch_id = Uuid::from_str(&args.batch_id)
        .map_err(|_| anyhow::anyhow!("Invalid batch_id; expect UUID"))?;

    let pool = PgPool::connect(&args.database_url).await?;

    // Resolve actor (for audit); permission gate is ensured by requiring admin or explicit permission
    let actor_email = args.actor_email.or_else(|| std::env::var("ADMIN_EMAIL").ok()).unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
    let actor_user: Option<(Uuid, bool)> = sqlx::query_as(
        r#"SELECT id, is_admin FROM users WHERE lower(email) = lower($1) AND is_active = true LIMIT 1"#,
    )
    .bind(&actor_email)
    .fetch_optional(&pool)
    .await?;
    let (actor_id, is_admin) = actor_user.ok_or_else(|| anyhow::anyhow!(format!("Actor user not found: {}", actor_email)))?;

    // Permission check: admin short-circuit or explicit permission
    if !is_admin {
        let has_perm: Option<bool> = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
              SELECT 1 FROM role_permissions rp
              JOIN user_roles ur ON ur.role_id = rp.role_id
              JOIN permissions p ON p.id = rp.permission_id
              WHERE ur.user_id = $1 AND p.code = 'animal.legacy_import.approve'
            )"#,
        )
        .bind(actor_id)
        .fetch_one(&pool)
        .await?;
        if !has_perm.unwrap_or(false) {
            return Err(anyhow::anyhow!("actor lacks animal.legacy_import.approve"));
        }
    }

    // Resolve import user id (accept disabled accounts)
    let import_user_id: Option<Uuid> = {
        let uid: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM users WHERE lower(email) = lower($1) LIMIT 1",
        )
        .bind(&args.import_user_email)
        .fetch_optional(&pool)
        .await?;
        if uid.is_none() {
            return Err(anyhow::anyhow!(format!(
                "import user not found: {}",
                args.import_user_email
            )));
        }
        uid
    };

    // Run approval
    let actor = erp_backend::middleware::ActorContext::User(erp_backend::middleware::CurrentUser {
        id: actor_id,
        email: actor_email.clone(),
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });

    let (approved, skipped, conflict) = erp_backend::services::legacy_import::LegacyImportService::approve_batch(
        &pool,
        &actor,
        batch_id,
        import_user_id,
        args.dry_run,
    )
    .await
    .map_err(|e| anyhow::anyhow!(format!("approve failed: {}", e)))?;

    println!(
        "{}",
        serde_json::json!({
            "batch_id": batch_id,
            "dry_run": args.dry_run,
            "approved": approved,
            "skipped": skipped,
            "conflict": conflict
        })
    );

    Ok(())
}

