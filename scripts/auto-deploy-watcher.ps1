# iPig System - Auto Deploy Watcher (R51)
#
# 用途：每 N 分鐘 (預設由 Task Scheduler 觸發) 檢查 origin/main 是否有新 commit，
# 有則自動觸發 deploy-prod.ps1。為「pull-based auto deploy」設計。
#
# 安全：
#   - Lock file 避免兩個 watcher 同時跑 deploy（造成 docker build 衝突）
#   - 只在 main branch + 工作目錄 clean 時動作，否則 abort（讓 user 看到狀況）
#   - Deploy 失敗 → log + 等下次 polling 重試（不會 panic）
#
# 安裝：先跑 scripts/install-auto-deploy.ps1 註冊 Task Scheduler
#
# Log: $env:LOCALAPPDATA\ipig-auto-deploy.log（rotation 由 user 手動，預期極少）

$ErrorActionPreference = "Stop"

# 路徑常量
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogFile = Join-Path $env:LOCALAPPDATA "ipig-auto-deploy.log"
$LockFile = Join-Path $env:TEMP "ipig-auto-deploy.lock"
$DeployScript = Join-Path $PSScriptRoot "deploy-prod.ps1"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $line = "[$ts] [$Level] $Message"
    Add-Content -Path $LogFile -Value $line -Encoding utf8
    Write-Host $line
}

# Lock：避免兩個 watcher 同時跑
# 過期門檻必須 >= install-auto-deploy.ps1 的 ExecutionTimeLimit（30 分），否則一次
# 合法跑 20~30 分的 deploy 會被下一輪誤判 lock 過期清掉、啟動並行 docker build（#399）。
# 取 35 分留 5 分 buffer。
if (Test-Path $LockFile) {
    $lockAge = (Get-Date) - (Get-Item $LockFile).CreationTime
    if ($lockAge.TotalMinutes -lt 35) {
        Write-Log "另一個 watcher 還在跑（lock $($lockAge.TotalMinutes.ToString('0')) 分鐘），跳過本輪。" "WARN"
        exit 0
    }
    Write-Log "Lock file 過期 > 35 分鐘（可能 deploy 卡死）；移除 lock 繼續。" "WARN"
    Remove-Item $LockFile -Force
}
New-Item -Path $LockFile -ItemType File -Force | Out-Null

try {
    Set-Location $RepoRoot

    # Pre-flight：必在 main + 工作目錄 clean
    $branch = git rev-parse --abbrev-ref HEAD
    if ($branch -ne "main") {
        Write-Log "目前 branch 是 '$branch'，watcher 只在 main 動作。abort。" "WARN"
        exit 0
    }

    $dirty = git status --porcelain
    if ($dirty) {
        Write-Log "工作目錄有未 commit 變更，watcher 不動作避免覆寫。abort。" "WARN"
        exit 0
    }

    # Fetch + 比對
    git fetch --quiet origin main 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Log "git fetch 失敗（exit $LASTEXITCODE）。網路問題？下一輪重試。" "ERROR"
        exit 0
    }

    $localSha = git rev-parse HEAD
    $remoteSha = git rev-parse origin/main

    if ($localSha -eq $remoteSha) {
        # 一般狀況：log 一行 tick（DEBUG level）證明 watcher 健康但無事可做。
        # 2026-05-18 加入此行 — R51 因 S4U+HighestAvailable bad combo 曾 silently
        # 死過 ~6 小時無人發覺，因為原版「靜默 exit」遇上「沒新 commit」就完全
        # 沒輸出。tick log 讓我們區分「watcher 沒 fire」vs「watcher fire 但無事」。
        Write-Log "tick — no new commits (HEAD $($localSha.Substring(0,7)))" "DEBUG"
        exit 0
    }

    # 有新 commit → 跑 deploy
    Write-Log "偵測到新 commit ($localSha → $remoteSha)，觸發 deploy-prod.ps1。" "INFO"

    # 列出將套用的 commits
    $commits = git log --oneline "$localSha..$remoteSha"
    $commits | ForEach-Object { Write-Log "  $_" "INFO" }

    # 呼叫 deploy 時放寬 $ErrorActionPreference："Stop" 模式下 git pull 寫
    # "From https://..." 到 stderr 會被 PowerShell 5.1 視為 NativeCommandError
    # 拋出，導致 docker build/up 沒跑就被外層 catch 抓走（首次實戰觀察）。
    # 改用 "Continue" 並用 $LASTEXITCODE 判斷成敗（git/docker 都遵循 exit code）。
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $DeployScript *>&1 | Tee-Object -FilePath $LogFile -Append
        $deployExit = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $prevEAP
    }

    if ($deployExit -eq 0) {
        Write-Log "Deploy 成功。" "INFO"
    } else {
        Write-Log "Deploy 失敗（exit $deployExit）。下一輪 watcher 不會重試（git 已 fast-forward），需手動處理：docker compose logs api" "ERROR"
    }
}
catch {
    Write-Log "Watcher unhandled exception: $_" "ERROR"
}
finally {
    Remove-Item $LockFile -Force -ErrorAction SilentlyContinue
}
