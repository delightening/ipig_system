@echo off
REM Download and install VS Build Tools (C++ workload) for link.exe
REM Run as Administrator: right-click -> Run as administrator
setlocal

set "VS_URL=https://aka.ms/vs/17/release/vs_BuildTools.exe"
set "DOWNLOAD_DIR=%TEMP%\vs_buildtools"
set "INSTALLER=%DOWNLOAD_DIR%\vs_BuildTools.exe"

echo.
echo === Visual Studio Build Tools Installer ===
echo.

REM Check admin (simple check)
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [X] Please run as Administrator
    echo     Right-click this file -^> Run as administrator
    pause
    exit /b 1
)

if not exist "%DOWNLOAD_DIR%" mkdir "%DOWNLOAD_DIR%"

REM Download
if not exist "%INSTALLER%" (
    echo [1/2] Downloading vs_BuildTools.exe ...
    powershell -NoProfile -Command "Invoke-WebRequest -Uri 'https://aka.ms/vs/17/release/vs_BuildTools.exe' -OutFile '%INSTALLER%' -UseBasicParsing"
    if %errorlevel% neq 0 (
        echo [X] Download failed
        pause
        exit /b 1
    )
    echo       Done.
) else (
    echo [1/2] Installer exists: %INSTALLER%
)

REM Install
echo [2/2] Installing C++ Build Tools (10-30 min) ...
echo.
"%INSTALLER%" --quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended

if %errorlevel% equ 0 (
    echo.
    echo [OK] Build Tools installed.
    echo.
    echo Next: Close terminal, reopen, then run:
    echo   . .\scripts\load-msvc-env.ps1
    echo   cd backend
    echo   cargo build
) else (
    echo [X] Installer exited with error
)
echo.
pause
