-- 修補：已核准的 ADJ 調整單有指定儲位但未寫入 storage_location_inventory
-- 將遺漏的儲位庫存補回

INSERT INTO storage_location_inventory (id, storage_location_id, product_id, on_hand_qty, batch_no, expiry_date, updated_at)
SELECT
    gen_random_uuid(),
    dl.storage_location_id,
    dl.product_id,
    SUM(dl.qty),
    dl.batch_no,
    dl.expiry_date,
    NOW()
FROM document_lines dl
JOIN documents d ON dl.document_id = d.id
WHERE d.doc_type = 'ADJ'
  AND d.status = 'approved'
  AND dl.storage_location_id IS NOT NULL
GROUP BY dl.storage_location_id, dl.product_id, dl.batch_no, dl.expiry_date
ON CONFLICT (storage_location_id, product_id, COALESCE(batch_no, ''), COALESCE(expiry_date, '1900-01-01'::date))
DO UPDATE SET
    on_hand_qty = storage_location_inventory.on_hand_qty + EXCLUDED.on_hand_qty,
    updated_at = NOW();

-- 更新受影響儲位的 current_count
UPDATE storage_locations sl SET
    current_count = (
        SELECT COUNT(DISTINCT product_id)
        FROM storage_location_inventory
        WHERE storage_location_id = sl.id
          AND on_hand_qty > 0
    ),
    updated_at = NOW()
WHERE sl.id IN (
    SELECT DISTINCT dl.storage_location_id
    FROM document_lines dl
    JOIN documents d ON dl.document_id = d.id
    WHERE d.doc_type = 'ADJ'
      AND d.status = 'approved'
      AND dl.storage_location_id IS NOT NULL
);
