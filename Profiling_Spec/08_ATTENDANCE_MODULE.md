# Attendance Module Specification

> **Version**: 1.0  
> **Last Updated**: 2026-01-17  
> **Audience**: HR Team, Developers

---

## 1. Purpose

The Attendance Module provides comprehensive timekeeping and leave management for internal staff, including:

- **Work Attendance** - Clock-in/out tracking
- **Overtime Management** - Overtime recording and comp time generation
- **Leave Management** - Leave requests with approval workflow
- **Balance Tracking** - Annual leave and comp time balances with expiration
- **Google Calendar Sync** - Two-way synchronization with shared calendar

---

## 2. Scope

| In Scope | Out of Scope |
|----------|--------------|
| Internal employees only (`is_internal = true`) | External contractors |
| Clock-in/out tracking | Biometric integration |
| Overtime with comp time generation | Payroll calculation |
| Leave requests and approvals | Complex shift scheduling |
| Annual leave and comp time | Project-based time tracking |
| Google Calendar sync | Microsoft/Outlook sync |

---

## 3. Business Rules

### 3.1 Comp Time (補休)

| Rule | Description |
|------|-------------|
| Generation | Created from approved overtime records |
| Multiplier | Weekday: 1.0x, Weekend: 1.33x, Holiday: 1.66x or 2.0x |
| Expiration | **1 year** from overtime date |
| Usage Order | FIFO (oldest expires first) |
| Minimum | 0.5 hours |

### 3.2 Annual Leave (特休)

| Rule | Description |
|------|-------------|
| Entitlement | Based on years of service (Labor Standards Act) |
| Grant Time | Beginning of calendar year |
| Expiration | **2 years** from grant year end (e.g., 2025 grant expires 2027-12-31) |
| Carry Forward | Unused days carry to next year within expiration |
| Minimum | 0.5 days |

### 3.3 Leave Entitlement by Seniority

| Years of Service | Annual Leave Days |
|------------------|-------------------|
| 6 months - 1 year | 3 days (prorated) |
| 1 - 2 years | 7 days |
| 2 - 3 years | 10 days |
| 3 - 5 years | 14 days |
| 5 - 10 years | 15 days |
| 10+ years | 15 + 1 per year (max 30) |

### 3.4 Leave Types

| Type | Code | Approval Level | Documentation Required |
|------|------|----------------|------------------------|
| 特休假 | ANNUAL | L1 (≤3 days), L2 (>3 days) | No |
| 事假 | PERSONAL | L1 (≤1 day), L2 (>1 day) | No |
| 病假 | SICK | L1 (≤3 days), HR (>3 days) | Doctor's note (>3 days) |
| 補休假 | COMPENSATORY | L1 | No |
| 婚假 | MARRIAGE | L2 + HR | Marriage certificate |
| 喪假 | BEREAVEMENT | L1 + HR | Death certificate |
| 產假 | MATERNITY | L2 + HR + GM | Medical certificate |
| 陪產假 | PATERNITY | L2 + HR | Birth certificate |
| 生理假 | MENSTRUAL | L1 | No |
| 公假 | OFFICIAL | L2 | Official request |
| 無薪假 | UNPAID | L2 + HR + GM | Reason required |

---

## 4. Approval Workflow

### 4.1 Approval Levels

| Level | Role | Description |
|-------|------|-------------|
| L1 | Direct Manager | First-level approval |
| L2 | Department Head | Second-level approval |
| HR | IACUC Staff (執行秘書) | Administrative approval |
| GM | Admin | Final approval for special cases |

### 4.2 Workflow Diagram

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│  DRAFT  │───►│PENDING  │───►│PENDING  │───►│PENDING  │───►│APPROVED │
│         │    │  L1     │    │  L2     │    │  HR     │    │         │
└─────────┘    └────┬────┘    └────┬────┘    └────┬────┘    └─────────┘
     │              │              │              │
     │              ▼              ▼              ▼
     │         ┌─────────┐    ┌─────────┐    ┌─────────┐
     │         │REJECTED │    │REJECTED │    │REJECTED │
     │         └─────────┘    └─────────┘    └─────────┘
     │
     ▼
