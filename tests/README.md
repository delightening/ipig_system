# iPig System 測試套件說明

> **最後更新：** 2026-02-17

## 目錄結構

```
tests/
├── README.md                    # 本說明文件
├── run_all_tests.py             # 統一測試入口（8 模組，共享 Context）
│
├── test_base.py                 # 共用基底模組（BaseApiTester）
├── test_context.py              # 共享測試 Context（SharedTestContext）
├── test_fixtures.py             # 統一帳號註冊表 + 角色子集定義
│
├── test_aup_full.py             # 1. AUP 審查流程測試
├── test_erp_full.py             # 2. ERP 倉庫管理測試
├── test_animal_full.py          # 3. 動物管理系統測試
├── test_blood_panel.py          # 4. 血液檢驗面板測試
├── test_hr_full.py              # 5. HR 人員管理測試
├── test_amendment_full.py       # 6. Amendment 變更申請測試
├── test_erp_permissions.py      # 7. ERP 權限測試
├── test_aup_integration.py      # 8. AUP 全功能整合測試
│
├── audit_verify.py              # 安全審計獨立驗證工具
├── cleanup_test_data.ps1        # 測試資料清理腳本（PowerShell）
│
├── spec/                        # 測試規格文件
│   └── test_spec.md
├── results/                     # 測試結果輸出
│   └── YYYY_MM_DD_HH_MM_*.txt
└── _archive/                    # 過時/偵錯用腳本（歸檔保留）
    ├── aup_test_standalone.py
    ├── audit_check_deep.py
    ├── audit_check_quick.py
    └── debug_csrf.py
```

---

## 快速開始

### 前置條件

1. Docker 容器 `ipig-db` 和 `ipig-backend` 正在運行
2. Python 虛擬環境已建立（`tests/.venv`）

### 執行全部測試

```powershell
cd d:\Coding\ipig_system
.venv\Scripts\python.exe tests/run_all_tests.py
```

### 指定模組

```powershell
# 單一模組
.venv\Scripts\python.exe tests/run_all_tests.py --aup          # AUP 審查
.venv\Scripts\python.exe tests/run_all_tests.py --erp          # ERP 倉庫
.venv\Scripts\python.exe tests/run_all_tests.py --animal       # 動物管理
.venv\Scripts\python.exe tests/run_all_tests.py --blood        # 血液檢驗
.venv\Scripts\python.exe tests/run_all_tests.py --hr           # HR 人員管理
.venv\Scripts\python.exe tests/run_all_tests.py --amendment    # Amendment 變更
.venv\Scripts\python.exe tests/run_all_tests.py --erp-perm     # ERP 權限
.venv\Scripts\python.exe tests/run_all_tests.py --aup-integ    # AUP 整合

# 組合使用
.venv\Scripts\python.exe tests/run_all_tests.py --aup --erp --animal
```

### 執行模式

```powershell
# 預設：共享 Context（各測試延遲登入，避免 JWT 過期）
.venv\Scripts\python.exe tests/run_all_tests.py

# 各自登入模式
.venv\Scripts\python.exe tests/run_all_tests.py --no-shared

# 測試後自動清理
.venv\Scripts\python.exe tests/run_all_tests.py --cleanup
```

---

## 八大測試模組

### 1. AUP 完整審查流程 (`test_aup_full.py`)

**涵蓋：** 14 步完整 AUP 生命週期

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

**測試帳號：** 8 個 — PI、VET、IACUC_STAFF、IACUC_CHAIR、REVIEWER×3、REV_OTHER

---

### 2. ERP 完整倉庫管理 (`test_erp_full.py`)

**涵蓋：** 5 個階段

| 階段 | 操作 | 數量 |
|------|------|------|
| Phase 1 | 建立倉庫、貨架、產品、供應商 | 3 倉庫 × 7 貨架、50 產品、2 供應商 |
| Phase 2 | 採購入庫 (PO → GRN) | 3 張 PO + 3 張 GRN |
| Phase 3 | 銷貨出庫 (SO → DO) | 5 個品項出庫 |
| Phase 4 | 倉庫間調撥 (TR) | 倉庫1 → 倉庫2 |
| Phase 5 | 庫存驗證 | 在手庫存 + 帳簿查詢 |

**測試帳號：** 2 個 — WAREHOUSE_MANAGER、ADMIN_STAFF_ERP

---

### 3. 動物管理系統 (`test_animal_full.py`)

**涵蓋：** 9 個階段

