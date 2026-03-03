# ============================================
# 本機 CI 全域測試腳本
# ============================================
# 模擬 GitHub Actions CI，涵蓋以下項目：
#
# 【核心測試】
#   - Backend: cargo test
#   - Frontend: tsc check (+ Vitest)
#   - E2E: Playwright
#
# 【安全與守衛】
#   - Security: cargo audit
#   - Security: cargo deny
#   - Guard: SQL injection
#   - Guard: unsafe code
#   - Backend: clippy
#   - Security: npm audit
#   - Security: Trivy container scan
#
# Port 對照（docker-compose.test.ci-local.yml）：
#   PostgreSQL: 15432 | API: 18000 | Web: 18080
#
# 前置需求：Rust、Node.js 22、Docker
# ============================================

param(
    [switch]$SkipBackend,
    [switch]$SkipFrontend,
    [switch]$SkipE2E,
    [switch]$SkipSecurity,
    [switch]$SkipTrivy
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not (Test-Path "$ProjectRoot\backend\Cargo.toml")) {
    $ProjectRoot = Resolve-Path "."
}
Set-Location $ProjectRoot

$COMPOSE_TEST = "docker-compose.test.yml"
$COMPOSE_CI_LOCAL = "docker-compose.test.ci-local.yml"
$FAILED = $false

function Write-Step { param($Msg) Write-Host "`n==> $Msg" -ForegroundColor Cyan }
function Write-Ok   { param($Msg) Write-Host "  OK: $Msg" -ForegroundColor Green }
function Write-Err  { param($Msg) Write-Host "  FAIL: $Msg" -ForegroundColor Red; $script:FAILED = $true }

function Invoke-Step {
    param([string]$Name, [scriptblock]$Block)
    Write-Step $Name
    try {
        & $Block
        Write-Ok $Name
        return $true
    } catch {
        Write-Err "$Name - $_"
        return $false
    }
}

# ----- 1. Security: cargo audit -----
if (-not $SkipSecurity) {
    $null = Invoke-Step "Security: cargo audit" {
        Set-Location "$ProjectRoot\backend"
        cargo install cargo-audit --locked 2>$null
        cargo update 2>$null
        cargo audit --ignore RUSTSEC-2023-0071 --ignore RUSTSEC-2024-0370 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "cargo audit exited with $LASTEXITCODE" }
        Set-Location $ProjectRoot
    }
}

# ----- 2. Security: cargo deny -----
if (-not $SkipSecurity) {
    $null = Invoke-Step "Security: cargo deny" {
        Set-Location $ProjectRoot
        cargo install cargo-deny --locked 2>$null
        cargo deny --manifest-path backend/Cargo.toml check
        if ($LASTEXITCODE -ne 0) { throw "cargo deny exited with $LASTEXITCODE" }
    }
}

# ----- 3. Guard: SQL injection -----
$null = Invoke-Step "Guard: SQL injection" {
    Set-Location "$ProjectRoot\backend"
    $files = Get-ChildItem -Path src -Filter "*.rs" -Recurse
    foreach ($f in $files) {
        # -CaseSensitive: avoid false positives like "Failed to delete file"
        $m = Select-String -Path $f.FullName -Pattern 'format!\s*\(.*\b(SELECT|INSERT|UPDATE|DELETE|DROP|ALTER)\b' -AllMatches -CaseSensitive
        if ($m) { throw "Potential SQL injection: $($f.FullName)" }
    }
    Set-Location $ProjectRoot
}

# ----- 4. Guard: unsafe code -----
$null = Invoke-Step "Guard: unsafe code" {
    Set-Location "$ProjectRoot\backend"
    $files = Get-ChildItem -Path src -Filter "*.rs" -Recurse
    foreach ($f in $files) {
        $m = Select-String -Path $f.FullName -Pattern '^\s*unsafe\s' -AllMatches
        if ($m) { Write-Warning "Found unsafe code in $($f.FullName) - please verify" }
    }
    Set-Location $ProjectRoot
}

# ----- 5. Backend: cargo check -----
if (-not $SkipBackend) {
    $null = Invoke-Step "Backend: cargo check" {
        Set-Location "$ProjectRoot\backend"
        $env:SQLX_OFFLINE = "true"
        cargo check --release
        if ($LASTEXITCODE -ne 0) { throw "cargo check exited with $LASTEXITCODE" }
        Set-Location $ProjectRoot
    }
}

# ----- 6. Backend: cargo test (需 Postgres) -----
if (-not $SkipBackend) {
    Write-Step "Backend: cargo test"
    try {
        docker compose -f $COMPOSE_TEST -f $COMPOSE_CI_LOCAL up -d db-test
        Start-Sleep -Seconds 5
        for ($i = 1; $i -le 30; $i++) {
            docker exec ipig-db-test pg_isready -U postgres -d ipig_db_test 2>&1 | Out-Null
            if ($LASTEXITCODE -eq 0) { break }
            Start-Sleep -Seconds 1
        }
        $env:DATABASE_URL = "postgres://postgres:password@localhost:15432/ipig_db_test"
        $env:SQLX_OFFLINE = "false"
        $env:ADMIN_EMAIL = "admin@ipig.local"
        $env:ADMIN_INITIAL_PASSWORD = "ci_test_admin_password_2024"
        $env:DISABLE_ACCOUNT_LOCKOUT = "true"

        Set-Location "$ProjectRoot\backend"
        cargo install sqlx-cli --no-default-features --features postgres --force 2>$null
        sqlx migrate run
        if ($LASTEXITCODE -ne 0) { throw "sqlx migrate failed" }
        cargo test --verbose -- --test-threads=1
        if ($LASTEXITCODE -ne 0) { throw "cargo test exited with $LASTEXITCODE" }
        Set-Location $ProjectRoot
        Write-Ok "Backend: cargo test"
    } catch {
        Write-Err "Backend: cargo test - $_"
    }
}