┌─────────┐
│CANCELLED│
└─────────┘
```

### 4.3 Special Cases

| Scenario | Handling |
|----------|----------|
| Urgent Leave (緊急請假) | Submit during/after leave, flagged for expedited review |
| Retroactive Leave (事後補請) | Submitted after leave date, requires explanation |
| Self-approval | Managers cannot approve their own requests (escalate to next level) |

---

## 5. Database Schema

### 5.1 Core Tables

#### attendance_records
| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Employee |
| work_date | DATE | Work date |
| clock_in_time | TIMESTAMPTZ | Check-in time |
| clock_out_time | TIMESTAMPTZ | Check-out time |
| regular_hours | NUMERIC(5,2) | Normal working hours |
| overtime_hours | NUMERIC(5,2) | Extra hours worked |
| status | VARCHAR(20) | normal, late, early_leave, absent, leave, holiday |

#### overtime_records
| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Employee |
| overtime_date | DATE | Date of overtime |
| start_time | TIMESTAMPTZ | OT start |
| end_time | TIMESTAMPTZ | OT end |
| hours | NUMERIC(5,2) | Hours worked |
| overtime_type | VARCHAR(20) | weekday, weekend, holiday |
| multiplier | NUMERIC(3,2) | Comp time multiplier |
| comp_time_hours | NUMERIC(5,2) | Generated comp time |
| comp_time_expires_at | DATE | Expiration date (1 year) |
| status | VARCHAR(20) | draft, pending, approved, rejected |

#### annual_leave_entitlements
| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Employee |
| entitlement_year | INTEGER | Year (e.g., 2025) |
| entitled_days | NUMERIC(5,2) | Days granted |
| used_days | NUMERIC(5,2) | Days used |
| expires_at | DATE | Expiration (2 years) |
| is_expired | BOOLEAN | Fully expired flag |

#### comp_time_balances
| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Employee |
| overtime_record_id | UUID | Source overtime |
| original_hours | NUMERIC(5,2) | Hours earned |
| used_hours | NUMERIC(5,2) | Hours used |
| earned_date | DATE | Date earned |
| expires_at | DATE | Expiration (1 year) |
| is_expired | BOOLEAN | Fully expired flag |

#### leave_requests
| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Requester |
| leave_type | leave_type ENUM | Type of leave |
| start_date | DATE | Leave start |
| end_date | DATE | Leave end |
| total_days | NUMERIC(5,2) | Total days |
| reason | TEXT | Reason for leave |
| status | leave_status ENUM | Current status |
| current_approver_id | UUID | Next approver |

---

## 6. Google Calendar Sync

### 6.1 Architecture

- **Approach**: Shared calendar with dedicated Gmail account
- **Sync Frequency**: Twice daily (8 AM, 6 PM Taiwan time)
- **Direction**: Primarily iPig → Google, with conflict detection
- **Visibility**: All staff with calendar access can see events

### 6.2 Event Format

```json
{
  "summary": "[請假] 王小明 - 特休",
  "description": "iPig Leave ID: abc-123\nType: 特休假\nStatus: 已核准",
  "start": { "date": "2026-01-20" },
  "end": { "date": "2026-01-21" },
  "extendedProperties": {
    "private": {
      "ipig_leave_id": "abc-123",
      "ipig_leave_type": "ANNUAL",
      "ipig_sync_version": "3"
    }
  }
}
```

### 6.3 Sync Rules

| iPig Event | Google Action |
|------------|---------------|
| Leave approved | Create event |
| Leave updated | Update event |
| Leave cancelled/revoked | Delete event |

### 6.4 Conflict Handling

| Google Change | iPig Response |
|---------------|---------------|
| Event deleted | Flag for review (do NOT delete leave) |
| Time changed | Flag for review |
| Title changed | Ignore (non-critical) |

Conflicts are stored in `calendar_sync_conflicts` and require admin resolution:
- **Keep iPig version**: Re-push to Google
- **Accept Google changes**: Update iPig (may require new approval)
- **Dismiss**: Mark resolved, no action

---

## 7. API Endpoints

### 7.1 Attendance

```
GET    /api/hr/attendance              # List attendance records
POST   /api/hr/attendance/clock-in     # Clock in
POST   /api/hr/attendance/clock-out    # Clock out
PUT    /api/hr/attendance/:id          # Manual correction
```

### 7.2 Overtime

```
GET    /api/hr/overtime                # List overtime records
POST   /api/hr/overtime                # Submit overtime
PUT    /api/hr/overtime/:id            # Update overtime
DELETE /api/hr/overtime/:id            # Delete (draft only)
POST   /api/hr/overtime/:id/submit     # Submit for approval
POST   /api/hr/overtime/:id/approve    # Approve
POST   /api/hr/overtime/:id/reject     # Reject
```

### 7.3 Leave

```
GET    /api/hr/leaves                  # List leave requests
POST   /api/hr/leaves                  # Create request
GET    /api/hr/leaves/:id              # Get details
PUT    /api/hr/leaves/:id              # Update (draft only)
DELETE /api/hr/leaves/:id              # Delete (draft only)
POST   /api/hr/leaves/:id/submit       # Submit for approval
POST   /api/hr/leaves/:id/approve      # Approve
POST   /api/hr/leaves/:id/reject       # Reject
POST   /api/hr/leaves/:id/cancel       # Cancel (before start)
POST   /api/hr/leaves/:id/revoke       # Revoke (after start)
```

### 7.4 Balances

```
GET    /api/hr/balances/annual         # Annual leave balances
GET    /api/hr/balances/comp-time      # Comp time balances
GET    /api/hr/balances/summary        # Combined summary
```

### 7.5 Calendar Sync

```
GET    /api/hr/calendar/status         # Sync status
POST   /api/hr/calendar/connect        # Configure calendar
POST   /api/hr/calendar/disconnect     # Remove config
POST   /api/hr/calendar/sync           # Manual sync trigger
PUT    /api/hr/calendar/settings       # Update settings
GET    /api/hr/calendar/conflicts      # List conflicts
POST   /api/hr/calendar/conflicts/:id/resolve  # Resolve conflict
```

---

## 8. UI Components

### 8.1 Navigation

```
👥 人員管理
  ├── 出勤打卡
  ├── 加班申請
  ├── 請假申請
  ├── 假期餘額
  └── 行事曆設定 (Admin)
