# start_named_tunnel.ps1
# Cloudflare Named Tunnel launcher
# SEC-15: Persistent tunnel setup
param(
    [string]$TunnelName = "ipig-system",
    [string]$ConfigPath = "deploy/cloudflared-config.yml"
)

$projectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $projectRoot

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  Cloudflare Named Tunnel Starting..." -ForegroundColor Cyan
Write-Host "  Tunnel Name: $TunnelName"
Write-Host "=====================================" -ForegroundColor Cyan

# Check cloudflared is installed
$cfPath = Get-Command cloudflared -ErrorAction SilentlyContinue
if (-not $cfPath) {
    Write-Host "[ERROR] cloudflared not found." -ForegroundColor Red
    exit 1
}

# Origin certificate (required for named tunnels). Set if not already set.
$defaultCertPath = Join-Path $env:USERPROFILE ".cloudflared\cert.pem"
if (-not $env:TUNNEL_ORIGIN_CERT) {
    $env:TUNNEL_ORIGIN_CERT = $defaultCertPath
}
if (-not (Test-Path -LiteralPath $env:TUNNEL_ORIGIN_CERT)) {
    Write-Host "[INFO] Origin certificate not found. Running login now..." -ForegroundColor Yellow
    Write-Host "  (Browser will open for Cloudflare sign-in.)" -ForegroundColor Gray
    $loginProc = Start-Process -FilePath "cloudflared" -ArgumentList "tunnel", "login" -NoNewWindow -Wait -PassThru
    if ($loginProc.ExitCode -ne 0) {
        Write-Host "[ERROR] Login failed or was cancelled (exit code $($loginProc.ExitCode))." -ForegroundColor Red
        exit 1
    }
    if (-not (Test-Path -LiteralPath $env:TUNNEL_ORIGIN_CERT)) {
        Write-Host "[ERROR] Certificate still not at: $env:TUNNEL_ORIGIN_CERT" -ForegroundColor Red
        exit 1
    }
    Write-Host "[OK] Login complete. Starting tunnel..." -ForegroundColor Green
}

# --- Read hostname from config and update APP_URL (email links) ---
$configFullPath = Join-Path $projectRoot $ConfigPath
$appUrl = $null
if (Test-Path -LiteralPath $configFullPath) {
    $configContent = Get-Content $configFullPath -Raw -ErrorAction SilentlyContinue
    if ($configContent -match 'hostname:\s*(\S+)') {
        $hostname = $Matches[1].Trim()
        $appUrl = "https://$hostname"
    }
}
if ($appUrl) {
    Write-Host "[UPDATE] Updating APP_URL for email links..." -ForegroundColor Cyan
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
            Add-Content $FilePath "`nAPP_URL=$NewUrl"
            Write-Host "  [ADD] $FilePath -> $NewUrl" -ForegroundColor Green
        }
    }
    $rootEnv = Join-Path $projectRoot ".env"
    $backendEnv = Join-Path $projectRoot "backend\.env"
    Update-EnvFile -FilePath $rootEnv -NewUrl $appUrl
    Update-EnvFile -FilePath $backendEnv -NewUrl $appUrl
    Write-Host "[RESTART] Restarting Docker API container..." -ForegroundColor Cyan
    $dockerCheck = Get-Command docker -ErrorAction SilentlyContinue
    if ($dockerCheck) {
        $composeFile = Join-Path $projectRoot "docker-compose.yml"
        if (Test-Path $composeFile) {
            Push-Location $projectRoot
            docker compose up -d api
            Pop-Location
            Write-Host "  [OK] API container restarted with APP_URL=$appUrl" -ForegroundColor Green
        } else {
            Write-Host "  [SKIP] docker-compose.yml not found" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  [SKIP] Docker not found" -ForegroundColor Yellow
    }
} else {
    Write-Host "[SKIP] Could not read hostname from $ConfigPath, APP_URL not updated" -ForegroundColor Yellow
}

# Run tunnel
cloudflared tunnel --config $ConfigPath run $TunnelName
