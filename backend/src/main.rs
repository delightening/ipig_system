use std::sync::Arc;

use axum::{
    http::{header, HeaderValue, Method},
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

use services::scheduler::SchedulerService;
use std::time::Duration;
use uuid::Uuid;

pub use error::{AppError, Result};

/// 解析資料庫 URL 以提取連接資訊（用於日誌，隱藏密碼）
fn parse_database_url_for_logging(url: &str) -> String {
    // 嘗試解析 postgres://user:password@host:port/dbname
    if let Some(at_pos) = url.find('@') {
        if let Some(slash_pos) = url[at_pos..].find('/') {
            let host_part = &url[at_pos + 1..at_pos + slash_pos];
            if let Some(user_part) = url.find("://") {
                let user = &url[user_part + 3..at_pos];
                if let Some(colon_pos) = user.find(':') {
                    let username = &user[..colon_pos];
                    return format!("postgres://{}:***@{}", username, host_part);
                }
            }
        }
    }
    // 如果解析失敗，只顯示前綴
    if url.starts_with("postgres://") {
        "postgres://***@***".to_string()
    } else {
        "***".to_string()
    }
}

/// 確保預設管理員帳號存在
async fn ensure_admin_user(pool: &sqlx::PgPool) -> Result<()> {
    let email = "admin@ipig.local";
    let display_name = "系統管理員";
    let password = "admin123";
    
    // 使用 AuthService 生成正確的密碼 hash
    let password_hash = services::AuthService::hash_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to hash admin password: {}", e))?;
    
    // 檢查用戶是否已存在
    let existing_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    
    let user_id = if let Some(id) = existing_id {
        // 用戶已存在：更新密碼 hash 和狀態（確保密碼正確）
        sqlx::query(
            "UPDATE users SET password_hash = $1, is_active = true, must_change_password = false, updated_at = NOW() WHERE id = $2"
        )
        .bind(&password_hash)
        .bind(id)
        .execute(pool)
        .await?;
        tracing::info!("[Admin] Existing admin user password reset: {}", email);
        id
    } else {
        // 用戶不存在：創建新用戶
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password, created_at, updated_at) VALUES ($1, $2, $3, $4, true, false, NOW(), NOW())"
        )
        .bind(id)
        .bind(email)
        .bind(&password_hash)
        .bind(display_name)
        .execute(pool)
        .await?;
        tracing::info!("[Admin] New admin user created: {} / {}", email, password);
        id
    };
    
    // 確保用戶有管理員角色
    let role_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'SYSTEM_ADMIN' OR code = 'admin' LIMIT 1")
        .fetch_optional(pool)
        .await?;
    
    if let Some(role_id) = role_id {
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(user_id)
            .bind(role_id)
            .execute(pool)
            .await?;
    }
    
    Ok(())
}

/// 確保資料庫 schema 完整（用程式碼取代部分 migration）
async fn ensure_schema(pool: &sqlx::PgPool) -> Result<()> {
    // 確保 pigs 表有 breed_other 欄位（用於 breed = 'other' 時存放自訂品種名）
    sqlx::query(r#"
        DO $$ BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'pigs' AND column_name = 'breed_other'
            ) THEN
                ALTER TABLE pigs ADD COLUMN breed_other VARCHAR(100);
            END IF;
        END $$;
    "#)
    .execute(pool)
    .await?;
    
    tracing::info!("[Schema] ✓ Schema integrity verified");
    Ok(())
}

/// 確保必要的權限存在於資料庫
/// 用於補充 migration 中未包含的權限
async fn ensure_required_permissions(pool: &sqlx::PgPool) -> Result<()> {
    // 需要確保存在的權限清單
    let required_permissions = vec![
        ("pig.source.manage", "管理豬隻來源", "pig", "可管理豬隻來源資料"),
    ];
    
    for (code, name, module, description) in required_permissions {
        sqlx::query(r#"
            INSERT INTO permissions (id, code, name, module, description, created_at)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())
            ON CONFLICT (code) DO NOTHING
        "#)
        .bind(code)
        .bind(name)
        .bind(module)
        .bind(description)
        .execute(pool)
        .await?;
    }
    
    tracing::info!("[Permissions] ✓ Required permissions verified");
    Ok(())
}

