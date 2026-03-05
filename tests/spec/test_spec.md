# iPig System 整合測試規格書

> **最後更新：** 2026-02-16
> **測試框架：** 自訂 `BaseApiTester`（Python + requests，HTTP 整合測試）
> **執行環境：** `.venv` 內的 Python 3.12+
> **後端：** Rust / Axum，所有 API 位於 `/api/` 前綴下

---

## 設計原則

| 原則 | 說明 |
|------|------|
| **帳密不硬編碼** | 管理員帳密從 `.env` 讀取 `TEST_ADMIN_EMAIL` / `TEST_ADMIN_PASSWORD` |
| **測試帳號自動建立** | 每個測試模組宣告 `TEST_USERS` dict，由 `BaseApiTester.setup_test_users()` 自動建立（已存在則跳過） |
| **UTF-8 統一編碼** | 所有 `.py` 檔案首行 `# -*- coding: utf-8 -*-`，CRLF / LF 不影響執行 |
| **SEC-02 適配** | Token 從 `Set-Cookie: access_token=...` 提取，非 JSON body |
| **冪等設計** | 重複執行不會因資料衝突失敗（使用時間戳或隨機後綴命名） |
| **結果儲存** | 測試報告存入 `tests/results/YYYY_MM_DD_HH_MM_testname.txt` |

---

## 共用基底模組：test_base.py

### 功能

| 方法 | 說明 |
|------|------|
| `admin_login()` | 以 `.env` 中的 `TEST_ADMIN_EMAIL` + `TEST_ADMIN_PASSWORD` 登入 |
| `fetch_roles()` | 取得 `GET /api/roles` 建立 `code → id` 對應表 |
| `setup_test_users(users_config)` | 遍歷帳號設定，若帳號已存在則跳過，否則以管理員 token 呼叫 `POST /api/users` 建立 |
| `login_all(users_config)` | 逐一登入所有測試帳號，提取 `access_token` cookie |
| `_req(method, url, role, **kwargs)` | 統一 HTTP 請求，自動附加 Bearer token、記錄錯誤至 `last_error.json` |
| `step()` / `sub_step()` / `record()` | 格式化輸出與測試結果記錄 |
| `print_summary()` | 輸出彙總，回傳 `bool`（全通過 = True） |
| `cleanup_test_data()` | 執行 `cleanup_test_data.sql`（Docker 或本機 psql） |

### 改進重點

- 管理員帳密改為 `os.getenv()` 讀取
- 加入 `save_results(test_name)` 方法自動寫入 `tests/results/`
- 加入 `retry_request()` 支援偶發 timeout 重試

---

## 測試模組總覽

| # | 檔案 | 模組 | 角色 | 步驟數 |
|---|------|------|------|--------|
| 1 | `test_aup_full.py` | AUP 完整審查流程 + Amendment | 8 角色 | 22 步 |
| 2 | `test_erp_full.py` | ERP 倉庫管理（CRUD + 單據 + 報表） | 2 角色 | 6 階段 |
| 3 | `test_animal_full.py` | 動物管理系統 | 3 角色 | 多階段 |
| 4 | `test_blood_panel.py` | 血液檢查系統 | 3 角色 | 13 階段 |
| 5 | `test_hr_full.py` | HR 人員管理（出勤/加班/請假） | 3 角色 | 多階段 |
| 6 | `test_amendment_full.py` | Amendment 獨立流程測試 | 7 角色 | 14 步 |
| 7 | `test_erp_permissions.py` | ERP 角色權限驗證 | 5 角色 | 多階段 |

---

## 1. test_aup_full.py — AUP 完整審查流程

### 測試目標
驗證 AUP 計畫書從建立到核准的完整 14 步審查流程，以及 Amendment（Minor / Major）從建立到核准的完整流程。

### 測試角色

