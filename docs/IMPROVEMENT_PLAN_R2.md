# iPig 系統改善計畫 — 第二輪（P0–P2）

> **建立日期：** 2026-02-28  
> **範圍：** 安全性（P0）、效能與可靠性（P1）、品質與維運（P2）  
> **前置條件：** 第一輪 14 項改善已完成（見 `IMPROVEMENT_PLAN.md`）  
> **預計工時：** 15 項任務，約 3–4 個工作天

---

## 目錄

| 優先級 | 項目數 | 重點 |
|--------|--------|------|
| 🔴 P0 安全性 | 2 項 | XSS 防護、Rate Limiting 精細化 |
| 🟡 P1 效能與可靠性 | 6 項 | 大型依賴 lazy load、動物列表分頁、健康檢查深度、Alertmanager、外部服務重試、Query Key 統一 |
| 🔵 P2 品質與維運 | 7 項 | 表單驗證統一、i18n 補齊、Zustand selector、DB 維護、Dependabot、零停機遷移文件、架構圖 |

---

## 🔴 P0 — 安全性

### P0-R2-1：前端 XSS 防護 — `dangerouslySetInnerHTML` 清理

**現況：** 2 處直接渲染未清理的 SVG 內容：
- `frontend/src/components/animal/SacrificeFormDialog.tsx:388` — 渲染手寫簽名 SVG
- `frontend/src/components/ui/handwritten-signature-pad.tsx:162` — 渲染簽名預覽 SVG

**風險：** 若 SVG 內容中注入 `<script>` 或 `onload` 事件，可執行任意 JavaScript。

**實作步驟：**

1. 安裝 DOMPurify：
   ```bash
   npm install dompurify
   npm install -D @types/dompurify
   ```

2. 建立共用 sanitize helper（`frontend/src/lib/sanitize.ts`）：
   ```typescript
   import DOMPurify from 'dompurify'

   const SVG_CONFIG: DOMPurify.Config = {
     USE_PROFILES: { svg: true, svgFilters: true },
     ADD_TAGS: ['svg', 'path', 'line', 'circle', 'rect', 'polyline', 'g'],
     ADD_ATTR: ['d', 'fill', 'stroke', 'stroke-width', 'viewBox', 'xmlns', 'transform'],
     FORBID_TAGS: ['script', 'iframe', 'object', 'embed'],
     FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onmouseover'],
   }

   export function sanitizeSvg(svg: string): string {
     return DOMPurify.sanitize(svg, SVG_CONFIG)
   }
   ```

3. 替換 2 處：
   ```typescript
   // 前
   dangerouslySetInnerHTML={{ __html: sig.handwriting_svg }}
   // 後
   dangerouslySetInnerHTML={{ __html: sanitizeSvg(sig.handwriting_svg) }}
   ```

**影響檔案：**
- `frontend/src/lib/sanitize.ts`（新增）
- `frontend/src/components/animal/SacrificeFormDialog.tsx`
- `frontend/src/components/ui/handwritten-signature-pad.tsx`

**驗證方式：**
```typescript
// 測試：惡意 SVG 應被過濾
sanitizeSvg('<svg><script>alert("xss")</script><path d="M0,0"/></svg>')
// → '<svg><path d="M0,0"></path></svg>'
```

---

### P0-R2-2：Rate Limiting 寫入端點分級

**現況：** 僅兩層限制：
- 認證端點：100/min（`auth_rate_limit_middleware`）
- 一般 API：600/min（`api_rate_limit_middleware`）

**風險：** 攻擊者可在 600/min 配額內大量執行寫入操作（建立動物、提交計畫書等）。

**實作步驟：**

1. 在 `backend/src/middleware/rate_limiter.rs` 新增寫入端點限制器：
   ```rust
   /// 寫入端點速率限制（POST/PUT/DELETE：每分鐘 120 次）
   pub async fn write_rate_limit_middleware(
       State(state): State<AppState>,
       ConnectInfo(addr): ConnectInfo<SocketAddr>,
       request: Request<Body>,
       next: Next,
   ) -> Result<Response<Body>, StatusCode> {
       let method = request.method().clone();
       if matches!(method, Method::GET | Method::HEAD | Method::OPTIONS) {
           return Ok(next.run(request).await);
       }

       static WRITE_LIMITER: OnceLock<RateLimiterState> = OnceLock::new();
       let limiter = WRITE_LIMITER.get_or_init(|| {
           RateLimiterState::new(RateLimiterConfig {
               max_requests: 120,
               window: Duration::from_secs(60),
           })
       });
       // ... 同 api_rate_limit_middleware 的 check 邏輯
   }
   ```

