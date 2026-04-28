-- R30-7: electronic_signatures 加 hmac_version 欄位
-- Why: 既有 signature_data 用 plain SHA-256(signer_id:content_hash:timestamp:hash_input)，
--      attacker 拿到 content_hash + timestamp + (password_hash 或 "handwriting") 即可重算偽造。
--      改 HMAC-SHA256 + secret（共用 AUDIT_HMAC_KEY），attacker 無 key 不可偽造。
-- How:  新增 hmac_version SMALLINT DEFAULT 1，舊資料永久保持 v1（plain SHA-256）；
--      新簽章寫 v2（HMAC-SHA256）。verify 時依此欄位 dispatch 計算演算法。
--      對齊 R26-6 audit chain 的 hmac_version pattern（migration 037）。

ALTER TABLE electronic_signatures
    ADD COLUMN IF NOT EXISTS hmac_version SMALLINT NOT NULL DEFAULT 1;

COMMENT ON COLUMN electronic_signatures.hmac_version IS
'signature_data 編碼版本：1=SHA-256 legacy（pre-R30-7），2=HMAC-SHA256+secret。verify 時依此 dispatch。';
