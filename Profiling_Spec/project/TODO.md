# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-02-19

此文件撰寫方式

做完的去更新 [PROGRESS.md](PROGRESS.md)
還沒完成的放這裡

---

## ⛔ 禁止事項

1. 密碼過期策略
2. 密碼歷史紀錄（SEC-38：密碼歷史紀錄）

---

## 🔴 P2 — 中優先（安全強化 / 體驗優化）

| # | 項目 | 說明 | 範圍 | 難度 |
|---|------|------|------|------|
| 1 | **治療方式藥物選單 + 後台管理** | ✅ 後端 API + DB migration + 前端管理頁面已完成；⏳ 待改造 ObservationForm/SurgeryForm/EmergencyMedication/SacrificeForm 表單 | 前後端+DB | 🟡 中 |
| 2 | **疼痛評估紀錄時間軸** | 後端 API + 前端視覺化 + 疼痛評估表數位化（DB schema 已就位） | 前後端 | 🟡 中 |
| 3 | **敏感資料二級審計** | 密碼修改、權限變更等操作的加強稽核紀錄 | 後端 | 🟡 中 |
| 4 | **安全警報即時推送** | Email / 站內通知即時推送安全警報（WebSocket/SSE） | 前後端 | 🔴 高 |
| 5 | **SEC-37：HSTS 標頭** | 正式環境 Nginx 加入 `Strict-Transport-Security`（Cloudflare Tunnel 條件式注入） | Nginx | 🟢 低 |
| 6 | **手機端 Dialog 滾動與縮放修復** | `DialogContent` 基礎元件改用 flexbox 置中 + 內建 `max-h/overflow-y-auto`；viewport 加入 `maximum-scale=1, user-scalable=no` 防 iOS 自動縮放；全域 CSS 手機端 Dialog 適配（觸控滾動、input 字體 ≥16px）；清理 20+ 處重複 scroll class （參考 [手機端架構.md](手機端架構.md)） | 前端 | 🟡 中 |


---

## 🔵 P3 — 低優先（資安 / 基礎設施）

| # | 項目 | 說明 | 範圍 |
|---|------|------|------|
| 8 | **SEC-15：Named Tunnel 遷移** | 從 Cloudflare Quick Tunnel 遷移至 Named Tunnel | DevOps |
| 9 | **SEC-32：CORS Origin 動態化** | 從環境變數讀取 `ALLOWED_ORIGINS`，取代硬編碼（推遲）| 後端 |
| 10 | **SEC-33：敏感操作二級認證** | 高危操作要求重新輸入密碼確認（推遲）| 前後端 |
| 11 | **SEC-34：稽核日誌防篡改** | HMAC 雜湊鏈保護 audit_logs 不可竄改 | 後端 |
| 12 | **SEC-35：上傳目錄隔離** | UUID 重命名 + Nginx 禁止直接存取 + API 代理下載 | 後端+Nginx |
| 13 | **SEC-36：輸入長度限制與清理** | 全域 body size limit + 文字欄位 max length | 後端 |

---

## 🟣 P4 — 中期品質提升（測試 / 文件 / CI）

| # | 項目 | 說明 | 範圍 |
|---|------|------|------|
| 14 | **Rust 測試覆蓋率擴充** | 87 個測試通過，protocol/hr/facility 已有覆蓋。需 test DB 做整合測試 | 後端 |
| 15 | **前端 E2E 測試** | Playwright 自動化測試 | 前端 |
| 16 | **OpenAPI 文件完善** | utoipa 補全 API 文件 | 後端 |
| 17 | **CI/CD PostgreSQL service** | CI 中增加 PostgreSQL service container | DevOps |
| 18 | **SEC-41：容器安全掃描** | CI 中加入 `trivy` 掃描 Docker image 漏洞 | DevOps |

---

## ⚪ P5 — 長期演進

