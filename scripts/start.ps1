# iPig System Startup Script

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  iPig System - Startup Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
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
    Write-Host "[OK] Docker is available" -ForegroundColor Green
    Write-Host ""
    Write-Host "Starting Docker Compose services..." -ForegroundColor Yellow
    
    docker compose up -d
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "[SUCCESS] Services started!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Service URLs:" -ForegroundColor Cyan
        Write-Host "  Frontend: http://localhost:8080" -ForegroundColor White
        Write-Host "  API:      http://localhost:3000" -ForegroundColor White
        Write-Host "  Database: localhost:5433" -ForegroundColor White
        Write-Host ""
        Write-Host "View logs:" -ForegroundColor Yellow
        Write-Host "  docker compose logs -f" -ForegroundColor White
        Write-Host ""
        Write-Host "Stop services:" -ForegroundColor Yellow
        Write-Host "  docker compose down" -ForegroundColor White
    } else {
        Write-Host ""
        Write-Host "[ERROR] Failed to start services" -ForegroundColor Red
        Write-Host "Please check if Docker Desktop is running" -ForegroundColor Yellow
    }
} else {
    Write-Host "[WARNING] Docker is not available" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please choose a startup method:" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "1. Using Docker Compose (Recommended)" -ForegroundColor Cyan
    Write-Host "   Start Docker Desktop first, then run:" -ForegroundColor White
    Write-Host "   docker compose up -d" -ForegroundColor Green
    Write-Host ""
    Write-Host "2. Local Development Mode" -ForegroundColor Cyan
    Write-Host "   Backend:" -ForegroundColor White
    Write-Host "   cd backend" -ForegroundColor Green
    Write-Host "   cargo run" -ForegroundColor Green
    Write-Host ""
    Write-Host "   Frontend:" -ForegroundColor White
    Write-Host "   cd frontend" -ForegroundColor Green
    Write-Host "   npm install" -ForegroundColor Green
    Write-Host "   npm run dev" -ForegroundColor Green
}

Write-Host ""
