# 快速啟動指南

> 本指南適用於**本地或測試環境**。正式環境、備份還原、監控與維運請見 [DEPLOYMENT.md](DEPLOYMENT.md)。專案總覽與架構見 [README.md](../README.md)。

## 方式 1: Docker Compose（推薦）

### 前置條件
- Docker 與 Docker Compose 已安裝（建議 Docker 24+、Compose 2.20+）
- 啟動 Docker Desktop（Windows/Mac）

### 啟動步驟

```powershell
# 在專案根目錄執行
cp .env.example .env
# 編輯 .env，至少設定：POSTGRES_PASSWORD、JWT_SECRET、ADMIN_INITIAL_PASSWORD

docker compose up -d

# 查看日誌
docker compose logs -f

# 停止服務
docker compose down
```

### 服務入口
| 服務 | 網址 |
|------|------|
| 前端 | http://localhost:8080 |
| API | http://localhost:8000 |
| 資料庫 | localhost:5433 |

### 驗證
```powershell
curl http://localhost:8080/api/health
# 預期：{"status":"healthy",...}
```

---

## 方式 2: 本地開發模式

### 前置條件
- PostgreSQL 已安裝並運行
- 專案根目錄有 `.env`，或 `backend/.env` 含 `DATABASE_URL`、`JWT_SECRET`

### 啟動後端（終端機 1）

```powershell
cd backend
cp env.sample .env
# 編輯 .env 設定資料庫連線
cargo install sqlx-cli
sqlx database create
sqlx migrate run
cargo run
```

後端預設：http://localhost:3000（或依 .env 的 PORT）

### 啟動前端（終端機 2）

```powershell
cd frontend
npm install
npm run dev
```

前端：http://localhost:5173

---

## 環境變數（必填）

| 變數 | 說明 |
|------|------|
| `POSTGRES_PASSWORD` | 資料庫密碼（≥16 字元建議） |
| `JWT_SECRET` | JWT 簽名密鑰（須安全隨機值） |
| `ADMIN_INITIAL_PASSWORD` | 管理員初始密碼（Docker 首次登入用） |

生成 JWT_SECRET（PowerShell）：
```powershell
$jwt = [Convert]::ToBase64String((1..64 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
Write-Output $jwt
```

---

## 預設帳號

| 帳號 | 密碼 |
|------|------|
| admin@ipig.local | admin123 |

（正式環境請於 .env 設定 `ADMIN_INITIAL_PASSWORD` 並關閉 `SEED_DEV_USERS`，見 [DEPLOYMENT.md](DEPLOYMENT.md)。）

---

## 下一步

- **使用系統**：請參考 [USER_GUIDE.md](USER_GUIDE.md)（登入、AUP、動物管理、ERP）。
- **正式部署與維運**：請參考 [DEPLOYMENT.md](DEPLOYMENT.md)（系統需求、備份、監控、故障排除、容器自動更新）。