```

### 8.2 Staff Views

| Page | Description |
|------|-------------|
| Attendance | Clock in/out, weekly summary |
| Overtime | Submit OT, view history |
| Leave | Request leave, view history |
| Balances | View annual and comp time |

### 8.3 Admin Views

| Page | Description |
|------|-------------|
| Team Calendar | Full team leave calendar |
| Balance Overview | All users' balances |
| Pending Approvals | Requests awaiting approval |
| Sync Status | Calendar sync monitoring |
| Conflicts | Resolve sync conflicts |

---

## 9. Permissions

| Code | Description |
|------|-------------|
| hr.attendance.view.own | View own attendance |
| hr.attendance.clock | Clock in/out |
| hr.attendance.view.all | View all attendance |
| hr.attendance.correct | Correct records |
| hr.overtime.view.own | View own overtime |
| hr.overtime.create | Submit overtime |
| hr.overtime.approve | Approve overtime |
| hr.leave.view.own | View own leaves |
| hr.leave.create | Request leave |
| hr.leave.approve.l1 | L1 approval |
| hr.leave.approve.l2 | L2 approval |
| hr.leave.approve.hr | HR approval |
| hr.leave.approve.gm | GM approval |
| hr.balance.view.own | View own balances |
| hr.balance.view.all | View all balances |
| hr.balance.manage | Adjust balances |
| hr.calendar.config | Configure sync |
| hr.calendar.sync | Trigger sync |
| hr.calendar.conflicts | Resolve conflicts |

---

## 10. Notifications

| Event | Recipients | Channel |
|-------|------------|---------|
| Leave submitted | Approvers | Email, In-app |
| Leave approved | Requester | Email, In-app |
| Leave rejected | Requester | Email, In-app |
| Balance expiring (30 days) | User | Email |
| Balance expiring (7 days) | User | Email, In-app |
| Sync conflict detected | HR Admin | In-app |

---

## 11. Background Jobs

| Job | Schedule | Description |
|-----|----------|-------------|
| Calendar Sync (AM) | 08:00 daily | Push/pull changes |
| Calendar Sync (PM) | 18:00 daily | Push/pull changes |
| Expiration Check | 00:00 daily | Mark expired balances |
| Expiry Warnings | 00:00 daily | Send expiration notifications |
| Aggregate Daily Stats | 01:00 daily | Calculate aggregates |

---

## 12. Related Documents

- [Permissions & RBAC](./06_PERMISSIONS_RBAC.md) - Role assignments
- [API Specification](./05_API_SPECIFICATION.md) - Full endpoint details
- [Audit & Logging](./07_AUDIT_LOGGING.md) - Activity tracking

---

*Last updated: 2026-01-17*
