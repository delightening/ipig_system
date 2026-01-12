-- Assign EXPERIMENT_STAFF default pig management permissions (exclude vet)

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code IN (
    -- Pig basic management
    'animal.pig.view_all',
    'animal.pig.view_project',
    'animal.pig.create',
    'animal.pig.edit',
    'animal.pig.delete',
    'animal.pig.assign',
    'animal.pig.unassign',
    'animal.pig.import',
    'animal.pig.export',
    -- Records
    'animal.record.view',
    'animal.record.create',
    'animal.record.edit',
    'animal.record.delete',
    'animal.record.copy',
    'animal.record.observation',
    'animal.record.surgery',
    'animal.record.weight',
    'animal.record.vaccine',
    'animal.record.sacrifice',
    -- Exports
    'animal.export.medical',
    'animal.export.observation',
    'animal.export.surgery',
    'animal.export.experiment',
    'animal.export.all',
    -- Pathology
    'animal.pathology.view',
    'animal.pathology.upload',
    'animal.pathology.edit',
    'animal.pathology.delete'
)
ON CONFLICT DO NOTHING;
