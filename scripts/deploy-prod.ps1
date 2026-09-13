# iPig System - Prod Redeploy Script
#
# 用途：把 main branch 最新程式碼部署到 prod 機器的 Docker container
#
# 何時用：
#   - PR merge 後想讓改動生效
#   - 修改 backend Rust code 後
#   - 修改 frontend 後（前端在 web 容器內，要 rebuild）
#
# 參數（2026-09-13 參數化；不帶任何參數 = 舊行為 + outbox-worker）：
#   -Services <名稱[,...]>  要部署的服務，預設 api,web,outbox-worker
#                           ⚠️ 舊版寫死 `api web`，outbox-worker 從未被換版。
#                           infra／依賴更新要帶全部三個（見 CLAUDE.md §授權現況 (b)）。
#   -UseGhcr                部署路徑 A：疊 docker-compose.prod.yml，改用 GHCR 映像
#                           （pull 取代 build）。需 .env 有 GHCR_OWNER + 已 docker login ghcr.io。
#   -Tag <tag>              指定 IMAGE_TAG（建議 pin 短 sha，回滾才有座標）。僅 -UseGhcr 時有意義。
#   -SkipPull               不跑 git pull，部署當前 HEAD（回滾／部署特定 commit 用）。
#   -NonInteractive         不等鍵盤輸入（排程／自動化用）。無新 commit 時直接繼續 rebuild。
#   -DryRun                 只印出會執行的指令，不動 git、不動 docker。
#
# 範例：
#   .\scripts\deploy-prod.ps1                                   # 預設三服務，本機 build
#   .\scripts\deploy-prod.ps1 -Services api                     # 只換 api
#   .\scripts\deploy-prod.ps1 -UseGhcr -Tag 1f4c293             # 路徑 A，pin 短 sha
#   .\scripts\deploy-prod.ps1 -SkipPull -NonInteractive -DryRun # 看它會做什麼
#
# 流程：
#   1. 參數與服務名檢查（服務名對照 docker compose config --services）
#   2. 確認在 main branch、工作目錄 clean
#   3. git pull origin main（-SkipPull 時跳過）
#   4. build（本機）或 pull（GHCR）
#   5. docker compose up -d <services>
#   6. 健康檢查（服務清單含 api 時才做）
#
# 安全：
#   - 不動 DB / Postgres container（migration 由 backend 啟動自動跑）
#   - 失敗時不自動 rollback（保留錯誤狀態供 debug）
#   - 工作目錄不 clean 會拒絕跑（避免覆寫本地修改）
#   - 所有結束路徑皆顯式 exit（成功 0 / 失敗 1），呼叫端才能靠 $LASTEXITCODE 判定
#     （2026-08 code review Low-1：舊版成功路徑是 fall-through，會被殘留的 exit code 污染）

[CmdletBinding()]
param(
    [string[]]$Services = @('api', 'web', 'outbox-worker'),
    [switch]$UseGhcr,
    [string]$Tag,
    [switch]$SkipPull,
    [switch]$NonInteractive,
    [switch]$DryRun
)

# R51 follow-up：用 "Continue" 而非 "Stop"。
# 原因：PowerShell 5.1 在 "Stop" 模式下，native command（git / docker）寫到
# stderr 的 progress 訊息（"From https://..."、"#1 [internal] load metadata..." 等）
# 會被視為 NativeCommandError 拋出，導致 deploy 在 git pull 或 docker build
# 階段就中斷。本 script 對每個 native command 都已手動檢查 $LASTEXITCODE，
# 不依賴 EAP=Stop。
$ErrorActionPreference = "Continue"

# docker-compose.prod.yml 有 image: 覆寫的服務＝可從 GHCR 拉的服務。
# print-pdf 刻意不在內（商用字型不可隨 repo 散布，CI 建不出映像），
# 它在 -UseGhcr 模式下仍必須本機 build。
$GhcrBackedServices = @('api', 'web', 'outbox-worker', 'db-backup')

