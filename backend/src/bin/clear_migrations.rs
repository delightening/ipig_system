// Clear all migration records

use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    
    println!("Clearing all migration records...");
    let result = sqlx::query("DELETE FROM _sqlx_migrations")
        .execute(&pool)
        .await?;
    
    println!("[SUCCESS] Deleted {} migration record(s)", result.rows_affected());
    println!("You can now use SQLx CLI to resolve migrations properly.");
    
    Ok(())
}
