# iPig System - Prod Redeploy Script
#
# 用途：把 main branch 最新程式碼部署到 prod laptop 的 Docker container
#
# 何時用：
#   - PR merge 後想讓改動生效（例如 R46 reuse 降噪、本 PR audit 降噪）
#   - 修改 backend Rust code 後
#   - 修改 frontend 後（前端在 web 容器內，要 rebuild）
#
# 流程：
#   1. 確認在 main branch、工作目錄 clean
#   2. git pull origin main（顯示新 commits）
#   3. docker compose build api web（重建變動的 service image）
#   4. docker compose up -d api web（套用新 image，保留 DB / 其他服務）
#   5. 健康檢查 + 顯示新 commit summary
#
# 安全：
#   - 不動 DB / Postgres container（migration 由 backend 啟動自動跑）
#   - 失敗時不自動 rollback（保留錯誤狀態供 debug）
#   - 工作目錄不 clean 會拒絕跑（避免覆寫本地修改）

# R51 follow-up：用 "Continue" 而非 "Stop"。
# 原因：PowerShell 5.1 在 "Stop" 模式下，native command（git / docker）寫到
# stderr 的 progress 訊息（"From https://..."、"#1 [internal] load metadata..." 等）
# 會被視為 NativeCommandError 拋出，導致 deploy 在 git pull 或 docker build
# 階段就中斷。本 script 對每個 native command 都已手動檢查 $LASTEXITCODE，
# 不依賴 EAP=Stop。
$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  iPig System - Prod Redeploy" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Pre-flight: working tree clean + on main branch
$currentBranch = git rev-parse --abbrev-ref HEAD
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ABORT] git rev-parse 失敗（exit $LASTEXITCODE，可能不在 git repo 內）" -ForegroundColor Red
    exit 1
}
if ($currentBranch -ne "main") {
    Write-Host "[ABORT] 目前 branch 是 '$currentBranch'，需切到 main 才能 deploy。" -ForegroundColor Red
    Write-Host "        git checkout main" -ForegroundColor Yellow
    exit 1
}

$dirty = git status --porcelain
if ($dirty) {
    Write-Host "[ABORT] 工作目錄有未 commit 變更，停下避免覆寫：" -ForegroundColor Red
    Write-Host $dirty -ForegroundColor Yellow
    Write-Host "請先 commit / stash 後再 deploy。" -ForegroundColor Yellow
    exit 1
}

# 1. Pull latest main
Write-Host "[1/4] git pull origin main ..." -ForegroundColor Cyan
$beforeSha = git rev-parse HEAD
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ABORT] git rev-parse HEAD 失敗（exit $LASTEXITCODE）" -ForegroundColor Red
    exit 1
}
git pull --ff-only origin main
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ABORT] git pull 失敗（exit $LASTEXITCODE，可能 main 已分歧需 rebase）" -ForegroundColor Red
    exit 1
}
$afterSha = git rev-parse HEAD
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ABORT] git rev-parse HEAD（pull 後）失敗（exit $LASTEXITCODE）" -ForegroundColor Red
    exit 1
}

if ($beforeSha -eq $afterSha) {
    Write-Host "[OK] 已是最新 main，無新 commit 需要 deploy。" -ForegroundColor Green
    Write-Host "若仍要強制 rebuild，按 Enter 繼續，否則 Ctrl+C 取消。" -ForegroundColor Yellow
    Read-Host
} else {
    Write-Host "[OK] $beforeSha → $afterSha" -ForegroundColor Green
    Write-Host ""
    Write-Host "新 commits：" -ForegroundColor Yellow
    git log --oneline "$beforeSha..$afterSha"
    Write-Host ""
}

# 2. Build api + web images
Write-Host "[2/4] docker compose build api web ..." -ForegroundColor Cyan
docker compose build api web
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ABORT] docker build 失敗，請看上方錯誤訊息。" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] build 完成" -ForegroundColor Green
Write-Host ""

# 3. Restart api + web with new images（保留 DB / monitoring 容器）
Write-Host "[3/4] docker compose up -d api web ..." -ForegroundColor Cyan
docker compose up -d api web
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ABORT] docker compose up 失敗。" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] api + web 已套用新 image" -ForegroundColor Green
Write-Host ""

# 4. Health check
Write-Host "[4/4] 等待 api healthy ..." -ForegroundColor Cyan
$maxRetries = 30
$retry = 0
while ($retry -lt $maxRetries) {
    Start-Sleep -Seconds 2
    $health = docker inspect --format='{{.State.Health.Status}}' ipig-api 2>$null
    if ($health -eq "healthy") {
        Write-Host "[OK] api healthy" -ForegroundColor Green
        break
    }
    if ($health -eq "unhealthy") {
        Write-Host "[ABORT] api unhealthy — 查看 logs：" -ForegroundColor Red
        Write-Host "  docker compose logs --tail=50 api" -ForegroundColor Yellow
        exit 1
    }
    $retry++
}
if ($retry -ge $maxRetries) {
    Write-Host "[WARN] 健康檢查 60 秒超時，但容器仍 starting — 可手動再確認：" -ForegroundColor Yellow
    Write-Host "  docker ps --filter name=ipig-api" -ForegroundColor White
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Deploy 完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "驗證：" -ForegroundColor Yellow
Write-Host "  https://ipigsystem.asia/" -ForegroundColor White
Write-Host "  https://ipigsystem.asia/admin/audit?tab=alerts" -ForegroundColor White
Write-Host ""
Write-Host "查看 api 日誌：" -ForegroundColor Yellow
Write-Host "  docker compose logs -f api" -ForegroundColor White
Write-Host ""
