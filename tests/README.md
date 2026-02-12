# iPig System 測試套件說明

> **最後更新：** 2026-02-12

## 目錄結構

```
tests/
├── README.md                    # 本說明文件
├── run_all_tests.py             # 統一測試入口（主要使用）
├── test_base.py                 # 共用基底模組
├── test_aup_full.py             # AUP 審查流程測試
├── test_erp_full.py             # ERP 倉庫管理測試
├── test_animal_full.py          # 動物管理系統測試
├── test_erp_permissions.py      # ERP 權限測試（獨立）
├── audit_verify.py              # 安全審計驗證（獨立）
├── aup_test_standalone.py       # AUP 獨立腳本（早期版本，保留參考）
├── results/                     # 測試結果輸出
│   └── YYYY-MM-DD_*.txt|.log
└── 測試報告Feb 12th 2026.md     # Bug 修復報告
```

---

## 快速開始

### 前置條件

1. Docker 容器 `ipig-db` 和 `ipig-backend` 正在運行
2. Python 虛擬環境已建立

### 執行全部測試

```powershell
cd d:\Coding\ipig_system
.venv\Scripts\python.exe tests/run_all_tests.py
```

### 指定模組

```powershell
# 只跑 AUP
.venv\Scripts\python.exe tests/run_all_tests.py --aup

# 只跑 ERP
.venv\Scripts\python.exe tests/run_all_tests.py --erp

# 只跑動物管理
.venv\Scripts\python.exe tests/run_all_tests.py --animal

# 組合使用
.venv\Scripts\python.exe tests/run_all_tests.py --aup --erp
```

### 測試後清理資料

```powershell
# 跑完測試後自動清理（保留審計記錄）
.venv\Scripts\python.exe tests/run_all_tests.py --cleanup

# 組合：跑完 AUP 後清理
.venv\Scripts\python.exe tests/run_all_tests.py --aup --cleanup
```

---

## 三大測試模組

### 1. AUP 完整審查流程測試 (`test_aup_full.py`)

**涵蓋範圍：** 14 步完整 AUP 生命週期

| 步驟 | 操作 | 涉及角色 |
|------|------|----------|
| 1 | PI 建立計畫書並提交 | PI |
| 2 | 行政預審 + 留言 | IACUC_STAFF |
| 3 | PI 回覆預審意見 | PI |
| 4 | 要求修訂 → PI 修改重送 | IACUC_STAFF → PI |
| 5 | 醫療審查 (VET) | IACUC_STAFF → VET → PI |
| 6 | 解決 VET 審查意見 | VET |
| 7-8 | 指派委員 → 倫理審查 | IACUC_STAFF |
| 9 | 3 名指派委員發表意見 | REVIEWER×3 |
| 10 | 非指定委員留言 | REV_OTHER |
| 11 | PI 回覆所有委員 | PI |
| 12 | Staff 要求修正 | IACUC_STAFF |
| 13 | 解決意見 → PI 重送 | REVIEWER×3 → PI |
| 14 | 主委最終核定 (APPROVED) | IACUC_CHAIR |

**建立的測試帳號：** 8 個
- PI、VET、IACUC_STAFF、IACUC_CHAIR、REVIEWER×3、REV_OTHER

```powershell
# 獨立執行
.venv\Scripts\python.exe tests/test_aup_full.py
```

---

### 2. ERP 完整倉庫管理測試 (`test_erp_full.py`)

**涵蓋範圍：** 5 個階段

| 階段 | 操作 | 數量 |
|------|------|------|
| Phase 1 | 建立倉庫、貨架、產品、供應商 | 3 倉庫 × 7 貨架、50 產品、2 供應商 |
| Phase 2 | 採購入庫 (PO → GRN) | 3 張 PO + 3 張 GRN |
| Phase 3 | 銷售出庫 (SO → DO) | 5 個品項出庫 |
| Phase 4 | 倉庫間調撥 (TR) | 倉庫1 → 倉庫2 |
| Phase 5 | 庫存驗證 | 在手庫存 + 帳簿查詢 |

**建立的測試帳號：** 2 個
- WAREHOUSE_MANAGER、ADMIN_STAFF

```powershell
# 獨立執行
.venv\Scripts\python.exe tests/test_erp_full.py
```

---

