# docker-up.ps1
# Wrapper for `docker compose up -d --build` that validates the current tunnel URL
# Usage: .\scripts\docker-up.ps1

param(
    [switch]$NoBuild,
    [switch]$SkipCheck
)

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent
$envFile = Join-Path $projectRoot ".env"

Write-Host ""
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  iPig System - Docker Up" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# --- 檢查目前 .env 中的 APP_URL 和 CORS ---
if (Test-Path $envFile) {
    $envContent = Get-Content $envFile -Raw -Encoding UTF8

    # 讀取 APP_URL
    $appUrl = ""
    if ($envContent -match 'APP_URL=(.+)') {
        $appUrl = $Matches[1].Trim()
    }

    # 讀取 CORS_ALLOWED_ORIGINS
    $corsOrigins = ""
    if ($envContent -match 'CORS_ALLOWED_ORIGINS=(.+)') {
        $corsOrigins = $Matches[1].Trim()
    }

    Write-Host "[INFO] Current configuration:" -ForegroundColor Cyan
    Write-Host "  APP_URL:              $appUrl" -ForegroundColor White
    Write-Host "  CORS_ALLOWED_ORIGINS: $corsOrigins" -ForegroundColor White
    Write-Host ""

    if (-not $SkipCheck) {
        # 檢查是否為 Cloudflare tunnel URL
        $isTunnel = $appUrl -match '\.trycloudflare\.com'

        if ($isTunnel) {
            # 驗證 tunnel 是否仍在運作
            Write-Host "[CHECK] Verifying tunnel URL is accessible..." -ForegroundColor Yellow
            try {
                $response = Invoke-WebRequest -Uri $appUrl -Method Head -TimeoutSec 5 -UseBasicParsing -ErrorAction Stop
                Write-Host "  [OK] Tunnel is active (HTTP $($response.StatusCode))" -ForegroundColor Green
            } catch {
                Write-Host "  [WARN] Tunnel URL is NOT reachable!" -ForegroundColor Red
                Write-Host "         $appUrl" -ForegroundColor Yellow
                Write-Host ""
                Write-Host "  The tunnel may have expired. Options:" -ForegroundColor Yellow
                Write-Host "    1. Run .\scripts\start_tunnel.ps1 to create a new tunnel" -ForegroundColor White
                Write-Host "    2. Use -SkipCheck to proceed anyway" -ForegroundColor White
                Write-Host ""

                $response = Read-Host "Continue anyway? (y/N)"
                if ($response -ne 'y' -and $response -ne 'Y') {
                    Write-Host "Aborted." -ForegroundColor Yellow
                    exit 0
                }
            }
        } elseif ($appUrl -eq 'http://localhost' -or $appUrl -eq 'https://your-domain.example.com' -or [string]::IsNullOrWhiteSpace($appUrl)) {
            Write-Host "  [WARN] APP_URL is set to default/placeholder: $appUrl" -ForegroundColor Yellow
            Write-Host "         Email links (login, password reset) will use this URL." -ForegroundColor Yellow
            Write-Host "         Run .\scripts\start_tunnel.ps1 to set up a public tunnel." -ForegroundColor Yellow
            Write-Host ""
        } else {
            Write-Host "  [OK] APP_URL looks configured" -ForegroundColor Green
        }

        # 檢查 CORS 是否包含 APP_URL
        if (-not [string]::IsNullOrWhiteSpace($appUrl) -and $appUrl -ne 'http://localhost' -and $corsOrigins -notmatch [regex]::Escape($appUrl)) {
            Write-Host "  [WARN] CORS_ALLOWED_ORIGINS does not include APP_URL!" -ForegroundColor Yellow
            Write-Host "         Frontend via tunnel may fail API calls." -ForegroundColor Yellow
            Write-Host "         Run .\scripts\start_tunnel.ps1 to fix this automatically." -ForegroundColor Yellow
            Write-Host ""
        }
    }
} else {
    Write-Host "[WARN] .env file not found at $envFile" -ForegroundColor Red
    Write-Host "  Copy .env.example to .env and configure it first." -ForegroundColor Yellow
    exit 1
}

# --- 執行 docker compose up ---
Write-Host ""
$buildFlag = if ($NoBuild) { "" } else { "--build" }
$cmd = "docker compose up -d $buildFlag"
Write-Host "[RUN] $cmd" -ForegroundColor Green
Write-Host ""

Push-Location $projectRoot
if ($NoBuild) {
    docker compose up -d
} else {
    docker compose up -d --build
}
$exitCode = $LASTEXITCODE
Pop-Location

if ($exitCode -eq 0) {
    Write-Host ""
    Write-Host "=====================================" -ForegroundColor Green
    Write-Host "  Services started successfully!" -ForegroundColor Green
    Write-Host "=====================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Local:  http://localhost:8080" -ForegroundColor White
    if ($appUrl -match '\.trycloudflare\.com') {
        Write-Host "  Tunnel: $appUrl" -ForegroundColor Yellow
    }
    Write-Host ""
} else {
    Write-Host ""
    Write-Host "[ERROR] docker compose failed (exit code: $exitCode)" -ForegroundColor Red
    exit $exitCode
}
