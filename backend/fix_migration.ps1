# Fix Migration 9 Issue

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Fix Migration 9 Issue" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Read DATABASE_URL from .env
$envFile = ".env"
if (-not (Test-Path $envFile)) {
    Write-Host "[ERROR] .env file not found!" -ForegroundColor Red
    exit 1
}

$envContent = Get-Content $envFile -Raw
if ($envContent -match "DATABASE_URL=([^\r\n]+)") {
    $databaseUrl = $matches[1]
    Write-Host "[OK] Found DATABASE_URL" -ForegroundColor Green
} else {
    Write-Host "[ERROR] DATABASE_URL not found in .env" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Connecting to database..." -ForegroundColor Yellow
Write-Host "Database URL: $($databaseUrl -replace ':[^:@]+@', ':***@')" -ForegroundColor Gray
Write-Host ""

# Check if psql is available
$psqlAvailable = $false
try {
    $null = Get-Command psql -ErrorAction Stop
    $psqlAvailable = $true
} catch {
    $psqlAvailable = $false
}

if ($psqlAvailable) {
    Write-Host "[OK] psql is available" -ForegroundColor Green
    Write-Host ""
    Write-Host "Deleting migration 9 record..." -ForegroundColor Yellow
    
    # Parse database URL
    if ($databaseUrl -match "postgres://([^:]+):([^@]+)@([^:]+):(\d+)/(.+)") {
        $user = $matches[1]
        $password = $matches[2]
        $dbHost = $matches[3]
        $port = $matches[4]
        $database = $matches[5]
        
        $env:PGPASSWORD = $password
        $result = psql -h $dbHost -p $port -U $user -d $database -c "DELETE FROM _sqlx_migrations WHERE version = 9;" 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[SUCCESS] Migration 9 record deleted!" -ForegroundColor Green
            Write-Host ""
            Write-Host "You can now restart the backend service." -ForegroundColor Cyan
        } else {
            Write-Host "[ERROR] Failed to delete migration record" -ForegroundColor Red
            Write-Host $result -ForegroundColor Red
        }
        
        Remove-Item Env:\PGPASSWORD
    } else {
        Write-Host "[ERROR] Invalid DATABASE_URL format" -ForegroundColor Red
    }
} else {
    Write-Host "[WARNING] psql is not available" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Please use one of the following methods:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Method 1: Install PostgreSQL client tools" -ForegroundColor Yellow
    Write-Host "  Then run this script again" -ForegroundColor White
    Write-Host ""
    Write-Host "Method 2: Use SQLx CLI" -ForegroundColor Yellow
    Write-Host "  sqlx migrate revert --database-url `"$databaseUrl`"" -ForegroundColor White
    Write-Host ""
    Write-Host "Method 3: Manual SQL command" -ForegroundColor Yellow
    Write-Host "  Connect to database and run:" -ForegroundColor White
    Write-Host "  DELETE FROM _sqlx_migrations WHERE version = 9;" -ForegroundColor Green
}

Write-Host ""
