# ============================================
# iPig System - 測試資料清理腳本
#
# 功能：刪除測試建立的業務記錄，保留安全審計資料
# 用法：.\cleanup_test_data.ps1
# ============================================

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  iPig System — 測試資料清理工具" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "此腳本將清除以下測試資料：" -ForegroundColor Yellow
Write-Host "  • 測試帳號（保留所有擁有 admin 角色的帳號）" -ForegroundColor Yellow
Write-Host "  • 動物紀錄（保留動物來源種子資料）" -ForegroundColor Yellow
Write-Host "  • AUP 計畫、審查、修正案" -ForegroundColor Yellow
Write-Host "  • ERP 倉庫、產品、單據、庫存" -ForegroundColor Yellow
Write-Host "  • HR 出勤、加班、請假" -ForegroundColor Yellow
Write-Host "  • 設施管理資料" -ForegroundColor Yellow
Write-Host ""
Write-Host "以下資料將被保留（不會被清除）：" -ForegroundColor Green
Write-Host "  ✓ 稽核日誌 (audit_logs)" -ForegroundColor Green
Write-Host "  ✓ 使用者活動日誌 (user_activity_logs)" -ForegroundColor Green
Write-Host "  ✓ 登入事件 (login_events)" -ForegroundColor Green
Write-Host "  ✓ 使用者會話 (user_sessions)" -ForegroundColor Green
Write-Host "  ✓ 安全警報 (security_alerts)" -ForegroundColor Green
Write-Host "  ✓ 計畫活動歷程 (protocol_activities)" -ForegroundColor Green
Write-Host "  ✓ 系統角色與權限定義" -ForegroundColor Green
Write-Host ""

# ============================================
# 雙重確認
# ============================================
$confirm1 = Read-Host "確定要清除測試資料嗎？(Y/N)"
if ($confirm1 -ne "Y" -and $confirm1 -ne "y") {
    Write-Host ""
    Write-Host "已取消操作" -ForegroundColor Cyan
    exit 0
}

$confirm2 = Read-Host "請輸入 CLEANUP 以確認"
if ($confirm2 -ne "CLEANUP") {
    Write-Host ""
    Write-Host "確認失敗，已取消操作" -ForegroundColor Cyan
    exit 0
}

Write-Host ""
Write-Host "開始清理測試資料..." -ForegroundColor Yellow
Write-Host ""

# ============================================
# 偵測環境
# ============================================
$dockerAvailable = $false
try {
    docker info 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        # 確認 ipig-db 容器執行中
        $containerRunning = docker ps --filter "name=ipig-db" --format "{{.Names}}" 2>&1
        if ($containerRunning -match "ipig-db") {
            $dockerAvailable = $true
        }
    }
}
catch {
    $dockerAvailable = $false
}

$sqlFile = Join-Path (Split-Path $PSScriptRoot -Parent) "backend\cleanup_test_data.sql"

if (-not (Test-Path $sqlFile)) {
    Write-Host "[ERROR] 找不到 SQL 腳本: $sqlFile" -ForegroundColor Red
    exit 1
}