function Abort {
    param([string]$Message, [string]$Hint)
    Write-Host "[ABORT] $Message" -ForegroundColor Red
    if ($Hint) { Write-Host "        $Hint" -ForegroundColor Yellow }
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  iPig System - Prod Redeploy" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# ---------------------------------------------------------------
# 0. 參數檢查
# ---------------------------------------------------------------
# 自己拆逗號：直接呼叫（.\deploy-prod.ps1 -Services api,web）時 PowerShell 會拆，
# 但 `powershell -File ... -Services api,web` 不會，整串會變成單一元素。兩種寫法都要能用。
$Services = @($Services | ForEach-Object { $_ -split ',' } | ForEach-Object { "$_".Trim() } | Where-Object { $_ })
if ($Services.Count -eq 0) {
    Abort "-Services 不能是空清單。"
}

if ($Tag -and -not $UseGhcr) {
    Abort "-Tag 只在 -UseGhcr 模式下有意義（本機 build 不看 IMAGE_TAG）。" `
          "要部署特定 commit 請用：git checkout <sha> 後加 -SkipPull"
}

# 服務名對照 compose 真實定義，避免打錯字到 up -d 才爆（訊息難讀）
$knownServices = docker compose config --services 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host $knownServices -ForegroundColor Yellow
    Abort "docker compose config 失敗（exit $LASTEXITCODE）。Docker daemon 沒起來？"
}
$knownServices = @($knownServices | ForEach-Object { "$_".Trim() } | Where-Object { $_ })

$unknown = @($Services | Where-Object { $knownServices -notcontains $_ })
if ($unknown.Count -gt 0) {
    Abort "未知的服務名：$($unknown -join ', ')" "合法服務名：$($knownServices -join ', ')"
}

# compose 檔組合：路徑 A 疊 prod overlay
$composeArgs = @()
if ($UseGhcr) {
    $composeArgs = @('-f', 'docker-compose.yml', '-f', 'docker-compose.prod.yml')

    # GHCR_OWNER 缺了的話 image 會解析成 ghcr.io//ipig-api:...，錯誤訊息完全看不出原因
    $ghcrOwner = $env:GHCR_OWNER
    if (-not $ghcrOwner -and (Test-Path '.env')) {
        $match = Select-String -Path '.env' -Pattern '^\s*GHCR_OWNER\s*=\s*(\S+)' | Select-Object -First 1
        if ($match) { $ghcrOwner = $match.Matches[0].Groups[1].Value }
    }
    if (-not $ghcrOwner) {
        Abort "-UseGhcr 需要 GHCR_OWNER，但 .env 與環境變數皆查無。" `
              "見 TODO.md R114-1：.env 加 GHCR_OWNER / IMAGE_TAG + docker login ghcr.io（屬必問項，由使用者執行）"
    }

    if ($Tag) { $env:IMAGE_TAG = $Tag }  # shell 環境變數優先於 .env
}

$pullServices  = @()
$buildServices = @()
if ($UseGhcr) {
    $pullServices  = @($Services | Where-Object { $GhcrBackedServices -contains $_ })
    $buildServices = @($Services | Where-Object { $GhcrBackedServices -notcontains $_ })
} else {
    $buildServices = $Services
}

$mode = if ($UseGhcr) { "A（GHCR 映像）" } else { "B（本機 build）" }
Write-Host "服務：  $($Services -join ', ')" -ForegroundColor White
Write-Host "路徑：  $mode" -ForegroundColor White
if ($UseGhcr) {
    $tagShown = if ($Tag) { $Tag } else { "（沿用 .env 的 IMAGE_TAG，未設則 latest）" }
    Write-Host "映像 tag：$tagShown" -ForegroundColor White
    if ($buildServices.Count -gt 0) {
        Write-Host "本機 build：$($buildServices -join ', ')（無 GHCR 映像）" -ForegroundColor Yellow
    }
}
if ($DryRun) { Write-Host "模式：  DryRun（只印指令，不執行）" -ForegroundColor Magenta }
Write-Host ""

$totalSteps = 3
if (-not $SkipPull) { $totalSteps++ }
if ($Services -contains 'api') { $totalSteps++ }
$step = 0

