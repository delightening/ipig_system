# iPig 系統維運手冊 (Operations)

> **版本**：1.0  
> **建立日期**：2026-03-01  
> **適用範圍**：正式環境維運、on-call、故障排除

本文件定義 iPig 系統的服務擁有者、on-call 輪值、升級流程與故障排除指引。  
**相關文件**：[DEPLOYMENT.md](../DEPLOYMENT.md)、[CREDENTIAL_ROTATION.md](../security-compliance/CREDENTIAL_ROTATION.md)、[ARCHITECTURE.md](../ARCHITECTURE.md)

---

## 1. 服務擁有者與聯絡方式

| 角色 | 職責 | 聯絡方式 |
|------|------|----------|
| **服務擁有者** | 系統整體責任、功能決策、資源配置 | *請填入：email / 分機* |
| **技術負責人** | 架構、部署、效能、安全 | *請填入：email / 分機* |
| **資料庫管理員** | DB 備份、還原、遷移、效能調校 | *請填入：email / 分機* |

**部署環境資訊：**

| 環境 | 用途 | 主機 / URL |
|------|------|------------|
| 正式環境 | 生產服務 | *請填入* |
| 測試環境 | UAT / 整合測試 | *請填入* |
| 開發環境 | 開發與驗證 | 本地 Docker |

---

## 2. On-call 輪值與排程

### 2.1 輪值原則

- 正式環境需有 on-call 人員，於非上班時間處理緊急故障
- 建議採週輪值或雙人輪值（主/副）
- 輪值表應提前一週公告

### 2.2 輪值表範本

| 週次 | 主值班 | 副值班 | 備註 |
|------|--------|--------|------|
| *YYYY-Www* | *姓名* | *姓名* | |
| *YYYY-Www+1* | *姓名* | *姓名* | |

### 2.3 緊急聯絡流程

1. 監控告警觸發（Prometheus + Alertmanager）
2. Alertmanager 依規則通知 on-call 人員（Email / PagerDuty / Slack）
3. On-call 人員依 [§4 故障排除](#4-故障排除流程) 進行初步診斷
4. 若無法自行排除，依嚴重度升級至技術負責人或服務擁有者

---

## 3. 升級聯絡人與升級流程

### 3.1 升級路徑

```
L1: On-call 人員（第一線）
    ↓ 無法排除 / 影響範圍大
L2: 技術負責人
    ↓ 架構 / 資安 / 資料問題
L3: 服務擁有者
```

### 3.2 升級條件

| 條件 | 建議升級至 |
|------|------------|
| 服務中斷 > 15 分鐘 | L2 |
| 資料遺失或損壞風險 | L2 |
| 安全事件（入侵、外洩） | L2 → L3 |
| 需變更架構或採購 | L3 |

### 3.3 升級紀錄

建議於每次升級後紀錄：

- 時間與觸發原因
- 升級對象
- 處理結果與後續行動

---

## 4. 故障排除流程

### 4.1 健康檢查端點

| 端點 | 用途 |
|------|------|
| `GET /api/health` | 整體健康狀態（DB、GeoIP 等） |
| `GET /api/metrics` | Prometheus 指標 |

**健康檢查範例：**

```bash
curl -s http://localhost:8080/api/health | jq .
# 預期：{"status":"healthy","checks":{"database":{"status":"up"},...}}
```

### 4.2 常見問題與處理

#### 4.2.1 API 無法連線（502 / 503）

| 可能原因 | 檢查方式 | 處理 |
|----------|----------|------|
| API 容器未啟動 | `docker compose ps` | `docker compose up -d api` |
| 資料庫連線失敗 | 查看 API 日誌 | 檢查 `DATABASE_URL`、DB 容器狀態 |
| 記憶體不足 | `docker stats` | 增加主機資源或調整 DB pool |

#### 4.2.2 資料庫連線錯誤

| 可能原因 | 檢查方式 | 處理 |
|----------|----------|------|
| DB 容器未啟動 | `docker compose ps db` | `docker compose up -d db` |
| 密碼錯誤 | 檢查 `.env` 與 DB 設定 | 對齊 `POSTGRES_PASSWORD` |
| 連線池耗盡 | API 日誌 `pool` 相關錯誤 | 重啟 API 或調整 `DATABASE_MAX_CONNECTIONS` |

#### 4.2.3 登入失敗 / JWT 錯誤

| 可能原因 | 檢查方式 | 處理 |
|----------|----------|------|
| JWT_SECRET 變更 | 比對環境變數 | 若輪換後，使用者需重新登入 |
| Cookie 設定 | 檢查 `COOKIE_SECURE`、HTTPS | 正式環境需 `COOKIE_SECURE=true` |
| CSRF Token | 檢查 `csrf_token` Cookie | 清除瀏覽器 Cookie 重試 |

#### 4.2.4 Migration 失敗

| 可能原因 | 檢查方式 | 處理 |
|----------|----------|------|
| 遷移腳本錯誤 | 查看 API 啟動日誌 | 參考 [DB_ROLLBACK.md](../database/DB_ROLLBACK.md) 回滾 |
| 手動修改 schema | 比對 migrations 與 DB | 依既有 migration 補齊或建立修正 migration |

#### 4.2.5 磁碟空間不足

| 可能原因 | 檢查方式 | 處理 |
|----------|----------|------|
| 日誌過多 | `du -sh /var/log` | 輪替或刪除舊日誌 |
| 備份檔案 | 檢查備份目錄 | 依保留政策刪除過期備份 |
| Docker 映像/卷 | `docker system df` | `docker system prune -a`（謹慎） |

### 4.3 日誌位置與查詢

| 元件 | 日誌來源 |
|------|----------|
| API | `docker compose logs -f api`，或 JSON log 至 stdout |
| Nginx | 容器內 `/var/log/nginx/` |
| PostgreSQL | 容器內 `pg_log`（若啟用） |

### 4.4 重啟服務順序

建議順序（避免連線中斷）：

1. `docker compose restart db`（必要時）
2. 等待 DB 就緒（約 10–30 秒）
3. `docker compose restart api`
4. `docker compose restart web`（若需）

---

## 5. 維護窗口建議

- **例行維護**：建議於低流量時段（如週末凌晨）進行
- **憑證輪換**：依 [CREDENTIAL_ROTATION.md](../security-compliance/CREDENTIAL_ROTATION.md)，JWT 輪換會使全站 session 失效，需事先公告
- **版本升級**：依 [DEPLOYMENT.md](../DEPLOYMENT.md) 執行，並於升級前備份

---

*文件產出於 2026-03-01，請依實際組織架構填入聯絡方式與輪值表。*