| 角色 | 帳號 | 用途 |
|------|------|------|
| PI | `pi_int_test@example.com` | 計畫主持人，建立/修訂/提交計畫書 |
| VET | `vet_int_test@example.com` | 獸醫師，醫療審查 |
| IACUC_STAFF | `staff_int_test@example.com` | 行政人員，流程控制 |
| IACUC_CHAIR | `chair_int_test@example.com` | 主委，最終核定 |
| REVIEWER1~3 | `rev1~3_int_test@example.com` | 審查委員 |
| REV_OTHER | `revother_int_test@example.com` | 非指定委員（驗證自由留言） |

### 測試步驟與驗證

| 步驟 | 動作 | 驗證方式 |
|------|------|----------|
| 1 | PI 建立計畫書並提交 | `POST /protocols` → 201, `POST /protocols/:id/submit` → status = `SUBMITTED` |
| 2 | Staff 行政預審 + 留言 | `POST /protocols/:id/status` → `PRE_REVIEW`, `POST /reviews/comments` → 200 |
| 3 | PI 回覆預審意見 | `POST /reviews/comments/reply` → 200 |
| 4 | Staff 要求修訂 → PI 修改重送 | status → `PRE_REVIEW_REVISION_REQUIRED` → PI 修改 `PUT /protocols/:id` → 重送 `submit` → `RESUBMITTED` |
| 5 | 醫療審查 (VET) | status → `VET_REVIEW`, VET 留言, Staff 退回 `VET_REVISION_REQUIRED`, PI 修改重送 |
| 6 | 解決 VET 意見 | `POST /reviews/comments/:id/resolve` → 200 |
| 7-8 | 指派委員 → 倫理審查 | status → `UNDER_REVIEW`（附 reviewer_ids） |
| 9 | 3 名委員發表意見 | 各別 `POST /reviews/comments` |
| 10 | 非指定委員留言 | REV_OTHER 可留言 |
| 11 | PI 回覆所有委員 | 逐一 `POST /reviews/comments/reply` |
| 12 | Staff 要求修正 | status → `REVISION_REQUIRED` |
| 13 | 解決意見 → PI 重送 | resolve 所有 comments → PI submit |
| 14 | 主委最終核定 | IACUC_CHAIR status → `APPROVED` |
| 15-17 | Minor Amendment 流程 | PI 建立 → 提交 → IACUC_STAFF 分類 MINOR → 自動 `ADMIN_APPROVED` |
| 18-20 | Major Amendment 流程 | PI 建立 → 提交 → 分類 MAJOR → CLASSIFIED → 自動指派 → 開始審查 → 委員全部 APPROVE → 自動 `APPROVED` |
| 21 | 版本歷程 + 狀態歷程 | `GET /amendments/:id/versions` ≥ 1, `GET /amendments/:id/history` ≥ 4 |
| 22 | Protocol amendments 列表 | `GET /protocols/:id/amendments` ≥ 2, `GET /amendments/pending-count` 含 `count` |

---

## 2. test_erp_full.py — ERP 倉庫管理

### 測試目標
驗證 ERP 倉庫管理系統的完整流程：建置基礎資料、採購入庫、銷貨出庫、調撥作業、庫存驗證、報表產生。

### 測試角色

| 角色 | 帳號 | 用途 |
|------|------|------|
| WAREHOUSE_MANAGER | `wm_int_test@example.com` | 倉庫管理主要操作 |
| ADMIN_STAFF | `as_int_test@example.com` | 行政備援 |

### 測試步驟與驗證

| 階段 | 動作 | 驗證方式 |
|------|------|----------|
| Phase 1 | 建立 3 倉庫 + 每倉庫 7 貨架 + 50 產品 + 2 供應商 | 各自回傳 id，數量正確 |
| Phase 2 | 採購入庫：3 張 PO → 各建 GRN，50 產品分配到 21 貨架 | PO/GRN submit + approve 成功 |
| Phase 3 | 銷貨出庫：SO → DO，含 IACUC 篩選 | IACUC 篩選 `?iacuc_no=PIG-11401` 回傳正確 |
| Phase 4 | 調撥：倉庫 1 → 倉庫 2 (TR) | submit + approve 成功 |
| Phase 5 | 庫存驗證：on-hand + ledger + 各貨架 inventory | 有庫存記錄，品項含 `product_name` / `on_hand_qty` |
| Phase 6 | 報表驗證：5 種報表 | stock-on-hand / stock-ledger / purchase-lines / sales-lines / cost-summary 各回傳 200 |

