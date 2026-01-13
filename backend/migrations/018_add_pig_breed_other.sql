-- Add breed_other column to pigs table
ALTER TABLE pigs ADD COLUMN breed_other VARCHAR(100);

-- Update record_versions snapshot if needed (optional, handled by JSONB)
