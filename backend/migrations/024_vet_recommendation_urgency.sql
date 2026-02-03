-- Migration: Vet Recommendation Urgency Field
-- Description: Add is_urgent flag to vet_recommendations for notification channel control

-- Add urgency flag
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS is_urgent BOOLEAN DEFAULT false;

-- Add comment
COMMENT ON COLUMN vet_recommendations.is_urgent IS 'Is urgent recommendation - triggers both in-app and email notifications';

-- Create index for urgent recommendations query
CREATE INDEX IF NOT EXISTS idx_vet_recommendations_urgent ON vet_recommendations(is_urgent) WHERE is_urgent = true;
