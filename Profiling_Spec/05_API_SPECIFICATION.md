# API Specification

> **Version**: 2.0  
> **Last Updated**: 2026-01-18  
> **Audience**: Frontend Developers, Integration Partners

---

## 1. Overview

Base URL: `http://localhost:8080/api` (development) or `https://ipig.example.com/api` (production)

### Authentication
- All endpoints except auth require a valid JWT Access Token
- Include token in header: `Authorization: Bearer <token>`
- Access tokens expire in 15 minutes
- Refresh tokens expire in 7 days

### Response Format
```json
{
  "data": { ... },      // Success response
  "error": "message",   // Error response
  "message": "string"   // Optional message
}
```

---

## 2. Authentication API

### Public Endpoints (No Auth Required)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/auth/login` | User login |
| POST | `/auth/refresh` | Refresh access token |
| POST | `/auth/forgot-password` | Request password reset |
| POST | `/auth/reset-password` | Reset password with token |

### Protected Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/auth/logout` | User logout |
| GET | `/me` | Get current user info |
| PUT | `/me/password` | Change own password |

### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```
Response:
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "display_name": "User Name",
    "roles": ["admin"]
  }
}
```

---

## 3. User Management API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/users` | List users |
| POST | `/users` | Create user |
| GET | `/users/:id` | Get user |
| PUT | `/users/:id` | Update user |
| DELETE | `/users/:id` | Delete user |
| PUT | `/users/:id/password` | Reset user password |

### User Preferences

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/me/preferences` | Get all preferences |
| GET | `/me/preferences/:key` | Get preference |
| PUT | `/me/preferences/:key` | Set preference |
| DELETE | `/me/preferences/:key` | Delete preference |

---

## 4. Roles & Permissions API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/roles` | List roles |
| POST | `/roles` | Create role |
| GET | `/roles/:id` | Get role |
| PUT | `/roles/:id` | Update role |
| DELETE | `/roles/:id` | Delete role |
| GET | `/permissions` | List all permissions |

---

## 5. Protocol (AUP) API

### Protocols

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/protocols` | List protocols |
| POST | `/protocols` | Create protocol |
| GET | `/protocols/:id` | Get protocol |
| PUT | `/protocols/:id` | Update protocol |
| POST | `/protocols/:id/submit` | Submit for review |
| POST | `/protocols/:id/status` | Change status |
| GET | `/protocols/:id/versions` | Get versions |
| GET | `/protocols/:id/status-history` | Get status history |
| GET | `/protocols/:id/animal-stats` | Get animal statistics |

### Review System

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/reviews/assignments` | List review assignments |
| POST | `/reviews/assignments` | Assign reviewer |
| GET | `/reviews/comments` | List review comments |
| POST | `/reviews/comments` | Create comment |
| POST | `/reviews/comments/:id/resolve` | Resolve comment |
| POST | `/reviews/comments/reply` | Reply to comment |

### Co-Editors

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/protocols/:id/co-editors` | List co-editors |
| POST | `/protocols/:id/co-editors` | Assign co-editor |
| DELETE | `/protocols/:id/co-editors/:user_id` | Remove co-editor |

### My Projects

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/my-projects` | Get my protocols |

---

## 6. Animal (Pig) Management API

### Pig CRUD

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pigs` | List pigs |
| POST | `/pigs` | Create pig |
| GET | `/pigs/:id` | Get pig |
| PUT | `/pigs/:id` | Update pig |
| DELETE | `/pigs/:id` | Soft delete pig |
| GET | `/pigs/by-pen` | List pigs by pen |

### Batch Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/pigs/batch/assign` | Batch assign pigs to protocol |
| POST | `/pigs/batch/start-experiment` | Batch start experiment |

### Pig Sources

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pig-sources` | List sources |
| POST | `/pig-sources` | Create source |
| PUT | `/pig-sources/:id` | Update source |
| DELETE | `/pig-sources/:id` | Delete source |

### Observations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pigs/:id/observations` | List observations |
| POST | `/pigs/:id/observations` | Create observation |
| GET | `/pigs/:id/observations/with-recommendations` | With vet recommendations |
| POST | `/pigs/:id/observations/copy` | Copy observation |
| GET | `/observations/:id` | Get observation |
| PUT | `/observations/:id` | Update observation |
| DELETE | `/observations/:id` | Delete observation |
| POST | `/observations/:id/vet-read` | Mark vet read |
| GET | `/observations/:id/versions` | Get versions |
| GET | `/observations/:id/recommendations` | Get recommendations |
| POST | `/observations/:id/recommendations` | Add recommendation |

