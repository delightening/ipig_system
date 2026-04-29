// 應用程式啟動初始化模組
//
// 從 main.rs 拆分出的初始化邏輯，包含：
// - 資料庫連線建立與重試
// - 管理員帳號與開發環境帳號 seed
// - 權限與角色權限初始化
// - Schema 完整性檢查
// - Tracing 初始化
// - Migration 執行
// - 啟動配置完整性檢查
// - 伺服器組裝與 Graceful Shutdown

mod config_check;
mod database;
mod db_self_test;
mod migration;
mod permissions;
mod security_checks;
mod seed;
pub mod server;
mod tracing;

pub use config_check::log_startup_config_check;
pub use database::create_database_pool_with_retry;
pub use db_self_test::run_db_self_test;
pub use migration::run_migrations;
pub use permissions::{ensure_all_role_permissions, ensure_required_permissions};
pub use security_checks::check_jwt_key_file_permissions;
pub use seed::{ensure_admin_user, ensure_admin_user_after_import, ensure_schema, seed_dev_users};
pub use tracing::init_tracing;

/// R30-23 / R30-24 共用的 production 判定（從 APP_ENV 或 RUST_ENV 讀）。
pub fn is_production() -> bool {
    let app_env = std::env::var("APP_ENV").ok();
    let rust_env = std::env::var("RUST_ENV").ok();
    matches!(
        (app_env.as_deref(), rust_env.as_deref()),
        (Some("production"), _) | (_, Some("production"))
    )
}
