# Audit & Logging Specification

> **Version**: 1.0  
> **Last Updated**: 2026-01-17  
> **Audience**: Security Team, Developers, Compliance Officers

---

## 1. Purpose

The audit logging system provides comprehensive tracking of user activities for:

- **GLP Compliance** - Who did what, when, to which entity
- **Security Monitoring** - Detect suspicious activity
- **Administrative Insights** - Understand system usage patterns
- **Incident Investigation** - Trace actions during security events

---

## 2. Audit Requirements

### 2.1 GLP Compliance

As a system used in experimental animal research, iPig must maintain audit trails that satisfy Good Laboratory Practice (GLP) requirements:

| Requirement | Implementation |
|-------------|----------------|
| **Immutability** | Audit logs are append-only, no updates or deletes |
| **Completeness** | All data mutations are logged |
| **Attribution** | Every action linked to a user |
| **Timestamp** | Server-side timestamps, not modifiable |
| **Retention** | 7 years total (2 hot, 5 cold) |

### 2.2 Data Subject Rights

| Consideration | Policy |
|---------------|--------|
| Access requests | Users can request their own activity data |
| Deletion requests | Audit data exempt from right to erasure |
| Anonymization | Not applicable - attribution is required |

---

## 3. What to Log

### 3.1 MUST Log (High Priority)

| Category | Events | Example |
|----------|--------|---------|
| **Authentication** | Login success/failure, logout, password changes | User alice@example.com logged in |
| **Data Mutations** | Create, update, delete on all entities | Created pig ear_tag=A001 |
| **Approvals** | Status changes requiring authorization | Protocol P-2026-001 approved |
| **Exports** | Data exports, downloads | Exported pig medical records |
| **Admin Actions** | User management, role changes | Assigned VET role to user bob |
| **Sensitive Access** | Views of sensitive records | Viewed pathology report for pig A001 |

### 3.2 SHOULD Log (Medium Priority)

| Category | Events |
|----------|--------|
| **Page Views** | Access to sensitive pages (aggregated) |
| **Search Queries** | Searches over sensitive data |
| **Bulk Operations** | Batch updates, imports |

### 3.3 SHOULD NOT Log (Performance/Privacy)

| Avoid | Reason |
|-------|--------|
| Keystroke-level input | Privacy, performance |
| Mouse movements, scrolling | No audit value |
| Auto-save drafts | Too noisy |
| Health check requests | System noise |
| Static asset requests | No audit value |

---

## 4. Database Schema

### 4.1 user_activity_logs (Partitioned)

Main audit table, partitioned by quarter for performance.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| actor_user_id | UUID | Who performed the action |
| actor_email | VARCHAR(255) | Email snapshot at time of action |
| actor_display_name | VARCHAR(100) | Name snapshot |
| actor_roles | JSONB | Role codes at time of action |
| session_id | UUID | Link to user session |
| event_category | VARCHAR(50) | auth, data, admin, export, navigation |
| event_type | VARCHAR(100) | Specific event (pig.create, protocol.approve) |
| event_severity | VARCHAR(20) | info, warning, critical |
| entity_type | VARCHAR(50) | Type of affected entity |
| entity_id | UUID | ID of affected entity |
| entity_display_name | VARCHAR(255) | Human-readable identifier |
| before_data | JSONB | State before change |
| after_data | JSONB | State after change |
| changed_fields | TEXT[] | List of modified field names |
| ip_address | INET | Client IP |
| user_agent | TEXT | Browser/client info |
| request_path | VARCHAR(500) | API endpoint called |
| request_method | VARCHAR(10) | HTTP method |
| response_status | INTEGER | HTTP response code |
| is_suspicious | BOOLEAN | Flagged by anomaly detection |
| suspicious_reason | TEXT | Why it was flagged |
| created_at | TIMESTAMPTZ | Immutable timestamp |
| partition_date | DATE | Partition key |

**Partitions**: Quarterly (2026_q1, 2026_q2, etc.)

### 4.2 login_events

