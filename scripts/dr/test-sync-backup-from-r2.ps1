# sync-backup-from-r2.ps1 的回歸測試（不需要 rclone、不碰網路）。
#
# 涵蓋兩件 2026-08-23 CodeRabbit 於 PR #7 指出的缺陷：
#   (1) 月份前綴只抓「上個月 + 當月」，RetentionDays 跨兩個月界時會漏備份
#   (2) checksum I/O 錯誤時因 ErrorActionPreference=Stop 直接中止，
#       sync-status.json 停在上一次的 ok（假健康）
#
# 用法：powershell -ExecutionPolicy Bypass -File scripts\dr\test-sync-backup-from-r2.ps1
$ErrorActionPreference = "Stop"
$script:fails = 0

function Assert-Equal($name, $expected, $actual) {
    if ($expected -eq $actual) {
        Write-Host "  PASS  $name"
    } else {
        Write-Host "  FAIL  $name"
        Write-Host "        expected [$expected]"
        Write-Host "        actual   [$actual]"
        $script:fails++
    }
}

# ── 被測邏輯：與腳本內同一份月份前綴產生式 ──────────────────────
# 抽成函式才測得到；腳本本體維持 inline（它是排程直跑的腳本，不宜 dot-source 依賴）。
# ⚠️ 兩處若日後分歧，案例 1-4 會失效而不自知——所以本檔與腳本要一起改。
function Get-MonthPrefixes([datetime]$Now, [int]$RetentionDays) {
    $oldest = $Now.AddDays(-$RetentionDays)
    $cursor = [datetime]::new($oldest.Year, $oldest.Month, 1)
    $endMon = [datetime]::new($Now.Year, $Now.Month, 1)
    $months = [System.Collections.Generic.List[string]]::new()
    while ($cursor -le $endMon) {
        $months.Add($cursor.ToString("yyyy/MM"))
        $cursor = $cursor.AddMonths(1)
    }
    return $months
}

Write-Host "--- 案例 1（CodeRabbit 原例）：3/1 + RetentionDays=31 應涵蓋 1 月 ---"
$m = Get-MonthPrefixes ([datetime]::new(2026, 3, 1)) 31
Assert-Equal "月份數" 3 $m.Count
Assert-Equal "內容" "2026/01, 2026/02, 2026/03" ($m -join ', ')

Write-Host "--- 案例 2：舊寫法在同條件下只會有 2 個月（證明缺陷為真）---"
$old = @((Get-Date -Year 2026 -Month 3 -Day 1).AddMonths(-1).ToString("yyyy/MM"),
         [datetime]::new(2026, 3, 1).ToString("yyyy/MM")) | Select-Object -Unique
Assert-Equal "舊寫法月份數" 2 $old.Count
if ($old -contains "2026/01") {
    Write-Host "  FAIL  舊寫法竟然涵蓋 2026/01，缺陷前提不成立"
    $script:fails++
} else {
    Write-Host "  PASS  已證明舊寫法漏掉 2026/01"
}

Write-Host "--- 案例 3：vet 的實際設定（保留 30 天）在月初 ---"
$m = Get-MonthPrefixes ([datetime]::new(2026, 3, 2)) 30
Assert-Equal "3/2 + 30 天 → 涵蓋 1 月" $true ($m -contains "2026/01")

Write-Host "--- 案例 4：月中不需要多抓 ---"
$m = Get-MonthPrefixes ([datetime]::new(2026, 3, 20)) 14
Assert-Equal "3/20 + 14 天 → 只需 3 月" "2026/03" ($m -join ', ')

Write-Host "--- 案例 5：跨年 ---"
$m = Get-MonthPrefixes ([datetime]::new(2026, 1, 3)) 30
Assert-Equal "1/3 + 30 天 → 涵蓋 2025/12" $true ($m -contains "2025/12")
Assert-Equal "跨年內容" "2025/12, 2026/01" ($m -join ', ')

Write-Host "--- 案例 6：預設保留期 14 天，最多 2 個月 ---"
$maxMonths = 0
for ($d = 1; $d -le 28; $d++) {
    $n = (Get-MonthPrefixes ([datetime]::new(2026, 2, $d)) 14).Count
    if ($n -gt $maxMonths) { $maxMonths = $n }
}
Assert-Equal "14 天最多跨 2 個月" 2 $maxMonths

