-- Migration 016 Add mass login flag to login_events
ALTER TABLE login_events ADD COLUMN is_mass_login BOOLEAN DEFAULT false;

-- 更新索引以包含新欄位
DROP INDEX IF EXISTS idx_login_unusual;
CREATE INDEX idx_login_unusual ON login_events(user_id) 
    WHERE is_unusual_time OR is_unusual_location OR is_new_device OR is_mass_login;

COMMENT ON COLUMN login_events.is_mass_login IS '是否為短時間內大量登入（偵測同時登入異常）';
