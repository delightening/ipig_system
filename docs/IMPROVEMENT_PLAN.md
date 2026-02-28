# iPig 系統改善計畫（P0–P2）

> **建立日期：** 2026-02-28  
> **範圍：** 安全性（P0）、效能（P1）、程式碼品質（P2）  
> **預計工時：** 14 項任務，約 3–4 個工作天  
> **前置條件：** 所有功能開發已完成（TODO.md 0 pending）

---

## 目錄

| 優先級 | 項目數 | 重點 |
|--------|--------|------|
| 🔴 P0 安全性 | 3 項 | Docker 網路隔離、DB 埠口、Secrets 管理 |
| 🟡 P1 效能 | 5 項 | N+1 查詢、批次 INSERT、`.expect()` 移除、前端 memoization、DB 索引 |
| 🔵 P2 品質 | 6 項 | `is_admin()` 統一、UserResponse 提取、TypeScript 嚴格化、API 錯誤統一、MainLayout 拆分、Dockerfile 快取 |

---

## 🔴 P0 — 安全性（上線前必要）

### P0-S1：Docker 自訂網路隔離

**現況：** 所有服務使用預設 bridge 網路，容器間無隔離。  
**風險：** 前端容器可直接存取資料庫，攻擊者若攻破 web 容器即可橫向移動。  

**目標架構：**
```
Client → [waf] → (frontend 網路) → [web] → (backend 網路) → [api] → (database 網路) → [db]
                                                                ↑
                                                          [db-backup]
```

**實作步驟：**

1. 在 `docker-compose.yml` 底部定義三個自訂網路：
   ```yaml
   networks:
     frontend:
       driver: bridge
     backend:
       driver: bridge
     database:
       driver: bridge
   ```

2. 為各服務指派最小權限網路：
   | 服務 | frontend | backend | database |
   |------|----------|---------|----------|
   | web / web-dev | ✅ | ✅ | ❌ |
   | api | ❌ | ✅ | ✅ |
   | db | ❌ | ❌ | ✅ |
   | db-backup | ❌ | ❌ | ✅ |
   | waf | ✅ | ❌ | ❌ |

3. 在 `docker-compose.prod.yml` 中繼承相同網路定義。

4. 在 `docker-compose.waf.yml` 中將 `waf` 加入 `frontend` 網路。

**影響檔案：**
- `docker-compose.yml`
- `docker-compose.prod.yml`
- `docker-compose.waf.yml`

**驗證方式：**
```bash
# 確認 web 容器無法存取 db
docker exec ipig-web ping -c 1 db   # 應 timeout
# 確認 api 容器可存取 db
docker exec ipig-api ping -c 1 db   # 應成功
```

---

### P0-S2：生產環境移除資料庫對外埠口

**現況：** `docker-compose.yml` 中 `db` 服務暴露 `${POSTGRES_PORT:-5433}:5432`，生產環境 `docker-compose.prod.yml` 未覆蓋此設定。  
**風險：** 外部可直接連接 PostgreSQL，繞過應用層認證。

**實作步驟：**

1. 在 `docker-compose.prod.yml` 覆蓋 `db` 服務的 `ports`，移除對外映射：
   ```yaml
   db:
     ports: []
   ```
   > 開發環境保留 5433 供 DBeaver / pgAdmin 等工具使用。

2. （備選方案）若生產環境仍需要管理存取，限制為 localhost：
   ```yaml
   db:
     ports:
       - "127.0.0.1:5433:5432"
   ```

**影響檔案：**
- `docker-compose.prod.yml`

**驗證方式：**
```bash
# 生產環境啟動後
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
# 從主機嘗試連線應失敗
psql -h localhost -p 5433 -U postgres  # 應 Connection refused
```

---

### P0-S3：機密資料遷移至 Docker Secrets