### 3. 動物管理系統完整測試 (`test_animal_full.py`)

**涵蓋範圍：** 9 個階段

| 階段 | 操作 | 數量 |
|------|------|------|
| Phase 1 | 建立豬源 + 20 隻豬 + 驗證 | 20 隻（4 品種、2 性別） |
| Phase 2 | 體重紀錄 | 50 筆（各 2~3 筆） |
| Phase 3 | 觀察試驗紀錄 | 30 筆（含用藥） |
| Phase 4 | 手術紀錄 + 術後觀察 | 5 隻豬手術、15 筆術後觀察 |
| Phase 5 | 疫苗/驅蟲紀錄 | 20 筆 |
| Phase 6 | 犧牲/採樣紀錄 | 3 隻豬 |
| Phase 7 | 動物資料更新 | 狀態變更、IACUC 編號 |
| Phase 8 | 病理組織報告 | 3 隻豬 |
| Phase 9 | 紀錄時間軸完整性驗證 | 全部 20 隻 |

**建立的測試帳號：** 2 個
- VET、EXPERIMENT_STAFF

```powershell
# 獨立執行
.venv\Scripts\python.exe tests/test_animal_full.py
```

---

## 獨立工具

### ERP 權限測試 (`test_erp_permissions.py`)

驗證 WAREHOUSE_MANAGER、ADMIN_STAFF、EXPERIMENT_STAFF 三角色對 ERP 功能的存取權限（10 項測試案例）。

```powershell
.venv\Scripts\python.exe tests/test_erp_permissions.py
```

### 安全審計驗證 (`audit_verify.py`)

驗證安全審計 6 大端點：Dashboard、Activities、Logins、Sessions、Alerts、Protocol Activities。

```powershell
.venv\Scripts\python.exe tests/audit_verify.py
```

---

## 共用基底 (`test_base.py`)

所有測試模組共用的 `BaseApiTester` 類別，提供：

| 方法 | 功能 |
|------|------|
| `setup_test_users(users)` | 透過 Admin API 建立測試帳號（已存在則跳過） |
| `login_all(users)` | 登入所有測試帳號，存儲 token |
| `_req(method, url, role, **kwargs)` | HTTP 請求包裝（自動帶入 token、錯誤記錄） |
| `step(name)` / `sub_step(name)` | 測試步驟標記 |
| `record(name, success, detail)` | 記錄測試結果 |
| `print_summary()` | 輸出測試彙總 |
| `cleanup_test_data()` | 清理測試資料（保留審計記錄） |

---

## 測試資料清理

測試會建立大量業務資料。清理方式有三種：

### 方式 1：PowerShell 腳本（推薦）

```powershell
cd d:\Coding\ipig_system
.\cleanup_test_data.ps1
```
- 互動式雙重確認
- 自動偵測 Docker / 本機環境
- 清理前後顯示統計

### 方式 2：`--cleanup` 旗標

```powershell
.venv\Scripts\python.exe tests/run_all_tests.py --cleanup
```

### 方式 3：Python 呼叫

```python
from test_base import BaseApiTester
BaseApiTester.cleanup_test_data()
```

### 清理範圍

| 清除 | 保留 |
|------|------|
| 測試使用者帳號 | admin@ipig.local |
| 動物紀錄 | 動物來源種子資料 |
| AUP 計畫、審查、修正案 | 稽核日誌 (audit_logs) |
| ERP 倉庫、產品、單據 | 活動日誌 (user_activity_logs) |
| HR 出勤、加班、請假 | 登入事件 (login_events) |
| 設施管理 | 安全警報 (security_alerts) |
| | 角色與權限定義 |

---

## 測試結果存放規範

所有輸出檔統一放入 `results/`，命名格式：

```
YYYY-MM-DD_<描述>.txt
YYYY-MM-DD_<描述>.log
```

範例：`2026-02-12_test_results.txt`

---

## 環境變數

| 變數 | 預設值 | 說明 |
|------|--------|------|
| `API_BASE_URL` | `http://localhost:8000/api` | 後端 API 位址 |

可在 `.env` 中覆寫。

---

## 最新測試結果 (2026-02-12)

```
✅  AUP     — 14/14 通過
✅  ERP     — 9/9 通過
✅  Animal  — 21/21 通過
⏱ 總耗時: 5.3 秒
```
