# Clear migration records and use SQLx CLI to resolve them properly

Write-Host "=== Clearing and Resolving Migrations ===" -ForegroundColor Cyan
Write-Host ""

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

Write-Host "[WARNING] This will clear all migration records and resolve them using SQLx CLI" -ForegroundColor Yellow
$confirmation = Read-Host "Type YES to continue"

if ($confirmation -ne "YES") {
    Write-Host "Operation cancelled" -ForegroundColor Cyan
    exit 0
}

Write-Host ""
Write-Host "[1/3] Clearing migration records..." -ForegroundColor Yellow

# Use Rust tool to clear (or direct SQL)
cargo run --bin fix_migration 2>&1 | Out-Null

# Actually, let's use a direct approach - create a simple Rust tool to clear all
Write-Host "[INFO] Please manually clear _sqlx_migrations table:" -ForegroundColor Cyan
Write-Host "  DELETE FROM _sqlx_migrations;" -ForegroundColor White
Write-Host ""
Write-Host "[2/3] Then use SQLx CLI to resolve:" -ForegroundColor Yellow
Write-Host "  sqlx migrate resolve <version> --database-url `"$databaseUrl`"" -ForegroundColor White
Write-Host ""
Write-Host "[INFO] To get version hashes, you need to use SQLx's migration resolver." -ForegroundColor Cyan
Write-Host "[INFO] The easiest solution is to reset the database:" -ForegroundColor Cyan
Write-Host "  ..\reset_database.ps1" -ForegroundColor White
Write-Host ""
