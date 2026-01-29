# Version History

> **Last Updated**: 2026-01-29

---

## Document Versions

### Profiling_Spec v2.2 (2026-01-29)

**Codebase Profiling Update**

Updated documentation to reflect actual codebase implementation:

| Document | Changes |
|----------|---------|
| 04_DATABASE_SCHEMA.md | + Added migrations 011-013 (HR updates, GLP compliance) |
| 04_DATABASE_SCHEMA.md | + Added GLP Phase 2 tables (electronic_signatures, record_annotations, change_reasons) |
| 05_API_SPECIFICATION.md | + Added GLP Compliance API section (signatures, annotations) |
| 05_API_SPECIFICATION.md | + Added Protocol PDF Export API |
| 07_AUDIT_LOGGING.md | + Added GLP Phase 2 enhancements (signature logging, change reason tracking) |
| 08_ATTENDANCE_MODULE.md | + Updated overtime types (A/B/C/D system) |
| 08_ATTENDANCE_MODULE.md | + Updated comp time rules (only C/D generate comp time, 8hrs fixed) |

**New Features Documented**:
- Electronic signatures for sacrifice/observation records
- Record annotations for locked records (NOTE, CORRECTION, ADDENDUM)
- Change reason tracking for all UPDATE/DELETE operations
- Record locking after signature
- PDF export for AUP protocols
- New overtime type system (A/B/C/D replacing weekday/weekend/holiday)

**Database Migrations Added**:
- 011_hr_system_updates.sql - Overtime type refactoring
- 012_seed_attendance_records.sql - Attendance data seeding
- 013_glp_compliance.sql - Electronic signatures, annotations, locking

---

### Profiling_Spec v2.1 (2026-01-29)

**Documentation Consolidation**

Merged and simplified documentation to reduce redundancy:

| Document | Change |
|----------|--------|
| 02_CORE_DOMAIN_MODEL.md | → Merged into 04_DATABASE_SCHEMA.md (324→40 lines) |
| 04_DATABASE_SCHEMA.md | + Added ERD diagrams from database_erd.md |
| 04_DATABASE_SCHEMA.md | + Added business logic annotations |
| 06_PERMISSIONS_RBAC.md | → Merged into 03_MODULES_AND_BOUNDARIES.md (249→14 lines) |
| 08_ATTENDANCE_MODULE.md | Simplified to business rules only (398→150 lines) |
| 01_ARCHITECTURE_OVERVIEW.md | Simplified tech stack section (291→257 lines) |
| database_tables.md | ❌ Deleted (794 lines) |
| database_tables_annotated.md | ❌ Deleted (487 lines) |

**Summary**:
- Reduced total line count by ~40%
- Consolidated 4 database files into 1
- Merged RBAC into Modules document
- Focused HR module on business rules only

---

### Profiling_Spec v2.0 (2026-01-18)

**Complete Rewrite**

All specification documents have been rewritten to reflect the actual codebase implementation:

| Document | Version | Status |
|----------|---------|--------|
| 00_INDEX.md | 2.0 | ✅ Updated |
| 01_ARCHITECTURE_OVERVIEW.md | 1.0 | ✅ Created (original) |
| 02_CORE_DOMAIN_MODEL.md | 2.0 | ✅ Created |
| 03_MODULES_AND_BOUNDARIES.md | 2.0 | ✅ Created |
| 04_DATABASE_SCHEMA.md | 2.0 | ✅ Created |
| 05_API_SPECIFICATION.md | 2.0 | ✅ Created |
| 06_PERMISSIONS_RBAC.md | 2.0 | ✅ Created |
| 07_AUDIT_LOGGING.md | 1.0 | ✅ Created (original) |
| 08_ATTENDANCE_MODULE.md | 1.0 | ✅ Created (original) |
| 09_EXTENSIBILITY.md | 1.0 | ✅ Created (original) |
| 10_UI_UX_GUIDELINES.md | 2.0 | ✅ Created |
| 11_NAMING_CONVENTIONS.md | 2.0 | ✅ Created |
| 12_VERSION_HISTORY.md | 2.0 | ✅ Updated |

**Key Changes**:
- Updated index with accurate system overview and technology stack
- Created comprehensive Core Domain Model with all entities and enums
- Created Modules and Boundaries with detailed module breakdown
- Created complete Database Schema based on 10 migrations
- Created full API Specification covering 250+ endpoints
- Created Permissions & RBAC with all system roles and permissions
- Created UI/UX Guidelines for frontend development
- Created Naming Conventions for all code layers

---

### Profiling_Spec v1.0 (2026-01-17)

**Initial Release**

Created initial specification documentation framework:
- 00_INDEX.md - Index
- 01_ARCHITECTURE_OVERVIEW.md - System architecture
- 07_AUDIT_LOGGING.md - GLP-compliant audit logging
- 08_ATTENDANCE_MODULE.md - HR attendance module
- 09_EXTENSIBILITY.md - Extensibility design
- 12_VERSION_HISTORY.md - Version tracking

---

## Database Migration History

### 2026-01-18

| Migration | Description |
|-----------|-------------|
| 010_add_deleted_at_column.sql | Added deleted_at column for pig soft delete |

### 2026-01-17

| Migration | Description |
|-----------|-------------|
| 001_aup_system.sql | Core schema: users, roles, ERP, protocols, pigs, notifications |
| 002_erp_base_data.sql | SKU categories, product categories seed data |
| 003_seed_accounts.sql | Initial admin account and roles |
| 004_hr_system.sql | Attendance, overtime, leave management |
| 005_calendar_sync.sql | Google Calendar integration |
| 006_audit_system.sql | GLP-compliant audit logging with partitioned tables |
| 007_seed_data.sql | Reference data (pig sources, permissions) |
| 008_reset_admin.sql | Admin password reset utility |
| 009_add_roles_is_active.sql | Added is_active flag to roles table |

---

## Change Log Format

When updating documents, add entries in this format:

```markdown
### Document Name vX.Y (YYYY-MM-DD)

**Summary of Changes**

- Added: New section on XYZ
- Changed: Updated ABC to reflect new requirements
- Fixed: Corrected typo in DEF section
- Removed: Deprecated GHI section

**Breaking Changes** (if any)

- API endpoint `/old/path` renamed to `/new/path`

**Migration Notes** (if any)

- Run migration script `XXX.sql` before deploying
```

---

*This document is automatically updated when specification changes are made.*