# ---------------------------------------------------------------
# 1. Pre-flight: working tree clean + on main branch
# ---------------------------------------------------------------
$step++
Write-Host "[$step/$totalSteps] pre-flight（branch / 工作目錄）..." -ForegroundColor Cyan

$currentBranch = git rev-parse --abbrev-ref HEAD
if ($LASTEXITCODE -ne 0) {
    Abort "git rev-parse 失敗（exit $LASTEXITCODE，可能不在 git repo 內）"
}
if ($currentBranch -ne "main") {
    # -SkipPull 通常就是為了部署特定 commit（回滾），此時 detached HEAD 是正常的
    if ($SkipPull) {
        Write-Host "[WARN] 目前不在 main（'$currentBranch'），但 -SkipPull 指定，繼續。" -ForegroundColor Yellow
    } else {
        Abort "目前 branch 是 '$currentBranch'，需切到 main 才能 deploy。" "git checkout main"
    }
}

$dirty = git status --porcelain
if ($LASTEXITCODE -ne 0) {
    # 舊版沒檢查這行：git status 硬失敗且 stdout 空的話，dirty-tree 守門會被靜默略過
    Abort "git status 失敗（exit $LASTEXITCODE）"
}
if ($dirty) {
    Write-Host $dirty -ForegroundColor Yellow
    Abort "工作目錄有未 commit 變更，停下避免覆寫。" "請先 commit / stash 後再 deploy。"
}
Write-Host "[OK] branch=$currentBranch，工作目錄 clean" -ForegroundColor Green
Write-Host ""

# ---------------------------------------------------------------
# 2. Pull latest main
# ---------------------------------------------------------------
if ($SkipPull) {
    $headSha = git rev-parse --short HEAD
    Write-Host "[--] -SkipPull 指定，不拉新 commit，部署當前 HEAD $headSha" -ForegroundColor Yellow
    Write-Host ""
} else {
    $step++
    Write-Host "[$step/$totalSteps] git pull origin main ..." -ForegroundColor Cyan

    if ($DryRun) {
        Write-Host "  DRYRUN> git pull --ff-only origin main" -ForegroundColor Magenta
    } else {
        $beforeSha = git rev-parse HEAD
        if ($LASTEXITCODE -ne 0) { Abort "git rev-parse HEAD 失敗（exit $LASTEXITCODE）" }

        git pull --ff-only origin main
        if ($LASTEXITCODE -ne 0) {
            Abort "git pull 失敗（exit $LASTEXITCODE，可能 main 已分歧需 rebase）"
        }

        $afterSha = git rev-parse HEAD
        if ($LASTEXITCODE -ne 0) { Abort "git rev-parse HEAD（pull 後）失敗（exit $LASTEXITCODE）" }

        if ($beforeSha -eq $afterSha) {
            if ($NonInteractive) {
                Write-Host "[OK] 已是最新 main，無新 commit；-NonInteractive 指定，直接往下 rebuild。" -ForegroundColor Green
            } else {
                Write-Host "[OK] 已是最新 main，無新 commit 需要 deploy。" -ForegroundColor Green
                Write-Host "若仍要強制 rebuild，按 Enter 繼續，否則 Ctrl+C 取消。" -ForegroundColor Yellow
                Read-Host
            }
        } else {
            Write-Host "[OK] $beforeSha -> $afterSha" -ForegroundColor Green
            Write-Host ""
            Write-Host "新 commits：" -ForegroundColor Yellow
            git log --oneline "$beforeSha..$afterSha"
            Write-Host ""
        }
    }
}

# ---------------------------------------------------------------
# 3. 取得映像：pull（GHCR）或 build（本機）
# ---------------------------------------------------------------
$step++
Write-Host "[$step/$totalSteps] 取得映像 ..." -ForegroundColor Cyan