2. 新增檔案上傳專用限制器：
   ```rust
   /// 檔案上傳速率限制（每分鐘 30 次）
   pub async fn upload_rate_limit_middleware(...) {
       static UPLOAD_LIMITER: OnceLock<RateLimiterState> = OnceLock::new();
       let limiter = UPLOAD_LIMITER.get_or_init(|| {
           RateLimiterState::new(RateLimiterConfig {
               max_requests: 30,
               window: Duration::from_secs(60),
           })
       });
       // ...
   }
   ```

3. 在 `backend/src/routes.rs` 中將寫入端點套用 `write_rate_limit_middleware`，上傳路由套用 `upload_rate_limit_middleware`。

**影響檔案：**
- `backend/src/middleware/rate_limiter.rs`
- `backend/src/routes.rs`

---

## 🟡 P1 — 效能與可靠性

### P1-R2-3：大型依賴動態導入

**現況：** `jsPDF`（~180KB）和 `html2canvas`（~130KB）在 `CommentsTab.tsx` 靜態導入，即使使用者不匯出 PDF 也會載入。`react-grid-layout`（~50KB）在 `DashboardPage.tsx` 靜態導入。

**實作步驟：**

1. **CommentsTab.tsx** — PDF 匯出改為動態導入：
   ```typescript
   // 移除頂部的靜態導入
   // import jsPDF from 'jspdf'
   // import html2canvas from 'html2canvas'

   const handleExportPDF = async () => {
     setExporting(true)
     try {
       const [{ default: jsPDF }, { default: html2canvas }] = await Promise.all([
         import('jspdf'),
         import('html2canvas'),
       ])
       // ... 原有 PDF 生成邏輯
     } finally {
       setExporting(false)
     }
   }
   ```

2. **DashboardPage.tsx** — react-grid-layout lazy load：
   ```typescript
   // 移除靜態導入
   // import { Responsive, WidthProvider } from 'react-grid-layout/legacy'

   const GridLayout = React.lazy(async () => {
     const { Responsive, WidthProvider } = await import('react-grid-layout/legacy')
     await import('react-grid-layout/css/styles.css')
     const ResponsiveGrid = WidthProvider(Responsive)
     return { default: ResponsiveGrid }
   })
   ```

**預估 bundle 減少：** ~360KB（初始載入減少約 15-20%）

**影響檔案：**
- `frontend/src/components/protocol/CommentsTab.tsx`
- `frontend/src/pages/DashboardPage.tsx`

---

### P1-R2-4：動物列表 API 分頁

**現況：** `GET /api/animals` 回傳所有動物（`Vec<AnimalListItem>`），無 LIMIT/OFFSET。資料量增長後將嚴重影響效能。

**實作步驟：**

1. **後端 — 擴展 `AnimalQuery`**（`backend/src/models/animal.rs` 或 `requests.rs`）：
   ```rust
   #[derive(Debug, Deserialize)]
   pub struct AnimalQuery {
       // 既有過濾欄位...
       pub status: Option<String>,
       pub breed: Option<String>,
       // ...
       // 新增分頁
       pub page: Option<i64>,
       pub per_page: Option<i64>,
   }
   ```

2. **後端 — `AnimalService::list` 加入分頁**：
   ```rust
   pub async fn list(pool: &PgPool, query: &AnimalQuery) -> Result<PaginatedResponse<AnimalListItem>> {
       let page = query.page.unwrap_or(1).max(1);
       let per_page = query.per_page.unwrap_or(50).clamp(1, 200);
       let offset = (page - 1) * per_page;

       // 計算總數
       let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM animals WHERE is_deleted = false ...")
           .fetch_one(pool).await?;

       // 查詢分頁資料
       let items = sqlx::query_as::<_, AnimalListItem>("... LIMIT $N OFFSET $M")
           .fetch_all(pool).await?;

       Ok(PaginatedResponse { data: items, total, page, per_page })
   }
   ```