/// 確保所有角色擁有正確的權限
/// 在程式啟動時自動配置所有系統角色的權限
async fn ensure_all_role_permissions(pool: &sqlx::PgPool) -> Result<()> {
    // 定義每個角色的權限
    let role_permissions: Vec<(&str, Vec<&str>)> = vec![
        // ============================================
        // WAREHOUSE_MANAGER (倉庫管理員) - ERP 完整權限
        // ============================================
        ("WAREHOUSE_MANAGER", vec![
            // 倉庫管理
            "erp.warehouse.view", "erp.warehouse.create", "erp.warehouse.edit",
            // 產品管理
            "erp.product.view", "erp.product.create", "erp.product.edit",
            // 夥伴管理
            "erp.partner.view", "erp.partner.create", "erp.partner.edit",
            // 單據管理
            "erp.document.view", "erp.document.create", "erp.document.edit", 
            "erp.document.submit", "erp.document.approve",
            // 庫存管理
            "erp.inventory.view",
            // 採購
            "erp.purchase.create", "erp.purchase.approve",
            "erp.grn.create", "erp.pr.create",
            // 庫存操作
            "erp.stock.in", "erp.stock.out", "erp.stock.view", 
            "erp.stock.adjust", "erp.stock.transfer",
            "erp.stocktake.create",
            // 報表
            "erp.report.view", "erp.report.export", "erp.report.download",
        ]),
        
        // ============================================
        // PURCHASING (採購人員) - 採購相關權限
        // ============================================
        ("PURCHASING", vec![
            // 基本查詢
            "erp.warehouse.view", "erp.product.view",
            // 夥伴管理
            "erp.partner.view", "erp.partner.create", "erp.partner.edit",
            // 單據管理
            "erp.document.view", "erp.document.create", "erp.document.edit", "erp.document.submit",
            // 庫存查詢
            "erp.inventory.view",
            // 採購
            "erp.purchase.create", "erp.grn.create",
            // 庫存查詢
            "erp.stock.view",
            // 報表
            "erp.report.view",
        ]),
        
        // ============================================
        // PI (計畫主持人) - 計畫管理、豬隻查看
        // ============================================
        ("PI", vec![
            // 計畫管理
            "aup.protocol.view_own", "aup.protocol.create", "aup.protocol.edit", 
            "aup.protocol.submit", "aup.protocol.delete",
            // 審查流程
            "aup.review.view",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download", "aup.attachment.upload",
            // 版本管理
            "aup.version.view",
            // 豬隻管理
            "pig.pig.view_project",
            "pig.record.view",
            // 匯出
            "pig.export.medical", "pig.export.observation", "pig.export.surgery",
        ]),
        
        // ============================================
        // VET (獸醫師) - 審查計畫、豬隻管理、獸醫建議
        // ============================================
        ("VET", vec![
            // 計畫審查
            "aup.protocol.view_all", "aup.protocol.review",
            // 審查流程
            "aup.review.view", "aup.review.comment",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download",
            // 版本管理
            "aup.version.view",
            // 豬隻管理
            "pig.pig.view_all", "pig.pig.edit", "pig.pig.assign",
            "pig.record.view", "pig.record.create", "pig.record.edit",
            // 獸醫師功能
            "pig.vet.recommend", "pig.vet.read",
            // 匯出
            "pig.export.medical", "pig.export.observation", "pig.export.surgery", "pig.export.experiment",
        ]),
        
        // ============================================
        // REVIEWER (審查委員) - 計畫審查權限（僅能看被指派的計畫）+ 可查看所有豬隻
        // ============================================
        ("REVIEWER", vec![
            // 計畫審查（只能看自己被指派的）
            "aup.protocol.view_own", "aup.protocol.review",
            // 審查流程
            "aup.review.view", "aup.review.comment",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download",
            // 版本管理
            "aup.version.view",
            // 豬隻管理 - 僅查看，不含來源管理
            "pig.pig.view_all",
            "pig.record.view",
        ]),
        
        // ============================================
        // CHAIR (IACUC 主席) - 計畫核准、審查人員指派 + 可查看所有豬隻
        // ============================================
        ("CHAIR", vec![
            // 計畫管理
            "aup.protocol.view_all", "aup.protocol.review", 
            "aup.protocol.approve", "aup.protocol.change_status",
            // 審查流程
            "aup.review.view", "aup.review.comment", "aup.review.assign",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download",
            // 版本管理
            "aup.version.view",
            // 豬隻管理 - 僅查看，不含來源管理
            "pig.pig.view_all",
            "pig.record.view",
        ]),
        
        // ============================================
        // IACUC_STAFF (執行秘書) - 計畫管理、狀態變更
        // ============================================
        ("IACUC_STAFF", vec![
            // 計畫管理
            "aup.protocol.view_all", "aup.protocol.create", "aup.protocol.edit", 
            "aup.protocol.change_status",
            // 審查流程
            "aup.review.view", "aup.review.assign",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download", "aup.attachment.upload",
            // 版本管理
            "aup.version.view",
            // 豬隻管理
            "pig.pig.view_all", "pig.pig.create", "pig.pig.edit", 
            "pig.pig.assign", "pig.pig.import", "pig.pig.delete",
            "pig.record.view", "pig.record.create", "pig.record.edit", "pig.record.delete",
            // 豬隻來源管理
            "pig.source.manage",
            // 匯出
            "pig.export.medical", "pig.export.observation", "pig.export.surgery", "pig.export.experiment",
        ]),
        
        // ============================================
        // EXPERIMENT_STAFF (試驗工作人員) - 計畫編輯、豬隻紀錄、ERP 查詢
        // ============================================
        ("EXPERIMENT_STAFF", vec![
            // 計畫管理（含 co-editor 權限）
            "aup.protocol.view_own", "aup.protocol.create", "aup.protocol.edit", 
            "aup.protocol.submit", "aup.protocol.delete",
            // 審查流程
            "aup.review.view", "aup.review.reply",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download", 
            "aup.attachment.upload", "aup.attachment.delete",
            // 版本管理
            "aup.version.view",
            // 豬隻管理 - 可查看所有豬隻
            "pig.pig.view_all",
            "pig.record.view", "pig.record.create", "pig.record.edit",
            "pig.record.observation", "pig.record.surgery", 
            "pig.record.weight", "pig.record.vaccine", "pig.record.sacrifice",
            // 豬隻來源管理
            "pig.source.manage",
            // 匯出
            "pig.export.observation", "pig.export.surgery", "pig.export.experiment",
            // ERP 查詢（僅讀取）- 倉庫、產品、夥伴、庫存
            "erp.warehouse.view", "erp.product.view", "erp.partner.view",
            "erp.inventory.view", "erp.stock.view",
            // Dashboard 權限
            "dashboard.view",
        ]),
        
        // ============================================
        // CLIENT (委託人) - 計畫/豬隻查看（僅自己相關）
        // ============================================
        ("CLIENT", vec![
            // 計畫查看
            "aup.protocol.view_own",
            // 審查流程
            "aup.review.view",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download",
            // 版本管理
            "aup.version.view",
            // 豬隻查看
            "pig.pig.view_project",
            "pig.record.view",
            // 匯出
            "pig.export.medical", "pig.export.observation", "pig.export.surgery",
        ]),
    ];
    
    let mut total_assigned = 0;
    
    for (role_code, permissions) in &role_permissions {
        // 為角色分配權限
        let result = sqlx::query(r#"
            INSERT INTO role_permissions (role_id, permission_id)
            SELECT r.id, p.id
            FROM roles r
            CROSS JOIN permissions p
            WHERE r.code = $1
            AND p.code = ANY($2::text[])
            ON CONFLICT DO NOTHING
        "#)
        .bind(role_code)
        .bind(&permissions[..])
        .execute(pool)
        .await?;
        
        let assigned = result.rows_affected();
        if assigned > 0 {
            tracing::debug!("[Permissions] {} -> {} new permissions", role_code, assigned);
            total_assigned += assigned;
        }
    }
    
    if total_assigned > 0 {
        tracing::info!("[Permissions] ✓ {} total permissions assigned to all roles", total_assigned);
    } else {
        tracing::info!("[Permissions] ✓ All role permissions already configured");
    }
    
    Ok(())
}

/// 開發環境預設帳號資料
struct DevUser {
    email: &'static str,
    display_name: &'static str,
    roles: &'static [&'static str],
}

