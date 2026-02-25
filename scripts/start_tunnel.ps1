# start_tunnel.ps1
# Cloudflare Quick Tunnel launcher with auto APP_URL update
# Usage: .\start_tunnel.ps1 [-Port 8080]
param([int]$Port = 8080)

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host ""
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  Cloudflare Quick Tunnel Starting..." -ForegroundColor Cyan
Write-Host "  Local: http://localhost:$Port" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Check cloudflared is installed
$cfPath = Get-Command cloudflared -ErrorAction SilentlyContinue
if (-not $cfPath) {
    Write-Host "[ERROR] cloudflared not found." -ForegroundColor Red
    Write-Host "  Install: winget install Cloudflare.cloudflared" -ForegroundColor Yellow
    exit 1
}

# Kill existing cloudflared
$existing = Get-Process -Name "cloudflared" -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "[WARN] cloudflared already running (PID: $($existing.Id)), stopping..." -ForegroundColor Yellow
    Stop-Process -Name "cloudflared" -Force
    Start-Sleep -Seconds 2
}

# Temp log file for stderr
$logFile = Join-Path $env:TEMP "cloudflared_tunnel.log"
if (Test-Path $logFile) { Remove-Item $logFile -Force }

# Start cloudflared
Write-Host "[START] Creating tunnel..." -ForegroundColor Green
$process = Start-Process -FilePath "cloudflared" -ArgumentList "tunnel","--url","http://localhost:$Port" -RedirectStandardError $logFile -PassThru -NoNewWindow

# Wait for tunnel URL (max 30 sec)
$tunnelUrl = $null
$maxWait = 30
$waited = 0

while ($waited -lt $maxWait) {
    Start-Sleep -Seconds 2
    $waited += 2

    if (Test-Path $logFile) {
        $content = Get-Content $logFile -Raw -ErrorAction SilentlyContinue
        if ($content -and ($content -match 'https://[a-zA-Z0-9\-]+\.trycloudflare\.com')) {
            $tunnelUrl = $Matches.0
            break
        }
    }

    if ($process.HasExited) {
        Write-Host "[ERROR] cloudflared exited unexpectedly." -ForegroundColor Red
        if (Test-Path $logFile) { Get-Content $logFile }
        exit 1
    }

    Write-Host "  Waiting... ($waited sec)" -ForegroundColor DarkGray
}

if (-not $tunnelUrl) {
    Write-Host ""
    Write-Host "[TIMEOUT] Could not get tunnel URL in $maxWait seconds." -ForegroundColor Red
    Write-Host "cloudflared still running (PID: $($process.Id))." -ForegroundColor Yellow
    Write-Host "Check log: $logFile" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "=====================================" -ForegroundColor Green
Write-Host "  Tunnel Ready!" -ForegroundColor Green
Write-Host "  URL: $tunnelUrl" -ForegroundColor Yellow
Write-Host "=====================================" -ForegroundColor Green
Write-Host ""

# Copy to clipboard
Set-Clipboard -Value $tunnelUrl
Write-Host "[COPIED] URL copied to clipboard!" -ForegroundColor Magenta
Write-Host ""

# --- Update APP_URL in .env files ---
Write-Host "[UPDATE] Updating APP_URL in .env files..." -ForegroundColor Cyan

function Update-EnvFile {
    param([string]$FilePath, [string]$NewUrl)
    if (-not (Test-Path $FilePath)) {
        Write-Host "  [SKIP] $FilePath not found" -ForegroundColor DarkGray
        return
    }
    $content = Get-Content $FilePath -Raw -Encoding UTF8
    $pattern = 'APP_URL=.*'
    $replacement = "APP_URL=$NewUrl"
    if ($content -match $pattern) {
        $newContent = $content -replace $pattern, $replacement
        $newContent | Set-Content $FilePath -Encoding UTF8 -NoNewline
        Write-Host "  [OK] $FilePath -> $NewUrl" -ForegroundColor Green
    } else {
        # APP_URL not found, append it
        Add-Content $FilePath "`nAPP_URL=$NewUrl"
        Write-Host "  [ADD] $FilePath -> $NewUrl" -ForegroundColor Green
    }
}

$rootEnv = Join-Path $projectRoot ".env"
$backendEnv = Join-Path $projectRoot "backend\.env"

Update-EnvFile -FilePath $rootEnv -NewUrl $tunnelUrl
Update-EnvFile -FilePath $backendEnv -NewUrl $tunnelUrl

# --- Restart Docker API container to pick up new APP_URL ---
Write-Host ""
Write-Host "[RESTART] Restarting Docker API container..." -ForegroundColor Cyan

$dockerCheck = Get-Command docker -ErrorAction SilentlyContinue
if ($dockerCheck) {
    $composeFile = Join-Path $projectRoot "docker-compose.yml"
    if (Test-Path $composeFile) {
        Push-Location $projectRoot
        docker compose up -d api
        Pop-Location
        Write-Host "  [OK] API container restarted with new APP_URL!" -ForegroundColor Green
    } else {
        Write-Host "  [SKIP] docker-compose.yml not found, skipping restart" -ForegroundColor Yellow
    }
} else {
    Write-Host "  [SKIP] Docker not found, skipping restart" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=====================================" -ForegroundColor Green
Write-Host "  All done! System is ready." -ForegroundColor Green
Write-Host "  Tunnel: $tunnelUrl" -ForegroundColor Yellow
Write-Host "  Email links will use this URL" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Green
Write-Host ""
Write-Host "  - URL changes on each restart" -ForegroundColor DarkGray
Write-Host "  - Press Ctrl+C or close window to stop" -ForegroundColor DarkGray
Write-Host "  - cloudflared PID: $($process.Id)" -ForegroundColor DarkGray
Write-Host ""

# Keep running until user stops
Write-Host "Tunnel running... Ctrl+C to stop." -ForegroundColor DarkGray
Wait-Process -Id $process.Id -ErrorAction SilentlyContinue

if (-not $process.HasExited) {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
}
Write-Host ""
Write-Host "Tunnel stopped." -ForegroundColor Yellow
