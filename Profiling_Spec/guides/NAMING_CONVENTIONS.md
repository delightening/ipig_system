# Naming Conventions

> **Version**: 2.0  
> **Last Updated**: 2026-01-18  
> **Audience**: All Developers

---

## 1. General Principles

| Principle | Description |
|-----------|-------------|
| **Clarity** | Names should be self-documenting |
| **Consistency** | Follow established patterns |
| **Brevity** | Short but meaningful |
| **English** | Technical names in English, UI in Chinese |

---

## 2. Backend (Rust)

### 2.1 Files

| Type | Convention | Example |
|------|------------|---------|
| Module | snake_case | `pig.rs`, `google_calendar.rs` |
| Handler files | singular | `handlers/pig.rs` |
| Service files | singular | `services/pig.rs` |
| Model files | singular | `models/pig.rs` |

### 2.2 Code

| Type | Convention | Example |
|------|------------|---------|
| Struct | PascalCase | `PigObservation`, `LeaveRequest` |
| Enum | PascalCase | `PigStatus`, `LeaveType` |
| Enum Variant | PascalCase | `InExperiment`, `Approved` |
| Function | snake_case | `create_pig`, `get_leave_balance` |
| Variable | snake_case | `user_id`, `event_date` |
| Constant | SCREAMING_SNAKE | `MAX_LOGIN_ATTEMPTS` |
| Module | snake_case | `mod pig_surgery;` |

### 2.3 API Handlers

| Pattern | Example |
|---------|---------|
| List | `list_pigs`, `list_leaves` |
| Get | `get_pig`, `get_leave` |
| Create | `create_pig`, `create_leave` |
| Update | `update_pig`, `update_leave` |
| Delete | `delete_pig`, `delete_leave` |
| Action | `submit_leave`, `approve_leave` |

---

## 3. Database (PostgreSQL)

### 3.1 Tables

| Type | Convention | Example |
|------|------------|---------|
| Table | snake_case, plural | `pigs`, `users`, `leave_requests` |
| Junction table | both_entities | `user_roles`, `role_permissions` |
| History table | entity_history | `protocol_status_history` |

### 3.2 Columns

| Type | Convention | Example |
|------|------------|---------|
| Primary key | `id` | `id UUID PRIMARY KEY` |
| Foreign key | `entity_id` | `user_id`, `pig_id` |
| Boolean | `is_*` or `has_*` | `is_active`, `has_attachments` |
| Timestamp | `*_at` | `created_at`, `approved_at` |
| Date | `*_date` | `entry_date`, `work_date` |
| Status | `status` | `status VARCHAR(20)` |

### 3.3 Enums

| Type | Convention | Example |
|------|------------|---------|
| Enum type | snake_case | `pig_status`, `leave_type` |
| Enum values | lowercase | `unassigned`, `in_experiment` |

### 3.4 Indexes

| Pattern | Example |
|---------|---------|
| Primary | `idx_{table}_{columns}` |
| Foreign key | `idx_{table}_{fk}` |
| Composite | `idx_{table}_{col1}_{col2}` |

Examples:
- `idx_pigs_ear_tag`
- `idx_pigs_status`
- `idx_attendance_user_date`

---

## 4. Frontend (TypeScript/React)

### 4.1 Files

| Type | Convention | Example |
|------|------------|---------|
| Component | PascalCase | `PigDetail.tsx`, `LeaveForm.tsx` |
| Page | PascalCase + Page | `DashboardPage.tsx`, `PigsPage.tsx` |
| Hook | camelCase | `useAuth.ts`, `usePigs.ts` |
| Store | camelCase | `authStore.ts` |
| Type | camelCase | `types/hr.ts` |
| Utility | camelCase | `utils/format.ts` |

### 4.2 Components

| Type | Convention | Example |
|------|------------|---------|
| Component | PascalCase | `PigCard`, `LeaveRequestForm` |
| Props interface | ComponentNameProps | `PigCardProps` |
| Children | ReactNode | `children: ReactNode` |

### 4.3 Functions/Variables

| Type | Convention | Example |
|------|------------|---------|
| Function | camelCase | `handleSubmit`, `formatDate` |
| Variable | camelCase | `isLoading`, `userData` |
| Constant | SCREAMING_SNAKE | `API_BASE_URL` |
| State | camelCase | `[pigs, setPigs]` |

### 4.4 Types/Interfaces

| Type | Convention | Example |
|------|------------|---------|
| Interface | PascalCase | `User`, `Pig`, `LeaveRequest` |
| Enum | PascalCase | `PigStatus`, `LeaveType` |
| Type alias | PascalCase | `CreatePigRequest` |

---

## 5. API Routes

### 5.1 URL Patterns

| Pattern | Example |
|---------|---------|
| Resource list | `GET /pigs` |
| Resource detail | `GET /pigs/:id` |
| Resource create | `POST /pigs` |
| Resource update | `PUT /pigs/:id` |
| Resource delete | `DELETE /pigs/:id` |
| Nested resource | `GET /pigs/:id/observations` |
| Action | `POST /pigs/:id/observations/copy` |
| Status change | `POST /leaves/:id/approve` |

### 5.2 Naming

| Pattern | Example |
|---------|---------|
| Plural nouns | `/pigs`, `/users`, `/documents` |
| Kebab-case for multi-word | `/pig-sources`, `/leave-requests` |
| Action verbs at end | `/leaves/:id/approve` |
| Query params for filters | `?status=approved&user_id=...` |

---

## 6. CSS/Styling

### 6.1 Tailwind Classes

Follow Tailwind conventions:
- Utility-first approach
- Component composition
- Consistent spacing scale

### 6.2 Custom Classes

| Type | Convention | Example |
|------|------------|---------|
| Component class | kebab-case | `.pig-card`, `.leave-form` |
| Modifier | BEM-style | `.pig-card--active` |
| State | `is-*` | `.is-loading`, `.is-error` |

---

## 7. Git

### 7.1 Branch Names

| Type | Convention | Example |
|------|------------|---------|
| Feature | `feature/description` | `feature/leave-approval` |
| Bugfix | `fix/description` | `fix/pig-status-update` |
| Hotfix | `hotfix/description` | `hotfix/login-timeout` |

### 7.2 Commit Messages

| Type | Example |
|------|---------|
| Feature | `feat: add leave approval workflow` |
| Fix | `fix: correct pig status transition` |
| Docs | `docs: update API specification` |
| Refactor | `refactor: extract pig service functions` |
| Style | `style: format with rustfmt` |

---

## 8. Environment Variables

| Convention | Example |
|------------|---------|
| SCREAMING_SNAKE | `DATABASE_URL` |
| Prefix by service | `POSTGRES_*`, `SMTP_*` |
| Boolean as string | `ENABLE_DEBUG=true` |

Examples:
- `DATABASE_URL`
- `JWT_SECRET`
- `SMTP_HOST`
- `GOOGLE_APPLICATION_CREDENTIALS`

---

*Next: [Version History](./12_VERSION_HISTORY.md)*
