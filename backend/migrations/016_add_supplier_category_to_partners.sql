DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'supplier_category') THEN
        CREATE TYPE supplier_category AS ENUM ('drug', 'consumable', 'feed', 'equipment');
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'partners' AND COLUMN_NAME = 'supplier_category') THEN
        ALTER TABLE partners ADD COLUMN supplier_category supplier_category;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_partners_supplier_category ON partners(supplier_category);