Dedicated table for authentication events.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | User attempting login (null if not found) |
| email | VARCHAR(255) | Email used for login attempt |
| event_type | VARCHAR(20) | login_success, login_failure, logout |
| ip_address | INET | Client IP |
| user_agent | TEXT | Browser info |
| device_type | VARCHAR(50) | desktop, mobile, tablet |
| browser | VARCHAR(50) | Chrome, Firefox, etc. |
| os | VARCHAR(50) | Windows, macOS, etc. |
| is_unusual_time | BOOLEAN | Outside 7 AM - 10 PM |
| is_unusual_location | BOOLEAN | Different from normal IPs |
| is_new_device | BOOLEAN | First time seeing device |
| device_fingerprint | VARCHAR(255) | Device identifier |
| failure_reason | VARCHAR(100) | If failed: invalid_password, etc. |
| created_at | TIMESTAMPTZ | Timestamp |

### 4.3 user_sessions

Track active sessions for force-logout capability.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Session owner |
| started_at | TIMESTAMPTZ | Session start |
| ended_at | TIMESTAMPTZ | When session ended |
| last_activity_at | TIMESTAMPTZ | Last API request |
| refresh_token_id | UUID | Link to refresh token |
| ip_address | INET | Session IP |
| user_agent | TEXT | Browser info |
| page_view_count | INTEGER | Pages visited |
| action_count | INTEGER | Mutations performed |
| is_active | BOOLEAN | Currently active |
| ended_reason | VARCHAR(50) | logout, expired, forced_logout |

### 4.4 security_alerts

Anomaly detection alerts.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| alert_type | VARCHAR(50) | brute_force, unusual_login, etc. |
| severity | VARCHAR(20) | info, warning, critical |
| title | VARCHAR(255) | Alert title |
| description | TEXT | Detailed description |
| user_id | UUID | Related user |
| context_data | JSONB | Additional context |
| status | VARCHAR(20) | open, acknowledged, resolved |
| resolved_by | UUID | Who resolved it |
| resolved_at | TIMESTAMPTZ | When resolved |
| resolution_notes | TEXT | Resolution details |

---

## 5. Event Types

### 5.1 Authentication Events

| Event Type | Severity | Triggers |
|------------|----------|----------|
| auth.login_success | info | Successful login |
| auth.login_failure | warning | Failed login attempt |
| auth.logout | info | User logout |
| auth.token_refresh | info | Token refreshed |
| auth.password_change | info | Password changed by user |
| auth.password_reset | info | Password reset via email |
| auth.session_expired | info | Session timed out |

### 5.2 Data Events

| Event Type | Severity | Triggers |
|------------|----------|----------|
| {entity}.create | info | Entity created |
| {entity}.update | info | Entity updated |
| {entity}.delete | warning | Entity deleted |
| {entity}.status_change | info | Status field changed |

Where `{entity}` is: pig, protocol, document, user, role, etc.

### 5.3 Admin Events

| Event Type | Severity | Triggers |
|------------|----------|----------|
| admin.user.create | info | New user created |
| admin.user.role_change | warning | User roles modified |
| admin.user.deactivate | warning | User deactivated |
| admin.role.permission_change | warning | Role permissions modified |
| admin.session.force_logout | warning | Admin forced logout |

### 5.4 Export Events

| Event Type | Severity | Triggers |
|------------|----------|----------|
| export.pig_medical | info | Medical record exported |
| export.audit_logs | info | Audit logs exported |
| export.report | info | Report generated |

---

## 6. Anomaly Detection

### 6.1 Detection Rules

| Rule | Threshold | Severity |
|------|-----------|----------|
| Brute force | 5 failed logins in 15 min | critical |
| Unusual time | Login before 7 AM or after 10 PM | warning |
| Unusual location | New IP range | warning |
| High volume | >100 mutations in 1 hour | warning |
| After-hours data access | Sensitive data access outside hours | warning |

### 6.2 Alert Workflow

```
Detection → Alert Created → Notification Sent → Admin Reviews
                                                      │
                           ┌──────────────────────────┼──────────────────────────┐
                           ▼                          ▼                          ▼
                    Acknowledge            Investigate                    Dismiss
                    (mark as seen)         (link to session)              (false positive)
                           │                          │                          │
                           └──────────────────────────┼──────────────────────────┘
                                                      ▼
                                                  Resolve
                                              (add resolution notes)
```

