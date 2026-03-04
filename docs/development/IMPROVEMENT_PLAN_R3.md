# iPig 系統改善計畫 — 第三輪（P0–P2）

> **建立日期：** 2026-02-28  
> **範圍：** 安全性（P0）、效能與可靠性（P1）、品質與維運（P2）  
> **前置條件：** 第一輪 14 項 + 第二輪 15 項改善已完成  
> **預計工時：** 20 項任務，約 4–5 個工作天

---

## 目錄

| 優先級 | 項目數 | 重點 |
|--------|--------|------|
| 🔴 P0 安全性 | 4 項 | SQL 注入修正、IDOR 修補、.expect() 清理、容器非 root |
| 🟡 P1 效能與可靠性 | 6 項 | 搜尋 debounce、staleTime 調優、大型元件拆分、DashMap、DB Pool 指標、Skeleton 統一 |
| 🔵 P2 品質與維運 | 10 項 | any 型別消除、審計日誌、魔術數字、Error Boundary、SSL 範本、備份驗證、日誌聚合、環境驗證、無障礙、API 一致性 |

---

## 🔴 P0 — 安全性

### P0-R3-1：SQL 動態拼接修正

**現況：** 4 處使用字串拼接建構 SQL，未使用 `QueryBuilder`：
- `services/treatment_drug.rs:32` — `${0}` 格式拼接參數索引
- `services/report.rs` — 多處 `format!` 拼接 SQL
- `services/warehouse.rs:89-125` — 動態 WHERE 拼接
- `services/document/crud.rs:268-287` — 動態 SQL

**風險：** 參數索引不一致可能導致 SQL 注入。

**實作步驟：**

1. `treatment_drug.rs` — 改用 `sqlx::QueryBuilder`：
   ```rust
   // 前（危險）
   sql.push_str(&format!(" AND (name ILIKE ${0} OR display_name ILIKE ${0})", param_idx));

   // 後（安全）
   let mut qb = sqlx::QueryBuilder::new("SELECT ... FROM treatment_drugs WHERE 1=1");
   if let Some(keyword) = &query.keyword {
       let pattern = format!("%{}%", keyword);
       qb.push(" AND (name ILIKE ");
       qb.push_bind(pattern.clone());
       qb.push(" OR display_name ILIKE ");
       qb.push_bind(pattern);
       qb.push(")");
   }
   ```

2. `report.rs` — 將大型報表查詢逐步遷移至 `QueryBuilder`
3. `warehouse.rs` — 統一使用 `QueryBuilder` 替代手動參數索引
4. `document/crud.rs` — 同上

**影響檔案：**
- `backend/src/services/treatment_drug.rs`
- `backend/src/services/report.rs`
- `backend/src/services/warehouse.rs`
- `backend/src/services/document/crud.rs`

**驗證：** `cargo check` + 手動測試搜尋功能

---

### P0-R3-2：IDOR 漏洞修補

**現況：** 5+ 處端點存在 IDOR（Insecure Direct Object Reference）風險：
- `handlers/hr/leave.rs` — `get_leave` 先查 DB 再檢查權限
- `handlers/hr/overtime.rs` — 同上
- `handlers/amendment.rs:93` — `get_amendment` 先查後驗
- `handlers/document.rs` — 缺少權限檢查
- `handlers/user.rs:127` — `get_user` 未檢查所有權

**風險：** 攻擊者可猜測 UUID 存取不屬於自己的資源。

**實作步驟：**

1. 建立通用 ownership check pattern：
   ```rust
   fn check_resource_access(
       current_user: &CurrentUser,
       resource_owner_id: Uuid,
       admin_permission: &str,
   ) -> Result<()> {
       if current_user.user_id == resource_owner_id || current_user.has_permission(admin_permission) {
           Ok(())
       } else {
           Err(AppError::Forbidden("無權存取此資源".into()))
       }
   }
   ```

2. HR handlers — 先檢查權限，再查詢（或查詢時加入 `user_id` 過濾）：
   ```rust
   // leave handler
   let leave = LeaveService::get(&state.db, id).await?;
   if leave.user_id != current_user.user_id && !current_user.has_permission("hr.leave.approve") {
       return Err(AppError::Forbidden("..."));
   }
   ```

3. `get_amendment` — 加入計畫書所有權或審查者身份檢查
4. `get_user` — 非 admin 僅能查詢自己的資料
5. `document` handlers — 加入文件所有權檢查

