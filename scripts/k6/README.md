# k6 壓力測試指南

## 安裝 k6

### Windows
```powershell
# 方法 1：MSI 安裝（推薦）
Invoke-WebRequest -Uri "https://dl.k6.io/msi/k6-latest-amd64.msi" -OutFile "k6.msi"
Start-Process msiexec -ArgumentList "/i k6.msi /quiet" -Wait

# 方法 2：Chocolatey（需管理員權限）
choco install k6 -y
```

### macOS / Linux
```bash
brew install k6       # macOS
snap install k6       # Linux
```

## 執行壓力測試

```powershell
# 基本執行
k6 run scripts/k6/load-test.js

# 自訂參數
k6 run --vus 50 --duration 60s scripts/k6/load-test.js

# 指定環境
k6 run -e BASE_URL=https://your-domain.com -e TEST_USER=admin -e TEST_PASS=yourpass scripts/k6/load-test.js
```

## 效能基準

| 指標 | 目標 | 說明 |
|------|------|------|
| 一般 API P95 | < 500ms | `/api/animals`, `/api/protocols` 等 |
| 報表 API P95 | < 2000ms | `/api/reports/*` |
| 登入 P95 | < 1000ms | `/api/auth/login` |
| 錯誤率 | < 5% | 全部請求 |

## 測試結果

測試結果會自動儲存至 `tests/results/k6_*.json`。
