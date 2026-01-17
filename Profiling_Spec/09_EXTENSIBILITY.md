# Extensibility Specification

> **Version**: 1.0  
> **Last Updated**: 2026-01-17  
> **Audience**: Architects, Senior Developers

---

## 1. Purpose

This document describes the extensibility architecture of iPig, enabling the system to grow to support:

- **New Animal Species** - Beyond pigs (rabbits, mice, etc.)
- **New Facilities** - Additional research centers, buildings
- **New Roles & Departments** - Organizational growth
- **New Modules** - Future functionality

The guiding principle is: **"Grow like Notion, don't disturb users"**
- Additive changes over breaking changes
- Stable navigation and workflows
- Clear migration paths
- Minimal retraining required

---

## 2. Current State Analysis

### 2.1 Animals (Pigs)

| Current | Limitation |
|---------|------------|
| `pigs` table hardcoded | Adding species requires new tables |
| `breed` as ENUM | New breeds require migrations |
| Routes `/api/pigs/*` | URLs tied to species |
| Permissions `pig.*` | Per-species permission sets |
| `ear_tag` identifier | Not all species use ear tags |

### 2.2 Facilities/Locations

| Current | Limitation |
|---------|------------|
| `pen_location` as VARCHAR | No structure (e.g., "A01") |
| Zone colors in CSS | Hardcoded, not configurable |
| Building A/B in UI | Hardcoded toggle buttons |

### 2.3 Roles/Departments

| Current | Limitation |
|---------|------------|
| Flat role model | No hierarchy |
| No departments | Can't organize by team |
| No direct manager | Can't auto-route approvals |

---

## 3. New Architecture

### 3.1 Species Abstraction

```sql
-- Generic species configuration
CREATE TABLE species (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE, -- 'pig', 'rabbit', 'mouse'
    name VARCHAR(100),        -- '豬', '兔', '小鼠'
    config JSONB              -- Species-specific settings
);

-- Config example for pigs:
{
    "breeds": ["Minipig", "White", "Other"],
    "identifier_label": "耳號",
    "identifier_format": "###",
    "has_pen_assignment": true,
    "pen_prefixes": ["A", "B", "C", "D", "E", "F", "G"]
}
```

#### Migration Strategy

1. Create `species` table with `pig` seed data
2. Add `species_id` to `pigs` table (nullable initially)
3. Backfill existing pigs with pig species ID
4. Make `species_id` NOT NULL
5. Create `animals` view over `pigs` for generic access

### 3.2 Facility Hierarchy

```
Facility (設施)
    └── Building (棟舍)
            └── Zone (區域)
                    └── Pen (欄位)
```

```sql
CREATE TABLE facilities (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE,
    name VARCHAR(100),
    address TEXT
);

CREATE TABLE buildings (
    id UUID PRIMARY KEY,
    facility_id UUID REFERENCES facilities(id),
    code VARCHAR(20),     -- 'A', 'B'
    name VARCHAR(100),    -- 'A棟', 'B棟'
    config JSONB          -- Building-specific settings
);

CREATE TABLE zones (
    id UUID PRIMARY KEY,
    building_id UUID REFERENCES buildings(id),
    code VARCHAR(20),     -- 'A', 'B', 'C'...
    color VARCHAR(20),    -- '#4CAF50' (configurable!)
    layout_config JSONB   -- Row/column arrangement
);

CREATE TABLE pens (
    id UUID PRIMARY KEY,
    zone_id UUID REFERENCES zones(id),
    code VARCHAR(20),     -- 'A01', 'A02'
    capacity INTEGER,
    status VARCHAR(20)    -- 'available', 'occupied', 'maintenance'
);
```

#### Migration Strategy

1. Create new tables without foreign keys to `pigs`
2. Seed with current A/B棟 structure
3. Add `pen_id` to `pigs` (nullable)
4. Create script to match `pen_location` → `pen_id`
5. Run automatic migration for exact matches
6. Manual review for unmatched records
7. Keep `pen_location` for backward compatibility (deprecated)

### 3.3 Department & Manager Hierarchy

