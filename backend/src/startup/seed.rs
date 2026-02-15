// 資料庫 seed 模組
//
// 包含管理員帳號建立、schema 完整性檢查、開發環境預設帳號

use uuid::Uuid;

use crate::services;
use crate::Result;

/// 確保預設管理員帳號存在
/// 密碼從環境變數 ADMIN_INITIAL_PASSWORD 讀取，未設定則跳過建立
pub async fn ensure_admin_user(pool: &sqlx::PgPool) -> Result<()> {
    let email = "admin@ipig.local";
    let display_name = "系統管理員";
    
    // 檢查用戶是否已存在
    let existing_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    
    let user_id = if let Some(id) = existing_id {
        // 用戶已存在：不更新密碼 hash（保留現有密碼）
        tracing::info!("[Admin] Existing admin user found, preserving password: {}", email);
        id
    } else {
        // 用戶不存在：從環境變數取得初始密碼
        let password = std::env::var("ADMIN_INITIAL_PASSWORD")
            .unwrap_or_else(|_| {
                tracing::warn!("[Admin] ADMIN_INITIAL_PASSWORD not set, using generated password");
                Uuid::new_v4().to_string()
            });
        
        let password_hash = services::AuthService::hash_password(&password)
            .map_err(|e| anyhow::anyhow!("Failed to hash admin password: {}", e))?;
        
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password, created_at, updated_at) VALUES ($1, $2, $3, $4, true, true, NOW(), NOW())"
        )
        .bind(id)
        .bind(email)
        .bind(&password_hash)
        .bind(display_name)
        .execute(pool)
        .await?;
        tracing::info!("[Admin] New admin user created: {} (must change password on first login)", email);
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
pub async fn ensure_schema(pool: &sqlx::PgPool) -> Result<()> {
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
    
    // 確保 review_comments 表有 parent_comment_id 欄位（用於審查意見回覆功能）
    sqlx::query(r#"
        DO $$ BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'review_comments' AND column_name = 'parent_comment_id'
            ) THEN
                ALTER TABLE review_comments ADD COLUMN parent_comment_id UUID REFERENCES review_comments(id);
            END IF;
        END $$;
    "#)
    .execute(pool)
    .await?;
    
    // 確保 review_comments 表有 replied_by 欄位（用於記錄回覆者）
    sqlx::query(r#"
        DO $$ BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'review_comments' AND column_name = 'replied_by'
            ) THEN
                ALTER TABLE review_comments ADD COLUMN replied_by UUID REFERENCES users(id);
            END IF;
        END $$;
    "#)
    .execute(pool)
    .await?;
    
    tracing::info!("[Schema] ✓ Schema integrity verified");
    Ok(())
}

/// 開發環境預設帳號資料
struct DevUser {
    email: &'static str,
    display_name: &'static str,
    roles: &'static [&'static str],
}

/// 確保開發環境預設帳號存在（僅在 Docker 開發環境使用）
pub async fn seed_dev_users(pool: &sqlx::PgPool) -> Result<()> {
    // SEC-26: 密碼從環境變數讀取，不再硬編碼弱密碼
    let password = std::env::var("DEV_USER_PASSWORD")
        .unwrap_or_else(|_| {
            let generated = Uuid::new_v4().to_string();
            tracing::warn!(
                "[DevUser] DEV_USER_PASSWORD 未設定，使用隨機密碼: {}",
                generated
            );
            generated
        });
    
    // 使用 AuthService 生成正確的密碼 hash
    let password_hash = services::AuthService::hash_password(&password)
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
            roles: &["EXPERIMENT_STAFF", "WAREHOUSE_MANAGER", "PURCHASING", "ADMIN_STAFF"],
        },        
    ];
    
    for dev_user in &dev_users {
        // 檢查用戶是否已存在
        let existing_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
            .bind(dev_user.email)
            .fetch_optional(pool)
            .await?;
        
        let user_id = if let Some(id) = existing_id {
            // 用戶已存在：不更新密碼 hash（保留現有密碼）
            tracing::info!("[DevUser] Existing user found, preserving password: {}", dev_user.email);
            id
        } else {
            // 用戶不存在：創建新用戶
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password, created_at, updated_at) VALUES ($1, $2, $3, $4, true, true, true, NOW(), NOW())"
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
