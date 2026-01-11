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

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow!("DATABASE_URL is not set"))?;

    let pool = PgPool::connect(&database_url).await?;

    let email = "admin@ipig.local";
    let display_name = "System Admin";
    let password = "admin123";
    let password_hash = hash_password(password)?;

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

    let role_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'SYSTEM_ADMIN'")
        .fetch_optional(&pool)
        .await?;

    let role_id = if role_id.is_some() {
        role_id
    } else {
        sqlx::query_scalar("SELECT id FROM roles WHERE code = 'admin'")
            .fetch_optional(&pool)
            .await?
    };

    if let Some(role_id) = role_id {
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(user_id)
            .bind(role_id)
            .execute(&pool)
            .await?;
    }

    println!("Default admin ensured: {} / {}", email, password);

    Ok(())
}