**現況：** `JWT_SECRET`、`SMTP_PASSWORD`、`POSTGRES_PASSWORD` 透過環境變數傳遞，僅 `google_service_account` 使用 Docker Secrets。  
**風險：** 環境變數可透過 `docker inspect`、`/proc/1/environ` 洩漏。

**實作步驟：**

1. 建立 secrets 檔案（生產環境）：
   ```
   secrets/
   ├── google-service-account.json  (已存在)
   ├── jwt_secret.txt               (新增)
   ├── db_password.txt               (新增)
   └── smtp_password.txt             (新增)
   ```

2. 在 `docker-compose.prod.yml` 定義新 secrets：
   ```yaml
   secrets:
     google_service_account:
       file: ./secrets/google-service-account.json
     jwt_secret:
       file: ./secrets/jwt_secret.txt
     db_password:
       file: ./secrets/db_password.txt
     smtp_password:
       file: ./secrets/smtp_password.txt
   ```

3. 修改 `api` 服務，掛載 secrets 並移除對應環境變數：
   ```yaml
   api:
     secrets:
       - jwt_secret
       - db_password
       - smtp_password
     environment:
       JWT_SECRET_FILE: /run/secrets/jwt_secret
       DATABASE_PASSWORD_FILE: /run/secrets/db_password
       SMTP_PASSWORD_FILE: /run/secrets/smtp_password
   ```

4. 修改 `db` 服務使用 `POSTGRES_PASSWORD_FILE`：
   ```yaml
   db:
     environment:
       POSTGRES_PASSWORD_FILE: /run/secrets/db_password
     secrets:
       - db_password
   ```

5. **後端 `config.rs` 修改**：新增 `read_secret()` helper，優先從 `*_FILE` 讀取檔案內容，fallback 至環境變數：
   ```rust
   fn read_secret(env_key: &str) -> String {
       let file_key = format!("{}_FILE", env_key);
       if let Ok(path) = std::env::var(&file_key) {
           std::fs::read_to_string(&path)
               .map(|s| s.trim().to_string())
               .unwrap_or_else(|_| std::env::var(env_key).expect(&format!("{} not set", env_key)))
       } else {
           std::env::var(env_key).expect(&format!("{} not set", env_key))
       }
   }
   ```

6. 將 `config.rs` 中 `JWT_SECRET`、`SMTP_PASSWORD`、`DATABASE_URL` 的讀取改用 `read_secret()`。

**影響檔案：**
- `docker-compose.prod.yml`
- `backend/src/config.rs`
- `secrets/` 目錄（新增 3 個檔案，加入 `.gitignore`）

**驗證方式：**
```bash
# 確認 secrets 可被容器讀取
docker exec ipig-api cat /run/secrets/jwt_secret  # 應輸出 secret 值
# 確認環境變數中不再包含明文
docker inspect ipig-api | grep JWT_SECRET  # 不應出現明文值
```

> ⚠️ 開發環境保持使用 `.env` 環境變數（向後相容），secrets 僅在 `docker-compose.prod.yml` 中啟用。

---

## 🟡 P1 — 效能（高優先級）

### P1-S4：修復 N+1 查詢 — `RoleService::list()`

**現況：** `backend/src/services/role.rs:54-91`  
先查所有角色（1 次），再在迴圈中為每個角色查詢權限（N 次），共 1+N 次 DB 查詢。

**目標：** 合併為 1–2 次查詢。

**實作方案：**

