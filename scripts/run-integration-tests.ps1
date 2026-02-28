# iPig 後端整合測試腳本 (Windows)
#
# 前置需求：
# - PostgreSQL 已安裝且可連線
# - 已設定 TEST_DATABASE_URL 或 DATABASE_URL 指向專用測試資料庫
# - 建議使用獨立測試 DB（如 ipig_db_test），避免與開發 DB 混用
#
# 若出現 VersionMismatch(1) 錯誤，表示測試 DB 的 migration 紀錄與程式碼不符。
# 解法：drop 並重建測試 DB，或執行 cargo run --bin fix_migration <version> 後再 sqlx migrate run

$ErrorActionPreference = "Stop"
$BackendDir = Join-Path $PSScriptRoot ".." "backend"

# 檢查環境變數
$dbUrl = $env:TEST_DATABASE_URL
if (-not $dbUrl) {
    $dbUrl = $env:DATABASE_URL
}
if (-not $dbUrl) {
    Write-Host "錯誤：請設定 TEST_DATABASE_URL 或 DATABASE_URL 環境變數" -ForegroundColor Red
    Write-Host "範例：`$env:TEST_DATABASE_URL = 'postgres://user:pass@localhost:5432/ipig_db_test'" -ForegroundColor Yellow
    exit 1
}

Write-Host "使用資料庫：$($dbUrl -replace ':[^:@]+@',':***@')" -ForegroundColor Cyan

# 執行 migration（確保 DB schema 與程式碼一致）
Write-Host "`n執行 sqlx migrate run..." -ForegroundColor Cyan
Push-Location $BackendDir
try {
    sqlx migrate run
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Migration 失敗。若出現 VersionMismatch，請 drop 測試 DB 後重建，或執行 fix_migration。" -ForegroundColor Red
        exit 1
    }

    Write-Host "`n執行 cargo test（整合測試）..." -ForegroundColor Cyan
    cargo test
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
