# Reproduce E2E CI environment locally and run Playwright tests
# Mirrors .github/workflows/ci.yml e2e-test job
# Usage: .\scripts\run-ci-e2e-tests.ps1

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))

# E2E env (matches CI)
$env:E2E_BASE_URL = "http://localhost:8080"
$env:E2E_USER_EMAIL = "admin@ipigsystem.asia"
$env:E2E_USER_PASSWORD = "ci_test_admin_password_2024"
$env:E2E_ADMIN_EMAIL = "admin@ipigsystem.asia"
$env:E2E_ADMIN_PASSWORD = "ci_test_admin_password_2024"
$env:ADMIN_INITIAL_PASSWORD = "ci_test_admin_password_2024"  # 與後端 seed 一致，避免 .env 覆蓋
$env:CI = "true"

Write-Host "`n=== CI: E2E Playwright tests ===" -ForegroundColor Cyan
Write-Host "E2E_BASE_URL=http://localhost:8080, E2E_ADMIN_PASSWORD=ci_test_admin_password_2024" -ForegroundColor Gray

Write-Host "`n1. Copy .env.example to .env..." -ForegroundColor Cyan
if (Test-Path (Join-Path $RepoRoot ".env.example")) {
    Copy-Item (Join-Path $RepoRoot ".env.example") (Join-Path $RepoRoot ".env") -Force
}

Write-Host "`n2. Build and start Docker test env (db-test, api-test, web-test)..." -ForegroundColor Cyan
Push-Location $RepoRoot
docker compose --progress=plain -f docker-compose.test.yml build
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    Write-Host "Docker build failed" -ForegroundColor Red
    exit 1
}
docker compose -f docker-compose.test.yml up -d --wait --wait-timeout 300
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    Write-Host "Docker up failed" -ForegroundColor Red
    exit 1
}
Pop-Location

Write-Host "`n3. Verify services (web:8080, api:8000)..." -ForegroundColor Cyan
$ready = $false
for ($i = 1; $i -le 60; $i++) {
    try {
        $web = Invoke-WebRequest -Uri "http://localhost:8080/" -UseBasicParsing -TimeoutSec 3 -ErrorAction SilentlyContinue
        $api = Invoke-WebRequest -Uri "http://localhost:8000/api/health" -UseBasicParsing -TimeoutSec 3 -ErrorAction SilentlyContinue
        if ($web.StatusCode -eq 200 -and $api.StatusCode -eq 200) {
            $ready = $true
            break
        }
    } catch {}
    Write-Host "Waiting for services... ($i/60)" -ForegroundColor Yellow
    Start-Sleep -Seconds 2
}
if (-not $ready) {
    Write-Host "Services not ready. Check: docker compose -f docker-compose.test.yml logs" -ForegroundColor Red
    exit 1
}
Write-Host "Services ready" -ForegroundColor Green

Write-Host "`n4. Install frontend deps and Playwright browsers..." -ForegroundColor Cyan
Push-Location (Join-Path $RepoRoot "frontend")
pnpm install --frozen-lockfile
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    exit 1
}
pnpm exec playwright install --with-deps
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    exit 1
}

Write-Host "`n5. Clear stale auth state (CI 模擬：每次使用全新 session)..." -ForegroundColor Cyan
$authDir = Join-Path $RepoRoot "frontend\e2e\.auth"
if (Test-Path $authDir) {
    Remove-Item -Path $authDir -Recurse -Force
    Write-Host "Removed existing .auth folder" -ForegroundColor Gray
}

Write-Host "`n6. Run Playwright E2E tests..." -ForegroundColor Cyan
pnpm run test:e2e
$exitCode = $LASTEXITCODE
Pop-Location

Write-Host "`n7. Stop Docker (optional, run 'docker compose -f docker-compose.test.yml down' to clean)" -ForegroundColor Gray
exit $exitCode
