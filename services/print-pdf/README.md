# print-pdf — FastAPI HTML→PDF 列印服務

**狀態**：2026-05-15 取代以下 3 個服務並於 `docker-compose.yml` 移除：

- `services/word-convert/` (host Word/Excel COM daemon)
- `services/gotenberg/` (LibreOffice + Chromium PDF 引擎)
- `pdf-service/` (FastAPI 編排層，docxtpl + Gotenberg + Word daemon 三層 fallback)

整合方式：endpoint paths 完全對齊（`/render-aup/from-working-content` 等），backend
`PdfServiceClient` 不改一行，只把 `PDF_SERVICE_URL` 從 `pdf-service:3200` 改指
`print-pdf:9200`，11 個 handler 全部 transparent 切換。

涵蓋文件：

| ID | 中文名 | 原 docx 來源 | GLP 編號 |
|---|---|---|---|
| `pig_approval` | IACUC 審查同意書 | `templates/source/PIG-XXXXXX審查同意書.docx` | — |
| `review_result` | 審核結果單 | `templates/source/AD-04-01-10B審核結果.docx` | AD-04-01-10B |
| `review_reply` | 審查意見回覆表 | `templates/source/APIG-XXXXXX審查意見回覆表.docx` | — |
| `aup_protocol` | 動物試驗研究計畫書 (R32) | `templates/source/R32_template F.docx` | AD-04-01-01F |
| `vet_patrol` | 欄位狀態表（位置圖） | `templates/source/位置圖output.xlsx` | AD-05-01-02C |
| `vet_patrol_report` | 獸醫巡場報告 | — | — |
| `medical_record` | 實驗豬隻病歷總表 | — | — |
| `surgery` | 實驗豬隻手術紀錄表 | — | — |
| `blood_test` | 動物血液檢查紀錄 | — | — |
| `warehouse` | 倉庫現況報表 | — | — |
| `audit_log` | 操作日誌 | — | — |

## 字型需求（build 前置）

GLP pixel-match 表單（巡場報告 / 豬隻欄位表 / 計畫書）使用商用專有字型，**不隨 repo 散布**
（授權限制），已於根 `.gitignore` 排除 `services/print-pdf/fonts/`。**build 主機需自備**下列檔案於
`services/print-pdf/fonts/`，Dockerfile 才能 `COPY fonts ./fonts`：

- `kaiu.ttf`（標楷體 DFKai-SB）
- `times.ttf` `timesbd.ttf` `timesi.ttf` `timesbi.ttf`（Times New Roman 4 變體）
- `arial.ttf` `arialbd.ttf` `ariali.ttf` `arialbi.ttf`（Arial 4 變體）

非 GLP 文件用容器內 Noto Sans CJK / Liberation，不需上列字型。

## 渲染引擎：Chromium (Playwright)

> 2026-06 起本服務 PDF 引擎已由 WeasyPrint 全面改為 **Chromium (Playwright)**（常駐 browser，
> `page.pdf` 出 PDF）。原因：WeasyPrint 的 fontTools subset 會破壞標楷體 DFKai-SB 的
> point-matching composite glyph（破字），Chromium 如瀏覽器/Word 原生正確處理。TOC 頁碼改用
> 「兩遍渲染 + pypdf 文字搜尋回填」（Chromium 無 `target-counter`）。下表為當初選 WeasyPrint 的
> 歷史比較，已被取代，僅供參考。

## （歷史，已取代）為何 WeasyPrint 而非 Chromium / LibreOffice

| 痛點 | Chromium (Gotenberg) | LibreOffice (Word fallback) | WeasyPrint |
|---|---|---|---|
| TOC 自動頁碼 (`target-counter`) | ❌ 需 PagedJS、失敗多 | ❌ | ✅ 原生 |
| `@page` header/footer / 跨頁 thead | ✅ | 限制多 | ✅ |
| 自帶字型 / 不依賴 LibreOffice | ❌ | ❌ | ✅ |
| 並行渲染 | ✅ 多 tab | ❌ threads=1 | ✅ Python async |
| 中文 / CJK | ✅ Noto CJK | ⚠️ docx → docx_x000a_ bug | ✅ Noto CJK + 標楷體 |
| 服務數量 | 3（pdf-service + gotenberg + Word daemon） | — | **1** |

## API

| Method | Path | 用途 |
|---|---|---|
| GET | `/` | Web UI 測試台 |
| GET | `/healthz` | Liveness probe |
| GET | `/docs` | OpenAPI Swagger |
| GET | `/api/templates` | 列模板 + JSON Schema |
| GET | `/api/sample/{id}` | sample JSON（測試用） |
| GET | `/api/preview/{id}?format=html\|pdf` | sample 預覽 |
| POST | `/api/render/{id}?format=html\|pdf` | JSON in → PDF/HTML out |

