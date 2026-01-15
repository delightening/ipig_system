// Quick tool to check _sqlx_migrations table structure

use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    let pool = PgPool::connect(&database_url).await?;
    
    // Query table structure
    let columns: Vec<(String, String)> = sqlx::query_as(
        "SELECT column_name, data_type 
         FROM information_schema.columns 
         WHERE table_schema = 'public' 
         AND table_name = '_sqlx_migrations'
         ORDER BY ordinal_position"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("_sqlx_migrations table structure:");
    for (name, data_type) in &columns {
        println!("  {}: {}", name, data_type);
    }
    
    // Check existing records
    println!("\nExisting migration records:");
    let records: Vec<(i64,)> = sqlx::query_as(
        "SELECT version FROM _sqlx_migrations ORDER BY version"
    )
    .fetch_all(&pool)
    .await?;
    
    if records.is_empty() {
        println!("  (none)");
    } else {
        for (version,) in &records {
            println!("  Version: {}", version);
        }
    }
    
    Ok(())
}