```rust
pub async fn list(pool: &PgPool) -> Result<Vec<RoleWithPermissions>> {
    let rows = sqlx::query_as::<_, (Role, Option<Permission>)>(
        r#"
        SELECT r.id, r.code, r.name, r.description, r.is_internal, r.is_system, r.is_active,
               r.created_at, r.updated_at,
               p.id as "p_id?", p.code as "p_code?", p.name as "p_name?", p.description as "p_desc?"
        FROM roles r
        LEFT JOIN role_permissions rp ON r.id = rp.role_id
        LEFT JOIN permissions p ON rp.permission_id = p.id
        WHERE r.is_active = true
        ORDER BY r.code, p.code
        "#
    )
    .fetch_all(pool)
    .await?;

    // 在記憶體中依 role_id 分組
    let mut map: IndexMap<Uuid, RoleWithPermissions> = IndexMap::new();
    for (role, permission) in rows {
        let entry = map.entry(role.id).or_insert_with(|| RoleWithPermissions {
            id: role.id,
            code: role.code,
            name: role.name,
            description: role.description,
            is_internal: role.is_internal,
            is_system: role.is_system,
            is_active: role.is_active,
            permissions: Vec::new(),
            created_at: role.created_at,
            updated_at: role.updated_at,
        });
        if let Some(p) = permission {
            entry.permissions.push(p);
        }
    }

    Ok(map.into_values().collect())
}
```

> 注意：需要定義一個中間 row struct 來接收 JOIN 結果，因 sqlx 不直接支援 `(Role, Option<Permission>)` tuple。可用 `sqlx::FromRow` 自訂 struct 或改用兩次查詢（批次載入）。

**備選方案（兩次查詢）：**
```rust
// 1. 查所有角色
let roles = sqlx::query_as::<_, Role>("...").fetch_all(pool).await?;
let role_ids: Vec<Uuid> = roles.iter().map(|r| r.id).collect();

// 2. 一次查所有權限（用 ANY）
let perms = sqlx::query_as::<_, RolePermissionRow>(
    "SELECT rp.role_id, p.* FROM permissions p
     INNER JOIN role_permissions rp ON p.id = rp.permission_id
     WHERE rp.role_id = ANY($1)"
)
.bind(&role_ids)
.fetch_all(pool).await?;

// 3. 在記憶體中分組
```

**影響檔案：**
- `backend/src/services/role.rs`

---

### P1-S5：修復 N+1 查詢 — `UserService::list()`

**現況：** `backend/src/services/user.rs:71-118`  
先查所有使用者（1 次），再為每個使用者呼叫 `get_user_roles_permissions()`（每次 2 次查詢），共 1+2N 次。

**目標：** 合併為 3 次查詢（users + 全部 user_roles + 全部 user_permissions）。

**實作方案：**

```rust
pub async fn list(pool: &PgPool, keyword: Option<&str>, pagination: &PaginationParams) -> Result<Vec<UserResponse>> {
    // 1. 查使用者
    let users = /* 現有 query */;
    let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();

    // 2. 批次查角色
    let user_roles = sqlx::query_as::<_, UserRoleRow>(
        r#"SELECT ur.user_id, r.code
           FROM user_roles ur
           INNER JOIN roles r ON ur.role_id = r.id
           WHERE ur.user_id = ANY($1)"#
    )
    .bind(&user_ids)
    .fetch_all(pool).await?;

    // 3. 批次查權限
    let user_perms = sqlx::query_as::<_, UserPermRow>(
        r#"SELECT ur.user_id, p.code
           FROM user_roles ur
           INNER JOIN role_permissions rp ON ur.role_id = rp.role_id
           INNER JOIN permissions p ON rp.permission_id = p.id
           WHERE ur.user_id = ANY($1)"#
    )
    .bind(&user_ids)
    .fetch_all(pool).await?;

    // 4. 建立 HashMap<Uuid, (Vec<String>, Vec<String>)>
    let mut roles_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    let mut perms_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for row in user_roles { roles_map.entry(row.user_id).or_default().push(row.code); }
    for row in user_perms { perms_map.entry(row.user_id).or_default().push(row.code); }

    // 5. 組裝 UserResponse
    let result = users.into_iter().map(|user| {
        let roles = roles_map.remove(&user.id).unwrap_or_default();
        let permissions = perms_map.remove(&user.id).unwrap_or_default();
        build_user_response(user, roles, permissions)  // 見 P2-S9
    }).collect();

    Ok(result)
}
```

