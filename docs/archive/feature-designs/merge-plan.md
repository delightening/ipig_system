# R9 Worktree 合併計畫

日期：2026-03-15

## 分析結論

| 項目 | 結論 |
|------|------|
| Worktree 基礎 commit | `95cd4fd`（R9 任務加入前） |
| 主分支 HEAD | `298fc30`（已含 R9-1、R9-5 完成） |
| admin tab 6 個檔案 | worktree ≡ main，**跳過** |
| TODO.md / PROGRESS.md | worktree 版本無 R9 內容，**跳過**，只更新主分支版本 |
| 新建檔案 | **無**，全部為原地重構 |
| 跨 worktree 衝突 | **無**，各任務目標檔案完全不重疊 |

## 已完成（已在主分支）

- ✅ R9-1：`backend/src/services/pdf/service.rs`（worktree: `ac222752`）
- ✅ R9-5：`backend/src/services/accounting.rs`（worktree: `ad071136`）

## 待合併 Worktree 清單

| Worktree | 任務 | 要 checkout 的目標檔案 |
|----------|------|----------------------|
| `a1ac4d54` | R9-2 | `backend/src/services/animal/import_export.rs` |
| `a94933a7` | R9-3 | `backend/src/services/product.rs` |
| `a3ff777f` | R9-6 | `frontend/src/components/protocol/ProtocolContentView.tsx` |
| `aca99f0d` | R9-7 | `frontend/src/components/product/ProductImportDialog.tsx`<br>`frontend/src/components/animal/BloodTestTab.tsx`<br>`frontend/src/pages/DashboardPage.tsx`<br>`frontend/src/components/dashboard/index.ts` |
| `aebcee9a` | R9-8 | `frontend/src/hooks/usePermissionManager.ts` |
| `a4513c05` | R9-9+10 | `frontend/src/pages/reports/AccountingReportPage.tsx`<br>`frontend/src/pages/hr/HrLeavePage.tsx` |
| `aaeff3e2` | R9-13+14 | `frontend/src/pages/documents/components/DocumentLineEditor.tsx`<br>`frontend/src/pages/documents/hooks/useDocumentForm.ts` |
| `af830c36` | R9-15 | `frontend/src/pages/admin/AuditLogsPage.tsx`<br>`frontend/src/pages/admin/TrainingRecordsPage.tsx`<br>`frontend/src/pages/animals/AnimalDetailPage.tsx`<br>`frontend/src/pages/master/ProductEditPage.tsx` |

## 執行步驟

### Step 1：後端 2 個（可並行 checkout，再統一 commit）

```bash
git checkout worktree-agent-a1ac4d54 -- backend/src/services/animal/import_export.rs
git checkout worktree-agent-a94933a7 -- backend/src/services/product.rs
git commit -m "refactor(R9-2,R9-3): import_export.rs 與 product.rs 長函數拆分"
```

### Step 2：前端 R9-6（獨立 commit）

```bash
git checkout worktree-agent-a3ff777f -- frontend/src/components/protocol/ProtocolContentView.tsx
git commit -m "refactor(R9-6): ProtocolContentView 拆分 870→子元件"
```

### Step 3：前端 R9-7（4 個相關檔案一起 commit）

```bash
git checkout worktree-agent-aca99f0d -- \
  frontend/src/components/product/ProductImportDialog.tsx \
  frontend/src/components/animal/BloodTestTab.tsx \
  frontend/src/pages/DashboardPage.tsx \
  frontend/src/components/dashboard/index.ts
git commit -m "refactor(R9-7): ProductImportDialog + BloodTestTab + DashboardPage 拆分"
```

### Step 4：前端 R9-8

```bash
git checkout worktree-agent-aebcee9a -- frontend/src/hooks/usePermissionManager.ts
git commit -m "refactor(R9-8): usePermissionManager 拆分為 3 個子 Hook"
```

### Step 5：前端 R9-9+10

```bash
git checkout worktree-agent-a4513c05 -- \
  frontend/src/pages/reports/AccountingReportPage.tsx \
  frontend/src/pages/hr/HrLeavePage.tsx
git commit -m "refactor(R9-9,R9-10): AccountingReportPage + HrLeavePage 拆分"
```

### Step 6：前端 R9-13+14

```bash
git checkout worktree-agent-aaeff3e2 -- \
  frontend/src/pages/documents/components/DocumentLineEditor.tsx \
  frontend/src/pages/documents/hooks/useDocumentForm.ts
git commit -m "refactor(R9-13,R9-14): DocumentLineEditor + useDocumentForm 拆分"
```

### Step 7：前端 R9-15

```bash
git checkout worktree-agent-af830c36 -- \
  frontend/src/pages/admin/AuditLogsPage.tsx \
  frontend/src/pages/admin/TrainingRecordsPage.tsx \
  frontend/src/pages/animals/AnimalDetailPage.tsx \
  frontend/src/pages/master/ProductEditPage.tsx
git commit -m "refactor(R9-15): 4 個中大型元件拆分"
```

### Step 8：更新 docs（TODO.md + PROGRESS.md）

標記以下 R9 任務為 `[x]`：
- R9-2、R9-3、R9-6、R9-7、R9-8、R9-9、R9-10、R9-13、R9-14、R9-15

PROGRESS.md §9 新增：
```
2026-03-15 R9 技術債清理（R9-2/3/6/7/8/9/10/13/14/15）— 後端 2 個長函數拆分 + 前端 8 個超大元件重構
```

更新 TODO.md「待辦統計」。

```bash
git commit -m "docs: R9 TODO/PROGRESS 更新 — R9-2/3/6/7/8/9/10/13/14/15 完成"
```

### Step 9：Push

```bash
git push -u origin claude/analyze-technical-debt-wDUeQ
```

## 注意事項

- 每個 `git checkout <branch> -- <file>` 不會觸發 merge，直接覆蓋工作區檔案，無衝突風險
- 若某個 worktree 仍在執行中，等待其完成後再 checkout
- 完成後可用 `git worktree remove` 清理 worktree（可選）
