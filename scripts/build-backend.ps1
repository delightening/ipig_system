# 在 MSVC 環境下編譯後端（解決 link.exe not found）
# 用法：.\scripts\build-backend.ps1 [cargo 參數...]
# 例：  .\scripts\build-backend.ps1
#       .\scripts\build-backend.ps1 --release
$ErrorActionPreference = "Stop"
$rootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$backendDir = Join-Path $rootDir "backend"

# 載入 MSVC 環境
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptDir "load-msvc-env.ps1")

Set-Location $backendDir
$cargoArgs = if ($args.Count -gt 0) { $args } else { @("build") }
cargo @cargoArgs