**影響檔案：**
- `backend/src/services/user.rs`
- `backend/src/services/auth.rs`（提取 `get_user_roles_permissions` 的 SQL 邏輯供共用）

---

### P1-S6：迴圈 INSERT 改為批次操作

**現況：** 建立/更新角色時在迴圈中逐一 INSERT 權限；建立使用者時在迴圈中逐一指派角色。  
**位置：**
- `backend/src/services/role.rs:43-49`（建立角色）
- `backend/src/services/role.rs:166-171`（更新角色）
- `backend/src/services/user.rs:57-64`（建立使用者）

**實作方案：** 改用 PostgreSQL `UNNEST` 批次插入：

```rust
// 取代迴圈中的個別 INSERT
if !permission_ids.is_empty() {
    sqlx::query(
        r#"INSERT INTO role_permissions (role_id, permission_id)
           SELECT $1, unnest($2::uuid[])
           ON CONFLICT DO NOTHING"#
    )
    .bind(role_id)
    .bind(&permission_ids)
    .execute(pool)
    .await?;
}
```

```rust
// 使用者角色指派
if !role_ids.is_empty() {
    sqlx::query(
        r#"INSERT INTO user_roles (user_id, role_id)
           SELECT $1, unnest($2::uuid[])
           ON CONFLICT DO NOTHING"#
    )
    .bind(user_id)
    .bind(&role_ids)
    .execute(pool)
    .await?;
}
```

**影響檔案：**
- `backend/src/services/role.rs`
- `backend/src/services/user.rs`

---

### P1-S7：移除生產程式碼中的 `.expect()`

**現況：** `backend/src/handlers/auth.rs` 中有 6 處 `.expect()`，`backend/src/handlers/two_factor.rs` 中可能也有。  
**風險：** JSON 序列化或 HTTP Response 建構失敗時會造成 panic，導致整個 Axum worker thread 崩潰。

**位置清單（auth.rs）：**
| 行號 | 用途 | 替代方案 |
|------|------|----------|
| 80 | `serde_json::to_string(response).expect(...)` | `map_err(AppError::Internal)` |
| 88 | `Response::builder()...expect(...)` | `map_err(AppError::Internal)` |
| 149 | 2FA response 序列化 | 同上 |
| 154 | 2FA response 建構 | 同上 |
| 331 | 登出 JSON 序列化 | 同上 |
| 334 | 登出 Response 建構 | 同上 |

**實作方案：** 提取一個共用 helper 函數：

```rust
fn json_response_with_cookies(
    body: &impl Serialize,
    cookies: Vec<HeaderValue>,
) -> Result<Response<Body>, AppError> {
    let json = serde_json::to_string(body)
        .map_err(|e| AppError::Internal(format!("JSON 序列化失敗: {}", e)))?;

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json");

    for cookie in cookies {
        builder = builder.header(header::SET_COOKIE, cookie);
    }

    builder.body(Body::from(json))
        .map_err(|e| AppError::Internal(format!("Response 建構失敗: {}", e)))
}
```

**影響檔案：**
- `backend/src/handlers/auth.rs`
- `backend/src/handlers/two_factor.rs`

---

### P1-S8：新增資料庫複合索引

**現況：** 個別欄位有索引，但高頻查詢組合缺少複合索引。

**新增 migration `017_composite_indexes.sql`：**

```sql
-- 動物列表查詢：常以 status + is_deleted 篩選，created_at 排序
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_animals_status_deleted_created
ON animals(status, is_deleted, created_at DESC);

-- 協議列表查詢：常以 status + PI 篩選
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_protocols_status_pi_created
ON protocols(status, pi_user_id, created_at DESC);

-- 通知查詢：使用者的未讀通知
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notifications_user_read_created
ON notifications(user_id, is_read, created_at DESC);

-- 稽核日誌查詢：依 entity 查歷史
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_entity_created
ON user_activity_logs(entity_type, entity_id, created_at DESC);

-- 附件查詢：依 entity 查附件
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attachments_entity
ON attachments(entity_type, entity_id);
```

