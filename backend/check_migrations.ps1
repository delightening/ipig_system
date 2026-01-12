# 检查数据库迁移状态
# 用于检查迁移是否已正确运行

Write-Host "=== 数据库迁移状态检查 ===" -ForegroundColor Cyan
Write-Host ""

# 读取 .env 文件
$envFile = ".env"
if (-not (Test-Path $envFile)) {
    Write-Host "[ERROR] .env 文件不存在" -ForegroundColor Red
    exit 1
}

$envContent = Get-Content $envFile
$databaseUrl = $envContent | Select-String -Pattern "^DATABASE_URL=(.+)$" | ForEach-Object { $_.Matches.Groups[1].Value }

if (-not $databaseUrl) {
    Write-Host "[ERROR] 无法从 .env 文件中读取 DATABASE_URL" -ForegroundColor Red
    exit 1
}

Write-Host "数据库 URL: $($databaseUrl -replace ':[^:@]+@', ':***@')" -ForegroundColor Gray
Write-Host ""

# 检查 psql 是否可用
try {
    $null = Get-Command psql -ErrorAction Stop
    $psqlAvailable = $true
} catch {
    $psqlAvailable = $false
}

if (-not $psqlAvailable) {
    Write-Host "[WARNING] psql 不可用，无法直接查询数据库" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "请安装 PostgreSQL 客户端工具，或使用以下方法之一：" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "方法 1: 使用 SQLx CLI" -ForegroundColor Yellow
    Write-Host "  cargo install sqlx-cli --features postgres" -ForegroundColor White
    Write-Host "  sqlx migrate info --database-url `"$databaseUrl`"" -ForegroundColor White
    Write-Host ""
    Write-Host "方法 2: 查看服务器启动日志" -ForegroundColor Yellow
    Write-Host "  查找 '[Database] ✓ Migrations completed successfully' 消息" -ForegroundColor White
    Write-Host ""
    Write-Host "方法 3: 手动连接到数据库" -ForegroundColor Yellow
    Write-Host "  检查 _sqlx_migrations 表中的记录" -ForegroundColor White
    exit 0
}

# 解析数据库 URL
if ($databaseUrl -match "postgres://([^:]+):([^@]+)@([^:]+):(\d+)/(.+)") {
    $user = $matches[1]
    $password = $matches[2]
    $dbHost = $matches[3]
    $port = $matches[4]
    $database = $matches[5]
    
    Write-Host "连接到数据库: $database @ $dbHost`:$port" -ForegroundColor Cyan
    Write-Host ""
    
    $env:PGPASSWORD = $password
    
    # 查询迁移状态
    Write-Host "查询迁移记录..." -ForegroundColor Yellow
    $migrations = psql -h $dbHost -p $port -U $user -d $database -t -c "SELECT version, name, applied_at FROM _sqlx_migrations ORDER BY version;" 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[SUCCESS] 迁移记录：" -ForegroundColor Green
        Write-Host ""
        $migrations | Where-Object { $_.Trim() -ne '' } | ForEach-Object {
            $parts = $_ -split '\|'
            if ($parts.Length -eq 3) {
                $version = $parts[0].Trim()
                $name = $parts[1].Trim()
                $appliedAt = $parts[2].Trim()
                Write-Host "  版本 $version : $name" -ForegroundColor White
                Write-Host "    应用时间: $appliedAt" -ForegroundColor Gray
            }
        }
        Write-Host ""
        
        # 检查迁移 004
        Write-Host "检查迁移 004 (pig_observations 表的 deleted_at 和 copied_from_id 字段)..." -ForegroundColor Yellow
        $migration004 = psql -h $dbHost -p $port -U $user -d $database -t -c "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 4;" 2>&1
        
        if ($LASTEXITCODE -eq 0 -and ($migration004.Trim() -eq "1")) {
            Write-Host "[OK] 迁移 004 已应用" -ForegroundColor Green
            Write-Host ""
            
            # 检查表结构
            Write-Host "检查 pig_observations 表结构..." -ForegroundColor Yellow
            $columns = psql -h $dbHost -p $port -U $user -d $database -t -c "SELECT column_name FROM information_schema.columns WHERE table_name = 'pig_observations' ORDER BY ordinal_position;" 2>&1
            
            $hasDeletedAt = $columns | Select-String -Pattern "deleted_at"
            $hasCopiedFromId = $columns | Select-String -Pattern "copied_from_id"
            
            if ($hasDeletedAt) {
                Write-Host "[OK] deleted_at 字段存在" -ForegroundColor Green
            } else {
                Write-Host "[ERROR] deleted_at 字段不存在！" -ForegroundColor Red
            }
            
            if ($hasCopiedFromId) {
                Write-Host "[OK] copied_from_id 字段存在" -ForegroundColor Green
            } else {
                Write-Host "[ERROR] copied_from_id 字段不存在！" -ForegroundColor Red
            }
            
            if (-not $hasDeletedAt -or -not $hasCopiedFromId) {
                Write-Host ""
                Write-Host "[WARNING] 表结构不完整。建议重新运行迁移：" -ForegroundColor Yellow
                Write-Host "  1. 停止后端服务" -ForegroundColor White
                Write-Host "  2. 删除迁移 004 的记录：" -ForegroundColor White
                Write-Host "     DELETE FROM _sqlx_migrations WHERE version = 4;" -ForegroundColor Green
                Write-Host "  3. 重新启动后端服务" -ForegroundColor White
            }
        } else {
            Write-Host "[ERROR] 迁移 004 未应用！" -ForegroundColor Red
            Write-Host ""
            Write-Host "建议：" -ForegroundColor Yellow
            Write-Host "  重新启动后端服务以应用迁移" -ForegroundColor White
        }
    } else {
        Write-Host "[ERROR] 无法查询迁移记录" -ForegroundColor Red
        Write-Host $migrations -ForegroundColor Red
    }
    
    Remove-Item Env:\PGPASSWORD
} else {
    Write-Host "[ERROR] 无法解析 DATABASE_URL 格式" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== 检查完成 ===" -ForegroundColor Cyan