---

## 7. Retention Policy

| Storage Tier | Duration | Location | Access |
|--------------|----------|----------|--------|
| Hot | 2 years | Primary database | Full query |
| Archive | 5 years | Cold storage | Request-based |
| **Total** | **7 years** | | |

### 7.1 Archival Process

1. Quarterly partitions older than 2 years are archived
2. Archived partitions are compressed and moved to cold storage
3. Index metadata is retained for search
4. Restoration available within 24 hours

---

## 8. API Endpoints

All endpoints require `admin.audit.*` permissions.

### 8.1 Activity Logs

```
GET /api/admin/audit/activities
```

Query parameters:
- `user_id` - Filter by actor
- `entity_type` - Filter by entity type
- `entity_id` - Filter by specific entity
- `event_category` - auth, data, admin, export
- `event_type` - Specific event type
- `from`, `to` - Date range
- `is_suspicious` - Only suspicious activities
- `page`, `limit` - Pagination

### 8.2 Login Events

```
GET /api/admin/audit/logins
```

Query parameters:
- `user_id` - Filter by user
- `event_type` - login_success, login_failure, logout
- `is_unusual` - Only unusual logins
- `from`, `to` - Date range

### 8.3 Sessions

```
GET /api/admin/audit/sessions
POST /api/admin/audit/sessions/:id/force-logout
```

### 8.4 User Timeline

```
GET /api/admin/audit/users/:id/timeline
GET /api/admin/audit/users/:id/summary
```

### 8.5 Entity History

```
GET /api/admin/audit/entities/:type/:id/history
```

### 8.6 Security Alerts

```
GET /api/admin/audit/security-alerts
POST /api/admin/audit/security-alerts/:id/resolve
```

### 8.7 Export

```
POST /api/admin/audit/activities/export
```

---

## 9. UI Components

### 9.1 Admin Menu Structure

```
⚙️ 系統管理
  └── 安全審計
        ├── 活動日誌
        ├── 登入紀錄
        ├── 使用者分析
        └── 安全警報
```

### 9.2 Activity Log View

- Filterable data table
- Timeline visualization option
- Export to CSV/JSON
- Link to entity detail pages

### 9.3 User Profile View

- User information card
- Activity timeline
- Login history
- Session list
- Role change history

### 9.4 Entity History View

- Accessible from entity detail pages
- Shows all changes over time
- Diff view for changes
- Links to actors

---

## 10. Permissions

| Permission Code | Description |
|-----------------|-------------|
| admin.audit.view | View audit logs |
| admin.audit.view.activities | View activity records |
| admin.audit.view.logins | View login records |
| admin.audit.view.sessions | View sessions |
| admin.audit.export | Export audit data |
| admin.audit.force_logout | Force user logout |
| admin.audit.alerts.view | View security alerts |
| admin.audit.alerts.resolve | Resolve alerts |
| admin.audit.dashboard | View audit dashboard |

Default assignment: SYSTEM_ADMIN, PROGRAM_ADMIN

---

## 11. Implementation Notes

### 11.1 Middleware Integration

Activity logging is implemented as Axum middleware:

```rust
.route_layer(middleware::from_fn_with_state(
    state.clone(), 
    activity_logging_middleware
))
```

### 11.2 Async Logging

Logs are written asynchronously to avoid blocking requests:

```rust
tokio::spawn(async move {
    log_activity(db, event).await;
});
```

### 11.3 Performance Considerations

- Partitioned tables for efficient querying
- Indexes on common query patterns
- Separate table for login events (high volume)
- Daily aggregation for dashboard metrics

---

## 12. Related Documents

- [Database Schema](./04_DATABASE_SCHEMA.md) - Full table definitions
- [Permissions & RBAC](./06_PERMISSIONS_RBAC.md) - Role assignments
- [API Specification](./05_API_SPECIFICATION.md) - Endpoint details

---

*Last updated: 2026-01-17*