---

## 3. test_animal_full.py — 動物管理系統

### 測試目標
驗證動物管理系統完整生命週期：建立動物來源 → 建立動物 → 觀察紀錄 → 手術紀錄 → 體重紀錄 → 疫苗紀錄 → 犧牲紀錄 → 病理報告 → 獸醫建議 → 匯出功能。

### 測試角色

| 角色 | 帳號 | 用途 |
|------|------|------|
| EXPERIMENT_STAFF | `exp_animal_test@example.com` | 日常操作 |
| VET | `vet_animal_test@example.com` | 獸醫審閱/建議 |
| ADMIN | （管理員） | 管理操作 |

### 測試步驟與驗證

| 階段 | 動作 | API 端點 | 驗證 |
|------|------|----------|------|
| Phase 1 | 建立動物來源 | `POST /animal-sources` | 回傳 id |
| Phase 2 | 建立動物 | `POST /animals` | 回傳完整 animal 物件 |
| Phase 3 | 觀察紀錄 CRUD | `POST/GET/PUT/DELETE /animals/:id/observations` | 建立 → 更新 → 版本歷程 → 列表 |
| Phase 4 | 複製觀察紀錄 | `POST /animals/:id/observations/copy` | 新紀錄內容相同 |
| Phase 5 | 手術紀錄 CRUD | `POST/GET/PUT/DELETE /animals/:id/surgeries` | 同觀察紀錄模式 |
| Phase 6 | 複製手術紀錄 | `POST /animals/:id/surgeries/copy` | 新紀錄內容相同 |
| Phase 7 | 體重紀錄 CRUD | `POST/GET/PUT/DELETE /animals/:id/weights` | 建立多筆 → 列表 → 更新 → 刪除 |
| Phase 8 | 疫苗紀錄 CRUD | `POST/GET/PUT/DELETE /animals/:id/vaccinations` | 同模式 |
| Phase 9 | 犧牲紀錄 | `POST/GET /animals/:id/sacrifice` | upsert → 查詢 |
| Phase 10 | 病理報告 | `POST/GET /animals/:id/pathology` | upsert → 查詢 |
| Phase 11 | 獸醫建議 | `POST/GET /observations/:id/recommendations` | 建立建議 → 查詢 |
| Phase 12 | 獸醫已讀標記 | `POST /animals/:id/vet-read`, `/observations/:id/vet-read` | 回傳 200 |
| Phase 13 | VET 評論列表 | `GET /animals/vet-comments` | 有結果 |
| Phase 14 | 匯出功能 | `POST /animals/:id/export` | 回傳 200 |
| Phase 15 | 跨角色存取 | 各角色查詢 | ADMIN/VET/STAFF 皆可讀取 |

---

## 4. test_blood_panel.py — 血液檢查系統

### 測試目標
驗證血液檢查系統完整功能：Seed 資料驗證、模板 CRUD、Panel CRUD、血檢建立/查詢/更新/刪除（GLP 合規軟刪除）、跨角色存取、操作審計日誌。

### 測試角色

| 角色 | 帳號 | 用途 |
|------|------|------|
| ADMIN | （管理員，從 .env 讀取） | 審計日誌查詢 |
| VET | `vet_blood_test@example.com` | 獸醫讀取驗證 |
| EXPERIMENT_STAFF | `exp_blood_test@example.com` | 主要操作者 |

### 測試步驟與驗證

