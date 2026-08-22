//! 由後端 `permissions` 表產生前端的權限常數檔
//! `frontend/src/lib/permissions.generated.ts`。
//!
//! ## 為什麼
//!
//! 前端手寫權限字串、後端 `require_permission!` 手寫權限字串，兩邊靠人眼對齊。
//! 打錯一個字元的下場是「前端閘比後端鬆」（該藏的按鈕沒藏）或「該顯示的功能消失」，
//! 而兩者都不會有任何編譯期或執行期錯誤 —— 只會在使用者按下去時吃 403。
//! 見 `docs/audit/button-permission-gate-2026-08-07.md` §7-2。
//!
//! ## Usage
//! ```bash
//! # 對任一已跑過 migration + ensure_required_permissions 的 DB
//! cd backend
//! DATABASE_URL_FILE=../secrets/db_url_host.txt cargo run --bin gen_permission_constants
//! # 只想看內容不寫檔
//! cargo run --bin gen_permission_constants -- --stdout
//! ```
//!
//! 產出格式必須與 `tests/permission_constants_sync.rs` 的期待完全一致 ——
//! 兩者共用 [`erp_backend::services::permission_codegen`] 的同一份 render 函式，
//! 不要在任一端另寫一份格式化邏輯。

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;

use erp_backend::services::permission_codegen;

fn read_database_url() -> Result<String> {
    if let Ok(path) = std::env::var("DATABASE_URL_FILE") {
        return Ok(std::fs::read_to_string(&path)
            .with_context(|| format!("read DATABASE_URL_FILE {path}"))?
            .trim()
            .to_string());
    }
    std::env::var("DATABASE_URL").context("DATABASE_URL（或 DATABASE_URL_FILE）must be set")
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let to_stdout = std::env::args().any(|a| a == "--stdout");

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&read_database_url()?)
        .await
        .context("connect db")?;

    let codes = permission_codegen::fetch_permission_codes(&pool)
        .await
        .context("fetch permission codes")?;
    let rendered = permission_codegen::render_ts(&codes);

    if to_stdout {
        print!("{rendered}");
        return Ok(());
    }

    let path = permission_codegen::generated_ts_path();
    std::fs::write(&path, &rendered).with_context(|| format!("write {}", path.display()))?;
    println!("✓ 已寫入 {}（{} 個權限碼）", path.display(), codes.len());
    Ok(())
}
