# 安全性紀錄

本文件記錄專案中已評估的漏洞與處置方式，供稽核與後續追蹤使用。

---

## Rust 依賴漏洞（cargo audit）

CI 已設定 `cargo audit --ignore` 排除下列項目，以下為評估結論。

### RUSTSEC-2023-0071（rsa 0.9.10）

| 項目 | 說明 |
|------|------|
| 漏洞 | Marvin Attack：透過時序側通道潛在的金鑰恢復 |
| 依賴路徑 | sqlx-mysql（transitive）→ rsa |
| 專案使用 | 本專案僅使用 sqlx 的 **postgres** feature，未使用 mysql |
| 處置 | CI 以 `--ignore RUSTSEC-2023-0071` 排除；上游 sqlx 無修復版本 |
| 風險 | 低 — mysql 驅動未被編譯進最終二進位檔 |

### RUSTSEC-2024-0370（proc-macro-error）

| 項目 | 說明 |
|------|------|
| 漏洞 | proc-macro-error 已不再維護 |
| 依賴路徑 | utoipa → utoipa-gen → proc-macro-error |
| 處置 | CI 以 `--ignore RUSTSEC-2024-0370` 排除；追蹤 utoipa 上游更新 |
| 風險 | 低 — 僅在編譯期使用，不影響執行期 |

### Yanked crates（js-sys, wasm-bindgen）

| 項目 | 說明 |
|------|------|
| 說明 | 間接依賴的 js-sys、wasm-bindgen 曾被 yank |
| 處置 | CI 執行 `cargo update` 以取得非 yanked 版本 |
| 風險 | 低 — 更新後應可排除 |

---

## CVE-2026-25646（libpng 堆緩衝區溢位）

### 摘要

| 項目 | 說明 |
|------|------|
| CVE | CVE-2026-25646 |
| 元件 | libpng 1.6.54-r0（位於 frontend 映像之基礎映像，修復版 1.6.55-r0） |
| 類型 | 堆緩衝區溢位 |
| 目前處置 | 列入 `.trivyignore`，不修復 |
| 最後檢查 | 2026-02-28 — 已升級至 `georgjung/nginx-brotli:1.29.5-alpine`（Alpine 3.23.3），CVE 仍存在 |

### 適用範圍

- **映像**：frontend（`frontend/Dockerfile`）
- **基礎映像**：`georgjung/nginx-brotli:1.29.5-alpine`（Alpine 3.23.3, nginx 1.29.5）

### 採用 .trivyignore 之原因

- 修復需在映像內執行 `apk upgrade` 以更新 libpng。
- 該映像內 nginx 的 **Brotli 動態模組**（`.so`）為預編譯產物，與基底 Alpine 的 glibc/ABI 綁定；執行 `apk upgrade` 會更新核心套件，導致與既有 Brotli 模組 **ABI 不相容**，nginx 啟動時無法載入模組而失敗。
- 因此在此映像中「只升級 libpng」與「保留 Brotli 功能」無法同時達成，故暫時以忽略 CVE 方式處理。

### 風險評估：LOW

- 前端映像僅負責提供 **靜態資源**（HTML / JS / CSS 等），不對使用者上傳的 PNG 進行解析或處理。
- libpng 漏洞通常需攻擊者能提供惡意 PNG 並由受影響程式解析才會觸發；本服務不具此攻擊面，故評估為低風險。

### 長期作法

