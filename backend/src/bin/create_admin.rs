use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sqlx::PgPool;
use uuid::Uuid;

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Failed to hash password: {}", e))?
        .to_string();
    Ok(password_hash)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL is not set"))?;

    let pool = PgPool::connect(&database_url).await?;

    let email = "admin@ipigsystem.asia";
    let display_name = "System Admin";
    let password = std::env::var("ADMIN_INITIAL_PASSWORD")
        .map_err(|_| anyhow!("ADMIN_INITIAL_PASSWORD must be set (no default for security)"))?;
    let password_hash = hash_password(&password)?;

    let existing_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&pool)
        .await?;

    let user_id = if let Some(id) = existing_id {
        sqlx::query(
            "UPDATE users SET password_hash = $1, is_active = true, must_change_password = false, updated_at = NOW() WHERE id = $2",
        )
        .bind(&password_hash)
        .bind(id)
        .execute(&pool)
        .await?;
        id
    } else {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password, created_at, updated_at) VALUES ($1, $2, $3, $4, true, false, NOW(), NOW())",
        )
        .bind(id)
        .bind(email)
        .bind(&password_hash)
        .bind(display_name)
        .execute(&pool)
        .await?;
        id
    };

    // 兩個管理員角色代碼取其一，`SYSTEM_ADMIN` 優先。實務上 `roles` 表只有 `admin`
    // （實查 + `backend/migrations/` 全目錄 0 命中），所以走到的一律是後者；
    // 保留 `SYSTEM_ADMIN` 是為了「日後真的建了那個角色時本工具仍指到對的那個」。
    //
    // ⚠️ 原本寫成兩段查詢（先查 SYSTEM_ADMIN、`is_some()` 才不查第二次）。
    // 那個寫法讓 fallback 落在**另一個敘述**裡，`tests/system_admin_role_code_guard.rs`
    // 因此看不到它——守衛把第一段判成「比對 SYSTEM_ADMIN 卻沒有 fallback」。
    // 合併成單一查詢之後例外消失，不需要豁免標記，也少一次 round-trip。
    let role_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM roles WHERE code IN ('SYSTEM_ADMIN', 'admin') \
         ORDER BY CASE code WHEN 'SYSTEM_ADMIN' THEN 0 ELSE 1 END LIMIT 1",
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(role_id) = role_id {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&pool)
        .await?;
    }

    // R7-P0-3: 不輸出密碼至 stdout，避免容器日誌洩露
    println!("Default admin ensured: {}", email);

    Ok(())
}
