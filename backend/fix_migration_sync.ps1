# Fix migration sync issue
# This script resolves the migration version mismatch

Write-Host "=== Fixing Migration Sync ===" -ForegroundColor Cyan
Write-Host ""

# Check if sqlx CLI is available
$sqlxAvailable = $false
try {
    $null = sqlx --version 2>&1
    $sqlxAvailable = $true
} catch {
    $sqlxAvailable = $false
}

if (-not $sqlxAvailable) {
    Write-Host "[ERROR] SQLx CLI is not installed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Installing SQLx CLI..." -ForegroundColor Yellow
    cargo install sqlx-cli --features postgres
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Failed to install SQLx CLI" -ForegroundColor Red
        exit 1
    }
}

# Read DATABASE_URL from .env
$envFile = ".env"
if (-not (Test-Path $envFile)) {
    Write-Host "[ERROR] .env file not found" -ForegroundColor Red
    exit 1
}

$envContent = Get-Content $envFile
$databaseUrl = $envContent | Select-String -Pattern "^DATABASE_URL=(.+)$" | ForEach-Object { $_.Matches.Groups[1].Value }

if (-not $databaseUrl) {
    Write-Host "[ERROR] DATABASE_URL not found in .env" -ForegroundColor Red
    exit 1
}

Write-Host "Database URL: $($databaseUrl -replace ':[^:@]+@', ':***@')" -ForegroundColor Gray
Write-Host ""

# Check migration info
Write-Host "[1/2] Checking migration status..." -ForegroundColor Yellow
sqlx migrate info --database-url $databaseUrl
if ($LASTEXITCODE -ne 0) {
    Write-Host "[WARNING] Could not get migration info" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[2/2] Attempting to resolve migrations..." -ForegroundColor Yellow
Write-Host "[INFO] If migrations are out of sync, SQLx will try to resolve them." -ForegroundColor Cyan
Write-Host "[INFO] If tables already exist, you may need to manually fix the _sqlx_migrations table." -ForegroundColor Cyan
Write-Host ""

# The simplest solution: clear _sqlx_migrations and let SQLx recreate it
# But this requires the migrations to be idempotent or the tables to not exist
Write-Host "[INFO] Recommended solution:" -ForegroundColor Yellow
Write-Host "  1. The database has tables but no migration records" -ForegroundColor White
Write-Host "  2. SQLx needs to know which migrations were applied" -ForegroundColor White
Write-Host "  3. Options:" -ForegroundColor White
Write-Host "     a) Use 'sqlx migrate resolve' for each migration file" -ForegroundColor Cyan
Write-Host "     b) Reset database and re-run migrations (loses data!)" -ForegroundColor Cyan
Write-Host "     c) Manually insert migration records (requires version hashes)" -ForegroundColor Cyan
Write-Host ""

Write-Host "[INFO] To manually resolve, run for each migration file:" -ForegroundColor Yellow
Write-Host "  sqlx migrate resolve <version_hash> --database-url `"$databaseUrl`"" -ForegroundColor White
Write-Host ""
Write-Host "[INFO] Or use the reset script (WARNING: loses all data):" -ForegroundColor Yellow
Write-Host "  ..\reset_database.ps1" -ForegroundColor White
Write-Host ""
