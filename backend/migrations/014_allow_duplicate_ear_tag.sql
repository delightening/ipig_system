-- Allow duplicate ear_tag across different birth dates

ALTER TABLE pigs DROP CONSTRAINT IF EXISTS pigs_ear_tag_key;
