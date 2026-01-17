# Architecture Overview

> **Version**: 1.0  
> **Last Updated**: 2026-01-17  
> **Audience**: All team members

---

## 1. System Purpose

iPig (豬博士動物科技系統) is an integrated experimental animal management platform designed to:

1. **Manage IACUC Protocols** - Submit, review, and approve Animal Use Protocols (AUP)
2. **Track Experimental Animals** - Full lifecycle management from entry to completion
3. **Handle Inventory** - Procurement, stock management, and cost tracking
4. **Manage Personnel** - Attendance, leave, and time-off for internal staff

---

## 2. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          iPig Unified Portal                                 │
│                                                                             │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────────────┐  │
│  │   Login/Auth    │ ── │  Role-based     │ ── │   Module Router         │  │
│  │   (JWT)         │    │  Access Control │    │                         │  │
│  └─────────────────┘    └─────────────────┘    └─────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────────┐  │
│  │  AUP Review      │  │  iPig ERP        │  │  Animal Management       │  │
│  │  System          │  │  (Inventory)     │  │  System                  │  │
│  │                  │  │                  │  │                          │  │
│  │  • Protocol      │  │  • Procurement   │  │  • My Projects           │  │
│  │    Drafting      │  │  • Stock Mgmt    │  │  • Animal Records        │  │
│  │  • Review Flow   │  │  • Cost Track    │  │  • Health Monitoring     │  │
│  │  • Approval      │  │                  │  │  • Medical Records       │  │
│  └────────┬─────────┘  └────────┬─────────┘  └────────────┬─────────────┘  │
│           │                     │                         │                │
│  ┌────────┴─────────────────────┴─────────────────────────┴─────────────┐  │
│  │                      HR / Personnel System                            │  │
│  │                                                                       │  │
│  │  • Attendance Tracking    • Leave Management    • Overtime/Comp Time │  │
│  │  • Google Calendar Sync   • Balance Reports                          │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                           Shared Data Layer                                 │
│                                                                             │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌─────────────┐ ┌─────────────────┐  │
│  │ users   │ │ animals │ │protocols │ │  products   │ │  audit_logs     │  │
│  │         │ │ (pigs)  │ │          │ │             │ │                 │  │
│  └─────────┘ └─────────┘ └──────────┘ └─────────────┘ └─────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Technology Stack

### 3.1 Frontend

| Technology | Version | Purpose |
|------------|---------|---------|
| React | 18.x | UI Framework |
| TypeScript | 5.x | Type safety |
| Vite | 5.x | Build tool |
| React Router | 6.x | Client-side routing |
| TailwindCSS | 3.x | Utility-first CSS |
| shadcn/ui | - | Component library |
| Zustand | 4.x | State management |
| React Query | 5.x | Server state |

### 3.2 Backend

| Technology | Version | Purpose |
|------------|---------|---------|
| Rust | 1.75+ | Language |
| Axum | 0.7.x | Web framework |
| SQLx | 0.7.x | Database driver (async) |
| Tokio | 1.x | Async runtime |
| Serde | 1.x | Serialization |
| Argon2 | 0.5.x | Password hashing |
| jsonwebtoken | 9.x | JWT handling |
| lettre | 0.11.x | Email sending |

### 3.3 Database

| Technology | Version | Purpose |
|------------|---------|---------|
| PostgreSQL | 15.x | Primary database |
| Table partitioning | - | Audit log performance |
| JSONB | - | Flexible data storage |

### 3.4 Infrastructure

| Technology | Purpose |
|------------|---------|
| Docker | Containerization |
| Docker Compose | Local development orchestration |
| Nginx | Reverse proxy (production) |
| Gmail SMTP | Email delivery |

---

## 4. System Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                      Presentation Layer                          │
│  React SPA • Pages • Components • Hooks • Stores                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ HTTP/REST
┌─────────────────────────────────────────────────────────────────┐
│                        API Layer                                 │
│  Axum Router • Handlers • Middleware (Auth, Logging)            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Service Layer                               │
│  Business Logic • Validation • Orchestration                    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Data Layer                                │
│  SQLx Queries • Models • Migrations                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   PostgreSQL Database                            │
│  Tables • Views • Functions • Triggers                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. Authentication & Authorization

### 5.1 Authentication Flow