if ($pullServices.Count -gt 0) {
    $cmd = @('docker', 'compose') + $composeArgs + @('pull') + $pullServices
    if ($DryRun) {
        Write-Host "  DRYRUN> $($cmd -join ' ')" -ForegroundColor Magenta
    } else {
        Write-Host "  > $($cmd -join ' ')" -ForegroundColor DarkGray
        docker compose @composeArgs pull @pullServices
        if ($LASTEXITCODE -ne 0) {
            Abort "docker compose pull 失敗（exit $LASTEXITCODE）。" `
                  "未 docker login ghcr.io？或該 tag 不存在？"
        }
    }
}

if ($buildServices.Count -gt 0) {
    $cmd = @('docker', 'compose') + $composeArgs + @('build') + $buildServices
    if ($DryRun) {
        Write-Host "  DRYRUN> $($cmd -join ' ')" -ForegroundColor Magenta
    } else {
        Write-Host "  > $($cmd -join ' ')" -ForegroundColor DarkGray
        docker compose @composeArgs build @buildServices
        if ($LASTEXITCODE -ne 0) {
            Abort "docker build 失敗，請看上方錯誤訊息。"
        }
    }
}
if (-not $DryRun) { Write-Host "[OK] 映像就緒" -ForegroundColor Green }
Write-Host ""

# ---------------------------------------------------------------
# 4. 套用新映像（保留 DB / monitoring 容器）
# ---------------------------------------------------------------
$step++
Write-Host "[$step/$totalSteps] docker compose up -d ..." -ForegroundColor Cyan
$cmd = @('docker', 'compose') + $composeArgs + @('up', '-d') + $Services
if ($DryRun) {
    Write-Host "  DRYRUN> $($cmd -join ' ')" -ForegroundColor Magenta
} else {
    Write-Host "  > $($cmd -join ' ')" -ForegroundColor DarkGray
    docker compose @composeArgs up -d @Services
    if ($LASTEXITCODE -ne 0) {
        Abort "docker compose up 失敗。"
    }
    Write-Host "[OK] $($Services -join ', ') 已套用新映像" -ForegroundColor Green
}
Write-Host ""

# ---------------------------------------------------------------
# 5. Health check（只在部署了 api 時才有意義）
# ---------------------------------------------------------------
if ($Services -contains 'api') {
    $step++
    Write-Host "[$step/$totalSteps] 等待 api healthy ..." -ForegroundColor Cyan

    if ($DryRun) {
        Write-Host "  DRYRUN> docker inspect --format='{{.State.Health.Status}}' ipig-api（最多 30 次 / 60 秒）" -ForegroundColor Magenta
    } else {
        $maxRetries = 30
        $retry = 0
        while ($retry -lt $maxRetries) {
            Start-Sleep -Seconds 2
            $health = docker inspect --format='{{.State.Health.Status}}' ipig-api 2>$null
            if ($health -eq "healthy") {
                Write-Host "[OK] api healthy" -ForegroundColor Green
                break
            }
            if ($health -eq "unhealthy") {
                Abort "api unhealthy — 查看 logs：" "docker compose logs --tail=50 api"
            }
            $retry++
        }
        if ($retry -ge $maxRetries) {
            Write-Host "[WARN] 健康檢查 60 秒超時，但容器仍 starting — 可手動再確認：" -ForegroundColor Yellow
            Write-Host "  docker ps --filter name=ipig-api" -ForegroundColor White
        }
    }
    Write-Host ""
}

# ---------------------------------------------------------------
Write-Host "========================================" -ForegroundColor Cyan
if ($DryRun) {
    Write-Host "  DryRun 結束（未執行任何指令）" -ForegroundColor Magenta
} else {
    Write-Host "  Deploy 完成" -ForegroundColor Cyan
}
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "驗證：" -ForegroundColor Yellow
Write-Host "  https://ipigsystem.asia/" -ForegroundColor White
Write-Host "  https://ipigsystem.asia/admin/audit?tab=alerts" -ForegroundColor White
Write-Host ""
Write-Host "查看 api 日誌：" -ForegroundColor Yellow
Write-Host "  docker compose logs -f api" -ForegroundColor White
Write-Host ""

# 顯式 exit：呼叫端（auto-deploy-watcher.ps1）用 $LASTEXITCODE 判定成敗，
# fall-through 會讓殘留的 native command exit code 冒充本腳本的結果。
exit 0
