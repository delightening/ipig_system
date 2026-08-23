# DR 待命機：定期從物件儲存拉取正式機的加密資料庫備份到本機，
# 只做「下載 + checksum 驗證」，**不解密、不還原**。
#
# 備份由 scripts/backup/pg_backup.sh 產生，若設定了 BACKUP_GPG_RECIPIENT 即為 GPG 加密。
# 解密私鑰若依建議離線保存（見 .env.example 備份 GPG 章節），本腳本就無法、
# 也不應該自動完成完整還原——還原是需要人到場的動作。
#
# 用法（remote 與目錄由呼叫端提供，本檔不寫入任何環境專屬值）：
#   powershell -ExecutionPolicy Bypass -File scripts\dr\sync-backup-from-r2.ps1 `
#       -RcloneRemote "<remote>:<bucket>" -LocalDir "<本機暫存目錄>"
#
# 前提：
#   - 已安裝 rclone，且 rclone.conf 內有對應的 remote。
#   - ⚠️ 該 remote 建議使用**唯讀**憑證、Scope 限定到備份 bucket，
#     不要沿用正式機那組可寫可刪的憑證——本腳本只需要讀取。
#
# 各機器實際使用的 remote 名稱、bucket、排程與 DR 佈署屬營運情資，
# 記於本機 docs/runbooks/（不隨公開 repo 發布）。

param(
    [Parameter(Mandatory)][string]$RcloneRemote,
    [Parameter(Mandatory)][string]$LocalDir,
    [int]$RetentionDays = 14
)

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $LocalDir | Out-Null
$logFile = Join-Path $LocalDir "sync.log"

function Log($msg) {
    $line = "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')] $msg"
    Write-Host $line
    Add-Content -Path $logFile -Value $line
}

# sync-status.json 是本腳本唯一的對外契約，供呼叫端判斷「異地備份是否就緒」。
# ⚠️ **失敗時也必須寫**：否則失敗後檔案會留著上一次成功的舊內容，
# 讀的人會看到一個看似合理的舊狀態而誤判 DR 就緒（假健康的另一條路徑）。
#
# 契約：
#   status = "ok"     → 有至少一份通過 checksum 驗證的備份，latest_backup 即該份。
#                       stale=true 僅表示「不夠新」（>30h），檔案本身仍可用。
#   status = "failed" → 沒有可信的可用備份。latest_backup 為 null，reasons 列出逐項原因。
#                       **看到 failed 就不要據此進行任何 prod 操作。**
#
# ⚠️ 這份契約**保證的範圍**（2026-08-22 補述，兩條都是實測發現的，不是推論）：
#
#  (1) 它保證的是「**遠端**有一份通過驗證的備份」，**不保證**「本機這份沒被動過」。
#      實測：把本機備份檔移走後重跑，腳本在驗證前就先跑 rclone copy 把它補回來了
#      （自我修復，是好事）——但這代表**本機檔案遺失不會被回報為失敗**。
#      DR 情境下這兩件事不同：若 R2 也掛了而本機檔案剛好被誤刪，本檔不會事先警告。
#
#  (2) 它**無法回報「自己沒被執行」**。機器關機、Task Scheduler 停用或排程被誤刪時，
#      本檔會留著上一次成功的內容、status 仍是 "ok"。
#      **呼叫端務必同時檢查 last_sync_run 的新鮮度**（門檻取略大於排程間隔即可），
#      否則可能在動 prod 前看到一個三天前的 "ok"。
function Write-SyncStatus {
    param(
        [Parameter(Mandatory)][ValidateSet("ok", "failed")][string]$Status,
        $Latest, [double]$AgeHours, [int]$VerifiedCount, [bool]$Stale, $Reasons
    )
    $obj = [ordered]@{
        status                  = $Status
        last_sync_run           = (Get-Date -Format "o")
        verified_count          = $VerifiedCount
        latest_backup           = if ($Latest) { $Latest.Name } else { $null }
        latest_backup_mtime     = if ($Latest) { $Latest.LastWriteTime.ToString("o") } else { $null }
        latest_backup_age_hours = if ($Latest) { $AgeHours } else { $null }
        stale                   = [bool]$Stale
        # @($null) 會產生 [null] 而非 []，故先濾掉空值
        reasons                 = @($Reasons | Where-Object { $_ })
    }
    $obj | ConvertTo-Json -Depth 4 | Set-Content -Path (Join-Path $LocalDir "sync-status.json") -Encoding UTF8
}