**影響檔案：**
- `backend/src/handlers/hr/leave.rs`
- `backend/src/handlers/hr/overtime.rs`
- `backend/src/handlers/amendment.rs`
- `backend/src/handlers/document.rs`
- `backend/src/handlers/user.rs`

**驗證：** 以非 admin 帳號嘗試存取他人資源，應返回 403

---

### P0-R3-3：剩餘 .expect() 清理

**現況：** handlers/ 有 ~14 處、services/ 有 ~36 處 `.expect()`，分布在：

| 檔案 | 數量 | 類型 |
|------|------|------|
| `handlers/signature.rs` | 10 | 驗證後 unwrap |
| `services/auth.rs` | 6 | token/hash 操作 |
| `services/scheduler.rs` | 5 | 日期操作 |
| `services/email/*.rs` | 10 | SMTP host |
| `services/google_calendar.rs` | 4 | 時間解析 |
| `services/sku.rs` | 4 | 正則匹配 |
| `services/hr/*.rs` | 5 | 時區/日期 |
| `handlers/user.rs` | 2 | hash |
| `services/partner.rs` | 2 | 正則 |
| `handlers/animal/import_export.rs` | 2 | 解析 |

**風險：** 任何 `.expect()` 在生產環境觸發都會導致整個 request panic。

**實作步驟：**