```sql
CREATE TABLE departments (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE,
    name VARCHAR(100),
    parent_id UUID REFERENCES departments(id), -- Hierarchy
    manager_id UUID REFERENCES users(id)       -- Department head
);

-- Add to users
ALTER TABLE users ADD COLUMN department_id UUID;
ALTER TABLE users ADD COLUMN direct_manager_id UUID;
```

This enables:
- Automatic approval routing (user → manager → dept head)
- Department-based permissions
- Team-based views

### 3.4 Role Groups

Pre-defined bundles of roles for common job functions:

```sql
CREATE TABLE role_groups (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE,
    name VARCHAR(100),
    description TEXT
);

CREATE TABLE role_group_roles (
    role_group_id UUID REFERENCES role_groups(id),
    role_id UUID REFERENCES roles(id)
);
```

Example role groups:
- `INTERNAL_STAFF` → Basic HR permissions
- `EXPERIMENT_TEAM` → Animal management permissions
- `ADMIN_TEAM` → System administration permissions

---

## 4. Extension Points

### 4.1 API Design for Extensions

#### Current (Species-specific)
```
GET /api/pigs
GET /api/pigs/:id
POST /api/pigs
```

#### New (Generic + Alias)
```
# Generic endpoints
GET /api/animals?species=pig
GET /api/animals/:id
POST /api/animals { species_id: "...", ... }

# Backward-compatible aliases (deprecated warnings)
GET /api/pigs → redirects to /api/animals?species=pig
```

### 4.2 Permission Pattern

#### Current
```
pig.read, pig.create, pig.update, pig.delete
```

#### New
```
# Generic
animal.read, animal.create, animal.update, animal.delete

# Species-specific (optional, for fine-grained control)
animal.pig.read, animal.rabbit.read
```

### 4.3 UI Configuration

Instead of hardcoding zone colors:

```typescript
// From admin configuration or API
const zoneConfig = await fetchZoneConfig();
// { "A": { color: "#4CAF50", ... }, "B": { color: "#2196F3", ... } }
```

Facility selector becomes dynamic:
```typescript
const buildings = await fetchBuildings();
// Dynamic tabs instead of hardcoded A棟/B棟
```

---

## 5. Implementation Phases

### Phase 1: Database Foundation (Weeks 1-2) ✅

- [x] Create `species` table with pig seed
- [x] Create `facilities`, `buildings`, `zones`, `pens` tables
- [x] Seed current pen structure
- [x] Create `departments` table
- [x] Add `department_id`, `direct_manager_id` to users
- [x] Add `pen_id` to pigs
- [x] Create `pen_details` view

### Phase 2: Backend Abstraction (Weeks 3-4)

- [ ] Create `animals` view over `pigs`
- [ ] Add `/api/animals/*` routes
- [ ] Keep `/api/pigs/*` as aliases with deprecation headers
- [ ] Create `/api/facilities/*` CRUD
- [ ] Create `/api/buildings/*` CRUD
- [ ] Create `/api/zones/*` CRUD
- [ ] Create `/api/pens/*` CRUD
- [ ] Update permission checks to generic pattern

### Phase 3: Frontend Progressive Enhancement (Weeks 5-8)

- [ ] Create configurable zone color system
- [ ] Create dynamic facility/building selector
- [ ] Create facility management admin page
- [ ] Replace hardcoded pen references incrementally
- [ ] Feature flag new components

### Phase 4: Rollout & Cleanup (Weeks 9-12)

- [ ] Enable new UI for beta users
- [ ] Gather feedback and iterate
- [ ] Enable for all users
- [ ] Log usage of deprecated endpoints
- [ ] Announce deprecation timeline
- [ ] Remove deprecated code after 6 months

---

## 6. Backward Compatibility

### 6.1 Views for Legacy Queries

```sql
-- Keep pigs as the primary table name
-- OR create a view that looks like the old structure
CREATE VIEW legacy_pigs AS
SELECT 
    p.*,
    pn.code AS pen_location_normalized
FROM pigs p
LEFT JOIN pens pn ON p.pen_id = pn.id;
```