# ⚠️ 未預期的終止錯誤也必須留下 failed 狀態。
#
# `$ErrorActionPreference = "Stop"` 會讓 checksum 那段的 `Get-Content` / `Get-FileHash`
# 一旦遇到 I/O 錯誤（檔案被鎖、磁碟問題、權限）就直接中止腳本——**在任何
# Write-SyncStatus 之前**。於是 sync-status.json 停在上一次成功的 `status="ok"`，
# 呼叫端讀到一個看似合理的舊狀態而誤判 DR 就緒。這與上方 (2) 是同一類「假健康」，
# 差別在 (2) 是腳本沒被執行，這裡是腳本執行了但死在中途。
#
# trap 是 scope 層級的，涵蓋這行以後的所有終止錯誤，不必逐段包 try/catch。
# 內層再各自 try/catch 是因為：連寫 log 或寫狀態檔本身都可能失敗（例如磁碟滿），
# 那時仍要走到 exit 1，不能讓 trap 自己炸掉而回傳 0。
trap {
    $trapMsg = $_.Exception.Message
    try { Log "ERROR: 未預期的終止錯誤：$trapMsg" } catch { Write-Host "ERROR: $trapMsg" }
    try {
        Write-SyncStatus -Status "failed" -Reasons @("腳本因未預期錯誤中止：$trapMsg——本次同步結果不可信")
    } catch {
        Write-Host "ERROR: 連 sync-status.json 都寫不出來：$($_.Exception.Message)"
    }
    exit 1
}

Log "=== 開始同步：$RcloneRemote -> $LocalDir ==="

# 備份上傳路徑是 <remote>/YYYY/MM/（見 scripts/backup/pg_backup.sh 的 UPLOAD_DATE）。
#
# ⚠️ 必須涵蓋保留期內的**每一個**月份，不能只抓「上個月 + 當月」：
# `RetentionDays` 跨兩個月界時中間那些月份會被整個跳過。
# 例如 3/1 且 RetentionDays=31，1/30 的備份仍在保留期內卻永遠不會被複製
# ——而下方的保留期清除又只看本機檔案，所以那份備份等於從 DR 視野裡消失。
# （2026-08-23 CodeRabbit 於 PR #7 指出，實例成立。）
#
# rclone copy 對已存在的檔案是 no-op，多抓幾個月份的成本只有一次 list。
$oldest = (Get-Date).AddDays(-$RetentionDays)
$cursor = [datetime]::new($oldest.Year, $oldest.Month, 1)
$endMon = [datetime]::new((Get-Date).Year, (Get-Date).Month, 1)
$months = [System.Collections.Generic.List[string]]::new()
while ($cursor -le $endMon) {
    $months.Add($cursor.ToString("yyyy/MM"))
    $cursor = $cursor.AddMonths(1)
}
Log "保留期 $RetentionDays 天涵蓋 $($months.Count) 個月份前綴：$($months -join ', ')"

