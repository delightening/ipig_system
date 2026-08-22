# 豬隻欄位表（AD-05-01-02C）排版規則

> 對齊官方 xlsx 母檔。模板：`templates/vet_patrol.html`（Chromium/Playwright 渲染）。
> 本規則於 2026-06-26 與使用者逐版調整後定版（v21 驗收通過）。**修改任何儲存格／字級前先讀本檔**。

## 0. 核心規則：儲存格內容溢出處理（最重要）

**規則**：欄寬、列高**固定不變**。當某儲存格內容（耳號）放不下時，**只縮小該格的字級**塞進去，其餘儲存格維持原字級（12pt）。範例基準＝`820.821.822` 那格。

**為什麼要特別處理（技術根因）**：
- 表格儲存格設 `overflow:hidden` 時，`el.scrollWidth` 會等於 `el.clientWidth`，**量不到溢出的真實內容寬度** → 不能用 scrollWidth 偵測溢出。
- 正解：用 `Range.selectNodeContents(cell)` + `getClientRects()` 量「文字實際渲染寬度」（不受裁切影響）。

**演算法**（已實作於 `vet_patrol.html` 末的 `<script>`，字型載入後 `document.fonts.ready` 才執行）：
1. 跳過 G 區寬格（含 `.g-inner` 者，內容本來就放得下）。
2. `avail = cell.clientWidth * 0.80`（**只佔格寬 80%，留 20% 空隙** → 對齊官方對擁擠格用較小字級的觀感，非剛好塞滿）。
3. 若 `textWidth(cell) <= avail` → **不動，維持原字級**（不影響其他格）。
4. 否則逐步 `font-size -= 0.5px` 直到 `textWidth <= avail`（下限 4px）。
5. 完成後設 `data-fit-done="1"`；`main.py` 的 `_html_to_pdf_async` 以 `wait_for_function` 等此旗標 + 字型載入完才印 PDF。

**實測**：86 個有內容的格中，僅 `820.821.822` 縮到 ~8.2pt（其餘 85 格維持 12pt），完整不被裁、左右留空隙。

## 1. 頁面

| 項目 | 值 |
|---|---|
| 紙張 | A4 直式，**整份限一頁** |
| 邊距 | `1.7cm 1.8cm 1.2cm 1.8cm`（上 右 下 左；L/R≈xlsx 0.7"、上邊距讓 body 起點≈xlsx 0.75"）|
| 列高 | 固定 `6.0mm`（rowspan 格 max-height = N × 6.0mm，須同步）|

## 2. 字型（@font-face 唯一名，避免容器 fontconfig alias 碰撞）

| 名稱 | 檔案 | 用途 |
|---|---|---|
| `KaiEmbed` | `kaiu.ttf`（標楷體 DFKai-SB，原始檔，Chromium 原生處理 composite，**勿用 decompose 版**）| 中文 |
| `ArialEmbed` | `arial*.ttf` | 數字 / 英文 / `○` `●` 符號 |
| `TimesEmbed` | `times.ttf` `timesbd.ttf` | 版權英文 |

## 3. 各區字級 / 樣式（定版值）

| 區域 | 字級 | 其他 |
|---|---|---|
| 耳號格 `.tag-cell` | 12pt | `white-space:nowrap`、`padding:0 1.5px`、溢出走 §0 自適應 |
| 狀態 `○`/`●` `.status-cell` | **25pt** | `line-height:0.62`（圓圈大但**不撐高列**；table-cell 忽略 max-height，故靠 line-height 壓行框）|
| 區位標籤 A–G `.zone-label` | 14pt 粗體 | |
| 頁首（文件編號／公司／頁次） | **10pt** | 非粗體；標楷體 |
| 頁尾清單（□ 全場豬隻…） `.footer` | **12pt** | `line-height:1.4` |
| 清單前 □ 方塊 `.cb-box` | **20pt** | `line-height:0`（不撐高行）、`display:inline-block`、`vertical-align:0` |
| 版權（中＋英）`.copyright` | **9pt 粗體** | 內文（接清單下方，非頁底固定）；英文 Times、中文標楷體 faux-bold；`margin-top:8pt` |
| 耳號後底線 | **15 個底線** `_` | |

## 4. 頁眉頁尾位置

- 頁首：`@page @top-left/@top-center/@top-right`，內容 = 文件編號 AD-05-01-02C／公司名／頁次。
- 頁尾版權：**改為內文**（表格後 `.copyright` div），`@bottom-center { content: "" }` 清空 base.html 的頁底版權避免重複 → 版權緊貼清單下方（對齊官方）。

## 5. 列高調整注意

若需整體縮放（塞一頁），列高 `6.0mm` 與所有 `td[rowspan="N"]` 的 `max-height = N×6.0mm` **必須等比同步修改**（否則 carrier cell 畫線與 rowspan 對不齊）。
