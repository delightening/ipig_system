# Migration 同步腳本
# 用法: .\scripts\sync_migrations.ps1 [-Method <方法>]
#
# 此腳本提供三種方法來處理 migration 同步問題：
# 1. Clear - 清空 migration 記錄，讓系統重新執行（推薦）
# 2. FixChecksums - 使用 Rust 工具同步現有版本的 checksums
# 3. Manual - 手動處理（顯示詳細資訊）

param(
    [ValidateSet("Clear", "FixChecksums", "Manual")]
    [string]$Method = "Clear"
)

$ErrorActionPreference = "Stop"

# 顏色輸出函數
function Write-Info { param([string]$Message) Write-Host $Message -ForegroundColor Cyan }
function Write-Success { param([string]$Message) Write-Host $Message -ForegroundColor Green }
function Write-Warning { param([string]$Message) Write-Host $Message -ForegroundColor Yellow }
function Write-Error { param([string]$Message) Write-Host $Message -ForegroundColor Red }

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Migration 同步工具" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 讀取環境變數
$envFile = ".env"
if (-not (Test-Path $envFile)) {
    Write-Error "錯誤: 找不到 .env 檔案"
    exit 1
}

$envVars = @{}
Get-Content $envFile | ForEach-Object {
    if ($_ -match '^\s*([^#=]+)=(.*)$') {
        $key = $matches[1].Trim()
        $value = $matches[2].Trim()
        $envVars[$key] = $value
    }
}

$dbUser = $envVars["POSTGRES_USER"]
$dbPassword = $envVars["POSTGRES_PASSWORD"]
$dbName = $envVars["POSTGRES_DB"]

if (-not $dbUser) { $dbUser = "postgres" }
if (-not $dbName) { $dbName = "ipig_db" }
if (-not $dbPassword) {
    Write-Error "錯誤: .env 檔案中未設定 POSTGRES_PASSWORD"
    exit 1
}

$dbContainer = "ipig-db"

# 檢查容器
$containerRunning = docker ps --filter "name=^${dbContainer}$" --format "{{.Names}}" 2>&1
$containerRunning = ($containerRunning | Where-Object { $_ -eq $dbContainer })

if (-not $containerRunning) {
    Write-Error "錯誤: 資料庫容器 '$dbContainer' 未運行"
    exit 1
}

Write-Success "✓ 資料庫容器運行中"

# 檢查 migration 狀態
Write-Info "`n檢查 Migration 狀態..."
$env:PGPASSWORD = $dbPassword
$dbMigrations = docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -t -c `
    "SELECT version, description FROM _sqlx_migrations ORDER BY version;" 2>&1
$env:PGPASSWORD = $null

$migrationFiles = Get-ChildItem -Path "backend/migrations" -Filter "*.sql" | Sort-Object Name
$dbMigrationCount = ($dbMigrations | Measure-Object -Line).Lines
$fileMigrationCount = $migrationFiles.Count

Write-Info "  資料庫 Migration 記錄數: $dbMigrationCount"
Write-Info "  當前 Migration 檔案數: $fileMigrationCount"

if ($dbMigrationCount -eq $fileMigrationCount) {
    Write-Success "  ✓ Migration 數量一致"
} else {
    Write-Warning "  ⚠ Migration 數量不一致（差異: $([math]::Abs($dbMigrationCount - $fileMigrationCount))）"
}

# 顯示詳細資訊
Write-Info "`n資料庫中的 Migration 記錄："
$dbMigrations | ForEach-Object {
    if ($_ -match '^\s*(\d+)\s*\|\s*(.+)$') {
        Write-Host "    版本 $($matches[1]): $($matches[2].Trim())" -ForegroundColor White
    }
}

Write-Info "`n當前 Migration 檔案："
$migrationFiles | ForEach-Object {
    $name = $_.BaseName
    $parts = $name -split '_'
    $version = $parts[0]
    $description = $parts[1..($parts.Length-1)] -join '_'
    Write-Host "    $version : $description" -ForegroundColor White
}

# 根據方法處理
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  執行方法: $Method" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