3. **後端 — 定義 `PaginatedResponse<T>`**（若尚未存在）：
   ```rust
   #[derive(Debug, Serialize)]
   pub struct PaginatedResponse<T: Serialize> {
       pub data: Vec<T>,
       pub total: i64,
       pub page: i64,
       pub per_page: i64,
   }
   ```

4. **前端** — `AnimalsPage.tsx` 更新：
   - 使用 TanStack Query 的 `keepPreviousData` 避免分頁閃爍
   - 加入分頁 UI（頁碼、每頁筆數選擇）

**影響檔案：**
- `backend/src/models/animal.rs`（或 `requests.rs`）
- `backend/src/services/animal.rs`
- `backend/src/handlers/animal/animal_core.rs`
- `frontend/src/pages/animals/AnimalsPage.tsx`

---

### P1-R2-5：健康檢查深度擴充

**現況：** `GET /api/health` 僅檢查 DB（`SELECT 1`）。

**實作步驟：**

擴充 `backend/src/handlers/health.rs`：

```rust
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize)]
pub struct HealthChecks {
    pub database: ComponentCheck,
    pub disk_space: ComponentCheck,
    pub connection_pool: ComponentCheck,
}

#[derive(Debug, Serialize)]
pub struct ComponentCheck {
    pub status: &'static str,
    pub latency_ms: Option<u64>,
    pub detail: Option<String>,
}
```

新增檢查：
- **磁碟空間**：檢查 `UPLOAD_DIR` 所在分區使用率
- **連線池**：`state.db.size()` vs `state.db.max_connections()` 使用率
- 綜合 status：任一檢查失敗則整體為 `degraded`，DB 失敗為 `unhealthy`

**影響檔案：**
- `backend/src/handlers/health.rs`

---

### P1-R2-6：Prometheus Alertmanager 告警規則

**現況：** Prometheus 已部署但無告警規則。

**實作步驟：**

1. 建立 `deploy/prometheus-alerts.yml`：
   ```yaml
   groups:
     - name: ipig_alerts
       rules:
         - alert: HighErrorRate
           expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.1
           for: 2m
           labels:
             severity: critical
           annotations:
             summary: "5xx 錯誤率超過 10%"

         - alert: HighLatencyP95
           expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 2
           for: 5m
           labels:
             severity: warning
           annotations:
             summary: "API P95 延遲超過 2 秒"

         - alert: DatabaseConnectionPoolHigh
           expr: database_connections_active / database_connections_max > 0.85
           for: 3m
           labels:
             severity: warning
           annotations:
             summary: "資料庫連線池使用率超過 85%"

         - alert: DiskSpaceLow
           expr: node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"} < 0.15
           for: 5m
           labels:
             severity: warning
           annotations:
             summary: "磁碟剩餘空間低於 15%"
   ```

2. 更新 `deploy/prometheus.yml` 加入：
   ```yaml
   rule_files:
     - /etc/prometheus/alerts.yml
   ```

3. 更新 `docker-compose.monitoring.yml`，掛載告警規則：
   ```yaml
   prometheus:
     volumes:
       - ./deploy/prometheus.yml:/etc/prometheus/prometheus.yml:ro
       - ./deploy/prometheus-alerts.yml:/etc/prometheus/alerts.yml:ro
   ```

**影響檔案：**
- `deploy/prometheus-alerts.yml`（新增）
- `deploy/prometheus.yml`
- `docker-compose.monitoring.yml`

---

### P1-R2-7：外部服務重試機制

**現況：**
- `google_calendar.rs` — HTTP 呼叫無重試，失敗直接回傳錯誤
- `email/mod.rs` — SMTP 發送無重試

**實作步驟：**

1. 建立通用重試 helper（`backend/src/utils/retry.rs`）：
   ```rust
   use std::time::Duration;
   use tokio::time::sleep;

   pub async fn with_retry<F, Fut, T, E>(
       max_attempts: u32,
       initial_delay: Duration,
       operation: F,
   ) -> std::result::Result<T, E>
   where
       F: Fn() -> Fut,
       Fut: std::future::Future<Output = std::result::Result<T, E>>,
       E: std::fmt::Display,
   {
       let mut last_err = None;
       for attempt in 0..max_attempts {
           match operation().await {
               Ok(val) => return Ok(val),
               Err(e) => {
                   let delay = initial_delay * 2u32.pow(attempt);
                   tracing::warn!(
                       "操作失敗 (嘗試 {}/{}): {}，{}ms 後重試",
                       attempt + 1, max_attempts, e, delay.as_millis()
                   );
                   last_err = Some(e);
                   if attempt + 1 < max_attempts {
                       sleep(delay).await;
                   }
               }
           }
       }
       Err(last_err.unwrap())
   }
   ```