/// 確保開發環境預設帳號存在（僅在 Docker 開發環境使用）
async fn seed_dev_users(pool: &sqlx::PgPool) -> Result<()> {
    let password = "12345678";
    
    // 使用 AuthService 生成正確的密碼 hash
    let password_hash = services::AuthService::hash_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to hash dev user password: {}", e))?;
    
    // 開發環境預設帳號列表
    let dev_users = [
        DevUser {
            email: "monkey20531@gmail.com",
            display_name: "怡均",
            roles: &["IACUC_STAFF", "EXPERIMENT_STAFF"],
        },
        DevUser {
            email: "lisa82103031@gmail.com",
            display_name: "莉珊",
            roles: &["EXPERIMENT_STAFF"],
        },
        DevUser {
            email: "museum1925@gmail.com",
            display_name: "芮蓁",
            roles: &["EXPERIMENT_STAFF"],
        },
        DevUser {
            email: "keytyne@gmail.com",
            display_name: "映潔",
            roles: &["EXPERIMENT_STAFF", "WAREHOUSE_MANAGER"],
        },
        DevUser {
            email: "raying80@gmail.com",
            display_name: "永發",
            roles: &["EXPERIMENT_STAFF"],
        },
        DevUser {
            email: "smen1971@gmail.com",
            display_name: "意萍",
            roles: &["EXPERIMENT_STAFF", "WAREHOUSE_MANAGER", "PURCHASING"],
        },
    ];
    
    for dev_user in &dev_users {
        // 檢查用戶是否已存在
        let existing_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
            .bind(dev_user.email)
            .fetch_optional(pool)
            .await?;
        
        let user_id = if let Some(id) = existing_id {
            // 用戶已存在：更新密碼 hash（確保密碼正確）
            sqlx::query(
                "UPDATE users SET password_hash = $1, is_active = true, must_change_password = false, updated_at = NOW() WHERE id = $2"
            )
            .bind(&password_hash)
            .bind(id)
            .execute(pool)
            .await?;
            tracing::info!("[DevUser] Updated password for existing user: {}", dev_user.email);
            id
        } else {
            // 用戶不存在：創建新用戶
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password, created_at, updated_at) VALUES ($1, $2, $3, $4, true, true, false, NOW(), NOW())"
            )
            .bind(id)
            .bind(dev_user.email)
            .bind(&password_hash)
            .bind(dev_user.display_name)
            .execute(pool)
            .await?;
            tracing::info!("[DevUser] Created dev user: {} ({})", dev_user.display_name, dev_user.email);
            id
        };
        
        // 清除現有角色並重新指派
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;
        
        // 指派角色
        for role_code in dev_user.roles {
            let role_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM roles WHERE code = $1")
                .bind(*role_code)
                .fetch_optional(pool)
                .await?;
            
            if let Some(role_id) = role_id {
                sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                    .bind(user_id)
                    .bind(role_id)
                    .execute(pool)
                    .await?;
            } else {
                tracing::warn!("[DevUser] Role not found: {}", role_code);
            }
        }
    }
    
    tracing::info!("[DevUser] ✓ {} dev users seeded successfully", dev_users.len());
    Ok(())
}

