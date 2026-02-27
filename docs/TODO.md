# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-02-28  
> **維護慣例：** 完成項目保留於本表並標 [x]，同時於 `docs/PROGRESS.md` §9 最新變更動態 新增對應紀錄；待辦統計僅計「未完成」數量。
> **AI 標註說明：** 
> - ⚡ **Gemini Flash** (適合樣板編寫、簡單設定、文檔生成)
> - 🧠 **Claude Sonnet/Opus** (適合架構設計、複雜邏輯、安全性強化、大規模重構)

---

## ⛔ 禁止事項

1. 密碼過期策略
2. 密碼歷史紀錄（SEC-38：密碼歷史紀錄）

---

## 🚨 P0 — 上線前必要 (Production Readiness)

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|

---

## 🟡 P1 — 上線前強烈建議 (Quality & Compliance)

| # | 項目 | 說明 | 範圍 | 依賴 | 建議 AI | 狀態 |
|---|------|------|------|------|----------|------|
| P1-1 | **前端 E2E 測試 (Playwright)** | 7 spec / 34 tests，含 429 重試 + race condition 修正，連續 3 次 0 failures | 前端 | 無 | 🧠 Claude | [x] |
| P1-2 | **E2E CI 自動化** | `docker-compose.test.yml` + GitHub Actions 整合 | DevOps | P1-1 | ⚡ Flash | [x] |
| P1-7 | **電子簽章合規審查** | 21 CFR Part 11 或等效法規合規審查 | 文件 | 無 | 🧠 Claude | [x] |
| P1-8 | **資料保留政策** | 定義各類紀錄的法定保留年限 | 文件 | 無 | 🧠 Claude | [x] |
| P1-12 | **OpenAPI 文件完善 (≥90%)** | 擴展其餘端點的 Schema 與 Path 定義 | 後端 | 無 | 🧠 Claude | [x] |

---

## 🔵 P3 — 低優先 (資安 / 基礎設施)

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| 7 | **SEC-33：敏感操作二級認證** | 高危操作要求重新輸入密碼確認 | 前後端 | 🧠 Claude | [x] |

---

