# 命名慣例

> **版本**：7.0  
> **最後更新**：2026-03-01  
> **對象**：全體開發人員

---

## 1. 通用原則

| 原則 | 說明 |
|------|------|
| **清晰** | 名稱應具自我說明性 |
| **一致** | 遵循既定模式 |
| **簡潔** | 簡短但有意義 |
| **語言** | 技術名稱用英文，UI 用中文 |

---

## 2. 後端（Rust）

### 2.1 檔案

| 類型 | 慣例 | 範例 |
|------|------|------|
| 模組 | snake_case | `animal.rs`, `google_calendar.rs` |
| 處理器檔案 | 單數 | `handlers/animal.rs` |
| 服務檔案 | 單數 | `services/animal.rs` |
| 模型檔案 | 單數 | `models/animal.rs` |

### 2.2 程式碼

| 類型 | 慣例 | 範例 |
|------|------|------|
| Struct | PascalCase | `AnimalObservation`, `LeaveRequest` |
| Enum | PascalCase | `AnimalStatus`, `LeaveType` |
| Enum 變體 | PascalCase | `InExperiment`, `Approved` |
| 函數 | snake_case | `create_animal`, `get_leave_balance` |
| 變數 | snake_case | `user_id`, `event_date` |
| 常數 | SCREAMING_SNAKE | `MAX_LOGIN_ATTEMPTS` |
| 模組 | snake_case | `mod animal_surgery;` |

### 2.3 API 處理器

| 模式 | 範例 |
|------|------|
| 列表 | `list_animals`, `list_leaves` |
| 取得 | `get_animal`, `get_leave` |
| 建立 | `create_animal`, `create_leave` |
| 更新 | `update_animal`, `update_leave` |
| 刪除 | `delete_animal`, `delete_leave` |
| 動作 | `submit_leave`, `approve_leave` |

---

## 3. 資料庫（PostgreSQL）

### 3.1 資料表

| 類型 | 慣例 | 範例 |
|------|------|------|
| 資料表 | snake_case，複數 | `animals`, `users`, `leave_requests` |
| 關聯表 | 兩實體名稱 | `user_roles`, `role_permissions` |
| 歷程表 | entity_history | `protocol_status_history` |

### 3.2 欄位

| 類型 | 慣例 | 範例 |
|------|------|------|
| 主鍵 | `id` | `id UUID PRIMARY KEY` |
| 外鍵 | `entity_id` | `user_id`, `animal_id` |
| 布林 | `is_*` 或 `has_*` | `is_active`, `has_attachments` |
| 時間戳記 | `*_at` | `created_at`, `approved_at` |
| 日期 | `*_date` | `entry_date`, `work_date` |
| 狀態 | `status` | `status VARCHAR(20)` |

### 3.3 列舉

| 類型 | 慣例 | 範例 |
|------|------|------|
| 列舉類型 | snake_case | `animal_status`, `leave_type` |
| 列舉值 | 小寫 | `unassigned`, `in_experiment` |

### 3.4 索引

| 模式 | 範例 |
|------|------|
| 主要 | `idx_{table}_{columns}` |
| 外鍵 | `idx_{table}_{fk}` |
| 複合 | `idx_{table}_{col1}_{col2}` |

範例：
- `idx_animals_ear_tag`
- `idx_animals_status`
- `idx_attendance_user_date`

---

## 4. 前端（TypeScript/React）

### 4.1 檔案

| 類型 | 慣例 | 範例 |
|------|------|------|
| 元件 | PascalCase | `AnimalDetail.tsx`, `LeaveForm.tsx` |
| 頁面 | PascalCase + Page | `DashboardPage.tsx`, `AnimalsPage.tsx` |
| Hook | camelCase | `useAuth.ts`, `useAnimals.ts` |
| Store | camelCase | `authStore.ts` |
| 型別 | camelCase | `types/hr.ts` |
| 工具 | camelCase | `utils/format.ts` |

### 4.2 元件

| 類型 | 慣例 | 範例 |
|------|------|------|
| 元件 | PascalCase | `AnimalCard`, `LeaveRequestForm` |
| Props 介面 | ComponentNameProps | `AnimalCardProps` |
| Children | ReactNode | `children: ReactNode` |

### 4.3 函數/變數

| 類型 | 慣例 | 範例 |
|------|------|------|
| 函數 | camelCase | `handleSubmit`, `formatDate` |
| 變數 | camelCase | `isLoading`, `userData` |
| 常數 | SCREAMING_SNAKE | `API_BASE_URL` |
| State | camelCase | `[animals, setAnimals]` |

### 4.4 型別/介面

| 類型 | 慣例 | 範例 |
|------|------|------|
| 介面 | PascalCase | `User`, `Animal`, `LeaveRequest` |
| 列舉 | PascalCase | `AnimalStatus`, `LeaveType` |
| 型別別名 | PascalCase | `CreateAnimalRequest` |

---

## 5. API 路由

### 5.1 URL 模式

| 模式 | 範例 |
|------|------|
| 資源列表 | `GET /animals` |
| 資源詳情 | `GET /animals/:id` |
| 資源建立 | `POST /animals` |
| 資源更新 | `PUT /animals/:id` |
| 資源刪除 | `DELETE /animals/:id` |
| 巢狀資源 | `GET /animals/:id/observations` |
| 動作 | `POST /animals/:id/observations/copy` |
| 狀態變更 | `POST /leaves/:id/approve` |

### 5.2 命名

| 模式 | 範例 |
|------|------|
| 複數名詞 | `/animals`, `/users`, `/documents` |
| 多字詞用 kebab-case | `/animal-sources`, `/leave-requests` |
| 動作動詞放結尾 | `/leaves/:id/approve` |
| 查詢參數用於篩選 | `?status=approved&user_id=...` |

---

## 6. CSS/樣式

### 6.1 Tailwind Classes

遵循 Tailwind 慣例：
- 工具優先方法
- 元件組合
- 一致的間距比例

### 6.2 自訂類別

| 類型 | 慣例 | 範例 |
|------|------|------|
| 元件類別 | kebab-case | `.animal-card`, `.leave-form` |
| 修飾器 | BEM 風格 | `.animal-card--active` |
| 狀態 | `is-*` | `.is-loading`, `.is-error` |

---

## 7. Git

### 7.1 分支名稱

| 類型 | 慣例 | 範例 |
|------|------|------|
| 功能 | `feature/description` | `feature/leave-approval` |
| 錯誤修正 | `fix/description` | `fix/animal-status-update` |
| 緊急修正 | `hotfix/description` | `hotfix/login-timeout` |

### 7.2 提交訊息

| 類型 | 範例 |
|------|------|
| 功能 | `feat: 新增請假核准流程` |
| 修正 | `fix: 修正動物狀態轉換` |
| 文件 | `docs: 更新 API 規格` |
| 重構 | `refactor: 提取動物服務函數` |
| 樣式 | `style: 使用 rustfmt 格式化` |

---

## 8. 環境變數

| 慣例 | 範例 |
|------|------|
| SCREAMING_SNAKE | `DATABASE_URL` |
| 依服務加前綴 | `POSTGRES_*`, `SMTP_*` |
| 布林值用字串 | `ENABLE_DEBUG=true` |

範例：
- `DATABASE_URL`
- `JWT_SECRET`
- `SMTP_HOST`
- `GOOGLE_APPLICATION_CREDENTIALS`

---

*下一章：[版本歷程](../project/VERSION_HISTORY.md)*

*最後更新：2026-03-01*