| # | 項目 | 說明 | 範圍 |
|---|------|------|------|
| 19 | **前端元件庫文件化** | Storybook 建置 | 前端 |
| 20 | **效能監控（APM）整合** | 應用程式效能監控 | DevOps |
| 21 | **前端超長頁面重構** | `ProtocolEditPage` 已完成（4240→1830 行），配合需求變更時機漸進式重構 | 前端 |
| 22 | **SEC-39：Two-Factor Authentication** | TOTP 二階段驗證（Google Authenticator） | 前後端 |
| 23 | **SEC-40：Web Application Firewall** | ModSecurity 或 Cloudflare WAF | DevOps |
| 24 | **SEC-42：Secrets 管理集中化** | Docker Secrets / Vault 取代 `.env` 管理敏感設定 | DevOps |

---

## 🚀 v2.0 遠程規劃（iPig ERP）

| # | 項目 | 說明 |
|---|------|------|
| 25 | 條碼掃描功能 | 行動裝置支援 |
| 26 | 進階成本法（FIFO） | 成本計算升級 |
| 27 | 批號效期到期提醒進階設定 | 更細緻的提醒規則 |
| 28 | 作廢已核准單據沖銷機制 | Reversal Document |
| 29 | 庫位管理（Bin Location） | 倉儲精細化管理 |
| 30 | 會計 / ERP API 對接 | 外部系統整合 |

---

## 📊 待辦統計

| 優先級 | 數量 | 佔比 |
|--------|------|------|
| 🔴 P2 中優先 | 6 | 23% |
| 🔵 P3 低優先 | 6 | 23% |
| 🟣 P4 品質提升 | 5 | 19% |
| ⚪ P5 長期演進 | 6 | 23% |
| 🚀 v2.0 遠程 | 3 | 12% |
| **合計** | **26** | **100%** |

---

## 變更紀錄