加上對齊 pdf-service 的 adapter 端點：`/render-aup/from-working-content`、
`/render-medical-record/from-animal-data`、`/render-surgery/from-surgery-data`、
`/render-review-reply/from-review-data`、`/render-review-result/from-review-data`、
`/render-blood-test/from-blood-test-data`、`/render-audit-log/from-export-data`、
`/render-warehouse/from-report-data`、`/render-vet-patrol/from-animals`、
`/render-vet-patrol-report/from-report-data`、`/render-project-medical/from-project-data`。

### vet_patrol 自動填欄

- `header.patrol_date` 未填 → server 自動填今天（`Asia/Taipei`）
- `header.period` 未填 → 早上→`AM`、午後→`PM`

## 開發

```powershell
cd "<repo-root>\services\print-pdf"
python -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
uvicorn main:app --host 0.0.0.0 --port 9200 --reload
```

Windows WeasyPrint 需 GTK runtime（pango/cairo）。建議：

```powershell
# 經 MSYS2：pacman -S mingw-w64-x86_64-pango mingw-w64-x86_64-cairo mingw-w64-x86_64-gdk-pixbuf2
# 或下載 GTK-for-Windows-Runtime-Environment-Installer
```

容器內部已用 `python:3.12-slim` + apt 預裝 `libpango / libcairo / fonts-noto-cjk
/ fonts-arphic-uming`，無需手動安裝。

## 容器整合

`docker-compose.yml` 已切換（2026-05-15）：移除 `gotenberg` / `pdf-service`、
新增 `print-pdf`。backend `PDF_SERVICE_URL` 預設指向 `http://print-pdf:9200`。

```yaml
print-pdf:
  build: ./services/print-pdf
  container_name: ipig-print-pdf
  networks: [backend]
  ports: ["127.0.0.1:9210:9200"]       # host 9210 → container 9200，避開 Elasticsearch 預設 port
  healthcheck: ...
```

## Backend 整合

**Current（已實施 2026-05-15）**：endpoint paths 對齊，**不改 client 一行**。

```rust
// 既有 PdfServiceClient 不動；URL swap 即可
state.pdf_service.render_review_reply_from_review_data(&data, DocxRenderFormat::Pdf).await?
// 內部 POST 到 ${PDF_SERVICE_URL}/render-review-reply/from-review-data
// PDF_SERVICE_URL 現在指向 http://print-pdf:9200
```

**Future（計畫，未實施）**：未來想砍掉 `PdfServiceClient`（GotenbergClient/daemon
相關殘留 API）時，可改成更簡潔的 `PrintPdfClient`：

```rust
// 假想未來 client（尚未實作）
state.print_pdf.render("review_reply", &payload).await?
```

Schema shape 完全對齊（複製自 `pdf-service/app/schemas/`），無需動 adapter / DB query。

## 遷移狀態

| 文件 | 舊路徑 | 新路徑 | 狀態 |
|---|---|---|---|
| `aup_protocol` | Word daemon (GLP) | `print-pdf` | ✅ Cut over 2026-05-15 |
| `review_reply` | Word daemon (GLP) | `print-pdf` | ✅ 同上 |
| `review_result` | Word daemon (GLP) | `print-pdf` | ✅ 同上 |
| `vet_patrol` | Excel daemon (GLP) | `print-pdf` | ✅ 同上 |
| `vet_patrol_report` | Word daemon (GLP) | `print-pdf` | ✅ 同上 |
| `medical_record` | docx + daemon/fallback | `print-pdf` | ✅ 同上 |
| `project_medical` | medical_record × N + pypdf merge | `print-pdf` | ✅ 同上 |
| `surgery` | docx + daemon/fallback | `print-pdf` | ✅ 同上 |
| `blood_test` | docx + daemon/fallback | `print-pdf` | ✅ 同上 |
| `warehouse` | docx + daemon/fallback | `print-pdf` | ✅ 同上 |
| `audit_log` | docx + daemon/fallback | `print-pdf` | ✅ 同上 |
| `pig_approval` | （新文件） | `print-pdf` | ✅ 新增 |

Follow-up（見 `docs/TODO.md` R55-1）：`GotenbergClient` 變成 0-caller dead code，
觀察 ≥1 週後 surgical cleanup（目標 2026-05-22）。

## 已知限制

1. **R32 (`aup_protocol`) 的 checkbox sub-models**：parked template 部分區塊只展示 free-text，
   v2 checkbox 多選表（pain category B/C/D/E、anesthesia、aseptic 等）後續逐項補上。
2. **vet_patrol 版面**：採 zone-grouped 卡片式（A/B/C/D/E/F/G 區）；原 xlsx 是 vet/QA 手調的精細網格，
   未完全 1:1 重現。如需保留 xlsx 原版面，需另開 ticket。
3. **WeasyPrint flex 支援有限**：parked HTML 模板用了一些 `display:flex`，
   render 時可能需逐個 fix（已預先把 `signatures` 改成 `display:table`）。
