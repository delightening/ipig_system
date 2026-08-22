# iPig System - Install Auto Deploy Watcher (R51)
#
# 用途：註冊 Windows Task Scheduler task 跑 auto-deploy-watcher.ps1。
#
# 用法：在 prod laptop 的 PowerShell（**Administrator**）跑：
#   .\scripts\install-auto-deploy.ps1
#
# 卸載：
#   .\scripts\install-auto-deploy.ps1 -Uninstall
#
# 自訂 polling 間隔（預設 5 分鐘）：
#   .\scripts\install-auto-deploy.ps1 -IntervalMinutes 10

[CmdletBinding()]
param(
    [int]$IntervalMinutes = 5,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$TaskName = "iPig-Auto-Deploy-Watcher"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$WatcherScript = Join-Path $PSScriptRoot "auto-deploy-watcher.ps1"

if (-not (([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))) {
    Write-Host "[ABORT] 必須以管理員身份執行（Task Scheduler 註冊需要）。" -ForegroundColor Red
    Write-Host "在 PowerShell 圖示按右鍵 → 以系統管理員身份執行，再跑本 script。" -ForegroundColor Yellow
    exit 1
}

if ($Uninstall) {
    if (Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue) {
        Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
        Write-Host "[OK] $TaskName 已移除。" -ForegroundColor Green
    } else {
        Write-Host "[OK] 未找到 $TaskName，無需移除。" -ForegroundColor Green
    }
    exit 0
}

if (-not (Test-Path $WatcherScript)) {
    Write-Host "[ABORT] 找不到 watcher script: $WatcherScript" -ForegroundColor Red
    exit 1
}

# 若已存在 → 先 unregister 避免衝突
if (Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue) {
    Write-Host "[INFO] $TaskName 已存在，移除後重建。" -ForegroundColor Yellow
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
}

$action = New-ScheduledTaskAction `
    -Execute "powershell.exe" `
    -Argument "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$WatcherScript`""

$trigger = New-ScheduledTaskTrigger `
    -Once `
    -At (Get-Date).AddMinutes(1) `
    -RepetitionInterval (New-TimeSpan -Minutes $IntervalMinutes)

# 以當前 user 身分跑（避免 SYSTEM 跑 docker 權限問題；docker desktop 是 user-scoped）
$principal = New-ScheduledTaskPrincipal `
    -UserId "$env:USERDOMAIN\$env:USERNAME" `
    -LogonType Interactive `
    -RunLevel Highest

$settings = New-ScheduledTaskSettingsSet `
    -AllowStartIfOnBatteries `
    -DontStopIfGoingOnBatteries `
    -StartWhenAvailable `
    -ExecutionTimeLimit (New-TimeSpan -Minutes 30)

Register-ScheduledTask `
    -TaskName $TaskName `
    -Action $action `
    -Trigger $trigger `
    -Principal $principal `
    -Settings $settings `
    -Description "iPig 自動部署 watcher (R51)：每 $IntervalMinutes 分鐘 git fetch + 偵測 main 新 commit → 跑 deploy-prod.ps1" | Out-Null

Write-Host ""
Write-Host "[OK] $TaskName 已註冊。" -ForegroundColor Green
Write-Host "Polling 間隔: $IntervalMinutes 分鐘" -ForegroundColor White
Write-Host "Watcher script: $WatcherScript" -ForegroundColor White
Write-Host "Log file: $env:LOCALAPPDATA\ipig-auto-deploy.log" -ForegroundColor White
Write-Host ""
Write-Host "下一輪約 1 分鐘後跑。立刻測試：" -ForegroundColor Yellow
Write-Host "  Start-ScheduledTask -TaskName '$TaskName'" -ForegroundColor White
Write-Host ""
Write-Host "查看狀態：" -ForegroundColor Yellow
Write-Host "  Get-ScheduledTask -TaskName '$TaskName' | Get-ScheduledTaskInfo" -ForegroundColor White
Write-Host ""
Write-Host "查看 log（tail 30 行）：" -ForegroundColor Yellow
Write-Host "  Get-Content `"$env:LOCALAPPDATA\ipig-auto-deploy.log`" -Tail 30" -ForegroundColor White
Write-Host ""
