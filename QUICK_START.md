# 快速啟動指南

## 方式 1: Docker Compose（推薦）

### 前置條件
- 啟動 Docker Desktop

### 啟動步驟
```powershell
# 在專案根目錄執行
docker compose up -d

# 查看日誌
docker compose logs -f

# 停止服務
docker compose down
```

### 服務入口
- 前端: http://localhost:8080
- API: http://localhost:3000
- 資料庫: localhost:5433

---

## 方式 2: 本地開發模式

### 前置條件
- PostgreSQL 資料庫已安裝並運行
- 已設定 `backend/.env` 檔案（資料庫連線資訊）

### 啟動後端

在終端機 1：
```powershell
cd backend
cargo run
```

後端將在 http://localhost:3000 啟動

### 啟動前端

在終端機 2：
```powershell
cd frontend
npm install
npm run dev
```

前端將在 http://localhost:5173 啟動

---

## 環境變數設定

確保 `backend/.env` 檔案已正確設定：

```env
DATABASE_URL=postgres://postgres:YOUR_PASSWORD@localhost:5432/ipig_db
JWT_SECRET=YOUR_JWT_SECRET
```

生成 JWT_SECRET（PowerShell）：
```powershell
$jwt = [Convert]::ToBase64String((1..64 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
Write-Output $jwt
```

---

## 預設帳號

- 帳號: admin@ipig.local
- 密碼: admin123
