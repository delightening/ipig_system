# 本機 CI 全域測試

模擬 GitHub Actions CI 環境，於本機執行完整測試套件。

## 涵蓋項目

### 核心測試
| 項目 | 說明 |
|------|------|
| **Backend: cargo test** | Rust 後端單元測試（需 Postgres） |
| **Frontend: tsc check** | TypeScript 型別檢查 |
| **Frontend: Vitest** | 前端單元測試 |
| **E2E: Playwright** | 端對端整合測試 |

### 安全與守衛
| 項目 | 說明 |
|------|------|
| **Security: cargo audit** | Rust 依賴漏洞掃描 (RustSec) |
| **Security: cargo deny** | 授權／漏洞／供應鏈檢查 |
| **Guard: SQL injection** | 檢查 format! 拼接 SQL 字串 |
| **Guard: unsafe code** | 檢查 unsafe 程式碼區塊 |
| **Backend: clippy** | Rust 靜態分析與風格檢查 |
| **Security: npm audit** | npm 依賴漏洞掃描 |
| **Security: Trivy** | 容器映像漏洞掃描 |

## Port 對照

為避免與正式／開發環境衝突，使用 `docker-compose.test.ci-local.yml` overlay：

| 服務 | 正式/開發 | 本機 CI |
|------|-----------|---------|
| PostgreSQL | 5432/5433 | **15432** |
| API | 8000 | **18000** |
| Web | 8080 | **18080** |

## 執行方式

### 完整執行
```powershell
.\scripts\run-ci-local.ps1
```

### 跳過部分項目
```powershell
# 跳過 Security 相關（cargo audit、cargo deny、npm audit）
.\scripts\run-ci-local.ps1 -SkipSecurity

# 跳過 Trivy 容器掃描
.\scripts\run-ci-local.ps1 -SkipTrivy

# 跳過 E2E
.\scripts\run-ci-local.ps1 -SkipE2E

# 僅跑 Frontend + E2E
.\scripts\run-ci-local.ps1 -SkipBackend -SkipSecurity -SkipTrivy
```

### Bash (Linux/macOS/WSL)
```bash
./scripts/run-ci-local.sh
./scripts/run-ci-local.sh --skip-security --skip-e2e
```

## 前置需求

- **Rust**（cargo、rustup、clippy）
- **Node.js 22**
- **Docker**
- 腳本會嘗試安裝：cargo-audit、cargo-deny、sqlx-cli、Playwright 瀏覽器

## 相關檔案

- `scripts/run-ci-local.ps1`：PowerShell 腳本
- `scripts/run-ci-local.sh`：Bash 腳本
- `docker-compose.test.yml`：測試環境基礎設定
- `docker-compose.test.ci-local.yml`：本機 overlay（port 15432/18000/18080）
- `.github/workflows/ci.yml`：GitHub Actions 對應定義
