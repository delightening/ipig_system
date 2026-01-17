# iPig System Specification Index

> **Version**: 1.0  
> **Last Updated**: 2026-01-17  
> **Maintainer**: Development Team

---

## System Overview

iPig (豬博士動物科技系統) is an integrated experimental animal management platform with four subsystems:

1. **AUP Review System** - IACUC Animal Use Protocol submission and review
2. **iPig ERP** - Inventory and procurement management
3. **Animal Management System** - Pig tracking, experiments, and medical records
4. **HR/Personnel System** - Attendance, leave, and time-off management

---

## Document Index

### Core Architecture

| Document | Audience | Description |
|----------|----------|-------------|
| [Architecture Overview](./01_ARCHITECTURE_OVERVIEW.md) | All | System design, technology stack, deployment |
| [Core Domain Model](./02_CORE_DOMAIN_MODEL.md) | Developers | Entities, relationships, domain logic |
| [Modules and Boundaries](./03_MODULES_AND_BOUNDARIES.md) | Architects | Module decomposition, bounded contexts |

### Technical Specifications

| Document | Audience | Description |
|----------|----------|-------------|
| [Database Schema](./04_DATABASE_SCHEMA.md) | DB Admins, Developers | Table definitions, indexes, migrations |
| [API Specification](./05_API_SPECIFICATION.md) | Frontend, Integration | REST API reference |
| [Permissions & RBAC](./06_PERMISSIONS_RBAC.md) | Admins, Developers | Role-based access control |

### Module Specifications

| Document | Audience | Description |
|----------|----------|-------------|
| [Audit & Logging](./07_AUDIT_LOGGING.md) | Security, Compliance | Activity tracking, GLP compliance |
| [Attendance Module](./08_ATTENDANCE_MODULE.md) | HR, Developers | Timekeeping, leave, overtime |
| [Extensibility](./09_EXTENSIBILITY.md) | Architects | Extension points, abstraction layers |

### Guidelines

| Document | Audience | Description |
|----------|----------|-------------|
| [UI/UX Guidelines](./10_UI_UX_GUIDELINES.md) | Designers, Frontend | Design principles, patterns |
| [Naming Conventions](./11_NAMING_CONVENTIONS.md) | All Developers | Code, DB, API naming standards |
| [Version History](./12_VERSION_HISTORY.md) | All | Document change log |

---

## Quick Reference

### Technology Stack

| Layer | Technology |
|-------|------------|
| Frontend | React 18, TypeScript, Vite, TailwindCSS |
| Backend | Rust, Axum, SQLx |
| Database | PostgreSQL 15 |
| Authentication | JWT (Access + Refresh tokens) |
| Deployment | Docker, Docker Compose |

### Key Roles

| Role Code | Name | Description |
|-----------|------|-------------|
| `admin` | 系統管理員 | Full system access |
| `iacuc_staff` | 執行秘書 | Protocol management, HR access |
| `experiment_staff` | 試驗工作人員 | Animal records, experiments |
| `vet` | 獸醫師 | Animal health, recommendations |
| `warehouse` | 倉庫管理員 | ERP operations |
| `pi` | 計畫主持人 | Protocol submission |
| `client` | 委託人 | View commissioned projects |

### API Base URL

- Development: `http://localhost:8080/api`
- Production: `https://ipig.example.com/api`

---

## Versioning Rules

All specification documents follow semantic versioning:

- **Major version** (1.x → 2.x): Breaking changes to interfaces or data models
- **Minor version** (1.1 → 1.2): New features, backward compatible additions
- **Patch version** (1.1.1 → 1.1.2): Fixes, clarifications, typo corrections

### Change Process

1. Propose changes in a new branch
2. Update relevant documents
3. Update [Version History](./12_VERSION_HISTORY.md)
4. Review and merge
5. Notify affected teams

---

## Related Documents

| Document | Location | Description |
|----------|----------|-------------|
| System Spec (Legacy) | `/_Spec.md` | Original specification (being migrated) |
| HR Implementation Plan | `/HR_SYSTEM_IMPLEMENTATION_PLAN.md` | Detailed HR module plan |
| Role Permissions | `/role.md` | Legacy role documentation |
| ERP Spec | `/ERPSpec.md` | ERP module details |
| UI Spec | `/uiSpec.md` | UI component specifications |

---

## Diagrams

Architecture and data flow diagrams are stored in the `./diagrams/` subdirectory:

- `system_architecture.mmd` - Overall system architecture
- `domain_model.mmd` - Entity relationship diagram
- `data_flow.mmd` - Data flow between subsystems
- `module_dependencies.mmd` - Module dependency graph

---

## Contact

For questions about these specifications:

- **Technical Lead**: Development Team
- **System Owner**: 豬博士動物科技有限公司

---

*This is a living document. Last updated: 2026-01-17*
