# start_named_tunnel.ps1
# Cloudflare Named Tunnel launcher
# SEC-15: Persistent tunnel setup
param(
    [string]$TunnelName = "ipig-system",
    [string]$ConfigPath = "cloudflared-config.yml"
)

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
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

# Run tunnel
cloudflared tunnel --config $ConfigPath run $TunnelName
