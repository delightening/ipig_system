-- Optimistic Locking: add version column to key tables
-- Version starts at 1 and increments on each update

ALTER TABLE animals ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE protocols ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE animal_observations ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE animal_surgeries ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
