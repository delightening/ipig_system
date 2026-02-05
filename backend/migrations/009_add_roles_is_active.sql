-- ============================================
-- Migration 009: Add is_active column to roles table
-- 
-- The roles table was created with is_deleted but the
-- code expects is_active. This migration adds the 
-- is_active column.
-- ============================================

-- Add is_active column if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'roles' AND column_name = 'is_active'
    ) THEN
        ALTER TABLE roles ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true;
        
        -- Set is_active based on is_deleted (inverse relationship)
        UPDATE roles SET is_active = NOT is_deleted;
    END IF;
END $$;