### 6.2 Route Aliases

```rust
// In routes.rs
.route("/pigs", get(handlers::list_pigs_deprecated))  // Logs deprecation
.route("/animals", get(handlers::list_animals))        // New endpoint
```

### 6.3 Deprecation Headers

```rust
fn add_deprecation_header(response: &mut Response) {
    response.headers_mut().insert(
        "Deprecation", 
        HeaderValue::from_static("true")
    );
    response.headers_mut().insert(
        "Sunset",
        HeaderValue::from_static("2027-01-01")
    );
}
```

---

## 7. Adding a New Species (Example)

### 7.1 Database Changes

```sql
-- Add species
INSERT INTO species (code, name, config) VALUES (
    'rabbit', 
    '兔',
    '{"breeds": ["NZW", "JW"], "identifier_label": "耳號"}'
);

-- Create species-specific table (optional, if schema differs significantly)
-- OR just add to animals table if schema is similar enough
```

### 7.2 Backend Changes

If using existing `pigs` schema:
- Just add species_id filter in queries

If species needs custom fields:
- Create new table with common + custom columns
- Extend animal handlers to route to correct table

### 7.3 Frontend Changes

```typescript
// Already dynamic if using config-based approach
const species = await fetchSpecies();
// Renders species selector automatically
```

---

## 8. Adding a New Facility (Example)

### 8.1 Admin UI

1. Go to **系統管理 → 設施管理**
2. Click **新增設施**
3. Enter name, address
4. Add buildings under facility
5. Add zones under buildings
6. Add pens under zones

### 8.2 Data Entry

```sql
INSERT INTO facilities (code, name) VALUES ('BRANCH2', '二廠');
INSERT INTO buildings (facility_id, code, name) VALUES (?, 'A', 'A棟');
INSERT INTO zones (building_id, code, color) VALUES (?, 'A', '#4CAF50');
-- ... pens
```

### 8.3 UI Updates

No code changes needed if using dynamic facility loading:
- Facility selector automatically shows new facility
- Building tabs automatically populate
- Pen grid automatically generated from config

---

## 9. Navigation Evolution

### Current Structure
```
📊 儀表板
📋 動物使用計畫 (AUP)
🐷 動物管理
  └─ 豬隻
  └─ 我的計劃
📦 ERP
⚙️ 系統管理
```

### Future Structure (Additive)
```
📊 儀表板
📋 動物使用計畫 (AUP)
🐷 動物管理
  ├─ 所有動物
  ├─ 依物種瀏覽 ▶
  │   ├─ 豬隻
  │   ├─ 兔 (future)
  │   └─ 小鼠 (future)
  └─ 我的計劃
📦 ERP
🏢 設施管理 ▶ (Admin only)
  ├─ 設施/棟舍
  └─ 欄位配置
👥 人員管理 ▶ (HR)
  ├─ 出勤打卡
  └─ 請假管理
⚙️ 系統管理
  ├─ ...existing...
  └─ 安全審計 ▶ (new)
```

**Key Principle**: Existing navigation items stay in place. New items are added at the end or as sub-items.

---

## 10. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Data migration errors | Run migration on staging first; manual review for edge cases |
| Performance regression | Test query performance; add indexes proactively |
| User confusion | Phase rollout; collect feedback; provide training |
| Breaking integrations | 6-month deprecation period for old endpoints |
| Incomplete migration | Track unmigrated records; provide admin tools |

---

## 11. Success Metrics

| Metric | Target |
|--------|--------|
| Old endpoint usage | <10% after 3 months |
| New UI adoption | >90% after 1 month |
| Support tickets | No increase during rollout |
| Query performance | Same or better than current |
| User satisfaction | No complaints about navigation changes |

---

## 12. Related Documents

- [Database Schema](./04_DATABASE_SCHEMA.md) - Table definitions
- [API Specification](./05_API_SPECIFICATION.md) - Endpoint details
- [UI/UX Guidelines](./10_UI_UX_GUIDELINES.md) - Navigation principles

---

*Last updated: 2026-01-17*
