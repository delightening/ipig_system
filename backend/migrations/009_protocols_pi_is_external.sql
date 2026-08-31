-- `pi_is_external` 把「PI 是否為外部人員（無系統帳號）」從角色啟發式
-- 搬成建立/匯入當下就寫死的權威欄位（CodeRabbit #26 第 3 輪指出，heavy lift，
-- 原 PR #26 已知限制，見 task_65663042）。
--
-- ⚠️ 先講清楚問題出在哪，免得以為是新規則——裁定 16（PI 不可兼任同一計畫的
-- SD）需要知道 `protocols.pi_user_id` 記的是不是「真的 PI」：
--   - 有系統帳號的 PI 明確指定時，`pi_user_id` = 那個人，是真的 PI。
--   - 外部 PI（無系統帳號）時，`pi_user_id` 記的是匯入者/建立者本人（佔位），
--     不代表任何人是 PI，裁定 10／11 明確允許執秘自任 SD。
--
-- 舊做法（`update` 路徑）在每次判斷時**現查**「`pi_user_id == created_by`
-- 且該使用者現在有沒有 PI 角色」來猜是不是佔位——但角色會變動，兩個方向都會
-- 判錯：
--   真 PI 事後**失去** PI 角色 → 被誤判成佔位 → PI≠SD 閘失效（fail open）。
--   佔位建立者事後**取得** PI 角色 → 被誤判成真 PI → 合法的自任 SD 被誤擋
--     （fail closed）。
--
-- 本 migration 把這個判斷結果在 `create` / `import_approved`「當下」就固定
-- 寫死，`update` 之後只讀這個欄位，不再現查角色——判斷結果不再隨事後的
-- 角色異動而改變。

ALTER TABLE public.protocols
    ADD COLUMN pi_is_external boolean NOT NULL DEFAULT false;

COMMENT ON COLUMN public.protocols.pi_is_external IS
    'PI 是否為外部人員（無系統帳號），建立/匯入當下寫死，之後不可變更。'
    'true：pi_user_id 只是建立者/匯入者的佔位值，不代表任何人是 PI，'
    'PI≠SD（裁定 16）比對時略過。false：pi_user_id 是真正的 PI，比對時生效。'
    '不要用「pi_user_id == created_by」現查角色回推——角色會變、判斷結果不該變。'
    '見 services/protocol/core.rs 的 validate_and_authorize_sd。';

-- 回填：套用 `update` 原本用的同一套角色啟發式（pi_user_id = created_by 且
-- 該使用者當下沒有 PI 角色 → 視為佔位）。這是既有資料唯一能用的判斷依據
-- （沒有 ground truth），回填後的精確度與 migration 前完全一樣——差別只在
-- 於「今後」的角色異動不會再讓判斷結果跟著漂移。
DO $$
DECLARE
    filled integer;
BEGIN
    UPDATE public.protocols p
    SET pi_is_external = true
    WHERE p.pi_user_id = p.created_by
      AND NOT EXISTS (
          SELECT 1 FROM user_roles ur
          JOIN roles r ON r.id = ur.role_id
          WHERE ur.user_id = p.pi_user_id AND r.code = 'PI'
      );

    GET DIAGNOSTICS filled = ROW_COUNT;

    RAISE NOTICE '009: 回填 pi_is_external = true 共 % 筆', filled;
END $$;