Write-Host "--- 案例 7：trap 存在且會寫 failed（靜態檢查）---"
$src = Get-Content (Join-Path $PSScriptRoot "sync-backup-from-r2.ps1") -Raw
$hasTrap = $src -match '(?s)trap\s*\{.*?Write-SyncStatus\s+-Status\s+"failed".*?exit 1'
Assert-Equal "trap 內有 Write-SyncStatus failed + exit 1" $true $hasTrap
# ⚠️ 不可以用 `Get-FileHash` 當定位點——它也出現在 trap 自己的註解裡（在 trap 之前），
#    斷言會抓到註解而永遠為 False（2026-08-23 實際踩到）。用只會出現在程式碼的形式。
$posTrap = $src.IndexOf('trap {', [System.StringComparison]::Ordinal)
$posHash = $src.IndexOf('(Get-FileHash -Path', [System.StringComparison]::Ordinal)
Assert-Equal "找得到 trap" $true ($posTrap -ge 0)
Assert-Equal "找得到 checksum 計算" $true ($posHash -ge 0)
Assert-Equal "trap 定義早於 checksum 計算" $true ($posTrap -lt $posHash)

Write-Host "--- 案例 8：RetentionDays 下限（0 或負數會刪光本機備份）---"
# 為什麼是靜態檢查而不是真跑：真跑要有 rclone 與遠端。這裡驗的是「param 上有沒有
# 那道護欄」——沒有護欄時 PowerShell 會照收 0/-1，然後清除門檻變成現在或未來。
$hasRange = $src -match '\[ValidateRange\(1,\s*\[int\]::MaxValue\)\]\s*\[int\]\$RetentionDays'
Assert-Equal "RetentionDays 有 ValidateRange(1, MaxValue)" $true $hasRange

# 同時證明「沒有護欄會出事」：直接算門檻，不依賴腳本
foreach ($bad in @(0, -1)) {
    $cutoff = (Get-Date).AddDays(-$bad)
    $wouldDeleteEverything = $cutoff -ge (Get-Date).AddSeconds(-1)
    Assert-Equal "RetentionDays=$bad 的清除門檻會刪光（故必須擋）" $true $wouldDeleteEverything
}

Write-Host "--- 案例 9：status=ok 必須同時要求「已驗證」與「本次於遠端可見」---"
# CodeRabbit 指出的核心：rclone copy 不刪目的端多餘檔案，遠端清空時 copy 仍 exit 0，
# 本機舊檔留著、checksum 照過，於是寫出帶新鮮 last_sync_run 的 "ok"。
$hasRemoteSeen = $src -match '\$remoteSeen\s*=\s*\[System\.Collections\.Generic\.HashSet\[string\]\]'
Assert-Equal "有建立 remoteSeen 集合" $true $hasRemoteSeen

$hasLsf = $src -match 'rclone\s+lsf'
Assert-Equal "有用 rclone lsf 列遠端" $true $hasLsf

# 挑 latest 的那一行必須同時檢查 verifiedNames 與 remoteSeen 兩個集合
# ⚠️ 2026-08-24 改成多行 Where-Object 後（見案例 11），單行 regex 抓不到了，
#    改成抓整個 Where-Object 區塊（從 Where-Object 開始到對應的 } 為止）再判斷。
$srcLinesForLatest = $src -split "`n"
$whereIdx = 0..($srcLinesForLatest.Count - 1) | Where-Object { $srcLinesForLatest[$_] -match '\$latest\s*=' } | Select-Object -First 1
$latestBlock = ($srcLinesForLatest[$whereIdx..([Math]::Min($whereIdx + 8, $srcLinesForLatest.Count - 1))]) -join " "
Assert-Equal "latest 篩選同時檢查 verifiedNames 與 remoteSeen" $true `
    (($latestBlock -match 'verifiedNames\.Contains') -and ($latestBlock -match 'remoteSeen\.Contains'))

# 遠端空了但本機有驗證過的檔 → 必須有專屬的失敗理由，不能沿用「本機沒有備份」那句
Assert-Equal "有『本次在遠端一份都沒看到』的專屬失敗理由" $true `
    ($src -match '本次在遠端一份都沒看到')