### Surgeries

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pigs/:id/surgeries` | List surgeries |
| POST | `/pigs/:id/surgeries` | Create surgery |
| GET | `/pigs/:id/surgeries/with-recommendations` | With vet recommendations |
| POST | `/pigs/:id/surgeries/copy` | Copy surgery |
| GET | `/surgeries/:id` | Get surgery |
| PUT | `/surgeries/:id` | Update surgery |
| DELETE | `/surgeries/:id` | Delete surgery |
| POST | `/surgeries/:id/vet-read` | Mark vet read |
| GET | `/surgeries/:id/versions` | Get versions |

### Weights

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pigs/:id/weights` | List weights |
| POST | `/pigs/:id/weights` | Create weight |
| PUT | `/weights/:id` | Update weight |
| DELETE | `/weights/:id` | Delete weight |

### Vaccinations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pigs/:id/vaccinations` | List vaccinations |
| POST | `/pigs/:id/vaccinations` | Create vaccination |
| PUT | `/vaccinations/:id` | Update vaccination |
| DELETE | `/vaccinations/:id` | Delete vaccination |

### Sacrifice & Pathology

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pigs/:id/sacrifice` | Get sacrifice record |
| POST | `/pigs/:id/sacrifice` | Upsert sacrifice |
| GET | `/pigs/:id/pathology` | Get pathology report |
| POST | `/pigs/:id/pathology` | Upsert pathology |

### Data Import/Export

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pigs/import/batches` | List import batches |
| GET | `/pigs/import/template/basic` | Download basic template |
| GET | `/pigs/import/template/weight` | Download weight template |
| POST | `/pigs/import/basic` | Import basic data |
| POST | `/pigs/import/weights` | Import weight data |
| POST | `/pigs/:id/export` | Export pig medical data |
| POST | `/projects/:iacuc_no/export` | Export project data |

### Vet Dashboard

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/pigs/vet-comments` | Get vet comments |
| POST | `/pigs/:id/vet-read` | Mark pig vet read |

---

## 7. ERP API

### Warehouses

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/warehouses` | List warehouses |
| POST | `/warehouses` | Create warehouse |
| GET | `/warehouses/:id` | Get warehouse |
| PUT | `/warehouses/:id` | Update warehouse |
| DELETE | `/warehouses/:id` | Delete warehouse |

### Products

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/products` | List products |
| POST | `/products` | Create product |
| GET | `/products/:id` | Get product |
| PUT | `/products/:id` | Update product |
| DELETE | `/products/:id` | Delete product |
| GET | `/categories` | List categories |
| POST | `/categories` | Create category |

### SKU

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/sku/categories` | Get SKU categories |
| GET | `/sku/categories/:code/subcategories` | Get subcategories |
| POST | `/sku/generate` | Generate SKU |
| POST | `/sku/validate` | Validate SKU |
| POST | `/skus/preview` | Preview SKU |
| POST | `/products/with-sku` | Create product with SKU |

### Partners

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/partners` | List partners |
| POST | `/partners` | Create partner |
| GET | `/partners/generate-code` | Generate code |
| GET | `/partners/:id` | Get partner |
| PUT | `/partners/:id` | Update partner |
| DELETE | `/partners/:id` | Delete partner |

### Documents

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/documents` | List documents |
| POST | `/documents` | Create document |
| GET | `/documents/:id` | Get document |
| PUT | `/documents/:id` | Update document |
| DELETE | `/documents/:id` | Delete document |
| POST | `/documents/:id/submit` | Submit document |
| POST | `/documents/:id/approve` | Approve document |
| POST | `/documents/:id/cancel` | Cancel document |

### Inventory

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/inventory/on-hand` | Get on-hand inventory |
| GET | `/inventory/ledger` | Get stock ledger |
| GET | `/inventory/low-stock` | Get low stock alerts |

---

## 8. HR API

### Attendance

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/hr/attendance` | List attendance |
| POST | `/hr/attendance/clock-in` | Clock in |
| POST | `/hr/attendance/clock-out` | Clock out |
| GET | `/hr/attendance/stats` | Get statistics |
| PUT | `/hr/attendance/:id` | Correct attendance |

### Overtime

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/hr/overtime` | List overtime records |
| POST | `/hr/overtime` | Create overtime |
| GET | `/hr/overtime/:id` | Get overtime |
| PUT | `/hr/overtime/:id` | Update overtime |
| DELETE | `/hr/overtime/:id` | Delete overtime |
| POST | `/hr/overtime/:id/submit` | Submit for approval |
| POST | `/hr/overtime/:id/approve` | Approve overtime |
| POST | `/hr/overtime/:id/reject` | Reject overtime |

### Leave

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/hr/leaves` | List leave requests |
| POST | `/hr/leaves` | Create leave |
| GET | `/hr/leaves/:id` | Get leave |
| PUT | `/hr/leaves/:id` | Update leave |
| DELETE | `/hr/leaves/:id` | Delete leave |
| POST | `/hr/leaves/:id/submit` | Submit for approval |
| POST | `/hr/leaves/:id/approve` | Approve leave |
| POST | `/hr/leaves/:id/reject` | Reject leave |
| POST | `/hr/leaves/:id/cancel` | Cancel leave |
| POST | `/hr/leaves/attachments` | Upload attachment |