> `CREATE INDEX CONCURRENTLY` 不會鎖表，可在線上環境安全執行。

**影響檔案：**
- `backend/migrations/017_composite_indexes.sql`（新增）

---

## 🔵 P2 — 程式碼品質（中優先級）

### P2-S9：提取 `UserResponse` 建構邏輯 + `CurrentUser::is_admin()`

**現況（UserResponse）：** `backend/src/services/auth.rs` 中有 6 處手動建構 `UserResponse`，每處 ~20 行重複程式碼。欄位變更時需修改 6 處。

**現況（Admin 檢查）：** 約 20 個 handlers 中硬編碼 `"admin"` 字串做角色檢查，散布各處。`CurrentUser` 已有 `has_role()` 和 `has_permission()`，但缺少專用的 `is_admin()` 方法。

**實作步驟：**

1. **`CurrentUser::is_admin()`** — 在 `backend/src/middleware/auth.rs` 新增：
   ```rust
   impl CurrentUser {
       pub fn is_admin(&self) -> bool {
           self.roles.iter().any(|r| r == "admin" || r == "SYSTEM_ADMIN")
       }
   }
   ```

2. **`UserResponse::from_user()`** — 在 `backend/src/models/user.rs` 新增：
   ```rust
   impl UserResponse {
       pub fn from_user(user: User, roles: Vec<String>, permissions: Vec<String>) -> Self {
           Self {
               id: user.id,
               email: user.email,
               display_name: user.display_name,
               phone: user.phone,
               organization: user.organization,
               is_internal: user.is_internal,
               is_active: user.is_active,
               must_change_password: user.must_change_password,
               theme_preference: user.theme_preference,
               language_preference: user.language_preference,
               last_login_at: user.last_login_at,
               entry_date: user.entry_date,
               position: user.position,
               aup_roles: user.aup_roles,
               years_experience: user.years_experience,
               trainings: user.trainings.0,
               roles,
               permissions,
               totp_enabled: user.totp_enabled,
           }
       }
   }
   ```

3. 將 `auth.rs` 6 處和 `user.rs` 中的手動建構改為 `UserResponse::from_user(user, roles, permissions)`。

4. 將各 handler 中的 `current_user.roles.contains(&"admin".to_string())` 替換為 `current_user.is_admin()`。

**影響檔案：**
- `backend/src/middleware/auth.rs`
- `backend/src/models/user.rs`
- `backend/src/services/auth.rs`（6 處）
- `backend/src/services/user.rs`
- ~10 個 handler 檔案中的 admin 檢查

---

### P2-S10：前端 TypeScript 嚴格化

**現況：** 多處使用 `error: any`、`as any`、`data: any`，削弱型別安全。

**實作步驟：**

1. **定義 `ApiError` 型別**（`frontend/src/types/error.ts`，新增）：
   ```typescript
   export interface ApiErrorResponse {
     error: {
       message: string
       code: number
       blocking: boolean
       warning_type?: string
       existing_animals?: unknown[]
     }
   }

   export function getErrorMessage(error: unknown): string {
     if (axios.isAxiosError(error)) {
       return error.response?.data?.error?.message || error.message
     }
     if (error instanceof Error) return error.message
     return '未知錯誤'
   }
   ```

2. **替換 `catch (error: any)` 模式**：
   - 搜尋所有 `error: any` → 改為 `error: unknown`
   - 使用 `getErrorMessage(error)` 提取訊息
   - 預計影響 ~30 處

3. **修正已知 `any` 型別**：
   | 檔案 | 變數 | 改為 |
   |------|------|------|
   | `types/aup.ts` | `vet_review?: any` | 定義 `VetReview` interface |
   | `ReviewCommentsReport.tsx` | `protocol: any` | `Protocol` |
   | `AnimalTimelineView.tsx` | `raw?: any` | 定義 `TimelineRawData` interface |

