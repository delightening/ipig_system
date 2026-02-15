# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-02-15

---

## 🔴 P2 — 中優先（安全強化 / 體驗優化）

| # | 項目 | 說明 | 範圍 | 難度 |
|---|------|------|------|------|
| 1 | **豬隻→動物命名重構** | 將 `pig` 命名統一為 `animal`，提升擴充性 | 全端 | 🔴 高 |
| 2 | **AUP 參考文獻格式** | 參考農業部提供之格式調整 | 前端 | 🟢 低 |
| 3 | **疼痛評估紀錄時間軸** | 動物管理中的疼痛評估紀錄視覺化檢視 | 前端 | 🟡 中 |
| 4 | **敏感資料二級審計** | 密碼修改、權限變更等操作的加強稽核紀錄 | 後端 | 🟡 中 |
| 5 | **安全警報即時推送** | Email / 站內通知即時推送安全警報（WebSocket/SSE） | 前後端 | 🔴 高 |
| 6 | **獸醫師建議通知進階設定** | 獸醫師建議通知機制的進階設定項 | 前後端 | 🟡 中 |

### 豬隻→動物命名重構（待決定）

> 📋 **分析報告**：[implementation_plan.md](file:///C:/Users/jason/.gemini/antigravity/brain/c319adf4-3f08-4bca-9bf4-fbcf710d0f0a/implementation_plan.md)

**影響範圍**：DB 10 表 + 6 enum、後端 30+ 路由 + 20+ struct、前端 4 pages + 14 components + 50+ API 呼叫、測試 2 檔

**待決定事項**：
- [ ] 是否一併改資料庫（表名/enum/欄位），還是只改程式碼層面？
- [ ] `pig_breed` enum（miniature/white/LYD）是豬特有品種，是否改為通用設計？
- [ ] 是否需保留 `/pigs` 路由向後相容？
- [ ] 選擇方案：A 分層漸進（推薦）/ B 一次性全改 / C 只改前端

---

## 🔵 P3 — 低優先（資安 / 基礎設施）

| # | 項目 | 說明 | 範圍 |
|---|------|------|------|
| 7 | **SEC-15：Named Tunnel 遷移** | 從 Cloudflare Quick Tunnel 遷移至 Named Tunnel | DevOps |

---

## 🟣 P4 — 中期品質提升（測試 / 文件 / CI）

| # | 項目 | 說明 | 範圍 |
|---|------|------|------|
| 8 | **Rust 測試覆蓋率擴充** | 87 個測試通過，protocol/hr/facility 已有測試覆蓋。需 test DB 做整合測試 | 後端 |
| 9 | **前端 E2E 測試** | Playwright 自動化測試 | 前端 |
| 10 | **OpenAPI 文件完善** | utoipa 補全 API 文件 | 後端 |
| 11 | **CI/CD PostgreSQL service** | CI 中增加 PostgreSQL service container | DevOps |

---

## ⚪ P5 — 長期演進

| # | 項目 | 說明 | 範圍 |
|---|------|------|------|
| 12 | **前端元件庫文件化** | Storybook 建置 | 前端 |
| 13 | **資料庫備份自動化** | 排程備份 + 異地備份 | DevOps |
| 14 | **效能監控（APM）整合** | 應用程式效能監控 | DevOps |
| 15 | **前端超長頁面重構** | `ProtocolEditPage` 已完成（4240→1830行），配合需求變更時機漸進式重構 | 前端 |

---

## 🚀 v2.0 遠程規劃（iPig ERP）

| # | 項目 | 說明 |
|---|------|------|
| 16 | 條碼掃描功能 | 行動裝置支援 |
| 17 | 進階成本法（FIFO） | 成本計算升級 |
| 18 | 批號效期到期提醒進階設定 | 更細緻的提醒規則 |
| 19 | 作廢已核准單據沖銷機制 | Reversal Document |
| 20 | 庫位管理（Bin Location） | 倉儲精細化管理 |
| 21 | 會計 / ERP API 對接 | 外部系統整合 |

---
---

## ✅ 已完成項目紀錄

### 2026-02-15
- [x] **行動端適配（響應式設計）** — `MainLayout` overlay sidebar + 漢堡選單、高頻頁面表格/篩選/標題響應式、`index.css` 全域工具 class
- [x] **稽核日誌匯出 CSV/PDF** — 後端 export API + 前端 CSV（BOM）及 PDF（可列印 HTML）匯出
- [x] **活動紀錄分頁優化** — `ProtocolDetailPage.tsx` 歷程 Tab 前端分頁
- [x] **修復 Login 頁面 401 無限迴圈** — interceptor 改用 zustand `clearAuth()` + `isLoggingOut` 鎖
- [x] **修復 IP 異常偵測 bug** — SQL `::INET` 轉型修正
- [x] **GeoIP 地理位置異常偵測** — MaxMind GeoLite2-City 國家層級比對

### 2026-02-14
- [x] **安全性強化** — 真實 IP 記錄、移除預設帳密、安全警報修復
- [x] **基礎設施升級** — Docker Node.js v22、Rust/Nginx 版本固定
- [x] **Session 管理強化** — heartbeat 機制、`last_activity_at` 即時更新
- [x] **ERP 功能增強** — 客戶分類、銷售成本、UOM 統一
- [x] **AUP / Amendment 修復** — Section 8 解析、翻譯鍵、管理員檢視
- [x] **Email 修復** — 文字顏色、破圖、系統網址更新
- [x] **帳號管理** — 移除入職日期必填驗證
- [x] **PDF 報表分頁優化** — 每隻豬獨立分頁、封面摘要
- [x] **血液檢查組合管理頁面** — `BloodTestPanelsPage.tsx` CRUD
- [x] **技術債清理** — `document.rs`、`models/animal.rs` 拆分
- [x] **SEC-14** — 檔案上傳 Magic Number 驗證（14 測試通過）
- [x] **SEC-02** — Token 改存 HttpOnly Cookie
- [x] **T6 前端重構** — `ProtocolEditPage.tsx` 拆分為 10 個 Section 元件 + 4 個工具模組

### 2026-02-13
- [x] 血液檢查項目管理頁面、Panel 快速勾選、整合測試 28/28
- [x] ERP 站內通知整合、血液檢查移除審核步驟
- [x] 前端技術債基礎建設（Zod 驗證、錯誤處理、骨架屏、型別統一）
- [x] Amendment 整合測試 14 步驟、審查委員記錄
- [x] DB migration 整合（008-029 合併）、HR 測試修復

### 2026-02-12
- [x] 修復 `reply_comment` UTF-8 panic、Animal/AUP 測試修復
- [x] 動物狀態簡化（6→3）、時間軸增強、ID 遷移至 SERIAL

### 2026-02-11
- [x] 安全審計強化、整合測試腳本建立（AUP/ERP/Animal）
- [x] 動物管理 23 個操作接入審計、角色權限修復

### 2026-02-09 ~ 2026-02-10
- [x] 多帳號/大量登入偵測、安全警報修復
- [x] AUP 審查流程多輪往返、審查委員強制意見
- [x] 狀態歷程修復、PI 刪除草稿權限、翻譯標準化

### 更早
- [x] 資料分析模組（血液檢查統計/異常標記/圖表/匯出）
- [x] HR 特休管理系統、GLP 合規、PDF 匯出
- [x] 協編者權限修正、審查意見回覆、Google Calendar 整合
- [x] 資料匯入（Excel/CSV）、Bug 修復（豬隻計數、PigBreed enum、體重驗證）
- [x] v1.0 MVP 完成、檔案上傳服務、通知偏好設定

---

## 變更紀錄

| 日期 | 內容 |
|------|------|
| 2026-02-15 | Profiling_Spec 01-09 全部重寫（含新增 07_SECURITY_AUDIT.md）、README 更新 |
| 2026-02-15 | 行動端適配完成、稽核匯出、活動紀錄分頁、GeoIP 整合、TODO 重新排序 |
| 2026-02-14 | 安全性強化、Docker 升級、Session heartbeat、ERP 增強、AUP/Email 修復 |
| 2026-02-13 | 血液檢查流程、資料分析模組、Amendment 測試、前端技術債 |
| 2026-02-12 | 整合測試 Bug 修復、動物狀態簡化、時間軸增強、ID 遷移 |
| 2026-02-11 | 安全審計強化、整合測試腳本、角色權限修復 |
| 2026-02-10 | 多帳號/大量登入偵測、安全警報修復 |
| 2026-02-09 | 審查委員強制意見、多輪往返功能、翻譯標準化 |
| 2026-02-08 | AUP 歷程紀錄增強、動物列表體重排序 |
| 2026-02-03 | Login As 功能、版本比較強化、Dashboard 錯誤處理 |
| 2026-02-02 | UUID 遷移、Amendment 後端、安樂死工作流程 UI |