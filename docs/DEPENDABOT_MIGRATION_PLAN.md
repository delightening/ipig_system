# Dependabot PR 遷移計畫

> 依「可安全合併 → 需遷移測試 → Backend」三階段執行。驗證腳本：`scripts/verify-deps.sh` / `scripts/verify-deps.ps1`

## 一、總覽與 PR 分類表

| 分類 | PR | 套件 | 狀態 |
|------|-----|------|------|
| 可安全合併 | #4–7 | GitHub Actions (checkout, setup-node, cache, upload-artifact) | Phase 1 |
| | #18 | validator 0.19→0.20 | Phase 1 |
| | #20 | lucide-react 0.323→0.575 | Phase 1 |
| | #22 | axios 1.13.5→1.13.6 | Phase 1 |
| | #26 | @types/dompurify 3.0.5→3.2.0 | Phase 1 |
| 需遷移 | #23, #27 | zod 4 + @hookform/resolvers 5 | Phase 2 |
| | #25 | zustand 5 | Phase 2 |
| | #21 | date-fns 4 | Phase 2 |
| | #19, #12 | react-ecosystem, dev-dependencies | Phase 2 |
| 暫緩 | #24 | tailwind-merge 3（需 Tailwind v4） | Phase 2.5 可選 |
| | #15 | axum-extra 0.12（需 axum 0.8） | 待 axum 升級 |
| Backend | #8–14, #16–17 | thiserror, jsonwebtoken, utoipa 等 | Phase 3 |

---

## 二、Phase 1：可安全合併（已完成）

- GitHub Actions：checkout v6、setup-node v6、cache v5、upload-artifact v7
- validator 0.20、axios ^1.13.6、lucide-react ^0.575.0、@types/dompurify ^3.2.0

**驗證**：`./scripts/verify-deps.sh` 或 `.\scripts\verify-deps.ps1`

---

## 三、Phase 2：需遷移或測試

見計畫詳情：zod+resolvers → zustand → date-fns → 其他。每批完成後執行 `npm run build && npm run test:run`。

---

## 四、Phase 2.5：Tailwind v4 + tailwind-merge v3（可選）

需先升級 Tailwind CSS 至 v4，再升級 tailwind-merge。詳見計畫檔。

---

## 五、Phase 3：Backend 遷移

**已完成**：metrics-exporter-prometheus、thiserror、jsonwebtoken、tower、tokio-cron-scheduler。

**暫緩**（需大量遷移）：printpdf 0.9（API 變更）、utoipa 5 + utoipa-swagger-ui 9（需 axum 0.8）。

---

## 六、驗證速查

| 情境 | 指令 |
|------|------|
| Phase 1 全驗證 | `./scripts/verify-deps.sh` 或 `.\scripts\verify-deps.ps1` |
| Phase 2 單批完成 | `cd frontend && npm run build && npm run test:run` |
| Phase 3 單 crate 完成 | `cd backend && cargo check --release && cargo test` |
| 合併前完整 E2E | `cd frontend && npm run test:e2e` |

---

完整遷移細節、相依關係圖與各套件 breaking changes 見計畫原始檔。
