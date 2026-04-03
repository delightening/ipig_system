-- 更新管理員帳號 email：admin@ipig.local → admin@ipigsystem.asia
UPDATE users
SET email = 'admin@ipigsystem.asia', updated_at = NOW()
WHERE email = 'admin@ipig.local';
