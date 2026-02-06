-- Rename role CHAIR to IACUC_CHAIR to maintain consistency with the code
UPDATE roles SET code = 'IACUC_CHAIR', name = 'IACUC 主席' WHERE code = 'CHAIR';
