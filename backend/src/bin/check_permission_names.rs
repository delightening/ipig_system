use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    
    println!("\n=== 檢查權限名稱重複 ===");
    
    // 檢查重複的名稱（但不同的 code）
    let duplicate_names: Vec<(String, i64, String)> = sqlx::query_as(
        r#"
        SELECT name, COUNT(DISTINCT code)::bigint as code_count, 
               string_agg(code, ', ' ORDER BY code) as codes
        FROM permissions
        GROUP BY name
        HAVING COUNT(DISTINCT code) > 1
        ORDER BY code_count DESC, name
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    if duplicate_names.is_empty() {
        println!("[OK] 沒有發現重複的權限名稱");
    } else {
        println!("[WARNING] 發現 {} 個重複的權限名稱:\n", duplicate_names.len());
        for (name, code_count, codes) in &duplicate_names {
            println!("  '{}': {} 個不同的 code", name, code_count);
            println!("    Codes: {}", codes);
        }
    }
    
    // 檢查 record 相關的權限
    println!("\n=== 檢查 record 相關權限 ===");
    let record_perms: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT code, name, description FROM permissions WHERE code LIKE '%.record.%' ORDER BY code"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("找到 {} 個 record 相關權限:\n", record_perms.len());
    for (code, name, description) in &record_perms {
        println!("  {}: '{}' - {:?}", code, name, description);
    }
    
    Ok(())
}
