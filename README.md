# 進銷存管理系統 (ERP)

一套支援「採購、入庫、銷售、出庫、盤點、調撥、報表、權限」的進銷存系統。

## 技術架構

### 後端
- **語言**: Rust
- **框架**: Axum
- **資料庫**: PostgreSQL
- **ORM**: SQLx
- **認證**: JWT + Argon2

### 前端
- **框架**: React 18 + TypeScript
- **建構工具**: Vite
- **樣式**: Tailwind CSS
- **元件庫**: shadcn/ui (Radix UI)
- **狀態管理**: TanStack Query + Zustand
- **表格**: TanStack Table
- **表單**: React Hook Form + Zod

## 功能特色

### 基礎資料
- 產品管理（SKU、名稱、單位、批號/效期追蹤）
- 倉庫管理
- 供應商/客戶管理

### 採購流程
- 採購單 (PO)
- 採購入庫 (GRN)
- 採購退貨 (PR)

### 銷售流程
- 銷售單 (SO)
- 銷售出庫 (DO)
- 銷售退貨 (SR)

### 倉儲作業
- 庫存查詢
- 庫存流水
- 調撥單 (TR)
- 盤點單 (STK)
- 調整單 (ADJ)

### 系統功能
- RBAC 權限控制
- 單據狀態機（草稿→送審→核准）
- 稽核軌跡
- 低庫存警示

## 快速開始

### 使用 Docker Compose（推薦）

```bash
# 啟動所有服務
docker-compose up -d

# 查看日誌
docker-compose logs -f

# 停止服務
docker-compose down
```

服務啟動後：
- 前端: http://localhost
- API: http://localhost:3000
- 資料庫: localhost:5432

### 本地開發

#### 後端

```bash
cd backend

# 建立 .env 檔案
cp env.sample .env
# 編輯 .env 設定資料庫連線等

# 安裝 SQLx CLI
cargo install sqlx-cli

# 建立資料庫
sqlx database create

# 執行 migrations
sqlx migrate run

# 啟動開發伺服器
cargo run
```

#### 前端

```bash
cd frontend

# 安裝相依套件
npm install

# 啟動開發伺服器
npm run dev
```

## 預設帳號

- **Email**: admin@erp.local
- **密碼**: admin123

## API 文件

### 認證
- `POST /api/auth/login` - 登入
- `POST /api/auth/refresh` - 刷新 Token
- `POST /api/auth/logout` - 登出
- `GET /api/me` - 取得當前用戶

### 基礎資料
- `GET/POST /api/warehouses` - 倉庫列表/建立
- `GET/PUT/DELETE /api/warehouses/:id` - 倉庫操作
- `GET/POST /api/products` - 產品列表/建立
- `GET/PUT/DELETE /api/products/:id` - 產品操作
- `GET/POST /api/partners` - 夥伴列表/建立
- `GET/PUT/DELETE /api/partners/:id` - 夥伴操作

### 單據
- `GET/POST /api/documents` - 單據列表/建立
- `GET/PUT /api/documents/:id` - 單據操作
- `POST /api/documents/:id/submit` - 送審
- `POST /api/documents/:id/approve` - 核准
- `POST /api/documents/:id/cancel` - 作廢

### 庫存
- `GET /api/inventory/on-hand` - 庫存現況
- `GET /api/inventory/ledger` - 庫存流水
- `GET /api/inventory/low-stock` - 低庫存警示

## 專案結構

```
ERP/
├── backend/                  # Rust 後端
│   ├── src/
│   │   ├── main.rs          # 程式入口
│   │   ├── config.rs        # 設定
│   │   ├── error.rs         # 錯誤處理
│   │   ├── routes.rs        # 路由
│   │   ├── models/          # 資料模型
│   │   ├── handlers/        # 請求處理器
│   │   ├── services/        # 業務邏輯
│   │   └── middleware/      # 中間件
│   ├── migrations/          # 資料庫遷移
│   └── Cargo.toml
├── frontend/                 # React 前端
│   ├── src/
│   │   ├── main.tsx         # 程式入口
│   │   ├── App.tsx          # 主元件
│   │   ├── components/      # UI 元件
│   │   ├── pages/           # 頁面元件
│   │   ├── layouts/         # 佈局元件
│   │   ├── stores/          # 狀態管理
│   │   └── lib/             # 工具函式
│   └── package.json
├── docker-compose.yml        # Docker 編排
└── README.md
```

## 授權

MIT License
