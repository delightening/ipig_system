use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    
    // Check if users table exists (indicates migrations were applied)
    let users_exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'users'
        )"
    )
    .fetch_one(&pool)
    .await?;
    
    if !users_exists.0 {
        println!("[ERROR] Database schema doesn't exist. Please run migrations first.");
        return Ok(());
    }
    
    println!("[INFO] Database schema exists. Checking migration status...");
    
    // Check current migration status
    let existing_migrations: Vec<(i64, String)> = sqlx::query_as(
        "SELECT version, name FROM _sqlx_migrations ORDER BY version"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("\nExisting migrations in database:");
    if existing_migrations.is_empty() {
        println!("  (none)");
    } else {
        for (version, name) in &existing_migrations {
            println!("  Version {}: {}", version, name);
        }
    }
    
    println!("\n[INFO] The database has tables but no migration records.");
    println!("[INFO] SQLx uses content-based hashing for migration versions.");
    println!("[INFO] To fix this, you have two options:\n");
    println!("Option 1: Use SQLx CLI to resolve migrations:");
    println!("  cargo install sqlx-cli --features postgres");
    println!("  sqlx migrate info --database-url \"{}\"", database_url);
    println!("\nOption 2: Reset the database and re-run migrations:");
    println!("  Use the reset_database.ps1 script or manually:");
    println!("  DROP DATABASE ipig_db;");
    println!("  CREATE DATABASE ipig_db;");
    println!("  Then restart the backend (it will run migrations automatically)\n");
    
    println!("[INFO] Since the schema already exists, the easiest solution is");
    println!("[INFO] to use SQLx CLI's 'migrate resolve' or manually insert");
    println!("[INFO] the correct migration records with the version hashes.");
    
    Ok(())
}