/// 診斷資料庫連接錯誤類型
fn diagnose_database_error(error: &sqlx::Error) -> String {
    match error {
        sqlx::Error::Configuration(e) => format!("配置錯誤: {}", e),
        sqlx::Error::Database(e) => {
            let code = e.code().map(|c| c.to_string()).unwrap_or_else(|| "UNKNOWN".to_string());
            let message = e.message();
            format!("資料庫錯誤 [{}]: {}", code, message)
        }
        sqlx::Error::Io(e) => format!("網路/IO 錯誤: {} (可能原因: 資料庫服務未啟動、網路不通、防火牆阻擋)", e),
        sqlx::Error::Tls(e) => format!("TLS/SSL 錯誤: {}", e),
        sqlx::Error::Protocol(e) => format!("協議錯誤: {}", e),
        sqlx::Error::RowNotFound => "找不到資料列".to_string(),
        sqlx::Error::TypeNotFound { type_name } => format!("類型未找到: {}", type_name),
        sqlx::Error::ColumnIndexOutOfBounds { index, len } => {
            format!("欄位索引超出範圍: {} (長度: {})", index, len)
        }
        sqlx::Error::ColumnNotFound(name) => format!("欄位未找到: {}", name),
        sqlx::Error::ColumnDecode { index, source } => {
            format!("欄位解碼錯誤 (索引 {}): {}", index, source)
        }
        sqlx::Error::Decode(e) => format!("解碼錯誤: {}", e),
        sqlx::Error::PoolTimedOut => "連線池超時".to_string(),
        sqlx::Error::PoolClosed => "連線池已關閉".to_string(),
        sqlx::Error::WorkerCrashed => "背景工作程序崩潰".to_string(),
        _ => format!("未知錯誤: {}", error),
    }
}

