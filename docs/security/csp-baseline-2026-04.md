# CSP 收緊基準掃描（R31-1）

**建立日期**：2026-04-29
**任務**：R31-1（CSP report 基準掃描）
**目的**：為 R31-4 ~ R31-10 收緊步驟提供決策依據

---

## 1. 當前 enforce CSP（`frontend/security-headers.conf:12`）

```
default-src 'self';
script-src 'self' 'unsafe-inline' 'unsafe-eval' https://static.cloudflareinsights.com;
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
font-src 'self' https://fonts.gstatic.com data:;
img-src 'self' data: blob:;
connect-src 'self' https://cloudflareinsights.com https://www.google-analytics.com https://analytics.google.com;
frame-ancestors 'none';
report-uri /api/v1/csp-report
```

**已知破口**：
- `script-src 'unsafe-inline' 'unsafe-eval'` — XSS script 注入幾乎無防線
- `style-src 'unsafe-inline'` — inline style 注入無防線

---

## 2. 靜態分析結果（不需等 Report-Only 收集）

### 2.1 `frontend/index.html` inline 內容

| 位置 | 類型 | 說明 |
|---|---|---|
| line 32 | `<style>` | `#static-landing{display:none}` — 1 行 |
| line 33 | `<style>` | `#static-landing{display:block}`（noscript 包裹）— 1 行 |
| line 36-51 | `<script type="application/ld+json">` | Schema.org Organization |
| line 54-62 | `<script type="application/ld+json">` | Schema.org WebSite |
| line 65-89 | `<script type="application/ld+json">` | Schema.org SoftwareApplication |
| line 92-123 | `<script type="application/ld+json">` | Schema.org FAQPage |
| line 240 | `<script src=...>` | external module（無問題） |

> **JSON-LD 仍受 `script-src` 管制**（即使 type 不是 JS），瀏覽器一律視為 script。
> 4 個 block 內容固定 → R31-7 nonce 化或 hash 化都可。

### 2.2 React inline style 用量

```
grep "style={{" frontend/src/  → 238 處 / 32 檔案
grep ".style.|setProperty"     → 8 處 / 6 檔案
```

**結論**：量級遠超 R31-3 設定的 50-處門檻 → **`style-src 'unsafe-inline'` 標記為 R31-13 已接受風險**，不主動收緊。

熱區檔案（≥10 處）：
- `PainAssessmentTab.tsx` (25)
- `SurgeriesTab.tsx`、`ObservationsTab.tsx` (各 17)
- `InvitationsPage.tsx` (15)
- `WeightsTab.tsx`、`VaccinationsTab.tsx`、`SurgeriesTab.tsx`、`AmendmentsTab.tsx`、`PersonnelSection.tsx`、`BloodTestTab.tsx` (各 13)
- `ReviewersTab.tsx`、`AttachmentsTab.tsx` (各 11)

### 2.3 動態 eval / Function 構造

```
grep "eval(\|new Function(" frontend/src/  → 0 處
```

**結論**：prod build 不需要 `'unsafe-eval'`。Vite **dev server** HMR 內部使用 eval-like 行為，但 `frontend/security-headers.conf` 僅在 prod nginx 容器內生效（`frontend/Dockerfile:51` 注入）；本地 `vite dev` 不經 nginx，不受此 CSP 影響。

---

## 3. R31-1 Report-Only 部署（本 PR 內容）

新增 `Content-Security-Policy-Report-Only` header 與當前 enforce header 並存：

```
script-src 'self' https://static.cloudflareinsights.com;     # 移除 'unsafe-inline' + 'unsafe-eval'
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;  # 維持 'unsafe-inline'（接受風險）
其餘同 enforce
report-uri /api/v1/csp-report?mode=ro
```

後端 `csp_report_handler` 加 `Query<CspReportQuery>` 解析 `?mode=ro`，違規寫入 `security_alerts` 時以 `alert_type = CSP_VIOLATION_REPORT_ONLY` 區分。

### 觀察期：1 週

#### 預期 Report-Only 違規來源

| 來源 | violated_directive | blocked_uri | 對策 |
|---|---|---|---|
| index.html JSON-LD ×4 | `script-src` | `inline` | R31-7 加 nonce |
| index.html `<style>` ×2 | `style-src` | `inline` | 已 accept；hash 化可選 |
| Cloudflare Insights | （應已白名單） | — | 確認 `connect-src` 未漏 |
| Google Analytics | （應已白名單） | — | 確認 `connect-src` 未漏 |

#### 監控 SQL

```sql
SELECT
  context_data->>'violated_directive' AS directive,
  context_data->>'blocked_uri'        AS blocked,
  COUNT(*)                            AS hits
FROM security_alerts
WHERE alert_type = 'CSP_VIOLATION_REPORT_ONLY'
  AND created_at > NOW() - INTERVAL '7 days'
GROUP BY 1, 2
ORDER BY hits DESC;
```

---

## 4. 後續決策樹（一週後）

```
觀察 7 天 csp_report 結果
├── 只看到 index.html inline 違規 → 可進 R31-7（nonce 化）
├── 出現未預期第三方 connect-src 違規 → 補白名單；推遲 R31-10 enforce 切換
└── 出現大量 inline script 違規 → 有 React 注入 inline 風險，需先排查再收緊
```

---

## 5. 風險與回滾

- **本 PR 風險**：Report-Only 不擋資源，**無使用者可見影響**；最壞情況是 violation 量爆掉 `security_alerts` 表 → 觀察 24h 若超量可立即移除 Report-Only header。
- **回滾**：revert 本 commit 即可，不涉及 schema 或不可逆變更。
