# Daily Work Summary - February 2, 2026

## 🐷 iPig System Development

### Database & Backend Improvements

- **Migrated Pig IDs to UUID** - Converted pig module primary keys from INTEGER to UUID, updating database schemas, backend models, handlers, services, and frontend components while retaining integer `pig_no` for display purposes.

- **Added GIN Index Optimization** - Implemented GIN indexes on JSONB columns (`protocols.working_content` and `pig_observations.treatments`) for improved query performance.

- **Fixed Array Foreign Key Integrity** - Addressed integrity issues with `comp_time_source_ids` in `leave_requests` table by relying on the `leave_balance_usage` association table.

- **Automated Partition Maintenance** - Implemented scheduler to automatically create new partition tables for `user_activity_logs` to prevent insertion failures.

- **Fixed Compilation Errors** - Resolved `display_name` field error in `animal.rs` and corrected type mismatch (`Uuid` vs `i32`) in `create_weight` function.

---

### Animal Management Features

- **Emergency Medication Notifications** - Updated animal handlers to trigger emergency notifications, alerting veterinary staff and PIs in critical situations.

- **Quick Add Animal Dialog** - Implemented dialog for manually adding animals when ear tag is not found, with auto-formatted ear tag and fields for breed, gender, entry date, and birth date.

---

### Protocol & Review System

- **Reviewer Anonymization** - Implemented frontend logic to display reviewer names as "Reviewer A", "Reviewer B", etc. when viewed by PI or CLIENT roles (actual names shown for IACUC_STAFF, IACUC_CHAIR, SYSTEM_ADMIN).

- **AUP Form Translation** - Translated all section headers and subtitles in the AUP protocol editing form (`ProtocolEditPage.tsx`) from Chinese to English.

---

### Permission & Role Management

- **Consolidated Permission Categories** - Reorganized and translated permission categories, merging duplicate "Other" categories and grouping CRUD operations under parent modules. Established hierarchical structure:
  - 動物使用計畫
  - 動物管理
  - 庫存管理
  - 管理階級
  - 系統管理
  - 開發工具

---

### UI Implementation

- **Emergency Medication UI** - Developed button on Pig Detail Page with input dialog.
- **Euthanasia Workflow UI** - Implemented order creation, PI approval/appeal, and CHAIR arbitration interfaces.

---

## Summary

Today's focus was primarily on **database modernization** (UUID migration, indexing, partitioning) and **animal management features** (emergency notifications, euthanasia workflows). Significant progress was also made on the **protocol review system** with anonymization and translation work.