1. **handlers/signature.rs（10 處）** — 改為 `map_err(AppError::Internal)?`
2. **services/auth.rs（6 處）** — token 生成/hash 失敗改為 `?`
3. **services/email/*.rs（10 處）** — SMTP host 已處理 None，但 `expect()` 改為 `ok_or()?`
4. **services/scheduler.rs（5 處）** — 日期操作 `.expect()` 改為 `.ok_or_else(|| ...)?`
5. **其餘檔案** — 逐一替換

**替換模式：**
```rust
// 日期類
.and_hms_opt(0, 0, 0).expect("valid time")
→ .and_hms_opt(0, 0, 0).ok_or_else(|| AppError::Internal("invalid time".into()))?

// 正則類
Regex::new(pattern).expect("valid regex")
→ 提升為 static OnceLock<Regex> 或 lazy_static

// Hash/Token 類
hash(...).expect("hash failed")
→ hash(...).map_err(|e| AppError::Internal(format!("hash error: {}", e)))?
```

**影響檔案：** 12+ 個 Rust 原始碼檔
**驗證：** `cargo check` + `cargo clippy`

---

### P0-R3-4：前端容器非 root 運行

**現況：** `frontend/Dockerfile` 的 Nginx 以 root 執行。

**風險：** 容器逃逸時攻擊者可取得 root 權限。

**實作步驟：**

1. 修改 `frontend/Dockerfile` 最終階段：
   ```dockerfile
   FROM georgjung/nginx-brotli:1.29.5-alpine AS production

   # 建立非 root 使用者
   RUN chown -R nginx:nginx /var/cache/nginx /var/log/nginx /etc/nginx/conf.d && \
       touch /var/run/nginx.pid && \
       chown nginx:nginx /var/run/nginx.pid

   COPY --from=build /app/dist /usr/share/nginx/html
   COPY nginx.conf /etc/nginx/conf.d/default.conf

   USER nginx
   EXPOSE 8080
   ```

2. 修改 `nginx.conf` 監聽 8080（非特權端口）：
   ```nginx
   server {
       listen 8080;
       # ...
   }
   ```

3. 更新 `docker-compose.yml` 端口映射（若有變更）

**影響檔案：**
- `frontend/Dockerfile`
- `frontend/nginx.conf`
- `docker-compose.yml`（若需更新端口映射）

**驗證：** `docker compose build web && docker compose up web` → 確認正常運行

---

## 🟡 P1 — 效能與可靠性

### P1-R3-5：搜尋 debounce 補齊

**現況：** 4 個列表頁的搜尋輸入直接觸發 API，每次按鍵都發送請求：
- `AnimalsPage.tsx` — search 直接進 queryKey
- `PartnersPage.tsx` — 同上
- `WarehousesPage.tsx` — 同上
- `ProtocolsPage.tsx` — 同上

**實作步驟：**

1. 建立 `useDebounce` hook（若尚未存在）：
   ```typescript
   // hooks/useDebounce.ts
   export function useDebounce<T>(value: T, delay = 300): T {
     const [debounced, setDebounced] = useState(value)
     useEffect(() => {
       const timer = setTimeout(() => setDebounced(value), delay)
       return () => clearTimeout(timer)
     }, [value, delay])
     return debounced
   }
   ```

2. 在 4 個頁面中：
   ```typescript
   const [search, setSearch] = useState('')
   const debouncedSearch = useDebounce(search, 400)
   // queryKey 改用 debouncedSearch
   queryKey: ['animals', statusFilter, breedFilter, debouncedSearch, page],
   ```

**影響檔案：**
- `frontend/src/hooks/useDebounce.ts`（新增）
- `frontend/src/pages/animals/AnimalsPage.tsx`
- `frontend/src/pages/master/PartnersPage.tsx`
- `frontend/src/pages/master/WarehousesPage.tsx`
- `frontend/src/pages/protocols/ProtocolsPage.tsx`

---

### P1-R3-6：staleTime 調優

**現況：** 多頁使用 `staleTime: 0`，導致每次 window focus 都重新抓取。

**實作規則：**

| 資料特性 | staleTime | 範例 |
|----------|-----------|------|
| 即時資料（動物狀態） | 30 秒 | `animal` detail |
| 列表計數 | 5 分鐘 | `animals-count` |
| 參考資料（sources, species） | 10 分鐘 | `animal-sources`, `species` |
| 設定/偏好 | 30 分鐘 | `system-settings`, `user-preferences` |

**影響檔案：** 10+ 個頁面/元件的 `useQuery` 設定

---

### P1-R3-7：大型元件拆分

**現況：**

| 檔案 | 行數 | 建議拆分 |
|------|------|---------|
| `AnimalsPage.tsx` | 1886 | → AnimalListTable, AnimalPenView, AnimalFilters, AnimalAddDialog |
| `UsersPage.tsx` | 977 | → UserListTable, UserCreateDialog, UserEditDialog |
| `ProtocolContentView.tsx` | 843 | → 按 Section 拆分（Basic, Items, Surgery, Personnel） |
| `BloodTestTab.tsx` | 810 | → BloodTestList, BloodTestFormDialog, BloodTestDetailDialog |

**實作順序：** AnimalsPage（最大且最常使用）→ UsersPage → ProtocolContentView → BloodTestTab

---

### P1-R3-8：Rate Limiter 改用 DashMap

**現況：** `rate_limiter.rs` 使用 `Arc<Mutex<HashMap>>>`，每次請求都需獲取 Mutex 鎖。

**實作步驟：**

1. `Cargo.toml` 新增 `dashmap = "6"`
2. 替換 `RateLimiterState`：
   ```rust
   pub struct RateLimiterState {
       records: DashMap<String, RequestRecord>,
       config: RateLimiterConfig,
   }
   ```
3. `check_rate` 改為無鎖操作：
   ```rust
   fn check_rate(&self, ip: &str) -> (bool, u32) {
       let mut entry = self.records.entry(ip.to_string()).or_insert_with(|| RequestRecord { timestamps: Vec::new() });
       let now = Instant::now();
       entry.timestamps.retain(|t| now.duration_since(*t) < self.config.window);
       // ...
   }
   ```

**影響檔案：**
- `backend/Cargo.toml`
- `backend/src/middleware/rate_limiter.rs`

---

### P1-R3-9：DB 連線池 Prometheus 指標

**現況：** 健康檢查回傳 pool 狀態，但 `/metrics` 未包含 pool 指標。

**實作步驟：**

在 `handlers/metrics.rs` 的 metrics 收集中新增：
```rust
// 在 metrics handler 中加入
let pool_size = state.db.size();
let pool_idle = state.db.num_idle() as u32;
output.push_str(&format!("db_pool_size {}\n", pool_size));
output.push_str(&format!("db_pool_idle {}\n", pool_idle));
output.push_str(&format!("db_pool_active {}\n", pool_size - pool_idle));
```

**影響檔案：** `backend/src/handlers/metrics.rs`

---

### P1-R3-10：Skeleton Loading 統一

**現況：** 列表頁（Animals, Users, Warehouses, Partners）使用 `<Loader2>` spinner。

**實作步驟：**

1. 建立 `TableSkeleton` 元件：
   ```tsx
   export function TableSkeleton({ rows = 5, cols = 6 }: { rows?: number; cols?: number }) {
     return (
       <Table>
         <TableBody>
           {Array.from({ length: rows }).map((_, i) => (
             <TableRow key={i}>
               {Array.from({ length: cols }).map((_, j) => (
                 <TableCell key={j}><Skeleton className="h-4 w-full" /></TableCell>
               ))}
             </TableRow>
           ))}
         </TableBody>
       </Table>
     )
   }
   ```

2. 替換 4 個列表頁的 loading 狀態

**影響檔案：**
- `frontend/src/components/ui/table-skeleton.tsx`（新增）
- `AnimalsPage.tsx`, `UsersPage.tsx`, `WarehousesPage.tsx`, `PartnersPage.tsx`

---

## 🔵 P2 — 品質與維運

### P2-R3-11：Protocol 相關 `any` 型別消除

**現況：** `protocol/` 元件群有 ~44 處 `: any`，集中在：
- `ProtocolContentView.tsx`（13 處）
- `ProtocolEditPage.tsx`（14 處）
- `CommentsTab.tsx`（4 處）
- 其餘 protocol 元件（13 處）

**實作：** 定義 `ProtocolWorkingContent`, `ProtocolPerson`, `ProtocolItem` 等型別，替換 `any`。

---

### P2-R3-12：缺失審計日誌補齊

**現況：** 以下敏感操作缺少審計紀錄：
- HR 請假/加班核准
- 使用者軟刪除
- 角色權限變更（細節）
- 通知路由規則變更

**實作：** 在對應 handler 中呼叫 `AuditService::log()`。

---

### P2-R3-13：硬編碼魔術數字提取

**現況：** 30+ 處硬編碼數字。

**實作：** 建立 `backend/src/constants.rs` 或在 `config.rs` 中集中定義。

---

### P2-R3-14：Error Boundary 分層

**現況：** 僅根層級有 ErrorBoundary。

**實作：** 在 `AnimalDetailPage`、`ProtocolEditPage`、`DashboardPage` 外層加入頁面級 ErrorBoundary。

---

### P2-R3-15：SSL/TLS 配置範本

**實作：** 建立 `docs/operations/SSL_SETUP.md` + `nginx-ssl.conf.example`。

---

### P2-R3-16：備份自動驗證

**實作：** 修改 `scripts/backup/pg_backup.sh`，備份後執行 `pg_restore --list` + SHA256 校驗。

---

### P2-R3-17：日誌集中聚合方案

**實作：** 建立 `docker-compose.logging.yml`，整合 Loki + Promtail。

---

### P2-R3-18：環境變數驗證腳本

**實作：** 建立 `scripts/validate-env.sh` 檢查 `.env` 必填項目，Docker entrypoint 呼叫。

---

### P2-R3-19：無障礙改善

**實作：** 搜尋框 `aria-label`、Dialog 焦點管理、快捷鍵提示。

---

### P2-R3-20：API 設計一致性

**現況：** 權限不足回傳空列表（而非 403）；硬編碼角色名稱。

**實作：** 統一回傳 403 + 錯誤訊息；角色檢查統一使用權限系統。

---

## 預估工時

| 優先級 | 項目 | 預估 |
|--------|------|------|
| P0-R3-1 | SQL 修正 | 2h |
| P0-R3-2 | IDOR 修補 | 2h |
| P0-R3-3 | .expect() 清理 | 3h |
| P0-R3-4 | 容器非 root | 1h |
| P1-R3-5 | 搜尋 debounce | 1h |
| P1-R3-6 | staleTime 調優 | 1h |
| P1-R3-7 | 大型元件拆分 | 6h |
| P1-R3-8 | DashMap | 1h |
| P1-R3-9 | DB Pool 指標 | 0.5h |
| P1-R3-10 | Skeleton 統一 | 1h |
| P2-R3-11 | any 消除 | 3h |
| P2-R3-12 | 審計日誌 | 2h |
| P2-R3-13 | 魔術數字 | 2h |
| P2-R3-14 | Error Boundary | 1h |
| P2-R3-15 | SSL 範本 | 1h |
| P2-R3-16 | 備份驗證 | 1h |
| P2-R3-17 | 日誌聚合 | 1.5h |
| P2-R3-18 | 環境驗證 | 1h |
| P2-R3-19 | 無障礙 | 1.5h |
| P2-R3-20 | API 一致性 | 2h |
| **合計** | | **~34h** |
