# 豬博士 iPig 系統

豬博士 iPig 系統為一套整合型實驗動物管理平台，採用**統一入口門戶**架構，使用者透過單一登入點存取所有子系統功能。

## 子系統組成

| 子系統 | 主要功能 |
|-------|---------|
| **AUP 提交與審查系統** | IACUC 動物試驗計畫書的撰寫、提交、審查、核准流程 |
| **iPig ERP (進銷存管理系統)** | 物資採購、庫存、成本管理 |
| **實驗動物管理系統** | 豬隻分配、實驗紀錄、健康監控、病歷管理 |
| **人員管理系統** | 請假流程、補休累計、特休假管理 |

## 快速開始

詳細設定步驟請參考 **[QUICK_START.md](QUICK_START.md)**。

### 最簡啟動（Docker）

```powershell
# 複製環境設定
cp .env.demo .env
mkdir secrets
cp secrets.example/google-service-account.json.example secrets/google-service-account.json

# 啟動服務
docker compose up -d
```

### 服務入口

| 服務 | 網址 |
|-----|------|
| 前端 | http://localhost:8080 |
| API | http://localhost:3000 |

### 預設帳號

| 帳號 | 密碼 |
|-----|------|
| admin@ipig.local | admin123 |

## 技術架構

| 層級 | 技術 |
|-----|------|
| 後端 | Rust + Axum + PostgreSQL + SQLx |
| 前端 | React 18 + TypeScript + Vite + Tailwind CSS |
| 認證 | JWT + Argon2 |

## 相關文件

| 文件 | 說明 |
|-----|------|
| [_Spec.md](_Spec.md) | 系統完整規格書 |
| [QUICK_START.md](QUICK_START.md) | 快速啟動指南 |
| [ERPSpec.md](ERPSpec.md) | 進銷存系統詳細規格 |
| [AUP.md](AUP.md) | AUP 表單資料結構規格 |
| [STORAGE_SETUP.md](STORAGE_SETUP.md) | 檔案儲存設定說明 |

## 授權

MIT License