| 階段 | 操作 | 數量 |
|------|------|------|
| Phase 1 | 建立豬源 + 動物 + 驗證 | 5 隻（4 品種、2 性別） |
| Phase 2 | 體重紀錄 | 各 2~3 筆 |
| Phase 3 | 觀察試驗紀錄 | 含用藥 |
| Phase 4 | 手術紀錄 + 術後觀察 | 手術 + 術後觀察 |
| Phase 5 | 疫苗/驅蟲紀錄 | 批次建立 |
| Phase 6 | 犧牲/採樣紀錄 | 指定動物 |
| Phase 7 | 動物資料更新 | 狀態變更、IACUC 編號 |
| Phase 8 | 病理組織報告 | 指定動物 |
| Phase 9 | 紀錄時間軸完整性驗證 | 全部動物 |

**測試帳號：** 2 個 — VET_ANIMAL、EXPERIMENT_STAFF_ANIMAL

---

### 4. 血液檢驗面板 (`test_blood_panel.py`)

**涵蓋：** 13 個階段

| Phase | 操作 |
|-------|------|
| 1 | Seed 資料驗證 — 模板 & Panel 預設資料 |
| 2 | 模板 CRUD — 建立、更新、停用/恢復 |
| 3 | Panel CRUD — 建立、更新、停用/恢復 |
| 4 | 建立血液檢查紀錄 |
| 5 | 更新檢查結果 |
| 6 | 批次建立檢查紀錄 |
| 7 | 查詢/篩選/分頁 |
| 8 | 異常標記驗證 |
| 9 | 刪除紀錄 |
| 10 | 趨勢圖表資料 |
| 11 | 匯出 CSV |
| 12 | 權限驗證 |
| 13 | 操作審計日誌 |

**測試帳號：** 3 個 — ADMIN_BLOOD、VET_BLOOD、EXPERIMENT_STAFF_BLOOD

---

### 5. HR 人員管理 (`test_hr_full.py`)

**涵蓋：** 14 個階段

| Phase | 操作 |
|-------|------|
| 1 | 打卡上班/下班 |
| 2 | 出勤查詢 + 統計 |
| 3 | 加班申請建立 |
| 4 | 加班審批流程 |
| 5 | 請假申請建立 |
| 6 | 請假審批流程 |
| 7 | 請假駁回流程 |
| 8 | 補打卡申請 |
| 9 | 月報查詢 |
| 10 | 出勤報表 |
| 11 | 取消請假 |
| 12 | 假期餘額查詢 |
| 13 | 儀表板日曆 |
| 14 | 員工列表 |

**測試帳號：** 3 個 — ADMIN_HR、ADMIN_STAFF_HR、EXPERIMENT_STAFF_HR

---

### 6. Amendment 變更申請 (`test_amendment_full.py`)

**涵蓋：** 14 個步驟

| 步驟 | 操作 |
|------|------|
| 1 | 建立已核准 AUP（簡化流程） |
| 2 | PI 建立 Minor Amendment |
| 3 | PI 提交 Amendment |
| 4 | IACUC_STAFF 分類 Minor → 自動 ADMIN_APPROVED |
| 5 | PI 建立 Major Amendment |
| 6 | PI 提交 Major Amendment |
| 7 | 指派委員審查 |
| 8 | 委員核准 |
| 9 | 主委最終核定 |
| 10 | Amendment 版本更新驗證 |
| 11-14 | 列表查詢、狀態篩選、待處理數量 |

**測試帳號：** 7 個 — 與 AUP 共用（PI、VET、IACUC_STAFF、IACUC_CHAIR、REVIEWER×3）

---

### 7. ERP 權限測試 (`test_erp_permissions.py`)

驗證三角色存取權限（10 項測試案例）：

| 角色 | 預期權限 |
|------|----------|
| WAREHOUSE_MANAGER | 完整 CRUD |
| ADMIN_STAFF | 查看 + 部分操作 |
| EXPERIMENT_STAFF | 僅查看 + 領用 |

**測試帳號：** 3 個 — WAREHOUSE_MANAGER_PERM、ADMIN_STAFF_PERM、EXPERIMENT_STAFF_PERM

---

### 8. AUP 全功能整合 (`test_aup_integration.py`)

**涵蓋：** 6 個 Phase，跨案件流程驗證