**影響檔案：**
- `frontend/src/types/error.ts`（新增）
- `frontend/src/types/aup.ts`
- ~15 個使用 `catch (error: any)` 的檔案

---

### P2-S11：前端 API 錯誤統一處理

**現況：** `frontend/src/lib/api.ts` response interceptor 僅處理 401（token refresh），其他錯誤直接 reject。各元件的 `onError` 各自處理，格式不一致。

**實作步驟：**

1. **在 `api.ts` interceptor 中新增全域錯誤通知**（使用 Sonner toast）：
   ```typescript
   import { toast } from 'sonner'

   api.interceptors.response.use(
     (response) => response,
     async (error: AxiosError<ApiErrorResponse>) => {
       // 401 refresh 邏輯保持不變...

       // 非 401 錯誤的統一處理
       if (error.response) {
         const status = error.response.status
         const msg = error.response.data?.error?.message

         if (status === 403) {
           toast.error(msg || '權限不足')
         } else if (status === 500) {
           toast.error('伺服器錯誤，請稍後再試')
         } else if (status === 503) {
           toast.error('服務暫時不可用，請稍後再試')
         }
       } else if (error.code === 'ECONNABORTED') {
         toast.error('請求逾時，請檢查網路連線')
       } else if (!error.response) {
         toast.error('無法連線至伺服器')
       }

       return Promise.reject(error)
     }
   )
   ```

2. **在 TanStack Query 的全域 `onError` 設定中新增 mutation 錯誤通知**（`main.tsx` 的 `QueryClient`）：
   ```typescript
   const queryClient = new QueryClient({
     defaultOptions: {
       mutations: {
         onError: (error) => {
           // 如果 interceptor 已處理（403/500/503），不重複通知
           // 否則顯示通用錯誤
         }
       }
     }
   })
   ```

**影響檔案：**
- `frontend/src/lib/api.ts`
- `frontend/src/main.tsx`

---

### P2-S12：`MainLayout` 元件拆分

**現況：** `frontend/src/components/layout/MainLayout.tsx` 共 1,192 行，包含側邊欄、通知下拉選單、密碼變更對話框、行動端選單等所有邏輯。

**拆分方案：**

| 新元件 | 預估行數 | 職責 |
|--------|----------|------|
| `Sidebar.tsx` | ~300 行 | 側邊欄導航 + 折疊/展開 |
| `NotificationDropdown.tsx` | ~200 行 | 通知鈴鐺 + 下拉選單 + 已讀/未讀 |
| `PasswordChangeDialog.tsx` | ~150 行 | 密碼變更表單 Dialog |
| `MobileMenu.tsx` | ~150 行 | 行動端漢堡選單 |
| `MainLayout.tsx`（重構後） | ~400 行 | 佈局骨架 + 組合上述元件 |

**實作原則：**
- 各子元件自行管理 state 和 queries
- 透過 Zustand store（已有 `useAuthStore`、`useUiPreferences`）共享全域狀態
- 不引入新的 prop drilling

**影響檔案：**
- `frontend/src/components/layout/MainLayout.tsx`（重構）
- `frontend/src/components/layout/Sidebar.tsx`（新增）
- `frontend/src/components/layout/NotificationDropdown.tsx`（新增）
- `frontend/src/components/layout/PasswordChangeDialog.tsx`（新增）
- `frontend/src/components/layout/MobileMenu.tsx`（新增）

---

### P2-S13：前端關鍵元件 Memoization

**現況：** 多數列表項元件和 Tab 陣列未使用 `React.memo`、`useMemo`、`useCallback`，Tab 切換時造成不必要重渲染。

**實作步驟：**

1. **Tab 陣列 memoize**：
   ```typescript
   // AnimalDetailPage.tsx
   const tabs = useMemo(() => [
     { key: 'info', label: t('animal.tabs.info'), component: AnimalInfoTab },
     { key: 'observations', label: t('animal.tabs.observations'), component: ObservationsTab },
     // ...
   ], [t])  // 僅語言切換時重建
   ```

