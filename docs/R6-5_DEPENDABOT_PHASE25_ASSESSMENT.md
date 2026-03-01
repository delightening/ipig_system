# Dependabot Phase 2.5 依賴評估

> **評估日期：** 2026-03-01  
> **對應：** R6-5、`docs/DEPENDABOT_MIGRATION_PLAN.md`

---

## 一、待評估套件

| 套件 | 現版 | 目標 | 阻擋因素 | 建議 |
|------|------|------|----------|------|
| **printpdf** | 0.8.x | 0.9 | `PdfDocument::new` API 變更 | 待有 PDF 功能需求再遷移，逐一更新呼叫處 |
| **utoipa** | 4.x | 5 | 需 axum 0.8 | 與 axum 升級一併規劃 |
| **axum-extra** | 0.10.x | 0.12 | 需 axum 0.8 | 同 axum 生態升級 |
| **tailwind-merge** | 2.x | 3 | 需 Tailwind CSS v4 | 專案大改，建議單獨排程 |

---

## 二、相依關係

```text
axum 0.8 升級 → utoipa 5、axum-extra 0.12 可跟進
Tailwind v4 升級 → tailwind-merge 3 可跟進
printpdf 0.9 → 獨立，無前置
```

---

## 三、建議

1. **printpdf**：若有 PDF 相關 bug 或需求再評估，優先處理。
2. **axum 生態**：等 axum 0.8 穩定後，再規劃 utoipa 5、axum-extra 0.12。
3. **tailwind-merge**：與 Tailwind v4 遷移一起評估，影響面大，需專門排程。
4. **驗證**：升級後執行 `scripts/verify-deps.sh` / `verify-deps.ps1`，並跑完整 CI。
