# iPig 舊資料庫 Dump 還原腳本
# 用法: .\scripts\restore_old_dump.ps1 [-DumpPath <路徑>] [-SkipBackup] [-SkipMigrationSync]
#
# 此腳本會：
# 1. 檢查 dump 檔案格式和內容
# 2. 備份現有資料庫（可選）
# 3. Restore dump 檔案
# 4. 同步 migration 追蹤狀態（處理 _sqlx_migrations 表）
# 5. 驗證還原結果

param(
    [string]$DumpPath = "old_ipig.dump",
    [switch]$SkipBackup,
    [switch]$SkipMigrationSync,
    [switch]$DataOnly,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# 顏色輸出函數
function Write-Info { param([string]$Message) Write-Host $Message -ForegroundColor Cyan }
function Write-Success { param([string]$Message) Write-Host $Message -ForegroundColor Green }
function Write-Warning { param([string]$Message) Write-Host $Message -ForegroundColor Yellow }
function Write-Error { param([string]$Message) Write-Host $Message -ForegroundColor Red }

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  iPig 舊資料庫 Dump 還原工具" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. 檢查 Docker 和資料庫容器
Write-Info "檢查 Docker 環境..."
try {
    docker info 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Docker 未運行"
    }
} catch {
    Write-Error "錯誤: Docker 未運行或無法連接"
    exit 1
}

$dbContainer = "ipig-db"
$containerExists = docker ps -a --filter "name=^${dbContainer}$" --format "{{.Names}}" 2>&1
$containerExists = ($containerExists | Where-Object { $_ -eq $dbContainer })

if (-not $containerExists) {
    Write-Error "錯誤: 找不到資料庫容器 '$dbContainer'"
    Write-Info "請先執行: docker compose up -d"
    exit 1
}

$containerRunning = docker ps --filter "name=^${dbContainer}$" --format "{{.Names}}" 2>&1
$containerRunning = ($containerRunning | Where-Object { $_ -eq $dbContainer })

if (-not $containerRunning) {
    Write-Warning "資料庫容器未運行，正在啟動..."
    docker start $dbContainer
    Start-Sleep -Seconds 5
}

Write-Success "✓ Docker 環境正常"

# 2. 檢查 dump 檔案
Write-Info "`n檢查 dump 檔案: $DumpPath"
if (-not (Test-Path $DumpPath)) {
    Write-Error "錯誤: Dump 檔案不存在: $DumpPath"
    exit 1
}

$dumpSize = (Get-Item $DumpPath).Length / 1MB
Write-Info "  檔案大小: $([math]::Round($dumpSize, 2)) MB"

# 複製 dump 到容器
Write-Info "  複製 dump 檔案到容器..."
docker cp $DumpPath "${dbContainer}:/tmp/old_ipig.dump" 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Error "錯誤: 無法複製 dump 檔案到容器"
    exit 1
}

# 檢查 dump 格式
Write-Info "  檢查 dump 格式..."
$dumpInfo = docker exec $dbContainer pg_restore --list /tmp/old_ipig.dump 2>&1
$dumpCheckResult = $LASTEXITCODE

# 檢查輸出是否包含錯誤訊息
if ($dumpInfo -match "error|ERROR|FATAL" -or ($dumpCheckResult -ne 0 -and -not ($dumpInfo -match "Archive created"))) {
    Write-Error "錯誤: 無法讀取 dump 檔案，可能格式不正確"
    Write-Error "詳細錯誤: $dumpInfo"
    exit 1
}

$hasMigrationTable = docker exec $dbContainer pg_restore --list /tmp/old_ipig.dump 2>&1 | Select-String "_sqlx_migrations"
$hasData = docker exec $dbContainer pg_restore --list /tmp/old_ipig.dump 2>&1 | Select-String "TABLE DATA"

Write-Success "✓ Dump 檔案格式正確"
Write-Info "  包含 _sqlx_migrations 表: $(if ($hasMigrationTable) { '是' } else { '否' })"
Write-Info "  包含資料: $(if ($hasData) { '是' } else { '否' })"

# 3. 讀取環境變數
Write-Info "`n讀取資料庫連線資訊..."
$envFile = ".env"
if (-not (Test-Path $envFile)) {
    Write-Error "錯誤: 找不到 .env 檔案"
    exit 1
}

# 讀取 .env 檔案
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

Write-Success "✓ 資料庫連線資訊已讀取"
Write-Info "  資料庫: $dbName"
Write-Info "  使用者: $dbUser"