2. **事件處理函數 useCallback**：
   ```typescript
   const handleDelete = useCallback(async (id: string) => {
     // ...
   }, [deleteMutation])
   ```

3. **列表項 React.memo**：
   - 為各 Tab 元件（`ObservationsTab`、`SurgeriesTab` 等）加上 `React.memo`
   - 為重複渲染的列表行元件加上 `React.memo`

4. **大型對話框 lazy loading**：
   ```typescript
   const ExportDialog = React.lazy(() => import('./ExportDialog'))
   ```

**影響檔案：**
- `frontend/src/pages/animals/AnimalDetailPage.tsx`
- `frontend/src/pages/protocols/ProtocolDetailPage.tsx`
- `frontend/src/components/animal/*.tsx`（多個 Tab 元件）
- `frontend/src/components/protocol/*.tsx`（多個 Tab 元件）

---

### P2-S14：後端 Dockerfile 建置快取優化

**現況：** `backend/Dockerfile` 中 `COPY Cargo.toml Cargo.lock ./` 與 `COPY src ./src` 在同一 stage，任何源碼變更都會觸發完整的依賴重新編譯。CI 建置時間約 10–15 分鐘。

**實作方案：** 使用 `cargo-chef` 分離依賴層：

```dockerfile
# Stage 1: Chef - 規劃依賴
FROM rust:1.92 AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Planner - 生成依賴配方
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - 先編譯依賴，再編譯源碼
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# 編譯依賴（可被 Docker 層快取）
RUN cargo chef cook --release --recipe-path recipe.json
# 再複製源碼並編譯
COPY . .
RUN cargo build --release

# Stage 4: Runtime（不變）
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/erp-backend /usr/local/bin/
# ...
```

**預期效果：** 源碼變更時僅重新編譯應用程式碼（~1–2 分鐘），依賴層從快取載入。

**影響檔案：**
- `backend/Dockerfile`

---

## 📊 執行順序建議

| 順序 | 項目 | 依賴 | 預估工時 |
|------|------|------|----------|
| 1 | P0-S1 網路隔離 | 無 | 1h |
| 2 | P0-S2 DB 埠口 | 無 | 15min |
| 3 | P0-S3 Secrets 管理 | 無 | 2h |
| 4 | P2-S9 is_admin + UserResponse | 無 | 1.5h |
| 5 | P1-S4 Role N+1 | 無 | 1h |
| 6 | P1-S5 User N+1 | P2-S9 | 1.5h |
| 7 | P1-S6 批次 INSERT | 無 | 30min |
| 8 | P1-S7 移除 .expect() | 無 | 1h |
| 9 | P1-S8 複合索引 | 無 | 30min |
| 10 | P2-S10 TypeScript 嚴格化 | 無 | 2h |
| 11 | P2-S11 API 錯誤統一 | P2-S10 | 1h |
| 12 | P2-S12 MainLayout 拆分 | 無 | 3h |
| 13 | P2-S13 Memoization | 無 | 1.5h |
| 14 | P2-S14 Dockerfile 快取 | 無 | 1h |
| | **合計** | | **~18h** |

---

## ✅ 驗收標準

- [ ] `docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d` 啟動正常
- [ ] web 容器無法 ping db 容器
- [ ] `docker inspect` 中不含明文密碼
- [ ] 生產環境從主機無法直連 PostgreSQL
- [ ] `cargo build --release` 零 warning（排除 dead_code）
- [ ] `cargo clippy` 無新增 warning
- [ ] 無任何 `.expect()` 出現在 `src/handlers/` 目錄中
- [ ] `npx tsc --noEmit` 零 error
- [ ] `RoleService::list()` 和 `UserService::list()` 的 DB 查詢次數 ≤ 3
- [ ] MainLayout.tsx ≤ 500 行
