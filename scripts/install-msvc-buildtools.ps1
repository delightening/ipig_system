# Download and install Visual Studio Build Tools (C++ workload) for link.exe
# Requires: Run as Administrator
# Usage: .\scripts\install-msvc-buildtools.ps1

$ErrorActionPreference = "Stop"

$vsUrl = "https://aka.ms/vs/17/release/vs_BuildTools.exe"
$downloadDir = Join-Path $env:TEMP "vs_buildtools"
$installerPath = Join-Path $downloadDir "vs_BuildTools.exe"

Write-Host ""
Write-Host "=== Visual Studio Build Tools Installer ===" -ForegroundColor Cyan
Write-Host ""

# Check admin (required for VS install)
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[X] Run as Administrator:" -ForegroundColor Red
    Write-Host "    Right-click PowerShell -> Run as administrator" -ForegroundColor Yellow
    Write-Host "    Then: cd 'c:\System Coding\ipig_system'; .\scripts\install-msvc-buildtools.ps1" -ForegroundColor Cyan
    exit 1
}

# Create download dir
if (-not (Test-Path $downloadDir)) {
    New-Item -ItemType Directory -Path $downloadDir -Force | Out-Null
}

# Download
if (-not (Test-Path $installerPath)) {
    Write-Host "[1/2] Downloading vs_BuildTools.exe ..." -ForegroundColor Yellow
    try {
        Invoke-WebRequest -Uri $vsUrl -OutFile $installerPath -UseBasicParsing
    } catch {
        Write-Host "[X] Download failed: $_" -ForegroundColor Red
        exit 1
    }
    Write-Host "      Done." -ForegroundColor Green
} else {
    Write-Host "[1/2] Installer already exists: $installerPath" -ForegroundColor Gray
}

# Install C++ workload (VCTools = link.exe, cl.exe, etc.)
Write-Host "[2/2] Installing C++ Build Tools (VCTools workload) ..." -ForegroundColor Yellow
Write-Host "      This may take 10-30 minutes. Do not close this window." -ForegroundColor Gray
Write-Host ""

$args = @(
    "--quiet",
    "--wait",
    "--norestart",
    "--add", "Microsoft.VisualStudio.Workload.VCTools",
    "--includeRecommended"
)

$proc = Start-Process -FilePath $installerPath -ArgumentList $args -Wait -PassThru

if ($proc.ExitCode -eq 0) {
    Write-Host ""
    Write-Host "[OK] Build Tools installed successfully." -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Close and reopen your terminal" -ForegroundColor White
    Write-Host "  2. Run: . .\scripts\load-msvc-env.ps1" -ForegroundColor White
    Write-Host "  3. Run: cd backend; cargo build" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host ""
    Write-Host "[X] Installer exited with code $($proc.ExitCode)" -ForegroundColor Red
    Write-Host "     Try running the installer manually: $installerPath" -ForegroundColor Yellow
    exit 1
}