# 4. 備份現有資料庫（可選）
if (-not $SkipBackup) {
    Write-Info "`n檢查現有資料庫..."
    $env:PGPASSWORD = $dbPassword
    $tablesExist = docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public';" 2>&1
    $env:PGPASSWORD = $null
    $tableCount = ($tablesExist -replace '\s', '')
    
    if ($tableCount -and $tableCount -ne "0") {
        Write-Warning "  發現現有資料庫包含 $tableCount 個資料表"
        
        if ($Force) {
            Write-Info "  Force 模式：自動建立備份"
            $backupChoice = "y"
        } else {
            $backupChoice = Read-Host "  是否要備份現有資料庫？(y/N)"
        }
        
        if ($backupChoice -eq "y" -or $backupChoice -eq "Y") {
            $backupFile = "backup_before_restore_$(Get-Date -Format 'yyyyMMdd_HHmmss').dump"
            Write-Info "  建立備份: $backupFile"
            
            $env:PGPASSWORD = $dbPassword
            docker exec -e PGPASSWORD=$dbPassword $dbContainer pg_dump -U $dbUser -d $dbName -Fc -f "/tmp/backup.dump" 2>&1 | Out-Null
            $env:PGPASSWORD = $null
            if ($LASTEXITCODE -eq 0) {
                docker cp "${dbContainer}:/tmp/backup.dump" $backupFile 2>&1 | Out-Null
                if ($LASTEXITCODE -eq 0) {
                    Write-Success "  ✓ 備份完成: $backupFile"
                } else {
                    Write-Warning "  警告: 無法複製備份檔案到主機"
                }
            } else {
                Write-Warning "  警告: 備份失敗，但將繼續還原流程"
            }
        }
    } else {
        Write-Info "  資料庫為空或不存在"
    }
}

# 5. 確認還原
Write-Host ""
Write-Warning "========================================"
Write-Warning "  警告：此操作將覆蓋現有資料庫！"
Write-Warning "========================================"
Write-Host ""

if ($DataOnly) {
    Write-Info "模式: 僅還原資料（不包含結構）"
    Write-Info "  此模式需要資料庫結構已存在"
} else {
    Write-Info "模式: 完整還原（結構 + 資料）"
    Write-Info "  此操作將："
    Write-Info "    - 刪除現有資料庫物件（--clean）"
    Write-Info "    - 還原所有結構和資料"
}

if ($Force) {
    Write-Info "Force 模式：自動確認還原"
    $confirm = "yes"
} else {
    $confirm = Read-Host "輸入 'yes' 確認繼續"
}

if ($confirm -ne "yes") {
    Write-Info "已取消"
    exit 0
}

# 6. 執行還原
Write-Info "`n開始還原..."
$restoreArgs = @(
    "-U", $dbUser,
    "-d", $dbName,
    "--no-owner",
    "--no-acl",
    "--verbose"
)

if ($DataOnly) {
    $restoreArgs += "--data-only"
} else {
    $restoreArgs += "--clean"
    $restoreArgs += "--if-exists"
}

$restoreArgs += "/tmp/old_ipig.dump"

# 設定環境變數並執行還原
$env:PGPASSWORD = $dbPassword
try {
    # 使用 Invoke-Expression 來避免 PowerShell 將輸出視為錯誤
    $restoreOutput = & docker exec -e "PGPASSWORD=$dbPassword" $dbContainer pg_restore $restoreArgs 2>&1 | Out-String
    $restoreExitCode = $LASTEXITCODE
} catch {
    $restoreOutput = $_.Exception.Message
    $restoreExitCode = 1
} finally {
    $env:PGPASSWORD = $null
}

# pg_restore 在 --clean 時可能回傳非零（因物件不存在），但這通常是正常的
# 檢查輸出中是否有真正的錯誤（ERROR 或 FATAL），而不是警告
$hasRealErrors = $restoreOutput | Select-String -Pattern "ERROR|FATAL" -CaseSensitive

