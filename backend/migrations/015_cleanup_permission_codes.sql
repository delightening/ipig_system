BEGIN;

ALTER TABLE permissions ADD COLUMN IF NOT EXISTS name VARCHAR(200);
UPDATE permissions SET name = code WHERE name IS NULL;

CREATE TEMP TABLE perm_map AS
SELECT
    id AS old_id,
    code AS old_code,
    name,
    description,
    CASE
        WHEN code LIKE 'animal.%' THEN 'pig.' || substr(code, 8)
        WHEN code LIKE 'admin.%' THEN 'dev.' || substr(code, 7)
        WHEN code = 'user.create' THEN 'dev.user.create'
        WHEN code = 'user.read' THEN 'dev.user.view'
        WHEN code = 'user.update' THEN 'dev.user.edit'
        WHEN code = 'user.delete' THEN 'dev.user.delete'
        WHEN code LIKE 'role.%' THEN CASE
            WHEN code = 'role.create' THEN 'dev.role.create'
            WHEN code = 'role.read' THEN 'dev.role.view'
            WHEN code = 'role.update' THEN 'dev.role.edit'
            WHEN code = 'role.delete' THEN 'dev.role.delete'
            ELSE 'dev.role.' || substr(code, 6)
        END
        WHEN code LIKE 'notification.%' THEN 'dev.notification.' || substr(code, 14)
        WHEN code LIKE 'warehouse.%' THEN 'erp.warehouse.' || substr(code, 11)
        WHEN code LIKE 'product.%' THEN 'erp.product.' || substr(code, 9)
        WHEN code LIKE 'partner.%' THEN 'erp.partner.' || substr(code, 9)
        WHEN code LIKE 'document.%' THEN CASE
            WHEN code = 'document.read' THEN 'erp.document.view'
            WHEN code = 'document.update' THEN 'erp.document.edit'
            WHEN code = 'document.create' THEN 'erp.document.create'
            WHEN code = 'document.delete' THEN 'erp.document.delete'
            WHEN code = 'document.submit' THEN 'erp.document.submit'
            WHEN code = 'document.approve' THEN 'erp.document.approve'
            WHEN code = 'document.cancel' THEN 'erp.document.cancel'
            ELSE 'erp.document.' || substr(code, 10)
        END
        WHEN code LIKE 'po.%' THEN CASE
            WHEN code = 'po.create' THEN 'erp.purchase.create'
            WHEN code = 'po.approve' THEN 'erp.purchase.approve'
            ELSE 'erp.purchase.' || substr(code, 4)
        END
        WHEN code LIKE 'so.%' THEN CASE
            WHEN code = 'so.create' THEN 'erp.sales.create'
            WHEN code = 'so.approve' THEN 'erp.sales.approve'
            ELSE 'erp.sales.' || substr(code, 4)
        END
        WHEN code LIKE 'grn.%' THEN 'erp.grn.' || substr(code, 5)
        WHEN code LIKE 'pr.%' THEN 'erp.pr.' || substr(code, 4)
        WHEN code LIKE 'do.%' THEN 'erp.do.' || substr(code, 4)
        WHEN code LIKE 'sr.%' THEN 'erp.sr.' || substr(code, 4)
        WHEN code LIKE 'tr.%' THEN 'erp.stock.transfer'
        WHEN code LIKE 'stk.%' THEN 'erp.stocktake.create'
        WHEN code LIKE 'adj.%' THEN 'erp.stock.adjust'
        WHEN code LIKE 'stock.%' THEN CASE
            WHEN code = 'stock.read' THEN 'erp.stock.view'
            WHEN code = 'stock.adjust' THEN 'erp.stock.adjust'
            ELSE 'erp.stock.' || substr(code, 7)
        END
        WHEN code LIKE 'report.%' THEN CASE
            WHEN code = 'report.read' THEN 'erp.report.view'
            WHEN code = 'report.export' THEN 'erp.report.export'
            WHEN code = 'report.schedule' THEN 'erp.report.schedule'
            WHEN code = 'report.download' THEN 'erp.report.download'
            ELSE 'erp.report.' || substr(code, 8)
        END
        ELSE NULL
    END AS new_code,
    CASE
        WHEN code LIKE 'animal.%' THEN 'pig'
        WHEN code LIKE 'admin.%' OR code LIKE 'user.%' OR code LIKE 'role.%' OR code LIKE 'notification.%' THEN 'dev'
        WHEN code LIKE 'warehouse.%' OR code LIKE 'product.%' OR code LIKE 'partner.%' OR code LIKE 'document.%'
            OR code LIKE 'po.%' OR code LIKE 'grn.%' OR code LIKE 'pr.%' OR code LIKE 'so.%'
            OR code LIKE 'do.%' OR code LIKE 'sr.%' OR code LIKE 'tr.%' OR code LIKE 'stk.%'
            OR code LIKE 'adj.%' OR code LIKE 'stock.%' OR code LIKE 'report.%' THEN 'erp'
        ELSE NULL
    END AS new_module
FROM permissions
WHERE code LIKE 'animal.%'
    OR code LIKE 'admin.%'
    OR code LIKE 'user.%'
    OR code LIKE 'role.%'
    OR code LIKE 'notification.%'
    OR code LIKE 'warehouse.%'
    OR code LIKE 'product.%'
    OR code LIKE 'partner.%'
    OR code LIKE 'document.%'
    OR code LIKE 'po.%'
    OR code LIKE 'grn.%'
    OR code LIKE 'pr.%'
    OR code LIKE 'so.%'
    OR code LIKE 'do.%'
    OR code LIKE 'sr.%'
    OR code LIKE 'tr.%'
    OR code LIKE 'stk.%'
    OR code LIKE 'adj.%'
    OR code LIKE 'stock.%'
    OR code LIKE 'report.%';

INSERT INTO permissions (id, code, name, module, description, created_at)
SELECT gen_random_uuid(), new_code, name, new_module, description, NOW()
FROM perm_map
WHERE new_code IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM permissions p2 WHERE p2.code = perm_map.new_code);

UPDATE permissions p
SET module = pm.new_module
FROM perm_map pm
WHERE p.code = pm.new_code
  AND pm.new_module IS NOT NULL;

INSERT INTO role_permissions (role_id, permission_id)
SELECT DISTINCT rp.role_id, p_new.id
FROM role_permissions rp
JOIN perm_map pm ON rp.permission_id = pm.old_id
JOIN permissions p_new ON p_new.code = pm.new_code
ON CONFLICT DO NOTHING;

DELETE FROM role_permissions
WHERE permission_id IN (SELECT old_id FROM perm_map);

DELETE FROM permissions
WHERE code IN (SELECT old_code FROM perm_map);

COMMIT;
