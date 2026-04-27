# 財務模組 Phase 2–5 評估

> **評估日期：** 2026-03-01  
> **對應：** R6-4、PROGRESS.md 財務 SOC2 QAU 三項規劃

---

## 一、現況摘要

Phase 1 已完成：
- migration 022 會計基礎（科目/傳票/分錄）
- `AccountingService::post_document`
- GRN/DO 核准時自動過帳
- `AccountingReportPage` 試算表、傳票查詢、AP/AR 帳齡 Tab
- 路由 `/accounting`

---

## 二、Phase 2–5 規劃與評估

| Phase | 項目 | 工時估計 | 優先建議 |
|-------|------|----------|----------|
| **Phase 2 (AP)** | ap_payments 表、POST /accounting/ap-payments、GET /accounting/ap-aging、前端「新增付款」 | 2–3 天 | 中：若需實際記錄付款，優先 |
| **Phase 3 (AR)** | ar_receipts 表、POST /accounting/ar-receipts、GET /accounting/ar-aging、前端「新增收款」 | 2–3 天 | 中：與 Phase 2 對應 |
| **Phase 4 (GL)** | GET /accounting/trial-balance、/journal-entries、/chart-of-accounts | 1–2 天 | 部分已有，可擴充查詢條件 |
| **Phase 5 (UI)** | AccountingReportPage 四 Tab 擴充、ERP 報表中心入口 | 0.5 天 | 低：入口已存在 |

---

## 三、建議

1. **依業務需求排程**：若無立即記帳/收款需求，可維持 Phase 1 現況。
2. **Phase 2–3 成對**：AP 與 AR 建議一起規劃，保持一致性。
3. **Phase 4**：trial-balance、journal-entries 已有 API，可視需要補齊 chart-of-accounts 與進階篩選。
