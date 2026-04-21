# iPig System 架構文件

## 1. 部署架構

```mermaid
graph TB
    subgraph Internet
        Browser[瀏覽器]
    end

    subgraph Docker Host
        subgraph "frontend network"
            CF[Cloudflare Tunnel + WAF]
            Web[Nginx<br/>ipig-web :8080]
            WebDev[Vite Dev Server<br/>ipig-web-dev :5173]
        end

        subgraph "backend network"
            API[Rust API<br/>ipig-api :3000]
            Prometheus[Prometheus<br/>:9090]
            Grafana[Grafana<br/>:3001]
            Alertmanager[Alertmanager<br/>:9093]
        end

        subgraph "database network"
            DB[(PostgreSQL 16<br/>ipig-db :5432)]
            Backup[DB Backup<br/>cron + pg_dump]
        end
    end

    subgraph External
        Google[Google Calendar API]
        SMTP[SMTP Server]
    end

    Browser -->|HTTPS| CF
    CF --> Web
    Browser -->|dev| WebDev
    Web --> API
    WebDev --> API
    API --> DB
    API --> Google
    API --> SMTP
    Prometheus -->|scrape /metrics| API
    Alertmanager --> Prometheus
    Grafana --> Prometheus
    Backup --> DB
```

## 2. 資料流

```mermaid
sequenceDiagram
    participant B as Browser
    participant N as Nginx
    participant A as API (Axum)
    participant MW as Middleware
    participant H as Handler
    participant S as Service
    participant D as PostgreSQL

    B->>N: HTTP Request
    N->>A: Proxy /api/*
    A->>MW: Rate Limiter
    MW->>MW: Auth (JWT)
    MW->>MW: CSRF Check
    MW->>H: Handler
    H->>S: Business Logic
    S->>D: SQL Query
    D-->>S: Result
    S-->>H: Response
    H-->>B: JSON Response

    Note over MW: Write endpoints: 120/min<br/>Upload endpoints: 30/min<br/>Auth endpoints: 30/min<br/>General API: 600/min
```

## 3. 模組架構

```mermaid
graph LR
    subgraph Frontend["Frontend (React + TypeScript)"]
        Pages[Pages]
        Components[Components]
        Stores[Zustand Stores]
        API_Client[API Client<br/>Axios]
        QueryKeys[Query Key Factory]
        i18n[i18n<br/>zh-TW / en]
    end

    subgraph Backend["Backend (Rust + Axum)"]
        Routes[Routes]
        Middleware[Middleware<br/>Auth / CSRF / Rate Limit]
        Handlers[Handlers]
        Services[Services]
        Models[Models / Types]
        Scheduler[Scheduler<br/>tokio-cron]
    end

    subgraph DB["PostgreSQL"]
        Core[核心表<br/>users, roles, permissions]
        Animal[動物管理<br/>animals, observations,<br/>surgeries, weights]
        Protocol[AUP 計畫書<br/>protocols, reviews,<br/>amendments]
        HR[人資模組<br/>attendance, leaves,<br/>overtime]
        ERP[進銷存<br/>products, documents,<br/>stock_ledger]
        Facility[設施管理<br/>buildings, zones, pens]
        Audit[稽核/安全<br/>audit_logs, sessions,<br/>security_alerts]
    end

    Pages --> Components
    Pages --> API_Client
    Components --> Stores
    API_Client --> Routes
    Routes --> Middleware
    Middleware --> Handlers
    Handlers --> Services
    Services --> Models
    Services --> DB
    Scheduler --> Services
```

## 4. 認證流程

```mermaid
sequenceDiagram
    participant U as User
    participant F as Frontend
    participant A as API

    U->>F: 輸入帳密
    F->>A: POST /api/auth/login
    alt 需要 2FA
        A-->>F: { requires_2fa: true, temp_token }
        F->>U: 顯示 TOTP 輸入
        U->>F: 輸入 TOTP 代碼
        F->>A: POST /api/auth/2fa/verify
    end
    A-->>F: { token, refresh_token, user }
    F->>F: 儲存 JWT (httpOnly cookie)
    F->>F: Zustand store 更新

    loop 每次 API 請求
        F->>A: Authorization: Bearer <JWT>
        A->>A: 驗證 JWT 簽章 + 過期時間
        A->>A: 提取 claims (user_id, roles, permissions)
        alt Token 過期
            F->>A: POST /api/auth/refresh
            A-->>F: 新 JWT
        end
    end
```

## 5. 技術堆疊

| 層級 | 技術 |
|------|------|
| **前端** | React 19, TypeScript, Vite, TailwindCSS, shadcn/ui |
| **狀態管理** | Zustand (auth/UI), TanStack Query (server state) |
| **動畫/圖表** | Framer Motion, Recharts |
| **圖示** | Lucide React |
| **後端** | Rust, Axum, SQLx, Tokio |
| **後端輔助** | lettre (Email), utoipa (OpenAPI), totp-rs (2FA), tower-http (CORS/壓縮) |
| **資料庫** | PostgreSQL 16, Redis 7, pg_stat_statements |
| **認證** | JWT + Refresh Token + TOTP 2FA (admin) |
| **安全** | CSRF tokens, Rate limiting, DOMPurify, Argon2 hashing |
| **容器** | Docker Compose, 三層網路隔離, Docker Secrets |
| **監控** | Prometheus, Grafana, Alertmanager |
| **WAF** | Cloudflare WAF（經 Cloudflare Tunnel） |
| **CI/CD** | GitHub Actions, Dependabot, cargo-chef 快取 |

## 6. 目錄結構

```
ipig_system/
├── backend/
│   ├── src/
│   │   ├── config.rs            # 環境變數 + Docker Secrets
│   │   ├── constants.rs         # 應用常數 (ETAG_VERSION, APP_NAME 等)
│   │   ├── error.rs             # AppError 統一錯誤
│   │   ├── routes.rs            # 路由定義
│   │   ├── handlers/            # HTTP 處理器（按模組分資料夾）
│   │   ├── services/            # 業務邏輯
│   │   ├── middleware/          # Auth, CSRF, ETag, Rate Limiter
│   │   ├── models/              # DB 型別 + Request/Response
│   │   └── bin/                 # CLI 工具 (create_admin)
│   ├── migrations/              # SQL 遷移腳本 (001–010)
│   └── Dockerfile               # 多階段 cargo-chef 建置
├── frontend/
│   ├── src/
│   │   ├── pages/               # 路由頁面
│   │   ├── components/          # 共用/模組元件
│   │   ├── lib/                 # api, sanitize, queryKeys, validations
│   │   ├── stores/              # Zustand stores
│   │   ├── types/               # TypeScript 型別
│   │   └── locales/             # i18n (zh-TW, en)
│   └── Dockerfile
├── monitoring/
│   ├── prometheus/              # 告警規則 + 抓取設定
│   └── alertmanager/            # 告警路由設定
├── .github/
│   └── workflows/               # CI/CD (GitHub Actions)
├── docs/
│   ├── ARCHITECTURE.md          # 本文件
│   ├── TODO.md / PROGRESS.md    # 待辦與進度追蹤
│   ├── spec/          # 系統規格書（API/DB/RBAC/模組）
│   ├── development/             # 改善計劃 (R1–R7)
│   ├── ops/              # 運維手冊
│   ├── security/     # 安全合規文件
│   ├── db/                # DB 匯入匯出與回滾
│   └── runbooks/                # DR 演練與回滾流程
├── docker-compose.yml           # 核心服務
├── docker-compose.prod.yml      # 生產環境覆蓋
└── docker-compose.monitoring.yml # 監控堆疊
```