switch ($Method) {
    "Clear" {
        Write-Info "方法：清空 Migration 記錄"
        Write-Warning "此操作將清空 _sqlx_migrations 表，系統啟動時會重新執行所有 migrations"
        Write-Info "由於資料庫結構已存在，migrations 會快速完成（只會更新追蹤記錄）"
        Write-Host ""
        
        $confirm = Read-Host "輸入 'yes' 確認繼續"
        if ($confirm -ne "yes") {
            Write-Info "已取消"
            exit 0
        }
        
        Write-Info "`n清空 _sqlx_migrations 表..."
        $env:PGPASSWORD = $dbPassword
        $result = docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -c `
            "TRUNCATE TABLE _sqlx_migrations;" 2>&1
        $env:PGPASSWORD = $null
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "  ✓ Migration 記錄已清空"
            Write-Info "`n後續步驟："
            Write-Info "  1. 啟動系統: docker compose up -d"
            Write-Info "  2. 系統會自動重新執行所有 migrations（只會更新追蹤記錄，不會改變結構）"
        } else {
            Write-Error "錯誤: 無法清空 migration 記錄"
            Write-Error $result
            exit 1
        }
    }
    
    "FixChecksums" {
        Write-Info "方法：使用 Rust 工具同步 Checksums"
        Write-Info "此方法會更新現有 migration 版本的 checksums，但不會處理版本不匹配的問題"
        Write-Host ""
        
        if ($dbMigrationCount -ne $fileMigrationCount) {
            Write-Warning "警告: Migration 數量不一致，此方法可能無法完全解決問題"
            Write-Warning "建議使用 Clear 方法"
            Write-Host ""
        }
        
        $confirm = Read-Host "輸入 'yes' 確認繼續"
        if ($confirm -ne "yes") {
            Write-Info "已取消"
            exit 0
        }
        
        Write-Info "`n執行 Rust migration checksum 修復工具..."
        Write-Info "  切換到 backend 目錄..."
        
        Push-Location backend
        
        try {
            # 檢查 DATABASE_URL 環境變數
            if (-not $env:DATABASE_URL) {
                # 從 .env 構建 DATABASE_URL
                $databaseUrl = "postgres://${dbUser}:${dbPassword}@localhost:5433/${dbName}"
                $env:DATABASE_URL = $databaseUrl
                Write-Info "  設定 DATABASE_URL: postgres://${dbUser}:***@localhost:5433/${dbName}"
            }
            
            Write-Info "  執行: cargo run --bin fix_migration_checksum"
            $output = cargo run --bin fix_migration_checksum 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Success "  ✓ Checksum 同步完成"
                Write-Host $output
            } else {
                Write-Error "錯誤: Checksum 同步失敗"
                Write-Host $output
                exit 1
            }
        } finally {
            Pop-Location
            if ($env:DATABASE_URL -match "localhost:5433") {
                $env:DATABASE_URL = $null
            }
        }
    }
    
    "Manual" {
        Write-Info "方法：手動處理"
        Write-Host ""
        Write-Info "由於 Migration 版本不匹配，建議使用以下方法之一："
        Write-Host ""
        Write-Info "選項 1：清空 Migration 記錄（推薦）"
        Write-Host "  docker exec ipig-db psql -U postgres -d ipig_db -c 'TRUNCATE TABLE _sqlx_migrations;'"
        Write-Host "  然後重新啟動系統，讓它重新執行所有 migrations"
        Write-Host ""
        Write-Info "選項 2：使用 Rust 工具同步"
        Write-Host "  cd backend"
        Write-Host "  cargo run --bin fix_migration_checksum"
        Write-Host ""
        Write-Info "選項 3：手動刪除不匹配的 Migration 記錄"
        Write-Host "  只保留與當前 migration 檔案對應的版本（1-10）"
        Write-Host "  刪除版本 11-13 的記錄"
        Write-Host ""
        Write-Info "選項 4：使用 SQLx CLI 標記 migrations"
        Write-Host "  sqlx migrate info --database-url <DATABASE_URL>"
        Write-Host "  sqlx migrate resolve <version> --database-url <DATABASE_URL>"
        Write-Host ""
    }
}

Write-Host ""
Write-Success "========================================"
Write-Success "  處理完成！"
Write-Success "========================================"
Write-Host ""
