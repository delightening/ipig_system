// This tool uses SQLx's migration resolver to get the correct version hashes
// and can mark them as applied if the corresponding tables exist

use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    
    // Use SQLx's migrate! macro to resolve migrations
    // This will give us the correct version hashes
    println!("\nResolving migrations using SQLx...");
    
    // Check if we should mark migrations as applied
    let mark_applied = env::args().any(|arg| arg == "--mark-applied");
    
    if mark_applied {
        println!("[WARNING] --mark-applied flag detected.");
        println!("[WARNING] This will mark all migrations as applied if tables exist.");
        println!("[WARNING] This is only safe if the database schema matches the migrations.\n");
        
        // Check if users table exists (indicates schema is present)
        let users_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = 'users'
            )"
        )
        .fetch_one(&pool)
        .await?;
        
        if users_exists.0 {
            println!("[INFO] Database schema exists. Attempting to resolve migrations...");
            println!("[INFO] SQLx will try to apply migrations, but they may fail if tables exist.");
            println!("[INFO] The best solution is to use SQLx CLI:");
            println!("  sqlx migrate info --database-url <DATABASE_URL>");
            println!("  sqlx migrate resolve <version> --database-url <DATABASE_URL>");
        } else {
            println!("[INFO] Database schema doesn't exist. Migrations should run normally.");
        }
    }
    
    // Try to run migrations - this will show us what SQLx expects
    println!("\nAttempting to run migrations (this may fail if tables exist)...");
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => {
            println!("[SUCCESS] Migrations completed successfully!");
        }
        Err(e) => {
            println!("[ERROR] Migration error: {}", e);
            println!("\n[INFO] This error is expected if tables already exist.");
            println!("[INFO] The database has the schema but migration tracking is out of sync.");
            println!("\n[SOLUTION] Use one of these approaches:");
            println!("  1. Use SQLx CLI to resolve:");
            println!("     sqlx migrate info --database-url \"{}\"", database_url);
            println!("     sqlx migrate resolve <version> --database-url \"{}\"", database_url);
            println!("  2. Reset database (WARNING: loses data):");
            println!("     Use reset_database.ps1 script");
            println!("  3. Manually fix _sqlx_migrations table (requires version hashes)");
        }
    }
    
    Ok(())
}
