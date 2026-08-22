# 載入 MSVC 環境（link.exe）供 Rust cargo build 使用
# 用法：. .\scripts\load-msvc-env.ps1
$ErrorActionPreference = "Stop"

$vcvarsPaths = @(
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
)

$vcvars = $null
foreach ($p in $vcvarsPaths) {
    if (Test-Path $p) {
        $vcvars = $p
        break
    }
}

if (-not $vcvars) {
    Write-Host "[X] vcvars64.bat not found. Install Visual Studio Build Tools:" -ForegroundColor Red
    Write-Host "    1. https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Yellow
    Write-Host "    2. Select 'Desktop development with C++'" -ForegroundColor Yellow
    Write-Host "    3. Or: MSVC v143 + Windows SDK" -ForegroundColor Yellow
    Write-Host "    Then: . .\scripts\load-msvc-env.ps1" -ForegroundColor Cyan
    throw "link.exe env not ready"
}

$cmdLine = '"' + $vcvars + '" & set'
& $env:SystemRoot\system32\cmd.exe /c $cmdLine | ForEach-Object {
    if ($_ -match '^([^=]*)=(.*)$') {
        $k = $matches[1]
        if ($k -and $k[0] -ne [char]33) { Set-Item -Path "Env:$k" -Value $matches[2] -Force }
    }
}
Write-Host "[OK] MSVC env loaded, link.exe ready" -ForegroundColor Green
