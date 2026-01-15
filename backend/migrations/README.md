# Database Migrations Policy

This directory contains immutable database history.

Rules:
- Existing migration files MUST NEVER be edited.
- All changes require a new migration.
- Corrections are done via forward migrations.
- History is append-only.

Rationale:
Production databases may already depend on these migrations.
Changing them would cause irreversible inconsistencies.
