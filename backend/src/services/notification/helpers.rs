// 輔助方法

use uuid::Uuid;

use crate::error::AppError;

use super::NotificationService;

impl NotificationService {
    /// 取得計畫的 PI 與計劃負責人（SD）使用者（站內通知對象）。
    ///
    /// CO_EDITOR 角色拆除後，內部協作者由計劃負責人（SD, `study_director_user_id`）後繼，
    /// 故通知對象＝PI 成員 ＋ SD（取代原「PI + CO_EDITOR」）。
    pub async fn get_protocol_pi_and_sd(
        &self,
        protocol_id: Uuid,
    ) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN user_protocols up ON u.id = up.user_id
            WHERE up.protocol_id = $1
              AND up.role_in_protocol = 'PI'
              AND u.is_active = true
              AND u.deleted_at IS NULL
            UNION
            SELECT u.id, u.email, u.display_name
            FROM users u
            JOIN protocols p ON u.id = p.study_director_user_id
            WHERE p.id = $1
              AND u.is_active = true
              AND u.deleted_at IS NULL
            "#,
        )
        .bind(protocol_id)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// 取得被指派的審查委員
    pub async fn get_assigned_reviewers(
        &self,
        protocol_id: Uuid,
    ) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN review_assignments ra ON u.id = ra.reviewer_id
            WHERE ra.protocol_id = $1
              AND u.is_active = true
            "#,
        )
        .bind(protocol_id)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// 依角色代碼取得使用者
    pub async fn get_users_by_role(
        &self,
        role_code: &str,
    ) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true AND r.code = $1
            "#,
        )
        .bind(role_code)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// 取得在職的系統管理員（站內通知對象）。
    ///
    /// ⚠️ **不要用 `get_users_by_role(ROLE_SYSTEM_ADMIN)` 拿管理員。**
    /// `constants.rs` 有 `SYSTEM_ADMIN` 與 `admin` 兩個代碼，但 `roles` 表**只有 `admin`**
    /// （2026-08-26 實查正式庫 16 個角色；`backend/migrations/` 全目錄 grep `SYSTEM_ADMIN`
    /// 0 命中，`003_seed.sql:181` 只 seed `'admin'`）。用單一代碼查會**恆回空清單**，
    /// 而空清單在通知路徑上不會報錯——管理員就是收不到，沒有任何訊號。
    ///
    /// 回傳 (user_id, email, display_name)，形狀同 [`Self::get_users_by_role`]。
    ///
    /// ⚠️ `is_active` 與 `deleted_at` **兩個都要濾**（CodeRabbit 於 #32 第 7 輪指出）。
    /// 本檔的 [`Self::get_protocol_pi_and_sd`] 與 [`Self::find_active_user_contact`]
    /// 都是兩條一起濾，只有這裡漏了 `deleted_at`。
    ///
    /// 目前**不可觸發**：使用者停用路徑（`services/user.rs:778`
    /// `UPDATE users SET is_active = false, tokens_valid_after = NOW() ...`）只寫
    /// `is_active`，全 repo 搜不到任何地方寫 `users.deleted_at`，所以產不出
    /// 「已軟刪除但仍 active」的列。仍然補上：方向是 fail-closed（少發不會多發），
    /// 而且一旦日後有人開始寫 `users.deleted_at`，這個缺口會安靜地讓已刪除的
    /// 管理員繼續收通知——通知路徑不會報錯，沒有任何訊號。
    pub async fn get_admin_users(&self) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND u.deleted_at IS NULL
              AND r.code = ANY($1)
            "#,
        )
        .bind(vec![
            crate::constants::ROLE_SYSTEM_ADMIN.to_string(),
            crate::constants::ROLE_ADMIN_LEGACY.to_string(),
        ])
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// 依事件類型從 notification_routing 表動態查詢收件者
    /// 回傳 (user_id, email, display_name, channel)
    pub async fn get_recipients_by_event(
        &self,
        event_type: &str,
    ) -> Result<Vec<(Uuid, String, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name, nr.channel
            FROM notification_routing nr
            JOIN roles r ON nr.role_code = r.code
            JOIN user_roles ur ON ur.role_id = r.id
            JOIN users u ON u.id = ur.user_id
            WHERE nr.event_type = $1
              AND nr.is_active = true
              AND u.is_active = true
            "#,
        )
        .bind(event_type)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// 該事件是否已在 notification_routing 啟用 email（任一 active 列 channel 為 email/both）。
    /// 供「非角色路由」的通知（如直接指派的審查委員）做可切換的 email 開關。
    /// 查詢失敗或無設定 → false（預設不寄，admin 在路由設定開啟後才寄）。
    pub async fn is_email_enabled_for_event(&self, event_type: &str) -> bool {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM notification_routing
                WHERE event_type = $1 AND is_active = true AND channel IN ('email', 'both')
            )
            "#,
        )
        .bind(event_type)
        .fetch_one(&self.db)
        .await
        .unwrap_or(false)
    }

    /// 取得使用者的 email + display_name（僅 active 且未軟刪除）；不存在回 None。
    pub async fn find_active_user_contact(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(String, String)>, AppError> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT email, display_name FROM users WHERE id = $1 AND is_active = true AND deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 檢查特定事件是否需要 Email 通知（根據 routing 表的 channel 設定）
    pub fn should_send_email(channel: &str) -> bool {
        channel == "email" || channel == "both"
    }

    /// 檢查特定事件是否需要站內通知
    pub fn should_send_in_app(channel: &str) -> bool {
        channel == "in_app" || channel == "both"
    }
}