```
User                    Frontend                  Backend                 Database
  │                        │                         │                       │
  │──── Login ────────────►│                         │                       │
  │                        │──── POST /auth/login ──►│                       │
  │                        │                         │──── Verify ──────────►│
  │                        │                         │◄──── User + Roles ────│
  │                        │◄──── JWT Tokens ────────│                       │
  │◄─── Store Tokens ──────│                         │                       │
  │                        │                         │                       │
  │──── API Request ──────►│                         │                       │
  │                        │──── + Bearer Token ────►│                       │
  │                        │                         │──── Validate JWT ────►│
  │                        │                         │◄──── Claims ──────────│
  │                        │◄──── Response ──────────│                       │
  │◄─── Data ──────────────│                         │                       │
```

### 5.2 Token Structure

| Token Type | Lifetime | Storage | Purpose |
|------------|----------|---------|---------|
| Access Token | 15 minutes | Memory | API authentication |
| Refresh Token | 7 days | HttpOnly Cookie | Token renewal |

### 5.3 Authorization Model

- **Role-Based Access Control (RBAC)**
- Users can have multiple roles
- Roles contain multiple permissions
- Permissions are checked at API handler level

---

## 6. Deployment Architecture

### 6.1 Development Environment

```
┌──────────────────────────────────────────────────────────────┐
│                    Docker Compose                             │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Frontend   │  │   Backend   │  │    PostgreSQL       │  │
│  │  (Vite Dev) │  │   (Cargo)   │  │                     │  │
│  │  :5173      │  │   :8080     │  │   :5432 (internal)  │
│  │             │  │             │  │   :5433 (host)      │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### 6.2 Production Environment

```
┌──────────────────────────────────────────────────────────────┐
│                       Server                                  │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                     Nginx                                │ │
│  │  :80 (HTTP → HTTPS redirect)                            │ │
│  │  :443 (HTTPS)                                           │ │
│  └────────────────────────┬────────────────────────────────┘ │
│                           │                                  │
│           ┌───────────────┴───────────────┐                 │
│           ▼                               ▼                 │
│  ┌─────────────────┐           ┌─────────────────┐         │
│  │  Static Files   │           │    API Proxy    │         │
│  │  (React Build)  │           │    /api → :8080 │         │
│  └─────────────────┘           └────────┬────────┘         │
│                                         │                   │
│                                         ▼                   │
│                              ┌─────────────────┐            │
│                              │    Backend      │            │
│                              │    Container    │            │
│                              │    :8080        │            │
│                              └────────┬────────┘            │
│                                       │                     │
│                                       ▼                     │
│                              ┌─────────────────┐            │
│                              │   PostgreSQL    │            │
│                              │   Container     │            │
│                              └─────────────────┘            │
└──────────────────────────────────────────────────────────────┘
```

---

## 7. Key Design Decisions

### 7.1 Why Rust for Backend?

- **Performance**: Low latency for API responses
- **Memory Safety**: Prevents common security vulnerabilities
- **Type System**: Catches errors at compile time
- **Async Support**: Handles concurrent requests efficiently

### 7.2 Why PostgreSQL?

- **JSONB Support**: Flexible schema for varying data structures
- **Table Partitioning**: Efficient audit log storage
- **Advanced Features**: Window functions, CTEs, triggers
- **Reliability**: ACID compliance, mature ecosystem

### 7.3 Why JWT?

- **Stateless**: No session storage required
- **Scalable**: Easy horizontal scaling
- **Standard**: Well-understood, library support
- **Flexible**: Can include custom claims

---

## 8. Cross-Cutting Concerns

### 8.1 Logging

- **Request Logging**: All API requests logged with timing
- **Activity Logging**: User actions tracked for audit
- **Error Logging**: Structured error capture

### 8.2 Security

- **HTTPS Only**: All production traffic encrypted
- **CORS**: Restricted to allowed origins
- **Rate Limiting**: Prevent abuse (planned)
- **Input Validation**: Server-side validation on all inputs

### 8.3 Monitoring (Planned)

- Health check endpoints
- Performance metrics
- Error tracking

---

## 9. Related Documents

- [Core Domain Model](./02_CORE_DOMAIN_MODEL.md) - Entity details
- [Database Schema](./04_DATABASE_SCHEMA.md) - Table definitions
- [API Specification](./05_API_SPECIFICATION.md) - Endpoint reference

---

*Next: [Core Domain Model](./02_CORE_DOMAIN_MODEL.md)*