2. 在 `google_calendar.rs` 的 `fetch_events`、`create_event` 等方法中包裝：
   ```rust
   let response = with_retry(3, Duration::from_millis(500), || async {
       client.get(&url).bearer_auth(&token).send().await
           .map_err(|e| AppError::Internal(format!("Google Calendar API 失敗: {}", e)))
   }).await?;
   ```

3. 在 `email/mod.rs` 的 `send_email_smtp` 中包裝 `mailer.send()` 呼叫。

**影響檔案：**
- `backend/src/utils/retry.rs`（新增）
- `backend/src/utils/mod.rs`（新增或修改）
- `backend/src/services/google_calendar.rs`
- `backend/src/services/email/mod.rs`

---

### P1-R2-8：TanStack Query Key Factory

**現況：** Query Key 命名不一致（`['animals']` vs `['animals-by-pen']`），Mutation 後手動 invalidate 容易遺漏。

**實作步驟：**

建立 `frontend/src/lib/queryKeys.ts`：

```typescript
export const queryKeys = {
  animals: {
    all: ['animals'] as const,
    lists: () => [...queryKeys.animals.all, 'list'] as const,
    list: (filters?: Record<string, unknown>) => [...queryKeys.animals.lists(), filters] as const,
    details: () => [...queryKeys.animals.all, 'detail'] as const,
    detail: (id: string) => [...queryKeys.animals.details(), id] as const,
    byPen: (pen: string) => [...queryKeys.animals.all, 'by-pen', pen] as const,
  },
  protocols: {
    all: ['protocols'] as const,
    lists: () => [...queryKeys.protocols.all, 'list'] as const,
    list: (filters?: Record<string, unknown>) => [...queryKeys.protocols.lists(), filters] as const,
    detail: (id: string) => [...queryKeys.protocols.all, 'detail', id] as const,
    attachments: (id: string) => [...queryKeys.protocols.all, 'attachments', id] as const,
    versions: (id: string) => [...queryKeys.protocols.all, 'versions', id] as const,
  },
  users: {
    all: ['users'] as const,
    list: (filters?: Record<string, unknown>) => [...queryKeys.users.all, 'list', filters] as const,
    detail: (id: string) => [...queryKeys.users.all, 'detail', id] as const,
  },
  notifications: {
    all: ['notifications'] as const,
    unreadCount: () => [...queryKeys.notifications.all, 'unread-count'] as const,
  },
  warehouses: { all: ['warehouses'] as const },
  partners: { all: ['partners'] as const },
  roles: { all: ['roles'] as const },
  dashboard: { all: ['dashboard'] as const },
} as const
```

逐步替換頁面中的硬編碼 Query Key。**不需一次替換所有**，可在修改到相關頁面時逐步遷移。

**影響檔案：**
- `frontend/src/lib/queryKeys.ts`（新增）
- 各頁面逐步遷移（不阻擋交付）

---

## 🔵 P2 — 品質與維運

### P2-R2-9：表單驗證模式統一

**現況：** 3 個頁面使用原生表單 + 手動驗證，其餘使用 `react-hook-form` + `zod`：
- `PartnersPage.tsx:194-243` — 手動驗證統編、電話格式
- `AnimalsPage.tsx:268-344` — 手動驗證必填欄位
- `WarehousesPage.tsx` — 手動驗證

**實作步驟：**

1. 為每個頁面建立 zod schema：
   ```typescript
   // partners
   const partnerSchema = z.object({
     name: z.string().min(1, '名稱為必填'),
     tax_id: z.string().regex(/^\d{8}$/, '統編必須為 8 碼數字').optional().or(z.literal('')),
     phone: z.string().optional(),
     address: z.string().optional(),
   })
   ```

2. 將手動表單改為 `useForm` + `zodResolver`。

3. 使用 `FormField` 元件統一顯示錯誤訊息。

