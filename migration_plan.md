# v1.0 Schema Baseline Migration Plan

## Goals
- Freeze a stable, reproducible schema baseline for v1.0.
- Separate core auth/identity from domain modules.
- Provide a clear re-baselining path for new environments.

## Principles
- One baseline migration that can stand alone for a fresh install.
- Keep incremental migrations after baseline small and focused.
- Preserve data safety for existing environments.

## Proposed Cut Strategy

### Cut 0: Pre-baseline cleanup (optional, one-time)
Purpose: normalize any legacy or experimental migrations before freezing v1.0.
- Verify all existing migrations apply cleanly to an empty database.
- Resolve any inconsistent constraints, defaults, or types.
- Ensure seed data is deterministic (no conflicting codes).

Deliverable:
- A tag or branch with a clean, verified migration chain.

### Cut 1: v1.0 Baseline (single migration)
Purpose: one file that defines the full v1.0 schema and seed data.
- Create all tables, enums, indexes, constraints.
- Insert required seed data (roles, permissions, system config defaults).
- Ensure idempotency only if needed for local dev; otherwise baseline is single-run.

Deliverable:
- `001_v1_0_baseline.sql` (or similar) containing full schema.

### Cut 2: Post-baseline delta migrations
Purpose: all new work lands as incremental migrations after v1.0.
- Each migration addresses one coherent change.
- Add forward-only migrations (avoid re-editing baseline).

Deliverable:
- `002_...` onward, per feature/bugfix.

## Implementation Steps

### Step 1: Inventory current schema
- Export schema from a clean database after applying current migrations.
- Diff against `backend/migrations` to identify missing elements.

### Step 2: Generate baseline migration
- Create a new SQL file that includes:
  - Extensions (e.g. `pgcrypto` if used).
  - Enums
  - Tables
  - Indexes and constraints
  - Views/triggers if any
  - Seed data

### Step 3: Baseline verification
- Create a new database.
- Apply baseline only.
- Run a minimal smoke check (login, admin list, etc.).

### Step 4: Rebase existing environments
- Option A (new env only): use baseline for fresh installs.
- Option B (existing envs): keep existing migration chain; do not reapply baseline.
- Document the split: legacy chain vs baseline.

## Suggested File Layout

```
backend/migrations/
  001_v1_0_baseline.sql
  002_...
```

## Notes and Risks
- If `sqlx` uses the migration table for tracking, do not retroactively alter the baseline once deployed.
- Seed data must avoid duplicate unique keys across environments.
- Ensure default admin creation is handled either by seed data or a separate admin bootstrap task.

## Open Questions
- Should baseline include demo data, or only essential seeds?
- Do we need to preserve existing migration numbering, or restart at 001 for v1.0?
- Is there a requirement to support both Docker and local dev baselines?