| Phase | 動作 | 驗證 |
|-------|------|------|
| 1 | Seed 資料驗證 | 模板 ≥ 20 個，Panel ≥ 10 個，含必要 code/key（WBC/RBC/CBC/LIVER/KIDNEY...） |
| 2 | 模板 CRUD | 建立 → 更新 → 停用 → 啟用列表不含 → 全部列表含 → 恢復 |
| 3 | Panel CRUD + 項目管理 | 建立自訂 Panel → 更新名稱 → 替換項目 → 列表驗證 |
| 4 | 建立測試動物 | `POST /animal-sources` + `POST /animals` |
| 5 | 建立血液檢查（CBC + Liver） | `POST /animals/:id/blood-tests`，項目數 = CBC + Liver 項目數 |
| 6 | 建立第二筆（Kidney） | 同上 |
| 7 | 查詢與詳情驗證 | 列表 = 2 筆，含 `item_count` / `abnormal_count`，詳情項目完整 |
| 8 | 更新血液檢查 | 更新 lab_name / remark / 替換項目 → 還原 |
| 9 | VET 讀取驗證 | VET 可查詳情/列表/異常標記 |
| 10 | 軟刪除（GLP 合規） | DELETE → 列表中不出現 |
| 11 | 跨角色存取 | ADMIN/VET/STAFF 皆可讀 Panel/模板/血檢 |
| 12 | Panel 停用與恢復 | 停用 → 啟用列表不含 → 全部列表含 → 恢復 |
| 13 | 操作審計日誌 | 查詢 ANIMAL 類別活動、含 BLOOD_TEST_CREATE 事件、含操作者/實體資訊 |

---

## 5. test_hr_full.py — HR 人員管理

### 測試目標
驗證 HR 系統完整功能：出勤打卡、加班申請、請假申請（含審批流程）、假期餘額查詢。

### 測試角色

| 角色 | 帳號 | 用途 |
|------|------|------|
| ADMIN | （管理員） | 審批人/管理 |
| HR_MANAGER | `hr_mgr_test@example.com` | HR 管理員 |
| STAFF | `hr_staff_test@example.com` | 一般員工（申請人） |

### 測試步驟與驗證

| Phase | 動作 | API 端點 | 驗證 |
|-------|------|----------|------|
| 1 | 出勤打卡 | `POST /hr/attendance/clock-in` → `POST /hr/attendance/clock-out` | 打卡成功，列表有紀錄 |
| 2 | 出勤統計 | `GET /hr/attendance/stats` | 有統計資料 |
| 3 | 出勤修正 | `PUT /hr/attendance/:id` | 修正成功 |
| 4 | 建立加班申請 | `POST /hr/overtime` | 回傳 id，status = DRAFT |
| 5 | 提交加班申請 | `POST /hr/overtime/:id/submit` | status = SUBMITTED |
| 6 | 核准加班 | `POST /hr/overtime/:id/approve` | status = APPROVED |
| 7 | 建立請假申請 | `POST /hr/leaves` | 回傳 id |
| 8 | 提交請假 | `POST /hr/leaves/:id/submit` | status = SUBMITTED |
| 9 | 核准請假 | `POST /hr/leaves/:id/approve` | status = APPROVED |
| 10 | 駁回請假 | 另建 → 提交 → `POST /hr/leaves/:id/reject` | status = REJECTED |
| 11 | 取消請假 | 另建 → 提交 → `POST /hr/leaves/:id/cancel` | status = CANCELLED |
| 12 | 假期餘額 | `GET /hr/balances/annual` / `comp-time` / `summary` | 有資料回傳 |
| 13 | 儀表板日曆 | `GET /hr/dashboard/calendar` | 200 |
| 14 | 員工列表 | `GET /hr/staff` / `GET /hr/internal-users` | 有結果 |

---

## 6. test_amendment_full.py — Amendment 獨立流程

### 測試目標
獨立驗證 Amendment（變更申請）完整流程，與 AUP 測試中的 Amendment 重複部分提供更獨立的測試覆蓋。先以簡化流程核准一個 AUP，再測試 Minor + Major Amendment 完整路徑。

### 測試角色
（與 AUP 測試相同 7 角色）

### 測試步驟與驗證

