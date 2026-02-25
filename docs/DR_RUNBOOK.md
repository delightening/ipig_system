# iPig 災難復原手冊 (DR Runbook)

> **RPO 目標**：< 1 小時（最多損失 1 小時資料）
> **RTO 目標**：< 4 小時（4 小時內恢復服務）

---

## 1. 緊急聯絡

| 角色 | 負責人 | 聯絡方式 |
|------|--------|---------|
| 系統管理員 | _填入_ | _填入_ |
| 資料庫管理員 | _填入_ | _填入_ |
| 主管 | _填入_ | _填入_ |

---

## 2. 故障分級

| 等級 | 定義 | 回應時間 | 範例 |
|------|------|---------|------|
| P0 | 服務完全中斷 | 立即 | DB 崩潰、主機當機 |
| P1 | 核心功能異常 | 30 分鐘內 | 登入失敗、打卡無法使用 |
| P2 | 非核心功能異常 | 2 小時內 | 報表產出錯誤、通知未送達 |
| P3 | 輕微問題 | 下個工作日 | UI 顯示異常、翻譯錯誤 |

---

## 3. 復原程序

### 3.1 情境一：資料庫毀損

**症狀**：`/api/health` 回傳 503、API 日誌大量 DB 連線錯誤

```bash
# Step 1: 確認 DB 容器狀態
docker compose ps db
docker compose logs db --tail 50

# Step 2: 嘗試重啟
docker compose restart db
sleep 10
curl http://localhost:8080/api/health

# Step 3: 如重啟無效，從備份還原
docker compose down

# 找到最新備份檔
docker compose run --rm db-backup ls -lt /backups/ | head -5

# 還原（以實際檔名替換）
docker compose up -d db
sleep 10
gunzip -c /path/to/ipig_db_YYYYMMDD_HHMMSS.sql.gz | \
  docker compose exec -T db psql -U postgres ipig_db

# 如為 GPG 加密備份
gpg --decrypt /path/to/backup.sql.gz.gpg | \
  gunzip | docker compose exec -T db psql -U postgres ipig_db

# Step 4: 重啟所有服務
docker compose up -d
```

**驗證**：
- [ ] `curl /api/health` 回傳 200
- [ ] 登入功能正常
- [ ] 最近的資料存在

---

### 3.2 情境二：API 服務崩潰

**症狀**：前端顯示 502 Bad Gateway、Nginx 正常但 API 無回應

```bash
# Step 1: 查看 API 日誌
docker compose logs api --tail 100

# Step 2: 重啟 API
docker compose restart api

# Step 3: 如持續崩潰，回滾至上個版本
git log --oneline -5
git checkout <previous-commit>
docker compose build api
docker compose up -d api
```

---

### 3.3 情境三：主機完全毀損（全新部署）

```bash
# Step 1: 在新主機安裝 Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Step 2: 取得程式碼
git clone <repository-url> ipig_system
cd ipig_system

# Step 3: 還原 .env 設定
cp .env.example .env
# 填入正式環境設定（JWT_SECRET 必須與舊環境一致！）

# Step 4: 啟動基礎服務
docker compose up -d db
sleep 15

# Step 5: 從異地備份還原
rsync -az user@nas:/backups/ipig/ ./backups/
gunzip -c ./backups/latest.sql.gz | \
  docker compose exec -T db psql -U postgres ipig_db

# Step 6: 還原上傳檔案
rsync -az user@nas:/backups/ipig/uploads/ ./uploads/

# Step 7: 啟動所有服務
docker compose up -d

# Step 8: 更新 DNS / Cloudflare Tunnel
```

**驗證清單**：
- [ ] `/api/health` 回傳 200 + healthy
- [ ] 管理員帳號可登入
- [ ] 稽核紀錄完整
- [ ] 上傳檔案可存取
- [ ] 打卡功能正常（IP + GPS）

---

## 4. 備份驗證程序

建議每季度執行一次完整的備份還原演練。

### 演練步驟

```bash
# 1. 在測試環境建立獨立 DB
docker run -d --name dr-test-db \
  -e POSTGRES_PASSWORD=test123 \
  -p 5433:5432 postgres:16-alpine

# 2. 從備份還原
gunzip -c /path/to/latest_backup.sql.gz | \
  docker exec -i dr-test-db psql -U postgres -d postgres

# 3. 驗證資料完整性
docker exec dr-test-db psql -U postgres -d ipig_db \
  -c "SELECT count(*) FROM users;"
docker exec dr-test-db psql -U postgres -d ipig_db \
  -c "SELECT count(*) FROM activity_logs;"

# 4. 清理
docker rm -f dr-test-db
```

### 演練記錄表

| 日期 | 執行人 | 備份日期 | 還原耗時 | 資料完整 | 備註 |
|------|--------|---------|---------|---------|------|
| _YYYY-MM-DD_ | _姓名_ | _YYYY-MM-DD_ | _X 分鐘_ | ✅/❌ | |

---

## 5. 事後檢討範本

每次 P0/P1 事件後，填寫以下範本：

```
事件標題：
發生時間：
發現時間：
解決時間：
影響範圍：
根本原因：
時間線：
  - HH:MM 發現問題
  - HH:MM 開始處理
  - HH:MM 服務恢復
改善措施：
  1.
  2.
```
