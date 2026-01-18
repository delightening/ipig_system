# Permissions & RBAC

> **Version**: 2.0  
> **Last Updated**: 2026-01-18  
> **Audience**: Administrators, Developers

---

## 1. Overview

iPig uses Role-Based Access Control (RBAC) where:
- **Users** have multiple **Roles**
- **Roles** have multiple **Permissions**
- **Permissions** are checked at API handler level

---

## 2. System Roles

### 2.1 Core Roles

| Code | Chinese Name | Description | Internal |
|------|--------------|-------------|----------|
| `admin` | 系統管理員 | Full system access | Yes |
| `iacuc_staff` | 執行秘書 | Protocol management, HR approval | Yes |
| `experiment_staff` | 試驗工作人員 | Animal records, experiments | Yes |
| `vet` | 獸醫師 | Animal health, recommendations | Yes |
| `warehouse` | 倉庫管理員 | ERP operations | Yes |
| `pi` | 計畫主持人 | Protocol submission | No |
| `client` | 委託人 | View commissioned projects | No |

### 2.2 Role Flags

| Flag | Description |
|------|-------------|
| `is_internal` | Role for internal employees (enables HR access) |
| `is_system` | System-defined role, cannot be deleted |
| `is_active` | Active/inactive flag |
| `is_deleted` | Soft delete flag |

---

## 3. Permission Categories

### 3.1 Protocol Permissions

| Code | Description |
|------|-------------|
| `protocol.create` | Create protocols |
| `protocol.read` | View protocols |
| `protocol.update` | Edit protocols |
| `protocol.delete` | Delete protocols |
| `protocol.submit` | Submit for review |
| `protocol.review` | Review protocols |
| `protocol.approve` | Approve protocols |
| `protocol.manage_status` | Change protocol status |
| `protocol.assign_reviewer` | Assign reviewers |
| `protocol.view_all` | View all protocols |

### 3.2 Animal (Pig) Permissions

| Code | Description |
|------|-------------|
| `pig.create` | Create pig records |
| `pig.read` | View pig records |
| `pig.update` | Edit pig records |
| `pig.delete` | Delete pig records |
| `pig.assign` | Assign pigs to protocols |
| `pig.observation.create` | Create observations |
| `pig.observation.read` | View observations |
| `pig.surgery.create` | Create surgery records |
| `pig.surgery.read` | View surgery records |
| `pig.vet_recommendation` | Add vet recommendations |
| `pig.export` | Export medical data |
| `pig.import` | Import pig data |

### 3.3 ERP Permissions

| Code | Description |
|------|-------------|
| `warehouse.read` | View warehouses |
| `warehouse.manage` | Manage warehouses |
| `product.read` | View products |
| `product.manage` | Manage products |
| `partner.read` | View partners |
| `partner.manage` | Manage partners |
| `document.create` | Create documents |
| `document.read` | View documents |
| `document.approve` | Approve documents |
| `inventory.read` | View inventory |
| `inventory.adjust` | Adjust inventory |

### 3.4 HR Permissions

| Code | Description |
|------|-------------|
| `hr.attendance.view` | 查看出勤紀錄 |
| `hr.attendance.view_all` | 查看所有出勤 |
| `hr.attendance.clock` | 打卡 |
| `hr.attendance.correct` | 更正打卡 |
| `hr.overtime.view` | 查看加班紀錄 |
| `hr.overtime.create` | 申請加班 |
| `hr.overtime.approve` | 審核加班 |
| `hr.leave.view` | 查看請假 |
| `hr.leave.view_all` | 查看所有請假 |
| `hr.leave.create` | 申請請假 |
| `hr.leave.approve` | 審核請假 |
| `hr.leave.manage` | 管理假別 |
| `hr.balance.view` | 查看餘額 |
| `hr.balance.manage` | 管理餘額 |
| `hr.calendar.config` | 設定行事曆同步 |
| `hr.calendar.view` | 檢視行事曆同步狀態 |
| `hr.calendar.sync` | 手動觸發行事曆同步 |
| `hr.calendar.conflicts` | 處理行事曆同步衝突 |

### 3.5 Audit Permissions

| Code | Description |
|------|-------------|
| `audit.logs.view` | 查看稽核日誌 |
| `audit.logs.export` | 匯出稽核日誌 |
| `audit.timeline.view` | 查看活動時間軸 |
| `audit.alerts.view` | 查看安全警報 |
| `audit.alerts.manage` | 管理安全警報 |

### 3.6 Admin Permissions

| Code | Description |
|------|-------------|
| `user.create` | Create users |
| `user.read` | View users |
| `user.update` | Edit users |
| `user.delete` | Delete users |
| `user.manage_roles` | Assign roles |
| `role.read` | View roles |
| `role.manage` | Manage roles |
| `notification.manage` | Manage notifications |
| `system.admin` | Full system administration |

---

## 4. Default Role Assignments

### 4.1 系統管理員 (admin)
All permissions

### 4.2 執行秘書 (iacuc_staff)
- All protocol permissions
- HR approval permissions (`hr.leave.approve`, `hr.overtime.approve`)
- HR calendar management
- Basic pig view access
- Notification management

### 4.3 試驗工作人員 (experiment_staff)
- Protocol view (own projects)
- Full pig CRUD access
- Observation and surgery management
- Weight and vaccination management
- Sacrifice and pathology access
- HR attendance (own)
- HR leave request (own)

### 4.4 獸醫師 (vet)
- Protocol view
- Pig view access
- Vet recommendation permissions
- Observation/surgery read and comment
- HR attendance (own)
- HR leave request (own)

### 4.5 倉庫管理員 (warehouse)
- Full ERP access
- Inventory management
- Document workflow
- Partner management
- Report access

### 4.6 計畫主持人 (pi)
- Protocol CRUD (own)
- Protocol submit
- View assigned animals
- My Projects access

### 4.7 委託人 (client)
- Protocol view (commissioned only)
- My Projects view (assigned)

---

## 5. Permission Checking

### 5.1 Handler-Level Checks

Permissions are validated in handlers using middleware:

```rust
// Example from handlers
async fn create_pig(
    State(state): State<AppState>,
    claims: Claims,  // Contains user_id and roles
    Json(payload): Json<CreatePigRequest>,
) -> Result<Json<Pig>, ApiError> {
    // Check permission
    require_permission(&state.db, &claims.user_id, "pig.create").await?;
    
    // ... create logic
}
```

### 5.2 Permission Inheritance

- Users inherit all permissions from all assigned roles
- Multiple roles stack (union of permissions)
- No explicit deny mechanism

### 5.3 Special Access Rules

| Rule | Description |
|------|-------------|
| **Own Data** | Users can always view their own leave/attendance |
| **Protocol Owner** | PI can edit own protocols before submission |
| **Assigned Reviewer** | Reviewers can comment on assigned protocols |
| **Co-Editor** | Co-editors have edit access to assigned protocols |

---

## 6. Internal vs External Users

| Aspect | Internal (`is_internal=true`) | External |
|--------|-------------------------------|----------|
| HR Access | Full attendance, leave, overtime | None |
| Dashboard | Full widgets including HR | Limited widgets |
| Roles | Can have internal-only roles | Limited to pi, client |

---

## 7. Leave Approval Chain

| Leave Days | Approval Required |
|------------|-------------------|
| ≤1 day | L1 (Direct Manager) |
| 2-3 days | L1 + L2 (Dept Head) |
| >3 days | L1 + L2 + HR |
| Special leave types | L1 + L2 + HR + GM |

---

*Next: [Audit & Logging](./07_AUDIT_LOGGING.md)*