if ($restoreExitCode -ne 0) {
    if ($hasRealErrors) {
        Write-Warning "還原過程中出現錯誤"
        Write-Info "詳細輸出（最後 20 行）："
        $restoreOutput | Select-Object -Last 20 | ForEach-Object { Write-Host "  $_" }
        
        # 驗證還原是否真的失敗（檢查資料表是否存在）
        $env:PGPASSWORD = $dbPassword
        $verifyTables = docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -t -c `
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public';" 2>&1
        $env:PGPASSWORD = $null
        $tableCountAfter = ($verifyTables -replace '\s', '')
        
        if ($tableCountAfter -and $tableCountAfter -ne "0") {
            Write-Success "✓ 還原似乎成功（資料表數量: $tableCountAfter），繼續處理..."
        } else {
            Write-Error "還原失敗：資料庫中沒有資料表"
            exit 1
        }
    } else {
        Write-Info "還原完成（可能有一些警告，但沒有嚴重錯誤）"
    }
} else {
    Write-Success "✓ 還原完成"
}

# 7. 處理 migration 追蹤（如果 dump 包含 _sqlx_migrations）
if (-not $SkipMigrationSync -and $hasMigrationTable) {
    Write-Info "`n處理 migration 追蹤狀態..."
    
    # 檢查 _sqlx_migrations 表是否存在
    $env:PGPASSWORD = $dbPassword
    $migrationTableExists = docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -t -c `
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema='public' AND table_name='_sqlx_migrations');" 2>&1
    $env:PGPASSWORD = $null
    
    if ($migrationTableExists -match 't') {
        Write-Info "  _sqlx_migrations 表已存在"
        
        # 檢查 migration 記錄數量
        $env:PGPASSWORD = $dbPassword
        $migrationCount = docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -t -c `
            "SELECT COUNT(*) FROM _sqlx_migrations;" 2>&1
        $env:PGPASSWORD = $null
        $migrationCount = ($migrationCount -replace '\s', '')
        
        Write-Info "  現有 migration 記錄數: $migrationCount"
        
        # 檢查當前 migration 檔案數量
        $migrationFiles = Get-ChildItem -Path "backend\migrations" -Filter "*.sql" | Sort-Object Name
        Write-Info "  當前 migration 檔案數: $($migrationFiles.Count)"
        
        if ($migrationCount -ne $migrationFiles.Count) {
            Write-Warning "  警告: Migration 記錄數與檔案數不一致"
            Write-Info "  選項："
            Write-Info "    1. 保留 dump 中的 migration 記錄（可能與當前 migration 檔案不匹配）"
            Write-Info "    2. 清空 migration 記錄，讓系統重新執行 migrations"
            Write-Info "    3. 手動同步 migration 記錄（使用 fix_migration_checksums.ps1）"
            
            if ($Force) {
                Write-Info "  Force 模式：選擇選項 1（保留 dump 中的記錄）"
                $migrationChoice = "1"
            } else {
                $migrationChoice = Read-Host "  選擇 (1/2/3，預設 1)"
            }
            
            if ($migrationChoice -eq "2") {
                Write-Info "  清空 _sqlx_migrations 表..."
                $env:PGPASSWORD = $dbPassword
                docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -c `
                    "TRUNCATE TABLE _sqlx_migrations;" 2>&1 | Out-Null
                $env:PGPASSWORD = $null
                Write-Success "  ✓ Migration 記錄已清空"
                Write-Info "  系統啟動時將重新執行所有 migrations"
            } elseif ($migrationChoice -eq "3") {
                Write-Info "  請執行以下命令來同步 migration checksums:"
                Write-Host "    .\scripts\fix_migration_checksums.ps1" -ForegroundColor Yellow
            } else {
                Write-Info "  保留 dump 中的 migration 記錄"
                Write-Warning "  如果啟動時出現 migration checksum 錯誤，請執行 fix_migration_checksums.ps1"
            }
        } else {
            Write-Success "  ✓ Migration 記錄數與檔案數一致"
        }
    } else {
        Write-Info "  _sqlx_migrations 表不存在，系統啟動時會自動建立"
    }
} else {
    if ($SkipMigrationSync) {
        Write-Info "`n跳過 migration 同步（已指定 -SkipMigrationSync）"
    } else {
        Write-Info "`nDump 不包含 _sqlx_migrations 表，無需同步"
    }
}

# 8. 驗證還原結果
Write-Info "`n驗證還原結果..."
$env:PGPASSWORD = $dbPassword
$tableCount = docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -t -c `
    "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';" 2>&1
$tableCount = ($tableCount -replace '\s', '')

$userCount = docker exec -e PGPASSWORD=$dbPassword $dbContainer psql -U $dbUser -d $dbName -t -c `
    "SELECT COUNT(*) FROM users;" 2>&1
$env:PGPASSWORD = $null
$userCount = ($userCount -replace '\s', '')

Write-Success "✓ 驗證完成"
Write-Info "  資料表數量: $tableCount"
if ($userCount -and $userCount -ne "") {
    Write-Info "  使用者數量: $userCount"
}

# 9. 清理
Write-Info "`n清理暫存檔案..."
docker exec $dbContainer rm -f /tmp/old_ipig.dump 2>&1 | Out-Null
Write-Success "✓ 清理完成"

# 10. 總結
Write-Host ""
Write-Success "========================================"
Write-Success "  還原流程完成！"
Write-Success "========================================"
Write-Host ""
Write-Info "後續步驟："
Write-Info "  1. 檢查資料庫連線: docker exec $dbContainer psql -U $dbUser -d $dbName -c '\dt'"
Write-Info "  2. 如果出現 migration checksum 錯誤，執行: .\scripts\fix_migration_checksums.ps1"
Write-Info "  3. 啟動系統: docker compose up -d"
Write-Host ""
