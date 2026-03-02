# E2E 流程參考（CI 參照用）

> 本文件為 E2E 測試流程的精簡參考，供 GitHub Actions 與自動化腳本參照。完整說明見 [README.md](README.md)。

## 執行流程

### CI（GitHub Actions）

```
1. 設定 .env（cp .env.example .env）
2. 啟動測試環境：docker compose -f docker-compose.test.yml up -d --wait --wait-timeout 300
3. 確認服務就緒：curl http://localhost:8080 與 http://localhost:8000/api/health
4. 安裝依賴：cd frontend && npm ci && npx playwright install --with-deps
5. 執行測試：npm run test:e2e
```

### 本機

```
1. 啟動服務：docker compose up -d
2. 驗證配置：cd frontend && npx tsx e2e/scripts/verify-config.ts
3. 執行測試：cd frontend && npm run test:e2e
```

## 環境變數

| 變數 | CI 範例 | 本機 |
|------|---------|------|
| E2E_BASE_URL | http://localhost:8080 | 預設同上 |
| E2E_ADMIN_EMAIL | admin@ipig.local | 預設同上 |
| E2E_ADMIN_PASSWORD | ci_test_admin_password_2024 | ADMIN_INITIAL_PASSWORD 或 E2eTest123!（force-change 後） |
| E2E_USER_EMAIL | admin@ipig.local | 同 E2E_ADMIN 或獨立 |
| E2E_USER_PASSWORD | ci_test_admin_password_2024 | 同上 |

CI 須與 `docker-compose.test.yml` 的 `ADMIN_INITIAL_PASSWORD` 一致；本機 force-change 後 admin 密碼變為 `E2eTest123!`。

## Auth Setup 順序

1. **authenticate as admin**：登入 → 若有 force-change 則完成 → 儲存 admin.json
2. **authenticate as user**：登入 → 若有 force-change 則完成 → 儲存 user.json

admin 先於 user 是因為兩者皆用 admin 時，admin 完成 force-change 後 user 須以新密碼登入。

## 相關檔案

- `frontend/e2e/auth.setup.ts`：Auth setup
- `frontend/e2e/auth-helpers.ts`：登入、force-change、帳密取得
- `frontend/playwright.config.ts`：Playwright 設定
- `docker-compose.test.yml`：CI 測試環境
