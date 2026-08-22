// 資料庫 Migration 執行模組

use sqlx::PgPool;

use crate::config::Config;

/// 執行資料庫 migration；若 SKIP_MIGRATION_CHECK=true 則允許失敗後繼續啟動
pub async fn run_migrations(pool: &PgPool, config: &Config) -> anyhow::Result<()> {
    tracing::info!("[Database] Running migrations...");
    match sqlx::migrate!("./migrations").run(pool).await {
        Ok(_) => {
            tracing::info!("[Database] ✓ Migrations completed successfully");
            Ok(())
        }
        Err(e) => {
            if config.skip_migration_check {
                tracing::warn!(
                    "\n╔═══════════════════════════════════════════════════════════════╗\n\
                     ║        ⚠️  MIGRATION CHECK SKIPPED (SKIP_MIGRATION_CHECK=true)  ║\n\
                     ╠═══════════════════════════════════════════════════════════════╣\n\
                     ║ Database connection: ✓ ESTABLISHED                              ║\n\
                     ║ Database migrations: ⚠️  FAILED (但繼續啟動)                    ║\n\
                     ║ Error: {}                                                       ║\n\
                     ║                                                                 ║\n\
                     ║ 注意：此設定僅用於開發環境，例如從 dump 還原資料庫後。          ║\n\
                     ║ 請確保資料庫結構與 migration 檔案一致。                          ║\n\
                     ╚═══════════════════════════════════════════════════════════════╝",
                    e
                );
                tracing::warn!(
                    "[Database] ⚠️  Migration check failed but continuing (SKIP_MIGRATION_CHECK=true)"
                );
                Ok(())
            } else {
                tracing::error!(
                    "\n╔═══════════════════════════════════════════════════════════════╗\n\
                     ║           API STARTUP FAILED - MIGRATION ERROR                   ║\n\
                     ╠═══════════════════════════════════════════════════════════════╣\n\
                     ║ Database connection: ✓ ESTABLISHED                              ║\n\
                     ║ Database migrations: ❌ FAILED                                 ║\n\
                     ║ Error: {}                                                       ║\n\
                     ║                                                                 ║\n\
                     ║ 提示：如果資料庫結構已存在（例如從 dump 還原），                ║\n\
                     ║ 可以設定 SKIP_MIGRATION_CHECK=true 來跳過檢查。                 ║\n\
                     ╚═══════════════════════════════════════════════════════════════╝",
                    e
                );
                Err(anyhow::anyhow!("Database migration failed: {}", e))
            }
        }
    }
}
