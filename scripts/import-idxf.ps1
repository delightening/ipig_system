# 全庫 IDXF 匯入腳本
# 用法: .\scripts\import-idxf.ps1 -FilePath "C:\Users\admin\Downloads\ipig_export_20260301_110804.json"
# 需先登入取得 session，或提供 -Email -Password 參數

param(
    [Parameter(Mandatory=$true)]
    [string]$FilePath,
    [string]$ApiBase = "http://localhost:8080/api/v1",
    [string]$Email,
    [string]$Password
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $FilePath)) {
    Write-Error "檔案不存在: $FilePath"
    exit 1
}

$session = New-Object Microsoft.PowerShell.Commands.WebRequestSession

# 若提供帳密則先登入
if ($Email -and $Password) {
    Write-Host "登入中..."
    $loginBody = @{ email = $Email; password = $Password } | ConvertTo-Json
    try {
        $loginRes = Invoke-WebRequest -Uri "$ApiBase/auth/login" -Method POST `
            -ContentType "application/json" -Body $loginBody `
            -WebSession $session -UseBasicParsing
    } catch {
        Write-Error "登入失敗: $_"
        exit 1
    }
    Write-Host "登入成功"
}

# 上傳檔案
Write-Host "上傳匯入: $FilePath"
$form = @{ file = Get-Item -Path $FilePath }

try {
    $res = Invoke-RestMethod -Uri "$ApiBase/admin/data-import" -Method POST `
        -Form $form -WebSession $session
    Write-Host "匯入完成: $($res.tables_processed) 表, $($res.rows_inserted) 筆新增, $($res.rows_skipped) 筆略過"
    if ($res.errors.Count -gt 0) {
        Write-Host "錯誤: $($res.errors -join '; ')"
    }
    if ($res.skipped_details -and $res.skipped_details.Count -gt 0) {
        Write-Host "略過項目:"
        foreach ($s in $res.skipped_details) {
            $cnt = if ($s.count) { " ($($s.count) 筆)" } else { "" }
            Write-Host "  - $($s.table): $($s.reason)$cnt"
        }
    }
} catch {
    $err = $_.ErrorDetails.Message
    if ($err) {
        $obj = $err | ConvertFrom-Json -ErrorAction SilentlyContinue
        if ($obj.error.message) { Write-Error $obj.error.message; exit 1 }
    }
    throw
}