**影響檔案：**
- `frontend/src/pages/master/PartnersPage.tsx`
- `frontend/src/pages/animals/AnimalsPage.tsx`
- `frontend/src/pages/master/WarehousesPage.tsx`

---

### P2-R2-10：i18n 硬編碼字串補齊

**現況：** 部分驗證訊息、UI 標籤、陣列資料仍硬編碼中文。

**需補齊位置：**
| 檔案 | 內容 |
|------|------|
| `PartnersPage.tsx:199-223` | 錯誤訊息（格式錯誤、統編必須為…） |
| `AnimalsPage.tsx:272-307` | 驗證訊息（欄位為必填、耳號為必填…） |
| `AnimalsPage.tsx:89-92` | `penBuildings` 靜態陣列 |
| `WarehousesPage.tsx:56-65` | 驗證訊息 |

**實作步驟：** 提取為 i18n key 並加入 `zh-TW.json` 和 `en.json`。

**影響檔案：**
- `frontend/src/pages/master/PartnersPage.tsx`
- `frontend/src/pages/animals/AnimalsPage.tsx`
- `frontend/src/pages/master/WarehousesPage.tsx`
- `frontend/public/locales/zh-TW.json`
- `frontend/public/locales/en.json`

---

### P2-R2-11：Zustand Store Selector 優化

**現況：** 多處使用解構訂閱 `const { user, hasRole, ... } = useAuthStore()`，任何 store 欄位變更都觸發重渲染。

**實作步驟：**

搜尋所有 `useAuthStore()` 並改為 selector：
```typescript
// 前
const { user, hasRole, isAuthenticated } = useAuthStore()

// 後
const user = useAuthStore(s => s.user)
const hasRole = useAuthStore(s => s.hasRole)
const isAuthenticated = useAuthStore(s => s.isAuthenticated)
```

重點檔案（高頻渲染元件）：
- `frontend/src/layouts/MainLayout.tsx`
- `frontend/src/components/layout/Sidebar.tsx`
- `frontend/src/components/layout/NotificationDropdown.tsx`
- `frontend/src/App.tsx`

**影響檔案：** ~15-20 個使用 `useAuthStore` 的檔案

---

### P2-R2-12：資料庫維護自動化

**現況：** 無自動 VACUUM/ANALYZE，無 `pg_stat_statements` 慢查詢監控。

**實作步驟：**

1. 建立 `scripts/db/init-extensions.sql`：
   ```sql
   CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
   ```

2. 在 `docker-compose.yml` 掛載：
   ```yaml
   db:
     volumes:
       - ./scripts/db/init-extensions.sql:/docker-entrypoint-initdb.d/10-extensions.sql:ro
   ```

3. 建立 `scripts/db/maintenance.sh`：
   ```bash
   #!/bin/bash
   docker compose exec -T db psql -U postgres ipig_db -c "VACUUM ANALYZE;"
   docker compose exec -T db psql -U postgres ipig_db -c \
     "SELECT query, calls, mean_exec_time, total_exec_time
      FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"
   ```

4. 加入 `db-backup` 容器的 cron 排程（或建立獨立排程容器）。

**影響檔案：**
- `scripts/db/init-extensions.sql`（新增）
- `scripts/db/maintenance.sh`（新增）
- `docker-compose.yml`（db volumes）

---

### P2-R2-13：啟用 GitHub Dependabot

**現況：** `.github/dependabot.yml` 不存在。

**實作步驟：**

建立 `.github/dependabot.yml`：
```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/backend"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 5
    labels:
      - "dependencies"
      - "rust"

  - package-ecosystem: "npm"
    directory: "/frontend"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 5
    labels:
      - "dependencies"
      - "frontend"

  - package-ecosystem: "docker"
    directory: "/backend"
    schedule:
      interval: "monthly"
    labels:
      - "dependencies"
      - "docker"

  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "monthly"
    labels:
      - "dependencies"
      - "ci"
```

**影響檔案：**
- `.github/dependabot.yml`（新增）

---

### P2-R2-14：零停機遷移策略文件

**現況：** Migration 在啟動時同步執行，無策略文件指導如何避免破壞性遷移。

**實作步驟：** 建立 `docs/ZERO_DOWNTIME_MIGRATIONS.md`：