### Balances

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/hr/balances/annual` | Annual leave balances |
| GET | `/hr/balances/comp-time` | Comp time balances |
| GET | `/hr/balances/summary` | Balance summary |
| POST | `/hr/balances/annual-entitlements` | Create entitlement |
| POST | `/hr/balances/:id/adjust` | Adjust balance |

### Dashboard

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/hr/dashboard/calendar` | Dashboard calendar data |

### Calendar Sync

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/hr/calendar/status` | Get sync status |
| GET | `/hr/calendar/config` | Get config |
| PUT | `/hr/calendar/config` | Update config |
| POST | `/hr/calendar/connect` | Connect calendar |
| POST | `/hr/calendar/disconnect` | Disconnect calendar |
| POST | `/hr/calendar/sync` | Trigger sync |
| GET | `/hr/calendar/history` | Sync history |
| GET | `/hr/calendar/pending` | Pending syncs |
| GET | `/hr/calendar/conflicts` | List conflicts |
| GET | `/hr/calendar/conflicts/:id` | Get conflict |
| POST | `/hr/calendar/conflicts/:id/resolve` | Resolve conflict |
| GET | `/hr/calendar/events` | List calendar events |

---

## 9. Admin Audit API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/admin/audit/activities` | List activity logs |
| GET | `/admin/audit/activities/user/:user_id` | User timeline |
| GET | `/admin/audit/activities/entity/:type/:id` | Entity history |
| GET | `/admin/audit/logins` | Login events |
| GET | `/admin/audit/sessions` | List sessions |
| POST | `/admin/audit/sessions/:id/logout` | Force logout |
| GET | `/admin/audit/alerts` | Security alerts |
| POST | `/admin/audit/alerts/:id/resolve` | Resolve alert |
| GET | `/admin/audit/dashboard` | Audit dashboard |

---

## 10. Facility Management API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/facilities/species` | List species |
| POST | `/facilities/species` | Create species |
| GET | `/facilities/species/:id` | Get species |
| PUT | `/facilities/species/:id` | Update species |
| DELETE | `/facilities/species/:id` | Delete species |
| GET | `/facilities` | List facilities |
| POST | `/facilities` | Create facility |
| GET | `/facilities/:id` | Get facility |
| PUT | `/facilities/:id` | Update facility |
| DELETE | `/facilities/:id` | Delete facility |
| GET | `/facilities/buildings` | List buildings |
| POST | `/facilities/buildings` | Create building |
| GET | `/facilities/zones` | List zones |
| GET | `/facilities/pens` | List pens |
| GET | `/facilities/departments` | List departments |

---

## 11. Notifications API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/notifications` | List notifications |
| GET | `/notifications/unread-count` | Unread count |
| POST | `/notifications/read` | Mark as read |
| POST | `/notifications/read-all` | Mark all read |
| DELETE | `/notifications/:id` | Delete notification |
| GET | `/notifications/settings` | Get settings |
| PUT | `/notifications/settings` | Update settings |

---

## 12. Reports API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/reports/stock-on-hand` | On-hand report |
| GET | `/reports/stock-ledger` | Ledger report |
| GET | `/reports/purchase-lines` | Purchase report |
| GET | `/reports/sales-lines` | Sales report |
| GET | `/reports/cost-summary` | Cost report |
| GET | `/scheduled-reports` | List scheduled |
| POST | `/scheduled-reports` | Create scheduled |
| GET | `/report-history` | Report history |
| GET | `/report-history/:id/download` | Download report |

---

## 13. File Upload API

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/protocols/:id/attachments` | Protocol attachment |
| POST | `/pigs/:id/photos` | Pig photo |
| POST | `/pigs/:id/pathology/attachments` | Pathology attachment |
| POST | `/pigs/:id/sacrifice/photos` | Sacrifice photo |
| POST | `/vet-recommendations/:record_type/:record_id/attachments` | Vet rec attachment |
| GET | `/attachments` | List attachments |
| GET | `/attachments/:id` | Download attachment |
| DELETE | `/attachments/:id` | Delete attachment |

---

## 14. Alert API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/alerts/low-stock` | Low stock alerts |
| GET | `/alerts/expiry` | Expiry alerts |

---

## 15. Admin Triggers API

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/admin/trigger/low-stock-check` | Trigger low stock check |
| POST | `/admin/trigger/expiry-check` | Trigger expiry check |
| POST | `/admin/trigger/notification-cleanup` | Cleanup notifications |

---

*Next: [Permissions & RBAC](./06_PERMISSIONS_RBAC.md)*