- 追蹤基礎映像 [georgjung/nginx-brotli](https://hub.docker.com/r/georgjung/nginx-brotli/tags) 或上游 Alpine 是否釋出已修補 libpng 的新版。
- 當有更新版基礎映像可用時，升級基礎映像並從 `.trivyignore` 移除此 CVE。

### 方案 D 已採納：圖片處理分離原則

本專案**已採納方案 D**，作為架構原則：

- **frontend 映像**僅負責提供靜態資源（HTML / JS / CSS / 靜態資產），**不解析、不處理**使用者上傳的 PNG 或其他圖片格式。
- 若有**解析或處理使用者上傳圖片**的需求（例如縮圖、驗證、轉檔、OCR），必須由**獨立服務**實作；該服務使用可安全執行 `apk upgrade` 的基底映像（例如 `nginx:alpine`、`alpine`、或官方語言 runtime 映像），以確保 libpng 等依賴可及時修補，且不影響 frontend 的 Brotli 與現有建置。

如此可維持攻擊面隔離：frontend 維持低風險與 .trivyignore 現狀，需處理不可信圖片的工作集中在可升級的單一服務。

#### 未來新增「圖片處理服務」時之實作指引

| 項目 | 建議 |
|------|------|
| **基底映像** | 使用可安全 `apk upgrade` 的映像（如 `nginx:alpine`、`python:alpine`、`node:alpine` 等），建置時執行 `apk upgrade` 以取得 libpng 等安全更新。 |
| **職責** | 僅負責圖片相關操作：縮圖、格式轉換、尺寸/格式驗證、必要時病毒/惡意檔掃描。不負責一般靜態檔託管。 |
| **介面** | 由後端 API 或 BFF 呼叫（上傳檔先到後端，後端再轉交此服務處理），或透過佇列非同步處理；前端不直連此服務。 |
| **部署** | 獨立容器/服務，可與現有 docker-compose 或 K8s 並存；Trivy 掃描此映像時不應再依賴 .trivyignore 之 CVE-2026-25646（因可升級修補）。 |
| **文件** | 新服務需有簡短 README 與 Dockerfile 註解，註明「本服務可執行 apk upgrade，用於隔離圖片解析攻擊面」。 |

---

## 若採積極修復之處置選項

若未來風險接受度改變（例如需通過嚴格合規掃描、或前端開始處理使用者上傳圖片），可考慮以下其中一種作法。

### 選項 A：改用已修補之基礎映像（首選）

- **作法**：等待或選用已修補 libpng 的 `georgjung/nginx-brotli:alpine` 新 tag，將 `frontend/Dockerfile` 的 `FROM` 改為該 tag。
- **優點**：無需改架構，Brotli 與安全性更新兼得。
- **缺點**：依賴上游維護者釋出更新。

### 選項 B：自建 nginx + Brotli 映像

- **作法**：以官方 `nginx:alpine` 或 `alpine` 為基底，在 Dockerfile 中自行編譯 nginx 與 Brotli 模組；在**同一建置流程內**先 `apk upgrade` 再編譯，使 Brotli 與升級後的系統 ABI 一致。
- **優點**：可完全控制何時執行 `apk upgrade`，並保留 Brotli。
- **缺點**：需維護自建映像與建置腳本，建置時間較長。

### 選項 C：捨棄 Brotli，改用官方 nginx:alpine

- **作法**：將基礎映像改為 `nginx:alpine`，在 Dockerfile 中執行 `apk upgrade`，僅使用 nginx 內建 gzip，不再使用 Brotli。
- **優點**：可立即取得 libpng 等套件更新，設定簡單。
- **缺點**：失去 Brotli 壓縮，靜態資源壓縮率略降。

### 選項 D：分離靜態服務與潛在受影響元件（**已採納**）

- **作法**：若未來有「必須解析使用者上傳 PNG」的需求，改為在**其他服務**（例如獨立縮圖服務）處理，該服務使用可安全升級的基底映像；frontend 仍僅提供靜態檔，不解析 PNG。
- **優點**：攻擊面隔離，frontend 維持現狀與 Brotli。
- **缺點**：需額外服務與維運。
- **狀態**：本專案已採納此原則；詳見上文「方案 D 已採納：圖片處理分離原則」及未來新增圖片服務之實作指引。

### Pros / Cons 比較表

| 維度 | A. 改用已修補基礎映像 | B. 自建 nginx + Brotli | C. 捨棄 Brotli | D. 分離服務 |
|------|------------------------|-------------------------|-----------------|-------------|
| **Pros** | 不改架構；Brotli 與修補兼得；改動最小（只改 FROM） | 不依賴上游；可隨時 apk upgrade；保留 Brotli | 可立即修補；設定簡單；官方映像維護佳 | 攻擊面隔離；frontend 與 Brotli 不變；僅在「要解析 PNG」時才需加服務 |
| **Cons** | 依賴上游釋出更新，時程不可控 | 需維護自建映像與編譯腳本；建置時間長；升級 nginx/OpenSSL 時可能需重編 | 失去 Brotli，壓縮率略降、傳輸量略增 | 僅在「有解析 PNG 需求」時有意義；多一個服務要部署與維運 |
| **實作成本** | 低 | 高 | 低 | 中（依是否已有縮圖/圖片服務） |
| **可執行時機** | 待上游出新 tag | 隨時 | 隨時 | 當需求出現時 |

---

*最後更新：2026-02-28 — 基礎映像版本釘選至 1.29.5-alpine，CVE-2026-25646 經 Trivy 掃描確認仍存在（libpng 1.6.54-r0 → 修復版 1.6.55-r0），下次檢查排定 2026-Q2。*