/// 建立資料庫連線池，包含重試機制
/// 適用於 Docker Compose 環境，當資料庫尚未就緒時自動重試
async fn create_database_pool_with_retry(
    config: &config::Config,
) -> anyhow::Result<sqlx::PgPool> {
    let max_attempts = config.database_retry_attempts;
    let delay_seconds = config.database_retry_delay_seconds;
    
    // 解析資料庫 URL 用於日誌（隱藏密碼）
    let db_url_display = parse_database_url_for_logging(&config.database_url);
    
    tracing::info!(
        "Initializing database connection pool...\n  URL: {}\n  Max connections: {}\n  Retry attempts: {}\n  Retry delay: {}s",
        db_url_display,
        config.database_max_connections,
        max_attempts,
        delay_seconds
    );

    let mut last_error: Option<sqlx::Error> = None;

    for attempt in 1..=max_attempts {
        tracing::info!(
            "[Database Connection] Attempt {}/{}: Connecting to database...",
            attempt,
            max_attempts
        );

        match PgPoolOptions::new()
            .max_connections(config.database_max_connections)
            .connect(&config.database_url)
            .await
        {
            Ok(pool) => {
                // 驗證連線是否真的可用
                match sqlx::query("SELECT 1").execute(&pool).await {
                    Ok(_) => {
                        tracing::info!(
                            "[Database Connection] ✓ Successfully connected and verified (attempt {}/{})",
                            attempt,
                            max_attempts
                        );
                        return Ok(pool);
                    }
                    Err(e) => {
                        let diagnosis = diagnose_database_error(&e);
                        tracing::warn!(
                            "[Database Connection] Connection established but verification failed: {}",
                            diagnosis
                        );
                        last_error = Some(e);
                    }
                }
            }
            Err(e) => {
                let diagnosis = diagnose_database_error(&e);
                last_error = Some(e);
                
                if attempt < max_attempts {
                    tracing::warn!(
                        "[Database Connection] ✗ Attempt {}/{} failed: {}\n  Retrying in {} seconds...",
                        attempt,
                        max_attempts,
                        diagnosis,
                        delay_seconds
                    );
                    tokio::time::sleep(Duration::from_secs(delay_seconds)).await;
                } else {
                    tracing::error!(
                        "[Database Connection] ✗ All {} attempts failed. Last error: {}",
                        max_attempts,
                        diagnosis
                    );
                }
            }
        }
    }

    // 所有重試都失敗，輸出詳細診斷資訊
    let error_msg = if let Some(ref e) = last_error {
        diagnose_database_error(e)
    } else {
        "Unknown error".to_string()
    };

    tracing::error!(
        "\n╔═══════════════════════════════════════════════════════════════╗\n\
         ║           DATABASE CONNECTION FAILED - DIAGNOSIS                ║\n\
         ╠═══════════════════════════════════════════════════════════════╣\n\
         ║ Connection URL: {}                              ║\n\
         ║ Error: {}                                        ║\n\
         ║                                                                 ║\n\
         ║ Troubleshooting steps:                                          ║\n\
         ║ 1. Check if database service is running                        ║\n\
         ║ 2. Verify DATABASE_URL is correct                               ║\n\
         ║ 3. Check network connectivity                                   ║\n\
         ║ 4. Verify database credentials                                  ║\n\
         ║ 5. Check firewall rules                                          ║\n\
         ║ 6. In Docker: ensure 'depends_on' with healthcheck is set      ║\n\
         ╚═══════════════════════════════════════════════════════════════╝",
        db_url_display,
        error_msg
    );

    Err(anyhow::anyhow!(
        "Database connection failed after {} attempts. Error: {}",
        max_attempts,
        error_msg
    ))
}

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<config::Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "erp_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = config::Config::from_env()?;
    let config = Arc::new(config);

    // Create database pool with retry logic
    let pool = match create_database_pool_with_retry(&config).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!(
                "\n╔═══════════════════════════════════════════════════════════════╗\n\
                 ║              API STARTUP FAILED - DATABASE ERROR                ║\n\
                 ╠═══════════════════════════════════════════════════════════════╣\n\
                 ║ The API server cannot start because database connection       ║\n\
                 ║ failed. Please check the error messages above for details.    ║\n\
                 ║                                                                 ║\n\
                 ║ Database Status: ❌ UNAVAILABLE                                ║\n\
                 ║ Error: {}                                                       ║\n\
                 ╚═══════════════════════════════════════════════════════════════╝",
                e
            );
            return Err(e);
        }
    };

    // Run migrations
    tracing::info!("[Database] Running migrations...");
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => {
            tracing::info!("[Database] ✓ Migrations completed successfully");
        }
        Err(e) => {
            tracing::error!(
                "\n╔═══════════════════════════════════════════════════════════════╗\n\
                 ║           API STARTUP FAILED - MIGRATION ERROR                   ║\n\
                 ╠═══════════════════════════════════════════════════════════════╣\n\
                 ║ Database connection: ✓ ESTABLISHED                              ║\n\
                 ║ Database migrations: ❌ FAILED                                 ║\n\
                 ║ Error: {}                                                       ║\n\
                 ╚═══════════════════════════════════════════════════════════════╝",
                e
            );
            return Err(anyhow::anyhow!("Database migration failed: {}", e));
        }
    }

    tracing::info!("[Database] ✓ Connection established and migrations completed");

    // 確保 schema 完整性（程式碼方式補充 migration 未涵蓋的欄位）
    if let Err(e) = ensure_schema(&pool).await {
        tracing::warn!("Failed to ensure schema (non-fatal): {}", e);
    }

    // Ensure default admin user exists
    if let Err(e) = ensure_admin_user(&pool).await {
        tracing::warn!("Failed to ensure admin user (non-fatal): {}", e);
    }

    // 確保必要的權限存在
    if let Err(e) = ensure_required_permissions(&pool).await {
        tracing::warn!("Failed to ensure required permissions (non-fatal): {}", e);
    }

    // 確保所有角色擁有正確的權限
    if let Err(e) = ensure_all_role_permissions(&pool).await {
        tracing::warn!("Failed to ensure role permissions (non-fatal): {}", e);
    }

    // Seed development users if enabled
    if config.seed_dev_users {
        tracing::info!("[DevUser] SEED_DEV_USERS is enabled, seeding development users...");
        if let Err(e) = seed_dev_users(&pool).await {
            tracing::warn!("Failed to seed dev users (non-fatal): {}", e);
        }
    }

    // Start scheduler for background tasks
    let scheduler_result = SchedulerService::start(pool.clone(), config.clone()).await;
    match scheduler_result {
        Ok(_scheduler) => {
            tracing::info!("Background scheduler started");
        }
        Err(e) => {
            tracing::warn!("Failed to start scheduler (non-fatal): {}", e);
        }
    }

    // Create app state
    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:8080"),
            HeaderValue::from_static("http://10.0.4.34:8080"),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);

    // Build trace layer
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    // Build router
    let app = routes::api_routes(state)
        .layer(cors)
        .layer(trace_layer);

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
