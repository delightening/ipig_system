//! 修復 migration checksum 不匹配
//!
//! 當 "migration was previously applied but has been modified" 錯誤發生時，
//! 常見原因是 CRLF/LF 換行符差異（Windows vs Unix）。
//!
//! 用法：從 backend 目錄執行
//!   cargo run --bin fix_migration_checksum
//!
//! 會將 _sqlx_migrations 中所有已套用 migration 的 checksum 更新為當前檔案的 checksum。
//! 僅在開發環境使用；若資料重要請先備份。

use sqlx::{migrate::Migrator, PgPool};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;

    // 使用 CARGO_MANIFEST_DIR 確保從 backend 目錄找 migrations
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let migrations_path = Path::new(&manifest_dir).join("migrations");
    let migrator = Migrator::new(migrations_path.as_path()).await?;

    let mut updated = 0;
    for m in migrator.iter().filter(|m| !m.migration_type.is_down_migration()) {
        let version = m.version;
        let checksum: &[u8] = m.checksum.as_ref();
        let description = &m.description;

        let rows = sqlx::query(
            "UPDATE _sqlx_migrations SET checksum = $1, description = $2 WHERE version = $3 RETURNING version",
        )
        .bind(checksum)
        .bind(description)
        .bind(version)
        .fetch_optional(&pool)
        .await?;

        if rows.is_some() {
            println!("[OK] Updated checksum for migration {} ({})", version, description);
            updated += 1;
        }
    }

    if updated > 0 {
        println!("\nFixed {} migration(s). You can now run: sqlx migrate run", updated);
    } else {
        println!("No migrations found in _sqlx_migrations to update.");
    }

    pool.close().await;
    Ok(())
}
