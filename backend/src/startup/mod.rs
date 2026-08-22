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

/// R30-23 / R30-24 共用的 production 判定（High-4 #283：改為 fail-safe 預設）。
///
/// **預設視為 production**：只有 `APP_ENV`（優先）或 `RUST_ENV` 明確標記為
/// dev / debug / test / staging / ci / local 時才視為非 production。
///
/// 動機：原實作「未設定即非 production」，但 prod 以 base `docker-compose.yml` 直接部署
/// （`scripts/deploy-prod.ps1`，不疊 prod.yml）且 .env 從未設 `APP_ENV` → `is_production()`
/// 永遠 false → #283 的 config fail-fast / DB self-test 在真正 prod 從未生效。改為
/// fail-safe 後「漏設環境變數」不再靜默跳過啟動防呆。
///
/// 對應 compose：base `api` 設 `APP_ENV=${APP_ENV:-production}`（dev 可於 .env 覆寫為
/// development）；`docker-compose.test.yml` 的 `api-test` 設 `APP_ENV=test`（CI E2E 不 fail-fast）。
pub fn is_production() -> bool {
    let env = std::env::var("APP_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .unwrap_or_default();
    is_production_from_env(&env)
}

/// 純函式：依環境字串判定是否 production（fail-safe）。抽出供單元測試（避免動 env var）。
fn is_production_from_env(env: &str) -> bool {
    !matches!(
        env.trim().to_ascii_lowercase().as_str(),
        "development" | "dev" | "debug" | "test" | "testing" | "staging" | "ci" | "local"
    )
}

#[cfg(test)]
mod is_production_tests {
    use super::is_production_from_env;

    #[test]
    fn explicit_non_prod_markers_are_not_production() {
        for v in [
            "development",
            "dev",
            "debug",
            "test",
            "testing",
            "staging",
            "ci",
            "local",
            "DEV",
            " Test ",
        ] {
            assert!(!is_production_from_env(v), "{v:?} 應視為非 production");
        }
    }

    #[test]
    fn unset_or_unknown_defaults_to_production() {
        // High-4 #283 回歸：未設定 / 未知值一律 fail-safe 視為 production。
        for v in ["", "production", "prod", "unknown", "anything"] {
            assert!(is_production_from_env(v), "{v:?} 應視為 production");
        }
    }
}