if ($dockerAvailable) {
    Write-Host "[INFO] 使用 Docker 模式 (ipig-db 容器)" -ForegroundColor Cyan
    Write-Host ""

    # 顯示清理前統計
    Write-Host "[1/3] 清理前統計..." -ForegroundColor Yellow
    docker exec ipig-db psql -U postgres -d ipig_db -c "
        SELECT '使用者(非admin)' AS item, COUNT(*)::text AS count FROM users WHERE email != 'admin@ipig.local'
        UNION ALL SELECT '動物', COUNT(*)::text FROM animals
        UNION ALL SELECT '計畫', COUNT(*)::text FROM protocols
        UNION ALL SELECT '倉庫', COUNT(*)::text FROM warehouses
        UNION ALL SELECT '單據', COUNT(*)::text FROM documents
        UNION ALL SELECT '--- 保留: 稽核日誌', COUNT(*)::text FROM audit_logs
        UNION ALL SELECT '--- 保留: 活動日誌', COUNT(*)::text FROM user_activity_logs
        UNION ALL SELECT '--- 保留: 登入事件', COUNT(*)::text FROM login_events
        UNION ALL SELECT '--- 保留: 安全警報', COUNT(*)::text FROM security_alerts
    " 2>&1

    Write-Host ""
    Write-Host "[2/3] 執行清理 SQL 腳本..." -ForegroundColor Yellow

    # 把 SQL 檔案複製到容器並執行
    docker cp $sqlFile ipig-db:/tmp/cleanup_test_data.sql
    docker exec ipig-db psql -U postgres -d ipig_db -f /tmp/cleanup_test_data.sql 2>&1

    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "[OK] SQL 腳本執行完畢" -ForegroundColor Green
    }
    else {
        Write-Host ""
        Write-Host "[ERROR] SQL 腳本執行失敗" -ForegroundColor Red
        exit 1
    }

    # 清理容器中的暫存檔
    docker exec ipig-db rm -f /tmp/cleanup_test_data.sql

    Write-Host ""
    Write-Host "[3/3] 驗證清理結果..." -ForegroundColor Yellow
    docker exec ipig-db psql -U postgres -d ipig_db -c "
        SELECT '使用者' AS item, COUNT(*)::text AS count FROM users
        UNION ALL SELECT '動物', COUNT(*)::text FROM animals
        UNION ALL SELECT '計畫', COUNT(*)::text FROM protocols
        UNION ALL SELECT '倉庫', COUNT(*)::text FROM warehouses
        UNION ALL SELECT '單據', COUNT(*)::text FROM documents
        UNION ALL SELECT '角色(保留)', COUNT(*)::text FROM roles
        UNION ALL SELECT '動物來源(保留)', COUNT(*)::text FROM animal_sources
        UNION ALL SELECT '稽核日誌(保留)', COUNT(*)::text FROM audit_logs
        UNION ALL SELECT '活動日誌(保留)', COUNT(*)::text FROM user_activity_logs
        UNION ALL SELECT '登入事件(保留)', COUNT(*)::text FROM login_events
        UNION ALL SELECT '安全警報(保留)', COUNT(*)::text FROM security_alerts
    " 2>&1

}
else {
    Write-Host "[INFO] Docker 不可用，嘗試使用本機 psql" -ForegroundColor Cyan
    Write-Host ""

    # 從 .env 讀取 DATABASE_URL
    $envFile = Join-Path $PSScriptRoot "backend\.env"
    if (-not (Test-Path $envFile)) {
        $envFile = Join-Path $PSScriptRoot ".env"
    }

    $databaseUrl = ""
    if (Test-Path $envFile) {
        Get-Content $envFile | ForEach-Object {
            if ($_ -match "^DATABASE_URL=(.+)") {
                $databaseUrl = $matches[1]
            }
        }
    }

    if ([string]::IsNullOrEmpty($databaseUrl)) {
        # 使用預設值
        $databaseUrl = "postgres://postgres:ipig_password_123@localhost:5432/ipig_db"
        Write-Host "[INFO] 使用預設連線: $databaseUrl" -ForegroundColor Yellow
    }

    # 替換 Docker 內部主機名
    $databaseUrl = $databaseUrl -replace "@db:", "@localhost:"

    Write-Host "[1/2] 執行清理 SQL 腳本..." -ForegroundColor Yellow
    psql $databaseUrl -f $sqlFile

    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "[OK] SQL 腳本執行完畢" -ForegroundColor Green
    }
    else {
        Write-Host ""
        Write-Host "[ERROR] SQL 腳本執行失敗" -ForegroundColor Red
        Write-Host "請確認 PostgreSQL 服務已啟動且連線資訊正確" -ForegroundColor Yellow
        exit 1
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  ✅ 測試資料清理完成！" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "保留帳號:" -ForegroundColor Cyan
Write-Host "  Email: admin@ipig.local" -ForegroundColor White
Write-Host "  Password: admin123" -ForegroundColor White
Write-Host ""
Write-Host "已保留所有安全審計資料（稽核日誌、活動日誌、登入事件、安全警報）" -ForegroundColor Cyan
Write-Host ""