foreach ($yearMonth in $months) {
    $remotePath = "$RcloneRemote/$yearMonth"
    Log "檢查 $remotePath ..."
    # --max-age 必須跟下面的保留期一致：否則會下載超過保留期的舊檔、隨即被清除，
    # 而下一輪又因本機不存在而重新下載——每天白白重傳一次（2026-08-21 首次實跑發現，
    # 當時下載 55 檔後刪掉 38 檔）。
    & rclone copy $remotePath $LocalDir --include "ipig_*.sql.gz.gpg" --include "ipig_*.sha256" `
        --max-age "${RetentionDays}d" --no-traverse 2>&1 |
        ForEach-Object { Log "  rclone: $_" }
    if ($LASTEXITCODE -ne 0) {
        Log "ERROR: rclone copy 失敗（$remotePath），exit code $LASTEXITCODE"
        # 同樣要寫狀態檔：否則會留下上一次成功的舊內容讓人誤判 DR 就緒。
        Write-SyncStatus -Status "failed" -Reasons @("rclone copy 失敗（$remotePath），exit code $LASTEXITCODE——本次未取得遠端最新備份")
        exit 1
    }
}

# =========================================================================
# 完整性驗證——只驗證，不解密。**fail-closed**。
#
# 2026-08-22（pr-agent 於 PR #30 指出兩個缺陷後改寫）。設計判準只有一條：
#   **任何無法通過 checksum 驗證的檔案，都不得被回報為「可用的最新快照」。**
# 理由：這支腳本的輸出（sync-status.json）會被當成「異地有可用備份、
# 可以安全動 prod」的依據（例如做 migration baseline 重設前的確認）。
# 假的「DR 健康」訊號比沒有訊號更危險——要回退時才發現最新那份從沒驗過。
# 所以配對不完整一律 exit 1，不是警告。
#
# 原本的兩個缺陷：
#  (1) 迴圈只走訪 .sha256 檔 → 沒有 checksum 檔的孤兒 .gpg **從頭到尾不會被檢查**，
#      卻能在下方被選為 $latest 並寫進 sync-status.json（假健康的主因）。
#  (2) .sha256 在、.gpg 缺 → 只印 WARNING 就 continue，整支仍 exit 0。
# =========================================================================
$verifiedNames = New-Object 'System.Collections.Generic.HashSet[string]'
$failures = New-Object System.Collections.ArrayList

$gpgFiles = @(Get-ChildItem -Path $LocalDir -Filter "ipig_*.sql.gz.gpg" -File -ErrorAction SilentlyContinue)
$shaFiles = @(Get-ChildItem -Path $LocalDir -Filter "ipig_*.sql.gz.gpg.sha256" -File -ErrorAction SilentlyContinue)

# (a) 每個備份檔都必須有對應的 .sha256——缺了代表它從未被驗證過（原缺陷 1）
foreach ($g in $gpgFiles) {
    if (-not (Test-Path (Join-Path $LocalDir ($g.Name + ".sha256")))) {
        [void]$failures.Add("$($g.Name)：缺少對應的 .sha256，無法驗證完整性")
    }
}

# (b) 每個 .sha256 都必須有對應的備份檔，且 checksum 必須吻合（原缺陷 2）
foreach ($s in $shaFiles) {
    $gpgName = $s.Name -replace '\.sha256$', ''
    $gpgPath = Join-Path $LocalDir $gpgName
    if (-not (Test-Path $gpgPath)) {
        [void]$failures.Add("$gpgName：.sha256 存在但備份檔缺漏，備份組不完整")
        continue
    }
    $expected = ((Get-Content $s.FullName -Raw).Trim() -split '\s+')[0].ToLower()
    $actual = (Get-FileHash -Path $gpgPath -Algorithm SHA256).Hash.ToLower()
    if ($actual -ne $expected) {
        [void]$failures.Add("$gpgName：checksum 不符（expected=$expected actual=$actual），下載不完整或檔案損毀")
    } else {
        [void]$verifiedNames.Add($gpgName)
    }
}

Log "checksum 驗證：$($verifiedNames.Count) 個通過，$($failures.Count) 個失敗"
foreach ($f in $failures) { Log "ERROR: $f" }

if ($failures.Count -gt 0) {
    Write-SyncStatus -Status "failed" -Reasons $failures
    Log "ERROR: 備份組驗證未通過（見上方逐項原因）。sync-status.json 已標記 failed——"
    Log "ERROR: 在人工釐清之前，不得把異地備份視為可用，也不要據此進行任何 prod 操作。"
    exit 1
}

# 本機只是暫存最近幾份，正本一直保留在 R2；清掉超過 RetentionDays 的本機副本
Get-ChildItem -Path $LocalDir -Filter "ipig_*" | Where-Object {
    $_.LastWriteTime -lt (Get-Date).AddDays(-$RetentionDays)
} | ForEach-Object {
    Log "清除過期本機副本：$($_.Name)"
    Remove-Item $_.FullName -Force
}

# ⚠️ $latest **只能**從「已通過 checksum 驗證」的集合裡挑（原缺陷 1 的修正點）。
# 保留期清除可能已刪掉部分檔案，故此處重新列檔並與 $verifiedNames 取交集。
$latest = Get-ChildItem -Path $LocalDir -Filter "ipig_*.sql.gz.gpg" -File -ErrorAction SilentlyContinue |
    Where-Object { $verifiedNames.Contains($_.Name) } |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1

if (-not $latest) {
    # 沒有任何「已驗證」的備份 = 沒有可用的 DR 快照。這同樣是失敗，不是警告：
    # 回報 exit 0 會讓呼叫端以為異地備份就緒。
    Write-SyncStatus -Status "failed" -Reasons @("本機沒有任何通過 checksum 驗證的備份檔（remote 上可能沒有檔案，或全數超過保留期被清除）")
    Log "ERROR: 沒有任何已驗證的備份可用——不得視為 DR 就緒。"
    exit 1
}

$ageHours = [math]::Round(((Get-Date) - $latest.LastWriteTime).TotalHours, 1)
$staleWarn = $ageHours -gt 30
Write-SyncStatus -Status "ok" -Latest $latest -AgeHours $ageHours -VerifiedCount $verifiedNames.Count -Stale $staleWarn
Log "=== 同步完成。已驗證 $($verifiedNames.Count) 份，最新可用備份：$($latest.Name)（$ageHours 小時前）==="
if ($staleWarn) {
    Log "WARNING: 最新備份已超過 30 小時，來源端的每日備份可能中斷，建議去查 db-backup log"
    Log "WARNING: （本項為警告不是失敗：檔案本身通過驗證、可用，只是不夠新。）"
}
