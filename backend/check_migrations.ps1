# 檢查資料庫遷移狀態
# 用於檢查遷移是否已正確運行

Write-Host "=== 資料庫遷移狀態檢查 ===" -ForegroundColor Cyan
Write-Host ""

# 讀取 .env 檔案
$envFile = ".env"
if (-not (Test-Path $envFile)) {
    Write-Host "[ERROR] .env 檔案不存在" -ForegroundColor Red
    exit 1
}

$envContent = Get-Content $envFile
$databaseUrl = $envContent | Select-String -Pattern "^DATABASE_URL=(.+)$" | ForEach-Object { $_.Matches.Groups[1].Value }

if (-not $databaseUrl) {
    Write-Host "[ERROR] 無法從 .env 檔案中讀取 DATABASE_URL" -ForegroundColor Red
    exit 1
}

Write-Host "資料庫 URL: $($databaseUrl -replace ':[^:@]+@', ':***@')" -ForegroundColor Gray
Write-Host ""

# 檢查 psql 是否可用
try {
    $null = Get-Command psql -ErrorAction Stop
    $psqlAvailable = $true
} catch {
    $psqlAvailable = $false
}

if (-not $psqlAvailable) {
    Write-Host "[WARNING] psql 不可用，無法直接查詢資料庫" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "請安裝 PostgreSQL 客戶端工具，或使用以下方法之一：" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "方法 1: 使用 SQLx CLI" -ForegroundColor Yellow
    Write-Host "  cargo install sqlx-cli --features postgres" -ForegroundColor White
    Write-Host "  sqlx migrate info --database-url `"$databaseUrl`"" -ForegroundColor White
    Write-Host ""
    Write-Host "方法 2: 查看伺服器啟動日誌" -ForegroundColor Yellow
    Write-Host "  查找 '[Database] ✓ Migrations completed successfully' 消息" -ForegroundColor White
    Write-Host ""
    Write-Host "方法 3: 手動連接到資料庫" -ForegroundColor Yellow
    Write-Host "  檢查 _sqlx_migrations 表中的記錄" -ForegroundColor White
    exit 0
}

# 解析資料庫 URL
if ($databaseUrl -match "postgres://([^:]+):([^@]+)@([^:]+):(\d+)/(.+)") {
    $user = $matches[1]
    $password = $matches[2]
    $dbHost = $matches[3]
    $port = $matches[4]
    $database = $matches[5]
    
    Write-Host "連接到資料庫: $database @ $dbHost`:$port" -ForegroundColor Cyan
    Write-Host ""
    
    $env:PGPASSWORD = $password
    
    # 查詢遷移狀態
    Write-Host "查詢遷移記錄..." -ForegroundColor Yellow
    $migrations = psql -h $dbHost -p $port -U $user -d $database -t -c "SELECT version, name, applied_at FROM _sqlx_migrations ORDER BY version;" 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[SUCCESS] 遷移記錄：" -ForegroundColor Green
        Write-Host ""
        $migrations | Where-Object { $_.Trim() -ne '' } | ForEach-Object {
            $parts = $_ -split '\|'
            if ($parts.Length -eq 3) {
                $version = $parts[0].Trim()
                $name = $parts[1].Trim()
                $appliedAt = $parts[2].Trim()
                Write-Host "  版本 $version : $name" -ForegroundColor White
                Write-Host "    應用時間: $appliedAt" -ForegroundColor Gray
            }
        }
        Write-Host ""
        
        # 檢查遷移 004
        Write-Host "檢查遷移 004 (pig_observations 表的 deleted_at 和 copied_from_id 欄位)..." -ForegroundColor Yellow
        $migration004 = psql -h $dbHost -p $port -U $user -d $database -t -c "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 4;" 2>&1
        
        if ($LASTEXITCODE -eq 0 -and ($migration004.Trim() -eq "1")) {
            Write-Host "[OK] 遷移 004 已應用" -ForegroundColor Green
            Write-Host ""
            
            # 檢查表結構
            Write-Host "檢查 pig_observations 表結構..." -ForegroundColor Yellow
            $columns = psql -h $dbHost -p $port -U $user -d $database -t -c "SELECT column_name FROM information_schema.columns WHERE table_name = 'pig_observations' ORDER BY ordinal_position;" 2>&1
            
            $hasDeletedAt = $columns | Select-String -Pattern "deleted_at"
            $hasCopiedFromId = $columns | Select-String -Pattern "copied_from_id"
            
            if ($hasDeletedAt) {
                Write-Host "[OK] deleted_at 欄位存在" -ForegroundColor Green
            } else {
                Write-Host "[ERROR] deleted_at 欄位不存在！" -ForegroundColor Red
            }
            
            if ($hasCopiedFromId) {
                Write-Host "[OK] copied_from_id 欄位存在" -ForegroundColor Green
            } else {
                Write-Host "[ERROR] copied_from_id 欄位不存在！" -ForegroundColor Red
            }
            
            if (-not $hasDeletedAt -or -not $hasCopiedFromId) {
                Write-Host ""
                Write-Host "[WARNING] 表結構不完整。建議重新運行遷移：" -ForegroundColor Yellow
                Write-Host "  1. 停止後端服務" -ForegroundColor White
                Write-Host "  2. 刪除遷移 004 的記錄：" -ForegroundColor White
                Write-Host "     DELETE FROM _sqlx_migrations WHERE version = 4;" -ForegroundColor Green
                Write-Host "  3. 重新啟動後端服務" -ForegroundColor White
            }
        } else {
            Write-Host "[ERROR] 遷移 004 未應用！" -ForegroundColor Red
            Write-Host ""
            Write-Host "建議：" -ForegroundColor Yellow
            Write-Host "  重新啟動後端服務以應用遷移" -ForegroundColor White
        }
    } else {
        Write-Host "[ERROR] 無法查詢遷移記錄" -ForegroundColor Red
        Write-Host $migrations -ForegroundColor Red
    }
    
    Remove-Item Env:\PGPASSWORD
} else {
    Write-Host "[ERROR] 無法解析 DATABASE_URL 格式" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== 檢查完成 ===" -ForegroundColor Cyan