Write-Host "--- 案例 10：lsf 與 copy 的 --max-age 必須一致（否則 remoteSeen 被過期物件汙染）---"
# CodeRabbit 第三輪指出：lsf 若不帶 --max-age，會收錄 copy 因過期而跳過的物件。
# 後果是「遠端只剩過期備份」（＝備份管線已死超過保留期）時，只要本機還留著同名舊檔
# 且 checksum 過得了，就會寫出帶新鮮 last_sync_run 的 "ok"。
# 本腳本保證的是「遠端有一份**在保留期內**且通過驗證的備份」，不是「曾經有過」。
# ⚠️ 必須排除註解行：本腳本的說明文字裡就出現過 `rclone copy` / `rclone lsf`，
#    不排除的話 -First 1 會抓到註解，測試就變成在檢查註解而不是指令（2026-08-24 實際踩到）。
$srcLines = $src -split "`n"
$isCode = { param($l) $l.TrimStart() -notmatch '^#' }
$lsfCmd = $srcLines | Where-Object { $_ -match 'rclone\s+lsf' -and (& $isCode $_) } | Select-Object -First 1
Assert-Equal "找得到非註解的 rclone lsf 指令行" $true ($null -ne $lsfCmd)
$lsfIdx = $srcLines.IndexOf($lsfCmd)
# lsf 的參數可能換行續接，取該行與其後 2 行一起判斷
$lsfBlock = ($srcLines[$lsfIdx..([Math]::Min($lsfIdx + 2, $srcLines.Count - 1))]) -join " "
Assert-Equal "rclone lsf 帶 --max-age" $true ($lsfBlock -match '--max-age')
Assert-Equal "lsf 的 --max-age 用 RetentionDays（與 copy 同一個變數）" $true `
    ($lsfBlock -match '--max-age\s+"\$\{RetentionDays\}d"')

# 兩邊都必須有，且用同一個運算式——只改一邊等於沒改
$copyCmd = $srcLines | Where-Object { $_ -match 'rclone\s+copy' -and (& $isCode $_) } | Select-Object -First 1
Assert-Equal "找得到非註解的 rclone copy 指令行" $true ($null -ne $copyCmd)
$copyIdx = $srcLines.IndexOf($copyCmd)
$copyBlock = ($srcLines[$copyIdx..([Math]::Min($copyIdx + 2, $srcLines.Count - 1))]) -join " "
Assert-Equal "rclone copy 也帶同樣的 --max-age" $true `
    ($copyBlock -match '--max-age\s+"\$\{RetentionDays\}d"')

# 反向驗證：確認這個檢測在「有事」時真的會叫——把 --max-age 從 lsf 拿掉後應該判為不通過
$lsfWithout = $lsfBlock -replace '--max-age\s+"\$\{RetentionDays\}d"', ''
Assert-Equal "檢測有效性：lsf 少了 --max-age 時本測試會失敗" $false ($lsfWithout -match '--max-age')

Write-Host "--- 案例 11：latest 篩選必須同時要求 .gpg 與 .sha256 都在 remoteSeen 裡 ---"
# CodeRabbit 第四輪指出：只查 remoteSeen.Contains($_.Name)（即 .gpg 本身），沒查
# 對應的 .sha256。後果：遠端的 .sha256 被刪、本機還留著舊的一份，checksum 照樣過，
# latest 照樣選中這對不完整的遠端配對。
Assert-Equal "latest 篩選同時要求 .gpg 與 .sha256 都在 remoteSeen 裡" $true `
    ($latestBlock -match 'remoteSeen\.Contains\("\$\(\$_\.Name\)\.sha256"\)')

# 反向驗證：拿掉 .sha256 檢查後，這個斷言必須判為不通過
$latestBlockWithoutSha = $latestBlock -replace 'remoteSeen\.Contains\("\$\(\$_\.Name\)\.sha256"\)\s*-?a?n?d?', ''
Assert-Equal "檢測有效性：latest 少了 .sha256 檢查時本測試會失敗" $false `
    ($latestBlockWithoutSha -match 'remoteSeen\.Contains\("\$\(\$_\.Name\)\.sha256"\)')

Write-Host ""
if ($script:fails -eq 0) { Write-Host "全部通過" } else { Write-Host "$script:fails 個案例失敗"; exit 1 }
