use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    
    // Check current migration status
    println!("\nChecking migration status...");
    let migrations: Vec<(i64,)> = sqlx::query_as(
        "SELECT version FROM _sqlx_migrations ORDER BY version"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("\nCurrent migrations in database:");
    for (version,) in &migrations {
        println!("  Version {}", version);
    }
    
    // Check for duplicate versions (primary key violation)
    println!("\nChecking for duplicate versions...");
    let duplicates: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT version, COUNT(*)::bigint as count 
         FROM _sqlx_migrations 
         GROUP BY version 
         HAVING COUNT(*) > 1"
    )
    .fetch_all(&pool)
    .await?;
    
    if !duplicates.is_empty() {
        println!("[WARNING] Found duplicate versions:");
        for (version, count) in &duplicates {
            println!("  Version {}: {} records", version, count);
            // Delete all but keep the one with the lowest row (using ctid)
            sqlx::query(
                "DELETE FROM _sqlx_migrations 
                 WHERE version = $1 
                 AND ctid NOT IN (
                     SELECT MIN(ctid) 
                     FROM _sqlx_migrations 
                     WHERE version = $1
                 )"
            )
            .bind(version)
            .execute(&pool)
            .await?;
            println!("  [FIXED] Removed duplicate records for version {}", version);
        }
    } else {
        println!("[OK] No duplicate versions found.");
    }
    
    let target_version = env::args()
        .nth(1)
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(9);

    // Check for a specific migration version
    println!("\nChecking for migration {}...", target_version);
    let migration_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM _sqlx_migrations WHERE version = $1"
    )
    .bind(target_version)
    .fetch_one(&pool)
    .await?;
    
    if migration_count.0 > 0 {
        println!(
            "Found {} migration {} record(s), deleting...",
            migration_count.0, target_version
        );
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1;")
            .bind(target_version)
            .execute(&pool)
            .await?;
        println!(
            "[SUCCESS] Migration {} record(s) deleted!",
            target_version
        );
    } else {
        println!("No migration {} record found.", target_version);
    }
    
    println!("\n[SUCCESS] Migration table cleaned up!");
    println!("You can now restart the backend service.");
    
    Ok(())
}
