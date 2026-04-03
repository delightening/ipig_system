# Reproduce CI environment locally and run backend cargo test
# Usage: .\scripts\run-ci-backend-tests.ps1
# Or with existing DB: .\scripts\run-ci-backend-tests.ps1 -UseExistingDb

param(
    [switch]$UseExistingDb
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
$BackendDir = Join-Path $RepoRoot "backend"

# CI env (matches .github/workflows/ci.yml backend-test)
$env:DATABASE_URL = "postgres://postgres:password@localhost:5432/ipig_db_test"
$env:SQLX_OFFLINE = "false"
$env:ADMIN_EMAIL = "admin@ipigsystem.asia"
$env:ADMIN_INITIAL_PASSWORD = "ci_test_admin_password_2024"
$env:DISABLE_ACCOUNT_LOCKOUT = "true"

Write-Host "`n=== CI: Backend cargo test ===" -ForegroundColor Cyan
Write-Host "DATABASE_URL=postgres://...@localhost:5432/ipig_db_test, SQLX_OFFLINE=false" -ForegroundColor Gray

if (-not $UseExistingDb) {
    Write-Host "`n1. Start db-test (fresh container for clean DB)..." -ForegroundColor Cyan
    Push-Location $RepoRoot
    docker compose -f docker-compose.test.yml up -d --force-recreate db-test
    if ($LASTEXITCODE -ne 0) {
        Pop-Location
        Write-Host "Failed to start db-test. Check Docker." -ForegroundColor Red
        exit 1
    }
    Pop-Location

    Write-Host "Waiting for PostgreSQL..."
    $ready = $false
    for ($i = 1; $i -le 30; $i++) {
        docker exec ipig-db-test pg_isready -U postgres -d ipig_db_test 2>$null
        if ($LASTEXITCODE -eq 0) {
            $ready = $true
            break
        }
        Start-Sleep -Seconds 2
    }
    if (-not $ready) {
        Write-Host "PostgreSQL not ready. Check: docker logs ipig-db-test" -ForegroundColor Red
        exit 1
    }
    Write-Host "PostgreSQL ready" -ForegroundColor Green
}

Write-Host "`n2. sqlx migrate run..." -ForegroundColor Cyan
Push-Location $BackendDir
sqlx migrate run
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    Write-Host "Migration failed" -ForegroundColor Red
    exit 1
}

Write-Host "`n3. cargo test --verbose (--test-threads=1 to avoid shared-DB collisions)..." -ForegroundColor Cyan
cargo test --verbose -- --test-threads=1
$exitCode = $LASTEXITCODE
Pop-Location

exit $exitCode