| 日期 | 內容 |
|------|------|
| 2026-02-19 | 📦 前端 Bundle 優化：React.lazy code-splitting（50 頁面）+ manualChunks（7 vendor chunks）+ auth.ts import 修正，主 chunk 3,267→242 KB（-92.6%） |
| 2026-02-18 | 💊 治療方式藥物選單規劃：三方案比較（直接 ERP / 獨立表 / 混合），建議方案 C（`treatment_drug_options` 表 + ERP 匯入 + DrugCombobox），影響 ObservationForm / SurgeryForm / EmergencyMedication 三表單 |
| 2026-02-18 | 🗃️ Migration 整合 10→8：GPS 4 欄位併入 005 CREATE TABLE + 010 seed 已存在於 008（刪除 009/010） |
| 2026-02-18 | 🔔 通知路由管理頁面：新增 `NotificationRoutingPage.tsx`（Table + CRUD Dialog + 啟停 Switch）、`App.tsx` 路由註冊、`MainLayout.tsx` 側邊欄導航 |
| 2026-02-17 | 🔔 進階通知觸發邏輯：AUP 全員意見完成/全部已解決/動物異常紀錄 → 非同步通知；修復 `observation.rs` 欄位引用、`review.rs` `Option<Uuid>` 型別、移除未使用變數 |
| 2026-02-17 | 🔔 通知路由進階設定：後端新增事件類型分類 API（5 大類 26 種事件）+ 角色清單 API + 5 條 seed data；前端 `NotificationRoutingSection.tsx` 升級為下拉選單 + 分類卡片 |
| 2026-02-17 | 💾 SEC-31 資料庫自動備份：`scripts/backup/` 建立 `pg_backup.sh`（pg_dump+gzip+30天清理+rsync）+ `Dockerfile.backup` + `entrypoint.sh`（cron 排程）+ `docker-compose.yml` 新增 `db-backup` 服務 + `BACKUP.md` 文件 |
| 2026-02-17 | 🧪 CI 測試修復：`enums.rs` display_name 與測試同步（Completed→"實驗完成"、White→"白豬"）+ 移除 2 個未使用 import（`IacucChangeInfo` re-export、`use super::*`），87 測試全通過 |
| 2026-02-17 | 🔔 啟動配置警告 Dialog：後端 `GET /admin/config-warnings` API + 前端 `MainLayout.tsx` Dialog，管理員登入後彈出三項配置狀態（⚠️/✅/ℹ️），按確認關閉，sessionStorage 防重複彈出 |
| 2026-02-17 | 🔧 啟動配置匯總框改進：三項狀態（地理圍籬/ADMIN_INITIAL_PASSWORD/SEED_DEV_USERS）永遠顯示，設定正確顯示 ✅、有問題顯示 ⚠️、未啟用顯示 ℹ️ |
| 2026-02-17 | 🔐 密碼更新不登出：後端 `change_own_password` 改為重新簽發 tokens + 回傳新 cookies、前端 `MainLayout.tsx` 移除 `logout()` 改用 `checkAuth()` |
| 2026-02-17 | 🔒 打卡 IP 限制：`ALLOWED_CLOCK_IP_RANGES` 環境變數 + CIDR 白名單驗證 + IP 寫入 DB + 前端 403 友善提示 |
| 2026-02-17 | 📍 GPS 定位打卡：Haversine 距離計算 + IP/GPS 擇一驗證 + 前端 Geolocation API + DB 記錄經緯度 + 辦公室座標 24.654053, 120.784923 |
| 2026-02-17 | 🔧 修復 `cleanup_test_data.sql`：3 處 bug（`google_calendar_config` 欄位名、`system_settings` 欄位名、89 個 FK 約束）+ 舊 `pigs`→`animals` 表名 + 10+ 個遺漏表，測試資料清理成功 |
| 2026-02-17 | 🧹 測試套件整合與清理：執行 `cleanup_test_data.ps1` 清理測試資料、4 個過時腳本移入 `_archive/`（`aup_test_standalone.py`、`audit_check_deep.py`、`audit_check_quick.py`、`debug_csrf.py`）、`README.md` 全面改寫（3→8 模組完整說明） |
| 2026-02-17 | 🔒 IACUC No. 變更保護：後端禁止實驗中動物更改 IACUC No. + IACUC_CHANGE 審計事件 + `GET /animals/:id/events` API；前端 AnimalEditPage 禁用下拉 + AnimalTimelineView IACUC 變更事件（amber 配色） |
| 2026-02-17 | 🐷 動物詳情頁新增「登記猝死」按鈕 + Dialog 表單（發現時間/地點/原因/備註/病理檢查），`in_experiment` 和 `completed` 狀態下顯示 |
| 2026-02-17 | 📝 Profiling_Spec v5.0 全面重寫：01-09 主文件、5 個模組文件、README、database_erd 全部更新。新增轉讓/猝死/手寫簽章/資料隔離/通知路由等功能文檔，更新統計數字（~293 端點、~73 資料表、8 migration） |
| 2026-02-17 | 🧪 測試修復：Animal 27/27 + AUP Integration 41/41 全部通過。修正犧牲動物狀態驗證、co-editor 權限角色、approve_protocol 重複 PRE_REVIEW、ear_tag 格式、vaccine_date 欄位、動物狀態機轉換、transfer 權限統一 EXP_STAFF |
| 2026-02-17 | 🗃️ 資料庫遷移整合：14 個 migration 合併為 8 個（血液/猝死/轉讓→003、AUP 審查→004、偏好→001、權限→002、通知+簽章→008），刪除 6 個舊檔 |
| 2026-02-16 | 🐷 第三波前端轉讓 UI：TransferTab 元件（Stepper + 6 步表單）+ Timeline transferred 事件 + EditPage 防護 |
| 2026-02-16 | 🐷 動物狀態生命週期重構（第二波後端完成）：transferred 狀態 + 轉讓 6 步 API（8 路由）+ DB migration 014 + Timeline 安樂死/猝死事件 |
| 2026-02-16 | 🐷 動物狀態生命週期重構（第一波完成）：新增 `euthanized`/`sudden_death` 狀態 + 狀態轉換驗證 + 猝死登記 API + 前端 Tab/顏色/多語系更新 |
| 2026-02-16 | 📱 行動裝置電子簽章模組加入 P2 待辦：跨功能手寫簽名擷取（轉讓/犧牲/安樂死/審查），獨立於動物狀態重構 |
| 2026-02-16 | 安全警報排序功能：`AdminAuditPage.tsx` 安全警報表支援按時間/類型/嚴重程度/狀態排序，前端 `useMemo` 排序邏輯 |
| 2026-02-16 | IACUC No. 大小寫修正：前端 5 檔 6 處 + 後端 3 檔 7 處 `IACUC NO.` → `IACUC No.` |
| 2026-02-16 | 修復時間軸未顯示實驗完成事件：`AnimalTimelineView.tsx` 加入犧牲紀錄與實驗完成項目，`AnimalDetailPage.tsx` 犧牲資料提前載入 |
| 2026-02-16 | 修復欄位表顯示錯誤：`AnimalsPage.tsx` `renderPenCell` 誤用外層 `animals` 變數，改為 `penAnimals` |
| 2026-02-16 | 分頁功能實作：`AdminAuditPage.tsx` 4 Tab 伺服器端分頁 + `UsersPage.tsx` 前端分頁（每頁 50 筆） |
| 2026-02-16 | 測試套件整合重構：`test_fixtures.py`（24 角色統一帳號）+ `test_context.py`（SharedTestContext 共享 token）+ 8 個測試模組 ctx 注入 + `run_all_tests.py` 一次性登入 + protocol_id 複用 + 動物數 20→5 + AUP 動物 5→2 |
| 2026-02-16 | 測試帳密環境變數化：10 個 `.py` 檔案 `password123` → `TEST_USER_PASSWORD` 環境變數、`test_base.py` DB fallback 修正 |
| 2026-02-16 | 測試套件重寫：3 個損壞測試（HR/Amendment/ERP Permissions）重建、所有 `.py` 硬編碼帳密移除、`run_all_tests.py` 從 5→7 模組、建立 `test_spec.md` |
| 2026-02-16 | 測試檔案 pig→animal 重構：`test_animal_full.py` 變數名、`test_blood_panel.py` bug 修正、`cleanup_test_data.ps1` SQL 更新、`audit_check_deep.py` entity_type 更新 |
| 2026-02-16 | NAMING_CONVENTIONS.md 重寫（pig→animal）：全面更新命名範例，版本升 3.0 |
| 2026-02-16 | AuditLogsPage.tsx 前端 pig→animal 修正：事件類型4個、實體類型7個、篩選器2處 |
| 2026-02-16 | Profiling_Spec 全面重寫（pig→animal）：12 個文件、版本升 4.0 |
| 2026-02-16 | 資料庫遷移重寫：pig → animal 在初始遷移中直接使用，刪除 012 rename 遷移，12→11 檔 |
| 2026-02-15 | 重新整理 TODO 結構：移除已完成項目至 PROGRESS.md、統一編號、新增統計摘要 |
| 2026-02-15 | 資安強化提案 SEC-31~42 規劃；SEC-20~28 共 9 項實作完成 |
| 2026-02-15 | 通知路由可配置化完成；Profiling_Spec 01-09 全部重寫 |
| 2026-02-15 | 行動端適配、稽核匯出、活動紀錄分頁、GeoIP 整合 |
| 2026-02-14 | 安全性強化、Docker 升級、Session heartbeat、ERP 增強 |
| 2026-02-13 | 血液檢查流程、資料分析模組、Amendment 測試、前端技術債 |
| 2026-02-12 | 整合測試 Bug 修復、動物狀態簡化、時間軸增強、ID 遷移 |
| 2026-02-11 | 安全審計強化、整合測試腳本、角色權限修復 |
| 2026-02-10 | 多帳號/大量登入偵測、安全警報修復 |
| 2026-02-09 | 審查委員強制意見、多輪往返功能、翻譯標準化 |
| 2026-02-08 | AUP 歷程紀錄增強、動物列表體重排序 |
| 2026-02-03 | Login As 功能、版本比較強化、Dashboard 錯誤處理 |
| 2026-02-02 | UUID 遷移、Amendment 後端、安樂死工作流程 UI |