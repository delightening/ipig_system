# Core Domain Model

> **注意**：此文件已整合至 [Database Schema](./04_DATABASE_SCHEMA.md)

請參考 **04_DATABASE_SCHEMA.md** 以獲取完整的：
- 實體關係圖 (ERD)
- 資料表定義與業務邏輯註解
- 列舉類型說明

---

## Cross-Entity Relationships

以下是跨實體關係的快速參考（詳細內容見 04_DATABASE_SCHEMA.md）：

### Protocol ↔ Pig
- 豬隻透過 `iacuc_no` 分配到實驗計畫
- 一個計畫可以有多隻豬
- 一隻豬只能分配到一個進行中的計畫

### User ↔ Role ↔ Permission
- 使用者可以有多個角色 (many-to-many via `user_roles`)
- 角色可以有多個權限 (many-to-many via `role_permissions`)
- 權限檢查在 handler 層級進行

### Overtime ↔ Comp Time
- 核准的加班產生補休時數
- 補休有效期限為加班日起 1 年
- 使用順序為 FIFO（先到期者先使用）

### Leave ↔ Balance
- 特休來自 `annual_leave_entitlements`（2年到期）
- 補休來自 `comp_time_balances`（1年到期）
- 請假從對應的餘額扣除

---

*Last updated: 2026-01-29*
