# iPig System - Database Reset Script
# This script will delete all data and re-run migrations to restore all accounts to default state

Write-Host ""
Write-Host "========================================" -ForegroundColor Red
Write-Host "  Database Reset Script" -ForegroundColor Red
Write-Host "========================================" -ForegroundColor Red
Write-Host ""
Write-Host "WARNING: This operation will delete all data!" -ForegroundColor Yellow
Write-Host "  - All user accounts will be deleted" -ForegroundColor Yellow
Write-Host "  - All business data will be deleted" -ForegroundColor Yellow
Write-Host "  - Database will be restored to initial state" -ForegroundColor Yellow
Write-Host ""

$confirmation = Read-Host "Type YES to continue"

if ($confirmation -ne "YES") {
    Write-Host ""
    Write-Host "Operation cancelled" -ForegroundColor Cyan
    exit 0
}

Write-Host ""
Write-Host "Starting database reset..." -ForegroundColor Yellow
Write-Host ""

# Check if Docker is available
$dockerAvailable = $false
try {
    docker info 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        $dockerAvailable = $true
    }
} catch {
    $dockerAvailable = $false
}

if ($dockerAvailable) {
    Write-Host "[INFO] Using Docker Compose mode" -ForegroundColor Cyan
    Write-Host ""
    
    # Step 1: Stop all services and remove volumes
    Write-Host "[1/4] Stopping services and removing volumes..." -ForegroundColor Yellow
    docker compose down -v
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[WARNING] Error stopping services (services may not be running)" -ForegroundColor Yellow
    }
    Write-Host "[OK] Services and volumes removed" -ForegroundColor Green
    Write-Host ""
    
    # Step 2: Confirm data cleared
    Write-Host "[2/4] Confirming database data cleared..." -ForegroundColor Yellow
    Write-Host "[OK] Database data cleared" -ForegroundColor Green
    Write-Host ""
    
    # Step 3: Restart services (will auto-run migrations)
    Write-Host "[3/4] Restarting services and running migrations..." -ForegroundColor Yellow
    docker compose up -d
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[OK] Services started" -ForegroundColor Green
        Write-Host ""
        
        # Step 4: Wait for database to be ready
        Write-Host "[4/4] Waiting for database to be ready..." -ForegroundColor Yellow
        $maxAttempts = 30
        $attempt = 0
        $dbReady = $false
        
        while ($attempt -lt $maxAttempts -and -not $dbReady) {
            Start-Sleep -Seconds 2
            $attempt++
            docker exec ipig-db pg_isready -U postgres -d ipig_db 2>&1 | Out-Null
            if ($LASTEXITCODE -eq 0) {
                $dbReady = $true
            } else {
                Write-Host "  Waiting... ($attempt/$maxAttempts)" -ForegroundColor Gray
            }
        }
        
        if ($dbReady) {
            Write-Host "[OK] Database is ready" -ForegroundColor Green
        } else {
            Write-Host "[WARNING] Database may not be fully ready yet, please wait" -ForegroundColor Yellow
        }
        
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "  Database reset completed!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "Default admin account:" -ForegroundColor Cyan
        Write-Host "  Email: admin@ipig.local" -ForegroundColor White
        Write-Host "  Password: admin123" -ForegroundColor White
        Write-Host ""
        Write-Host "Service URLs:" -ForegroundColor Cyan
        Write-Host "  Frontend: http://localhost:8080" -ForegroundColor White
        Write-Host "  API:      http://localhost:3000" -ForegroundColor White
        Write-Host ""
        Write-Host "View logs:" -ForegroundColor Yellow
        Write-Host "  docker compose logs -f" -ForegroundColor White
        Write-Host ""
    } else {
        Write-Host ""
        Write-Host "[ERROR] Failed to start services" -ForegroundColor Red
        Write-Host "Please check if Docker Desktop is running" -ForegroundColor Yellow
        exit 1
    }
} else {
    Write-Host "[INFO] Docker not available, using local development mode" -ForegroundColor Cyan
    Write-Host ""
    
    # Check .env file
    $envFile = "backend\.env"
    if (-not (Test-Path $envFile)) {
        Write-Host "[ERROR] backend\.env file not found" -ForegroundColor Red
        Write-Host "Please create backend\.env file (copy from env.sample)" -ForegroundColor Yellow
        exit 1
    }
    
    # Read DATABASE_URL
    $databaseUrl = ""
    Get-Content $envFile | ForEach-Object {
        if ($_ -match "^DATABASE_URL=(.+)") {
            $databaseUrl = $matches[1]
        }
    }
    
    if ([string]::IsNullOrEmpty($databaseUrl)) {
        Write-Host "[ERROR] DATABASE_URL not found in backend\.env" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "[INFO] Using database: $databaseUrl" -ForegroundColor Cyan
    Write-Host ""
    
    # Check if SQLx CLI is installed
    $sqlxAvailable = $false
    try {
        sqlx --version 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            $sqlxAvailable = $true
        }
    } catch {
        $sqlxAvailable = $false
    }
    
    if (-not $sqlxAvailable) {
        Write-Host "[ERROR] SQLx CLI is not installed" -ForegroundColor Red
        Write-Host ""
        Write-Host "Please install SQLx CLI:" -ForegroundColor Yellow
        Write-Host "  cargo install sqlx-cli" -ForegroundColor White
        Write-Host ""
        Write-Host "Or manually reset using SQL commands:" -ForegroundColor Yellow
        Write-Host "  1. Connect to database" -ForegroundColor White
        Write-Host "  2. Run: DROP DATABASE ipig_db;" -ForegroundColor White
        Write-Host "  3. Run: CREATE DATABASE ipig_db;" -ForegroundColor White
        Write-Host "  4. Run migrations: sqlx migrate run" -ForegroundColor White
        exit 1
    }
    
    # Step 1: Drop and recreate database
    Write-Host "[1/3] Dropping existing database..." -ForegroundColor Yellow
    sqlx database drop --database-url $databaseUrl --yes
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[WARNING] Error dropping database (database may not exist)" -ForegroundColor Yellow
    }
    Write-Host "[OK] Database dropped" -ForegroundColor Green
    Write-Host ""
    
    # Step 2: Create new database
    Write-Host "[2/3] Creating new database..." -ForegroundColor Yellow
    sqlx database create --database-url $databaseUrl
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Failed to create database" -ForegroundColor Red
        exit 1
    }
    Write-Host "[OK] Database created" -ForegroundColor Green
    Write-Host ""
    
    # Step 3: Run migrations
    Write-Host "[3/3] Running migrations..." -ForegroundColor Yellow
    Push-Location backend
    sqlx migrate run --database-url $databaseUrl
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Migrations failed" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Pop-Location
    Write-Host "[OK] Migrations completed" -ForegroundColor Green
    Write-Host ""
    
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "  Database reset completed!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Default admin account:" -ForegroundColor Cyan
    Write-Host "  Email: admin@ipig.local" -ForegroundColor White
    Write-Host "  Password: admin123" -ForegroundColor White
    Write-Host ""
}

Write-Host ""
