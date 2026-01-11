use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    
    println!("\n=== 檢查重複權限 ===");
    
    // 檢查重複的 code
    let duplicates: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT code, COUNT(*)::bigint as count
        FROM permissions
        GROUP BY code
        HAVING COUNT(*) > 1
        ORDER BY count DESC, code
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    if duplicates.is_empty() {
        println!("[OK] 沒有發現重複的權限 code");
    } else {
        println!("[WARNING] 發現 {} 個重複的權限 code:\n", duplicates.len());
        for (code, count) in &duplicates {
            println!("  {}: {} 筆記錄", code, count);
            
            // 顯示重複記錄的詳細資訊
            let records: Vec<(uuid::Uuid, String, Option<String>)> = sqlx::query_as(
                "SELECT id, name, description FROM permissions WHERE code = $1 ORDER BY created_at"
            )
            .bind(code)
            .fetch_all(&pool)
            .await?;
            
            for (id, name, description) in &records {
                println!("    - ID: {}, Name: '{}', Description: {:?}", id, name, description);
            }
        }
    }
    
    // 檢查空名稱的權限
    println!("\n=== 檢查空名稱的權限 ===");
    let empty_names: Vec<(String, String)> = sqlx::query_as(
        "SELECT code, name FROM permissions WHERE name = '' OR name IS NULL ORDER BY code"
    )
    .fetch_all(&pool)
    .await?;
    
    if empty_names.is_empty() {
        println!("[OK] 沒有發現空名稱的權限");
    } else {
        println!("[WARNING] 發現 {} 個空名稱的權限:\n", empty_names.len());
        for (code, name) in &empty_names {
            println!("  {}: name = '{}'", code, name);
        }
    }
    
    // 統計總數
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM permissions")
        .fetch_one(&pool)
        .await?;
    
    let unique: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT code)::bigint FROM permissions")
        .fetch_one(&pool)
        .await?;
    
    println!("\n=== 統計資訊 ===");
    println!("總權限數: {}", total.0);
    println!("唯一 code 數: {}", unique.0);
    if total.0 > unique.0 {
        println!("[WARNING] 有 {} 筆重複記錄需要清理", total.0 - unique.0);
    }
    
    Ok(())
}
