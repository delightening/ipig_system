// 應用程式啟動初始化模組
//
// 從 main.rs 拆分出的初始化邏輯，包含：
// - 資料庫連線建立與重試
// - 管理員帳號與開發環境帳號 seed
// - 權限與角色權限初始化
// - Schema 完整性檢查

mod database;
mod permissions;
mod seed;

pub use database::create_database_pool_with_retry;
pub use permissions::{ensure_all_role_permissions, ensure_required_permissions};
pub use seed::{ensure_admin_user, ensure_admin_user_after_import, ensure_schema, seed_dev_users};
