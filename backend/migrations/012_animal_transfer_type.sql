-- 轉讓類型：external = 轉給其他機構（完成時清空欄位），internal = 仍在機構內（保留欄位）
ALTER TABLE animal_transfers
ADD COLUMN IF NOT EXISTS transfer_type VARCHAR(20) NOT NULL DEFAULT 'internal';

COMMENT ON COLUMN animal_transfers.transfer_type IS 'external: 轉給其他機構; internal: 仍在機構內';
