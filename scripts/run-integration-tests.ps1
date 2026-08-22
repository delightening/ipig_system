# iPig 後端整合測試腳本 (Windows)
#
# 前置需求：
# - PostgreSQL 已安裝且可連線
# - 已設定 TEST_DATABASE_URL 指向**獨立的丟棄用**測試資料庫（如 ipig_db_test）
#
# 為什麼只認 TEST_DATABASE_URL、又要覆寫 DATABASE_URL：
#   開發機同時是 prod，DATABASE_URL 指向正式庫。本腳本先前的版本有兩個問題疊在一起——
#   (1) 找不到 TEST_DATABASE_URL 時會 fallback 到 DATABASE_URL；
#   (2) 算出來的 DSN **只用於印出來給人看**，實際跑的是裸的 `sqlx migrate run`，
#       而 sqlx-cli 只認 DATABASE_URL / .env，根本收不到那個變數。
#   合起來的後果：使用者照規範設好 TEST_DATABASE_URL，畫面顯示「使用資料庫：測試庫」，
#   142 支 migration 卻跑在 prod 上。顯示安全卻做不安全的事，比沒有顯示更危險。
#
#   現在改為：只認 TEST_DATABASE_URL（不 fallback），並把 DATABASE_URL 一併覆寫成
#   同一個值，確保這條路徑上沒有任何子行程看得到 prod DSN。
#
# 若出現 VersionMismatch(1) 錯誤，表示測試 DB 的 migration 紀錄與程式碼不符。
# 解法：drop 並重建測試 DB，或執行 rtk cargo run --bin fix_migration <version> 後再重跑。

$ErrorActionPreference = "Stop"
# 巢狀 Join-Path 而非三參數形式：`Join-Path a b c` 需要 PowerShell 7 的
# -AdditionalChildPath，Windows PowerShell 5.1（本機為 5.1.26100）只吃兩個位置參數，
# 會以 PositionalParameterNotFound 中止。
$BackendDir = Join-Path (Join-Path $PSScriptRoot "..") "backend"

# 檢查環境變數（fail-closed：刻意不 fallback 到 DATABASE_URL）
$dbUrl = $env:TEST_DATABASE_URL
if (-not $dbUrl) {
    Write-Host "錯誤：請設定 TEST_DATABASE_URL 指向獨立的丟棄用測試資料庫" -ForegroundColor Red
    Write-Host "刻意不 fallback 到 DATABASE_URL——開發機那條指向 prod，見 CLAUDE.md" -ForegroundColor Yellow
    Write-Host "範例：`$env:TEST_DATABASE_URL = 'postgres://user:pass@localhost:5432/ipig_db_test'" -ForegroundColor Yellow
    exit 1
}

# 只有「非空」不等於「是測試庫」：打錯字或直接把 prod DSN 貼過來，
# 之前一樣會被匯出成 DATABASE_URL、跑 migration、被測試寫入
# （CodeAnt / CodeRabbit 於 PR #46 各自獨立指出）。
#
# 這裡是**第二層**防線，判準刻意保守：資料庫名不得是 prod 的名字，且必須含 `test`。
# 名稱檢查本身是啟發式（改名就失效），真正的身分驗證在測試 harness 的
# tests/common/test_db.rs——它檢查標記表與核心業務表是否為空。但那道護欄管不到
# 本腳本的 `sqlx migrate run`（獨立的 sqlx-cli 進程），所以這裡要自己擋一次。
$ProdDbName = "ipig_db"
# 取資料庫名：去掉 query string 後取最後一段路徑。
$dbName = ($dbUrl -split '\?')[0].TrimEnd('/').Split('/')[-1]
if ($dbName -eq $ProdDbName) {
    Write-Host "錯誤：TEST_DATABASE_URL 指向 prod 資料庫 ``$ProdDbName``，已中止。" -ForegroundColor Red
    Write-Host "整合測試會跑 migration 並寫入 fixture 且不清理，絕不可對正式庫執行。" -ForegroundColor Yellow
    exit 1
}
if ($dbName -notlike "*test*") {
    Write-Host "錯誤：TEST_DATABASE_URL 的資料庫名為 ``$dbName``，看不出是測試庫（名稱需含 test），已中止。" -ForegroundColor Red
    Write-Host "若這確實是丟棄用資料庫，請改名後重試——這道檢查是為了擋住手滑貼到正式 DSN。" -ForegroundColor Yellow
    exit 1
}

Write-Host "使用資料庫：$($dbUrl -replace ':[^:@]+@',':***@')" -ForegroundColor Cyan

# 覆寫 DATABASE_URL：測試 harness 與 Config::from_env() 都會讀它，統一指向測試庫，
# 讓這條路徑上沒有任何行程能連到 prod。
$env:DATABASE_URL = $dbUrl

Push-Location $BackendDir
try {
    Write-Host "`n執行 sqlx migrate run..." -ForegroundColor Cyan
    # --database-url 必須明確帶上：裸的 `sqlx migrate run` 會忽略上面算出的 $dbUrl，
    # 改讀 DATABASE_URL / .env（舊版的 bug 就在這裡）。
    sqlx migrate run --database-url $dbUrl
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