```markdown
# 零停機資料庫遷移規範

## 原則
1. 新增欄位永遠使用 `NULL` 或 `DEFAULT`，避免 `NOT NULL` 無預設值
2. 刪除欄位分兩步：
   - v1: 停止在程式碼中使用該欄位
   - v2: 執行 `ALTER TABLE ... DROP COLUMN`
3. 重新命名欄位分三步：
   - v1: 新增新欄位 + 觸發器同步
   - v2: 程式碼改用新欄位
   - v3: 移除舊欄位 + 觸發器
4. 索引使用 `CREATE INDEX CONCURRENTLY`（已遵循）
5. 大表 ALTER TABLE 使用 `LOCK_TIMEOUT` 避免長時間鎖定

## 檢查清單
- [ ] Migration 在空 DB 與正式 DB 皆可執行
- [ ] 無 `DROP COLUMN` 與 `RENAME COLUMN`（或已分步）
- [ ] 無 `ALTER COLUMN ... SET NOT NULL`（除非欄位已全部填值）
- [ ] 索引使用 `CONCURRENTLY`
```

**影響檔案：**
- `docs/ZERO_DOWNTIME_MIGRATIONS.md`（新增）

---

### P2-R2-15：系統架構圖

**現況：** 缺少視覺化架構文件。

**實作步驟：** 建立 `docs/ARCHITECTURE.md`，使用 Mermaid 繪製：

1. **部署架構圖**（Docker 服務關係）
2. **資料流圖**（Client → WAF → Nginx → API → DB）
3. **模組關係圖**（後端 handlers → services → models）
4. **認證流程圖**（Login → JWT → Refresh → 2FA）

**影響檔案：**
- `docs/ARCHITECTURE.md`（新增）

---

## 📊 執行順序建議

| 順序 | 項目 | 依賴 | 預估工時 |
|------|------|------|----------|
| 1 | P0-R2-1 XSS 防護 | 無 | 30min |
| 2 | P0-R2-2 Rate Limiting | 無 | 1.5h |
| 3 | P1-R2-3 Lazy Import | 無 | 1h |
| 4 | P1-R2-4 動物分頁 | 無 | 2h |
| 5 | P1-R2-5 健康檢查 | 無 | 1h |
| 6 | P1-R2-6 Alertmanager | 無 | 1h |
| 7 | P1-R2-7 外部重試 | 無 | 1.5h |
| 8 | P1-R2-8 Query Key | 無 | 1h |
| 9 | P2-R2-9 表單統一 | 無 | 2h |
| 10 | P2-R2-10 i18n 補齊 | P2-R2-9 | 1h |
| 11 | P2-R2-11 Store selector | 無 | 1h |
| 12 | P2-R2-12 DB 維護 | 無 | 30min |
| 13 | P2-R2-13 Dependabot | 無 | 15min |
| 14 | P2-R2-14 遷移文件 | 無 | 30min |
| 15 | P2-R2-15 架構圖 | 無 | 1h |
| | **合計** | | **~16h** |

---

## ✅ 驗收標準

- [ ] `dangerouslySetInnerHTML` 所有使用處經過 DOMPurify 清理
- [ ] 寫入端點有獨立 rate limit（120/min）
- [ ] `jsPDF` + `html2canvas` 不在初始 bundle 中（可透過 `npx vite-bundle-visualizer` 驗證）
- [ ] `GET /api/animals?page=1&per_page=20` 回傳分頁結構
- [ ] `GET /api/health` 回傳 DB + 磁碟 + 連線池三項檢查
- [ ] Prometheus 載入告警規則（`curl localhost:9090/api/v1/rules` 可見）
- [ ] Google Calendar / SMTP 失敗時有重試日誌
- [ ] `cargo check` 零錯誤、`npx tsc --noEmit` 零錯誤
- [ ] `.github/dependabot.yml` 存在
- [ ] `docs/ZERO_DOWNTIME_MIGRATIONS.md` 與 `docs/ARCHITECTURE.md` 存在

---

## 修正說明

> ⚠️ 第一輪計畫中提到的「角色/權限快取」不再列入，因為分析後確認：  
> `auth_middleware` 從 JWT Claims 直接讀取 `roles` 和 `permissions`，**不查詢資料庫**。  
> 權限在簽發 JWT 時嵌入 token，直到 token 過期前都有效。此設計已足夠高效。
