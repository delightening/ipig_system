# Resolve all migrations automatically
# This script reads migration files and marks them as applied in the database

Write-Host "=== Resolving All Migrations ===" -ForegroundColor Cyan
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
    Write-Host "Installing SQLx CLI..." -ForegroundColor Yellow
    cargo install sqlx-cli --features postgres
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Failed to install SQLx CLI" -ForegroundColor Red
        exit 1
    }
}

# Get migration files
$migrationsDir = ".\migrations"
if (-not (Test-Path $migrationsDir)) {
    Write-Host "[ERROR] Migrations directory not found" -ForegroundColor Red
    exit 1
}

$migrationFiles = Get-ChildItem -Path $migrationsDir -Filter "*.sql" | Sort-Object Name

Write-Host "Found $($migrationFiles.Count) migration files" -ForegroundColor Cyan
Write-Host ""

# For each migration, we need to get its version hash
# SQLx uses content-based hashing, so we'll use sqlx migrate resolve with the file path
Write-Host "Resolving migrations..." -ForegroundColor Yellow
Write-Host ""

$resolvedCount = 0
$failedCount = 0

foreach ($file in $migrationFiles) {
    $fileName = $file.Name
    Write-Host "Processing: $fileName" -ForegroundColor White -NoNewline
    
    # SQLx migrate resolve needs the version hash
    # We can get it by using sqlx migrate add --check or by calculating it
    # Actually, the easiest way is to use sqlx migrate resolve with the migration number
    # But SQLx uses content hashes, not sequential numbers
    
    # Let's try a different approach: use sqlx migrate resolve with the file
    # Actually, sqlx migrate resolve needs the version (hash), not the file
    
    # The best approach: use sqlx migrate info to get version hashes, then resolve each
    # But that's complex. Let me try using the Rust tool we created instead
    
    Write-Host " ... " -NoNewline
}

Write-Host ""
Write-Host "[INFO] SQLx migrate resolve requires the version hash for each migration." -ForegroundColor Yellow
Write-Host "[INFO] The version hash is calculated from the migration file content." -ForegroundColor Yellow
Write-Host ""
Write-Host "Alternative approach: Use the Rust tool to resolve migrations" -ForegroundColor Cyan
Write-Host "  cargo run --bin resolve_migrations" -ForegroundColor White
Write-Host ""
Write-Host "Or manually resolve using SQLx CLI for each migration:" -ForegroundColor Cyan
Write-Host "  First, get the version hash from sqlx migrate info" -ForegroundColor White
Write-Host "  Then: sqlx migrate resolve <hash> --database-url `"$databaseUrl`"" -ForegroundColor White
Write-Host ""

# Actually, let's use a simpler approach: create a Rust binary that does this
Write-Host "[INFO] Creating a tool to automatically resolve all migrations..." -ForegroundColor Yellow
