// 資料庫連線建立模組
//
// 包含連線池建立、重試機制與錯誤診斷

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

use crate::config::Config;

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
pub async fn create_database_pool_with_retry(
    config: &Config,
) -> anyhow::Result<sqlx::PgPool> {
    let max_attempts = config.database_retry_attempts;
    let delay_seconds = config.database_retry_delay_seconds;
    
    // 解析資料庫 URL 用於日誌（隱藏密碼）
    let db_url_display = parse_database_url_for_logging(&config.database_url);
    
    tracing::info!(
        "Initializing database connection pool...\n  URL: {}\n  Max connections: {}\n  Min connections: {}\n  Acquire timeout: {}s\n  Retry attempts: {}\n  Retry delay: {}s",
        db_url_display,
        config.database_max_connections,
        config.database_min_connections,
        config.database_acquire_timeout_seconds,
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

        let statement_timeout_ms = config.database_statement_timeout_ms;
        match PgPoolOptions::new()
            .max_connections(config.database_max_connections)
            .min_connections(config.database_min_connections)
            .acquire_timeout(Duration::from_secs(config.database_acquire_timeout_seconds))
            .after_connect(move |conn, _meta| {
                Box::pin(async move {
                    if statement_timeout_ms > 0 {
                        sqlx::query(&format!(
                            "SET statement_timeout = {statement_timeout_ms}"
                        ))
                        .execute(conn)
                        .await?;
                    }
                    Ok(())
                })
            })
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
