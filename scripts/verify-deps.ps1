# 高效驗證：前端 + 後端並行，任一失敗即退出
# 用法：.\scripts\verify-deps.ps1
$ErrorActionPreference = "Stop"
$rootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $rootDir

$feJob = Start-Job -ScriptBlock {
    Set-Location $using:rootDir\frontend
    pnpm install --frozen-lockfile
    pnpm run build
    pnpm run test:run
}

$beJob = Start-Job -ScriptBlock {
    Set-Location $using:rootDir\backend
    cargo check --release
    cargo test
}

$feResult = Wait-Job $feJob | Receive-Job
$beResult = Wait-Job $beJob | Receive-Job

$feFailed = $feJob.State -eq "Failed"
$beFailed = $beJob.State -eq "Failed"

if ($feFailed) { Write-Host $feResult; Remove-Job $feJob -Force }
if ($beFailed) { Write-Host $beResult; Remove-Job $beJob -Force }
Remove-Job $feJob, $beJob -Force -ErrorAction SilentlyContinue

if ($feFailed -or $beFailed) {
    Write-Error "驗證失敗"
    exit 1
}
Write-Host "✅ 驗證通過"
