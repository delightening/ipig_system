-- ============================================
-- Migration 010: Add deleted_at column to pigs table
-- 
-- This migration ensures the deleted_at column exists
-- for soft delete functionality (used by SQL queries with
-- 'deleted_at IS NULL').
-- ============================================

-- Add deleted_at column to pigs table if it doesn't exist
ALTER TABLE pigs ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Create index for soft delete queries if it doesn't exist
CREATE INDEX IF NOT EXISTS idx_pigs_deleted_at ON pigs(deleted_at) WHERE deleted_at IS NULL;
