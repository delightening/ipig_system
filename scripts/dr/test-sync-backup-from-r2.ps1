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

Write-Host ""
if ($script:fails -eq 0) { Write-Host "全部通過" } else { Write-Host "$script:fails 個案例失敗"; exit 1 }