# ----- 7. Backend: clippy -----
if (-not $SkipBackend) {
    $null = Invoke-Step "Backend: clippy" {
        Set-Location "$ProjectRoot\backend"
        $env:SQLX_OFFLINE = "true"
        rustup component add clippy 2>$null
        cargo clippy --all-targets -- -D warnings -W clippy::unwrap_used
        if ($LASTEXITCODE -ne 0) { throw "clippy exited with $LASTEXITCODE" }
        Set-Location $ProjectRoot
    }
}

# ----- 8. Frontend: tsc + vitest -----
if (-not $SkipFrontend) {
    $null = Invoke-Step "Frontend: tsc check" {
        Set-Location "$ProjectRoot\frontend"
        npm ci
        npx playwright install chromium --with-deps 2>$null
        npx tsc --noEmit
        if ($LASTEXITCODE -ne 0) { throw "tsc exited with $LASTEXITCODE" }
        Set-Location $ProjectRoot
    }
    $null = Invoke-Step "Frontend: Vitest" {
        Set-Location "$ProjectRoot\frontend"
        npx vitest run
        if ($LASTEXITCODE -ne 0) { throw "vitest exited with $LASTEXITCODE" }
        Set-Location $ProjectRoot
    }
}

# ----- 9. Security: npm audit -----
if (-not $SkipSecurity) {
    $null = Invoke-Step "Security: npm audit" {
        Set-Location "$ProjectRoot\frontend"
        npm audit --audit-level=high
        if ($LASTEXITCODE -ne 0) { throw "npm audit exited with $LASTEXITCODE" }
        Set-Location $ProjectRoot
    }
}

# ----- 10. Security: Trivy scan -----
if (-not $SkipTrivy) {
    $trivyBackend = {
        docker build --no-cache --progress=plain -t ipig-api:ci "$ProjectRoot\backend"
        docker run --rm -v //var/run/docker.sock:/var/run/docker.sock -v "${ProjectRoot}/.trivyignore:/.trivyignore:ro" aquasec/trivy:latest image --exit-code 1 --ignore-unfixed --ignorefile /.trivyignore --severity CRITICAL,HIGH ipig-api:ci
    }
    $null = Invoke-Step "Trivy: backend image" $trivyBackend

    $trivyFrontend = {
        Set-Location "$ProjectRoot\frontend"
        npm ci
        npm run build
        Set-Location $ProjectRoot
        docker build --no-cache --progress=plain -t ipig-web:ci "$ProjectRoot\frontend"
        docker run --rm -v //var/run/docker.sock:/var/run/docker.sock -v "${ProjectRoot}/.trivyignore:/.trivyignore:ro" aquasec/trivy:latest image --exit-code 1 --ignore-unfixed --ignorefile /.trivyignore --severity CRITICAL,HIGH ipig-web:ci
    }
    $null = Invoke-Step "Trivy: frontend image" $trivyFrontend
}

# ----- 11. E2E: Playwright -----
if (-not $SkipE2E) {
    Write-Step "E2E: Playwright"
    try {
        if (-not (Test-Path ".env")) { Copy-Item ".env.example" ".env" }
        docker compose -f $COMPOSE_TEST -f $COMPOSE_CI_LOCAL up -d --wait --wait-timeout 300

        # 確認服務就緒
        $ok = $false
        for ($i = 1; $i -le 60; $i++) {
            try {
                $r1 = Invoke-WebRequest -Uri "http://localhost:18080/" -UseBasicParsing -TimeoutSec 2 -ErrorAction SilentlyContinue
                $r2 = Invoke-WebRequest -Uri "http://localhost:18000/api/health" -UseBasicParsing -TimeoutSec 2 -ErrorAction SilentlyContinue
                if ($r1.StatusCode -eq 200 -and $r2.StatusCode -eq 200) { $ok = $true; break }
            } catch { $null }
            Write-Host "  Waiting for services... ($i/60)"
            Start-Sleep -Seconds 2
        }
        if (-not $ok) { throw 'Services did not become ready in time' }

        Set-Location "$ProjectRoot\frontend"
        npm ci
        npx playwright install --with-deps 2>$null
        $env:E2E_BASE_URL = "http://localhost:18080"
        $env:E2E_USER_EMAIL = "admin@ipig.local"
        $env:E2E_USER_PASSWORD = "ci_test_admin_password_2024"
        $env:E2E_ADMIN_EMAIL = "admin@ipig.local"
        $env:E2E_ADMIN_PASSWORD = "ci_test_admin_password_2024"
        npm run test:e2e
        if ($LASTEXITCODE -ne 0) { throw "Playwright E2E exited with $LASTEXITCODE" }
        Set-Location $ProjectRoot
        Write-Ok "E2E: Playwright"
    } catch {
        Write-Err "E2E: Playwright - $_"
    } finally {
        docker compose -f $COMPOSE_TEST -f $COMPOSE_CI_LOCAL down 2>$null
    }
}

# ----- 若 Backend 有啟動 db-test 且沒跑 E2E，則 down -----
if ($SkipE2E -and -not $SkipBackend) {
    docker compose -f $COMPOSE_TEST -f $COMPOSE_CI_LOCAL down 2>$null
}

Write-Host ""
if ($FAILED) {
    Write-Host "Some steps failed - check errors above" -ForegroundColor Red
    exit 1
} else {
    Write-Host "All CI simulation steps completed" -ForegroundColor Green
    exit 0
}