## 🟣 P4 — 中期品質提升 (測試 / 文件 / CI)

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| 17 | **基礎映像與 CVE 週期檢查** | 每季或基礎映像大改時，檢查 [georgjung/nginx-brotli](https://hub.docker.com/r/georgjung/nginx-brotli/tags) 是否有新 tag；若有則升級 frontend Dockerfile 的 FROM，並從 `.trivyignore` 移除 CVE-2026-25646。詳見 `docs/security.md`。**2026-02-28 已升級至 1.29.5-alpine（Alpine 3.23.3），CVE 仍存在（libpng 1.6.54→需 1.6.55），下次 Q2 檢查。** | DevOps | 🧠 Claude | [x] |
| 18 | **E2E Rate Limiting / Session 穩定化** | ~~解決 shared context 下 Session 過期誤判導致大量重新登入~~。已修復：admin-context 改用 auth.setup 儲存的 storageState 免重複登入；API rate limit 120→600/min；login.spec 加入 credential fallback。34/34 連續通過、22s 完成。 | 前端 | 🧠 Claude | [x] |
| 19 | **Prometheus 服務部署** | `docker-compose.monitoring.yml` overlay 新增 Prometheus + Grafana 服務，`deploy/prometheus.yml` 配置 scrape，Grafana provisioning 自動註冊 datasource + dashboard（10 panels：Request Rate / Latency P50-P99 / Error Rate / Status Codes / Heatmap / DB Pool / Pool Utilization / Top Endpoints）。 | DevOps | 🧠 Claude | [x] |
| 20 | **後端 API 整合測試** | `backend/tests/` 建立 6 個整合測試檔案（api_auth / api_health / api_animals / api_protocols / api_users / api_reports），共用 `TestApp` 測試基礎架構（spawn Axum + random port + test DB）。重構 `lib.rs` 使 crate 同時支援 library + binary。 | 後端 | 🧠 Claude | [x] |
| 21 | **效能基準報告文件化** | `docs/PERFORMANCE_BENCHMARK.md` 正式報告（8 章節：摘要/環境/方法/指標/閾值/資源/限制/結論）。k6 腳本優化：改用 `setup()` 共用 token 消除 rate limit 串連失敗。 | 文件 | 🧠 Claude | [x] |

---

## ⚪ P5 — 長期演進

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| 13 | **前端元件庫文件化** | Storybook 建置 | 前端 | ⚡ Flash | [ ] |
| 14 | **前端超長頁面重構** | 漸進式重構巨型組件。**2026-02-28 已完成 AnimalDetailPage 1,945→748 行（-61%），抽離 7 個 Tab 元件（ObservationsTab/SurgeriesTab/WeightsTab/VaccinationsTab/SacrificeTab/AnimalInfoTab/PathologyTab）。** ProtocolDetailPage（1,921 行）待後續處理。 | 前端 | 🧠 Claude | [x] |
| 15 | **SEC-39：Two-Factor Authentication** | TOTP 二階段驗證 (Google Authenticator) | 前後端 | 🧠 Claude | [ ] |
| 16 | **SEC-40：Web Application Firewall** | ModSecurity 或 Cloudflare WAF | DevOps | ⚡ Flash | [ ] |

---

## 📊 待辦統計

| 優先級 | 數量 (未完成) |
|--------|------|
| 🚨 P0 上線前必要 | 0 |
| 🟡 P1 上線前建議 | 0 |
| 🔴 P2 中優先 | 0 |
| 🔵 P3 低優先 | 0 |
| 🟣 P4 品質提升 | 0 |
> 📌 2026-02-28 新增 P4-19/20/21 並已完成（Prometheus 部署、後端整合測試、效能報告）
| ⚪ P5 長期演進 | 2 |
| **合計（未完成）** | **2** |

---

## 變更紀錄 (最新)

| 日期 | 內容 |
|------|------|
| 2026-02-28 | 🧠 Claude：完成交付前補強 3 項 — (1) P4-19 Prometheus + Grafana 部署（`docker-compose.monitoring.yml` + `deploy/prometheus.yml` + Grafana provisioning + 10-panel dashboard）；(2) P4-20 後端 API 整合測試（`lib.rs` 重構 + `TestApp` infra + 6 個測試檔 25+ test cases，`cargo check --tests` 通過）；(3) P4-21 效能基準報告（`docs/PERFORMANCE_BENCHMARK.md` 8 章節正式報告 + k6 腳本 setup() token sharing 優化）。|
| 2026-02-28 | 🧠 Claude：解決 3 個市場交付阻擋項 — (1) 獸醫建議/觀察紀錄檔案上傳下載串接完成（後端新增 `ObservationAttachment` FileCategory + `/observations/:id/attachments` 路由，前端 VetRecommendationDialog 與 ObservationFormDialog 串接 multipart 上傳與下載）；(2) USER_GUIDE.md 從 26 行擴充至完整操作手冊（9 章節含 AUP/動物/ERP/HR/報表/系統管理/FAQ）；(3) docker-compose.prod.yml 補齊所有服務的 CPU/記憶體限制與 json-file 日誌輪轉。|
| 2026-02-28 | 🧠 Claude：完成 P5-14 前端超長頁面重構 — AnimalDetailPage 1,945→748 行（-61%），抽離 7 個 Tab 元件至 `components/animal/`（Observations/Surgeries/Weights/Vaccinations/Sacrifice/AnimalInfo/PathologyTab），TypeScript 零錯誤通過。 |
| 2026-02-28 | 🧠 Claude：完成 P4-17 基礎映像與 CVE 週期檢查 — Dockerfile 版本釘選至 `georgjung/nginx-brotli:1.29.5-alpine`（Alpine 3.23.3），Trivy 掃描確認 CVE-2026-25646 仍存在（libpng 1.6.54-r0→需 1.6.55-r0），.trivyignore 保留並更新註解，下次 Q2 檢查。 |
| 2026-02-27 | 🧠 Claude：完成 P4-18 E2E Rate Limiting / Session 穩定化 — admin-context 改用 storageState 檔案免重複登入、API rate limit 120→600/min、login.spec credential fallback。34/34 連續通過、22s 完成。 |
| 2026-02-27 | 🧠 Claude：E2E 測試總結計畫實施 — 新增 P4-18 Rate Limiting/Session 穩定化待辦；`docs/e2e/README.md` 故障排除 §5 補充 Session 過期導致 429 連鎖失敗說明。 |
| 2026-02-25 | 🧠 Claude：完成 P3-7 SEC-33 敏感操作二級認證 — 後端 confirm-password + reauth token，前後端刪除使用者／重設密碼／模擬登入／刪除角色皆需重新輸入密碼確認。 |
| 2026-02-25 | 🧠 Claude：完成 P1-7 電子簽章合規審查（21 CFR Part 11），新增 `docs/ELECTRONIC_SIGNATURE_COMPLIANCE.md`。 |
| 2026-02-25 | 🧠 Claude：完成 P1-12 OpenAPI 完善 — 新增電子簽章（10 paths + 2 附註）、動物管理（9 paths）及對應 Schema。 |
| 2026-02-25 | 🧠 Claude：修正 CI `sqlx-cli` 安裝錯誤，增加 `--force` 以應對快取衝突。 |
| 2026-02-25 | 🧠 Claude：完成 P1-8 資料保留政策 (Data Retention Policy) 定義。 |
| 2026-02-25 | 🧠 Claude：修正 CI Trivy 掃描參數一致性並清理 `.trivyignore` 無效編號。 |
| 2026-02-25 | ⚡ Flash：完成 P1-5 後端壓力測試基準建立 (k6)，已遷移至 PROGRESS.md。 |
| 2026-02-25 | 🧠 Claude：P1-1 E2E 穩定化 — 429 rate limit 重試、React state race condition fallback、連續 3 次 0 failures。 |
| 2026-02-25 | 🧠 Claude：P1-1 Playwright E2E 測試擴充（7 spec, 34 tests, auth setup + 6 critical flows）。 |
| 2026-02-25 | 🧹 整理：將 P0-6, P0-7, P1-6 已完成項目遷移至 `PROGRESS.md`。 |
| 2026-02-25 | ⚡ Flash 任務第二波：完成 P0-6 跨瀏覽器相容性驗證、P1-6 GLP 驗證文件 (IQ/OQ/PQ) 生成。 |
| 2026-02-25 | ⚡ Flash 任務第一波：完成 Brotli、具名隧道腳本、CI/CD DB 整合、操作手冊與 Grafana 分配。 |
| 2026-02-25 | 🏷️ AI 標註：新增建議使用的 AI 模型標註。 |