| Phase | 操作 |
|-------|------|
| 1 | 同時建立 2 個 AUP 案件（A / B 草稿） |
| 2 | 案件 A 快速走完審查 → APPROVED |
| 3 | 案件 B 退件修訂流程 → APPROVED |
| 4 | 案件 A 的 Minor + Major Amendment |
| 5 | 動物轉讓流程驗證 |
| 6 | 資料隔離性驗證 |

**測試帳號：** 10 個 — PI_A、PI_B、IACUC_STAFF、IACUC_CHAIR、REVIEWER×3、EXP_STAFF、REV_OTHER、VET

---

## 獨立工具

### 安全審計驗證 (`audit_verify.py`)

驗證安全審計 6 大端點：Dashboard、Activities、Logins、Sessions、Alerts、Protocol Activities。

```powershell
.venv\Scripts\python.exe tests/audit_verify.py
```

---

## 共用基底架構

### `test_base.py` — BaseApiTester

所有測試模組繼承的基底類別：

| 方法 | 功能 |
|------|------|
| `admin_login()` | 管理員登入（含 429 自動重試 + CSRF） |
| `setup_test_users(users)` | 透過 Admin API 建立測試帳號（已存在則跳過） |
| `login_all(users)` | 登入所有測試帳號，存儲 token |
| `_req(method, url, role, **kwargs)` | HTTP 請求包裝（含 CSRF、錯誤記錄、429 重試） |
| `step()` / `sub_step()` | 測試步驟標記 |
| `record(name, success, detail)` | 記錄測試結果 |
| `print_summary()` | 輸出測試彙總 |
| `save_results()` | 儲存至 `tests/results/` |
| `cleanup_test_data()` | 清理測試資料（保留審計記錄） |

### `test_context.py` — SharedTestContext

跨測試模組的共享 context，統一管理 token/帳號/共用資料。
當透過 `run_all_tests.py` 批次執行時，延遲初始化各角色帳號（避免 JWT 15 分鐘過期）。

| 方法 | 功能 |
|------|------|
| `initialize(required_roles)` | 一次性初始化指定角色（已初始化則跳過） |
| `inject_into(tester, roles)` | 注入 token/user_id 到子 tester |
| `get_shared() / set_shared()` | 跨測試共用資料存取 |

### `test_fixtures.py` — 帳號註冊表

集中定義所有測試帳號（`ALL_TEST_USERS`），避免各測試重複定義。
每個測試模組透過角色子集常數宣告所需帳號：

```python
AUP_ROLES       = ["IACUC_STAFF", "REVIEWER1", ...]
ANIMAL_ROLES    = ["VET_ANIMAL", "EXPERIMENT_STAFF_ANIMAL"]
BLOOD_PANEL_ROLES = ["ADMIN_BLOOD", "VET_BLOOD", "EXPERIMENT_STAFF_BLOOD"]
ERP_ROLES       = ["WAREHOUSE_MANAGER", "ADMIN_STAFF_ERP"]
HR_ROLES        = ["ADMIN_HR", "ADMIN_STAFF_HR", "EXPERIMENT_STAFF_HR"]
# ... 等等
```

---

## 測試資料清理

測試會建立大量業務資料。清理方式有三種：

### 方式 1：PowerShell 腳本（推薦）

```powershell
cd d:\Coding\ipig_system
.\tests\cleanup_test_data.ps1
```
- 互動式雙重確認（Y + CLEANUP）
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
| | 計畫活動歷程 (protocol_activities) |
| | 角色與權限定義 |

---

## 測試結果存放規範

所有輸出檔統一放入 `results/`，命名格式：

```
YYYY_MM_DD_HH_MM_<測試模組名>.txt
```

範例：`2026_02_17_01_55_AUP_全功能整合測試.txt`

---

## 環境變數

| 變數 | 預設值 | 說明 |
|------|--------|------|
| `API_BASE_URL` | `http://localhost:8000/api` | 後端 API 位址 |
| `TEST_ADMIN_EMAIL` | `admin@example.com` | 管理員帳號 |
| `TEST_ADMIN_PASSWORD` | `changeme` | 管理員密碼 |
| `TEST_USER_PASSWORD` | `password123` | 測試帳號通用密碼 |

可在 `.env` 中覆寫。

---

## 最新測試結果 (2026-02-17)

```
✅  AUP              — 14/14 通過
✅  ERP              — 9/9 通過
✅  Animal           — 通過
✅  Blood Panel      — 13 Phase 通過
✅  HR               — 14 Phase 通過
✅  Amendment        — 14/14 通過
✅  ERP Permissions  — 10/10 通過
✅  AUP Integration  — 6 Phase 通過
```