| 步驟 | 動作 | 驗證 |
|------|------|------|
| 1 | 建立已核准 AUP（簡化流程） | `APPROVED` status |
| 2-4 | Minor Amendment 流程 | 建立 → 提交 → 分類 MINOR → `ADMIN_APPROVED` |
| 5-9 | Major Amendment 流程 | 建立 → 提交 → 分類 MAJOR → 自動指派 → 開始審查 → 全部核准 → `APPROVED` |
| 10 | 版本歷程 | `GET /amendments/:id/versions` ≥ 1 |
| 11 | 狀態歷程 | `GET /amendments/:id/history` ≥ 4，含 DRAFT → SUBMITTED → CLASSIFIED → UNDER_REVIEW → APPROVED |
| 12 | Protocol amendments 列表 | `GET /protocols/:id/amendments` ≥ 2 |
| 13 | Amendment 列表查詢 + 狀態篩選 | `GET /amendments?status=APPROVED` ≥ 1 |
| 14 | 待處理數量 | `GET /amendments/pending-count` 含 `count` |

---

## 7. test_erp_permissions.py — ERP 角色權限驗證

### 測試目標
驗證 ERP 系統中不同角色的權限控制：哪些角色可以建立/管理倉庫/產品/供應商/貨架，哪些角色只能查看/領用。

### 測試角色

| 角色 | 帳號 | 預期權限 |
|------|------|----------|
| WAREHOUSE_MANAGER | `wm_perm_test@example.com` | 完整 CRUD |
| ADMIN_STAFF | `as_perm_test@example.com` | 完整 CRUD |
| EXPERIMENT_STAFF | `exp_perm_test@example.com` | 只讀 + 領用（SO/DO） |
| PI | `pi_perm_test@example.com` | 只讀 |
| VET | `vet_perm_test@example.com` | 只讀 |

### 測試步驟與驗證

| 階段 | 動作 | 驗證 |
|------|------|------|
| Phase 1 | WAREHOUSE_MANAGER 建立倉庫/產品/供應商/貨架 | 全部成功 (200/201) |
| Phase 2 | ADMIN_STAFF 建立倉庫/產品/供應商/貨架 | 全部成功 |
| Phase 3 | EXPERIMENT_STAFF 查看庫存 | `GET /inventory/on-hand` → 200 |
| Phase 4 | EXPERIMENT_STAFF 建立 SO（領用） | `POST /documents` (SO) → 成功 |
| Phase 5 | EXPERIMENT_STAFF 按 IACUC 查詢 | `GET /documents?iacuc_no=...` → 200 |
| Phase 6 | PI/VET 不可建立倉庫/產品 | `POST /warehouses` → 403, `POST /products` → 403 |
| Phase 7 | PI/VET 可查看庫存 | `GET /inventory/on-hand` → 200 |

---

## 執行方式

### 單一模組執行
```powershell
cd d:\Coding\ipig_system
.venv\Scripts\python.exe tests/test_aup_full.py
.venv\Scripts\python.exe tests/test_erp_full.py
.venv\Scripts\python.exe tests/test_animal_full.py
.venv\Scripts\python.exe tests/test_blood_panel.py
.venv\Scripts\python.exe tests/test_hr_full.py
.venv\Scripts\python.exe tests/test_amendment_full.py
.venv\Scripts\python.exe tests/test_erp_permissions.py
```

### 全部執行
```powershell
.venv\Scripts\python.exe tests/run_all_tests.py
```

### 指定模組
```powershell
.venv\Scripts\python.exe tests/run_all_tests.py --aup
.venv\Scripts\python.exe tests/run_all_tests.py --erp
.venv\Scripts\python.exe tests/run_all_tests.py --animal
.venv\Scripts\python.exe tests/run_all_tests.py --blood
.venv\Scripts\python.exe tests/run_all_tests.py --hr
```

### 測試後清理
```powershell
.venv\Scripts\python.exe tests/run_all_tests.py --cleanup
```

---

## 驗證計畫

1. **逐一執行**：7 個模組各自獨立執行，確認通過
2. **全部執行**：`run_all_tests.py` 整合執行，確認 5/5 通過
3. **結果儲存**：自動寫入 `tests/results/YYYY_MM_DD_HH_MM_testname.txt`
4. **冪等驗證**：連續執行兩次，確認第二次也通過（不因資料已存在而失敗）
