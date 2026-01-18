# iPig System Specification Index

> **Version**: 2.0  
> **Last Updated**: 2026-01-18  
> **Maintainer**: Development Team

---

## System Overview

**iPig (豬博士動物科技系統)** is an integrated experimental animal management platform with four subsystems:

| Subsystem | Description |
|-----------|-------------|
| **AUP Review System** | IACUC Animal Use Protocol submission, review, and approval workflow |
| **iPig ERP** | Inventory, procurement, SKU management, and cost tracking |
| **Animal Management System** | Pig lifecycle tracking, experiments, medical records, vet recommendations |
| **HR/Personnel System** | Attendance, leave, overtime, comp time, Google Calendar sync |

---

## Document Index

### Core Architecture

| Document | Audience | Description |
|----------|----------|-------------|
| [Architecture Overview](./01_ARCHITECTURE_OVERVIEW.md) | All | System design, tech stack, deployment |
| [Core Domain Model](./02_CORE_DOMAIN_MODEL.md) | Developers | Entities, relationships, enums |
| [Modules and Boundaries](./03_MODULES_AND_BOUNDARIES.md) | Architects | Module decomposition, bounded contexts |

### Technical Specifications

| Document | Audience | Description |
|----------|----------|-------------|
| [Database Schema](./04_DATABASE_SCHEMA.md) | DB Admins, Devs | Table definitions, migrations |
| [API Specification](./05_API_SPECIFICATION.md) | Frontend, Integration | Complete REST API reference |
| [Permissions & RBAC](./06_PERMISSIONS_RBAC.md) | Admins, Devs | Role-based access control |

### Module Specifications

| Document | Audience | Description |
|----------|----------|-------------|
| [Audit & Logging](./07_AUDIT_LOGGING.md) | Security, Compliance | GLP-compliant activity tracking |
| [HR Module](./08_ATTENDANCE_MODULE.md) | HR, Developers | Attendance, leave, overtime, calendar sync |
| [Extensibility](./09_EXTENSIBILITY.md) | Architects | Extension points, future expansion |

### Guidelines

| Document | Audience | Description |
|----------|----------|-------------|
| [UI/UX Guidelines](./10_UI_UX_GUIDELINES.md) | Designers, Frontend | Design patterns |
| [Naming Conventions](./11_NAMING_CONVENTIONS.md) | All Developers | Naming standards |
| [Version History](./12_VERSION_HISTORY.md) | All | Document changelog |

---

## Quick Reference

### Technology Stack

| Layer | Technology |
|-------|------------|
| Frontend | React 18, TypeScript, Vite 5, TailwindCSS, shadcn/ui, Zustand, React Query |
| Backend | Rust 1.75+, Axum 0.7, SQLx 0.7, Tokio, Serde |
| Database | PostgreSQL 15 |
| Authentication | JWT (Access + Refresh tokens) |
| Deployment | Docker, Docker Compose, Nginx |

### Key Roles

| Role Code | Chinese Name | Description |
|-----------|--------------|-------------|
| `admin` | 系統管理員 | Full system access |
| `iacuc_staff` | 執行秘書 | Protocol management, HR access |
| `experiment_staff` | 試驗工作人員 | Animal records, experiments |
| `vet` | 獸醫師 | Animal health, recommendations |
| `warehouse` | 倉庫管理員 | ERP operations |
| `pi` | 計畫主持人 | Protocol submission |
| `client` | 委託人 | View commissioned projects |

### API Base URL

- **Development**: `http://localhost:8080/api`
- **Production**: `https://ipig.example.com/api`

### Database Enums (Key)

| Enum | Values |
|------|--------|
| `pig_status` | unassigned, assigned, in_experiment, completed, transferred, deceased |
| `pig_breed` | miniature, white, LYD, other |
| `protocol_status` | DRAFT, SUBMITTED, PRE_REVIEW, UNDER_REVIEW, REVISION_REQUIRED, RESUBMITTED, APPROVED, APPROVED_WITH_CONDITIONS, DEFERRED, REJECTED, SUSPENDED, CLOSED, DELETED |
| `leave_type` | ANNUAL, PERSONAL, SICK, COMPENSATORY, MARRIAGE, BEREAVEMENT, MATERNITY, PATERNITY, MENSTRUAL, OFFICIAL, UNPAID |
| `leave_status` | DRAFT, PENDING_L1, PENDING_L2, PENDING_HR, PENDING_GM, APPROVED, REJECTED, CANCELLED, REVOKED |
| `doc_type` | PO, GRN, PR, SO, DO, SR, TR, STK, ADJ, RTN |

---

## Directory Structure

```
ipig_system/
├── backend/
│   ├── src/
│   │   ├── handlers/        # HTTP handlers
│   │   ├── services/        # Business logic
│   │   ├── models/          # Data models
│   │   ├── middleware/      # Auth, logging
│   │   ├── routes.rs        # Route definitions
│   │   └── main.rs          # Application entry
│   └── migrations/          # SQL migrations (001-010)
├── frontend/
│   └── src/
│       ├── components/      # Reusable components
│       ├── pages/           # Page components
│       ├── stores/          # Zustand stores
│       ├── types/           # TypeScript types
│       └── lib/             # Utilities
├── docker-compose.yml
└── Profiling_Spec/          # This documentation
```

---

*Last updated: 2026-01-18*
