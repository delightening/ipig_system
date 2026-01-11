use std::env;

use sqlx::PgPool;
use uuid::Uuid;

use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: create_test_user <email> <password> <display_name>");
        std::process::exit(1);
    }

    let email = &args[1];
    let password = &args[2];
    let display_name = &args[3];

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPool::connect(&database_url).await?;
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();
    let user_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
        VALUES ($1, $2, $3, $4, true, false)
        ON CONFLICT (email) DO UPDATE SET
            password_hash = EXCLUDED.password_hash,
            display_name = EXCLUDED.display_name,
            is_active = true,
            must_change_password = false,
            updated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(&password_hash)
    .bind(display_name)
    .execute(&pool)
    .await?;

    println!("Test user ready: {} ({})", email, display_name);
    Ok(())
}
