// 人員訓練紀錄 Service (GLP 合規)

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        CreateTrainingRecordRequest, PaginatedResponse, TrainingQuery, TrainingRecord,
        TrainingRecordWithUser, UpdateTrainingRecordRequest,
    },
    Result,
};

/// 讀取單筆訓練紀錄的授權判斷。
///
/// 抽成純函式的理由：原本這段寫在 `get` 裡，而且**寫錯了**——
/// `!has_manage && !has_manage_own && record.user_id != current_user.id`
/// 在持有 `training.manage_own` 時，`!has_manage_own` 為 false，
/// 整個 `&&` 直接短路，**owner 比對根本不會被求值**。
/// 於是任何持有 `manage_own` 的角色（`EXPERIMENT_STAFF`／`INTERN`／
/// `EQUIPMENT_MAINTENANCE`，都是一般員工）只要知道 id 就讀得到別人的訓練紀錄。
///
/// 同檔的 `update`／`delete` 沒有這個問題，因為它們拆成**兩道**檢查：
/// 先擋「完全沒有任何 training 權限」，再擋「有 manage_own 但不是自己的」。
/// `get` 想用一道條件同時做這兩件事，結果兩件都沒做對。
///
/// 現在的判準與 `list` 一致（見該函式：非 `training.manage` 者一律被強制
/// `user_filter = 自己`）：**有 `training.manage` 就能讀全部，否則只能讀自己的。**
/// `manage_own` 在這裡刻意不參與判斷——它的語意是「能維護自己的紀錄」，
/// 不該成為讀取別人紀錄的通行證。
fn can_read_training_record(has_manage: bool, record_user_id: Uuid, current_user_id: Uuid) -> bool {
    has_manage || record_user_id == current_user_id
}

pub struct TrainingService;

impl TrainingService {
    pub async fn list(
        pool: &PgPool,
        query: &TrainingQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<TrainingRecordWithUser>> {
        let has_view = current_user.has_permission("training.view");
        let has_manage = current_user.has_permission("training.manage");
        let has_manage_own = current_user.has_permission("training.manage_own");
        if !has_view && !has_manage && !has_manage_own {
            return Err(AppError::Forbidden("無權查看訓練紀錄".into()));
        }

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let user_filter = if has_manage {
            query.user_id
        } else {
            Some(current_user.id)
        };

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM training_records tr
            INNER JOIN users u ON tr.user_id = u.id
            WHERE ($1::uuid IS NULL OR tr.user_id = $1)
              AND ($2::text IS NULL OR tr.course_name ILIKE '%' || $2 || '%')
            "#,
        )
        .bind(user_filter)
        .bind(query.course_name.as_deref())
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, TrainingRecordWithUser>(
            r#"
            SELECT
                tr.id, tr.user_id, u.email as user_email, u.display_name as user_name,
                tr.course_name, tr.completed_at, tr.expires_at, tr.notes, tr.created_at
            FROM training_records tr
            INNER JOIN users u ON tr.user_id = u.id
            WHERE ($1::uuid IS NULL OR tr.user_id = $1)
              AND ($2::text IS NULL OR tr.course_name ILIKE '%' || $2 || '%')
            ORDER BY tr.completed_at DESC, tr.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_filter)
        .bind(query.course_name.as_deref())
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<TrainingRecord> {
        let record =
            sqlx::query_as::<_, TrainingRecord>("SELECT * FROM training_records WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound("訓練紀錄不存在".into()))?;

        let has_manage = current_user.has_permission("training.manage");
        if !can_read_training_record(has_manage, record.user_id, current_user.id) {
            return Err(AppError::Forbidden("無權查看此訓練紀錄".into()));
        }

        Ok(record)
    }

    pub async fn create(
        pool: &PgPool,
        payload: &CreateTrainingRecordRequest,
        current_user: &CurrentUser,
    ) -> Result<TrainingRecord> {
        let has_manage = current_user.has_permission("training.manage");
        let has_manage_own = current_user.has_permission("training.manage_own");
        if !has_manage && !has_manage_own {
            return Err(AppError::Forbidden("無權新增訓練紀錄".into()));
        }
        if has_manage_own && !has_manage && payload.user_id != current_user.id {
            return Err(AppError::Forbidden("僅能新增自己的訓練紀錄".into()));
        }
        payload.validate()?;

        let record = sqlx::query_as::<_, TrainingRecord>(
            r#"
            INSERT INTO training_records (user_id, course_name, completed_at, expires_at, notes)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(payload.user_id)
        .bind(&payload.course_name)
        .bind(payload.completed_at)
        .bind(payload.expires_at)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateTrainingRecordRequest,
        current_user: &CurrentUser,
    ) -> Result<TrainingRecord> {
        let has_manage = current_user.has_permission("training.manage");
        let has_manage_own = current_user.has_permission("training.manage_own");
        if !has_manage && !has_manage_own {
            return Err(AppError::Forbidden("無權編輯訓練紀錄".into()));
        }

        let existing =
            sqlx::query_as::<_, TrainingRecord>("SELECT * FROM training_records WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound("訓練紀錄不存在".into()))?;

        if has_manage_own && !has_manage && existing.user_id != current_user.id {
            return Err(AppError::Forbidden("僅能編輯自己的訓練紀錄".into()));
        }

        payload.validate()?;

        let course_name = payload
            .course_name
            .as_deref()
            .unwrap_or(&existing.course_name);
        let completed_at = payload.completed_at.unwrap_or(existing.completed_at);
        let expires_at = payload.expires_at.or(existing.expires_at);
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();

        let record = sqlx::query_as::<_, TrainingRecord>(
            r#"
            UPDATE training_records
            SET course_name = $2, completed_at = $3, expires_at = $4, notes = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(course_name)
        .bind(completed_at)
        .bind(expires_at)
        .bind(notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(pool: &PgPool, id: Uuid, current_user: &CurrentUser) -> Result<()> {
        let has_manage = current_user.has_permission("training.manage");
        let has_manage_own = current_user.has_permission("training.manage_own");
        if !has_manage && !has_manage_own {
            return Err(AppError::Forbidden("無權刪除訓練紀錄".into()));
        }

        let existing =
            sqlx::query_as::<_, TrainingRecord>("SELECT * FROM training_records WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound("訓練紀錄不存在".into()))?;

        if has_manage_own && !has_manage && existing.user_id != current_user.id {
            return Err(AppError::Forbidden("僅能刪除自己的訓練紀錄".into()));
        }

        let result = sqlx::query("DELETE FROM training_records WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("訓練紀錄不存在".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 這幾個 case 在修好之前會失敗——舊版的
    /// `!has_manage && !has_manage_own && record.user_id != current_user.id`
    /// 在 `has_manage_own = true` 時短路成 false，等同「一律放行」。
    /// 所以本組測試確實測得到那個缺陷，不是恆真的裝飾。
    ///
    /// 判斷抽成純函式正是為了讓它不必連資料庫就能被測到：
    /// 原本這段邏輯埋在 `get` 裡，而 `backend/tests/` 至今**沒有任何一支
    /// training 的測試**，等於這個授權判斷從來沒有任何網子接住。
    #[test]
    fn manage_can_read_anyones_record() {
        let me = Uuid::new_v4();
        let someone_else = Uuid::new_v4();
        assert!(can_read_training_record(true, someone_else, me));
        assert!(can_read_training_record(true, me, me));
    }

    #[test]
    fn without_manage_can_only_read_own_record() {
        let me = Uuid::new_v4();
        let someone_else = Uuid::new_v4();
        assert!(
            can_read_training_record(false, me, me),
            "沒有 training.manage 也該讀得到自己的（與 list 的 own-only 行為一致）"
        );
        assert!(
            !can_read_training_record(false, someone_else, me),
            "這正是修掉的那個洞：持有 manage_own 的一般員工不得讀別人的訓練紀錄"
        );
    }

    /// 明確釘住「manage_own 不是讀取別人紀錄的通行證」這個語意。
    /// 舊寫法就是把它當成了通行證。
    #[test]
    fn manage_own_is_not_a_pass_for_other_peoples_records() {
        let me = Uuid::new_v4();
        let someone_else = Uuid::new_v4();
        // has_manage = false 代表「只有 manage_own 或什麼都沒有」，兩者行為應相同
        assert!(!can_read_training_record(false, someone_else, me));
    }

    /// 把舊條件原樣寫下來與新判斷並排，讓「這次到底修掉了什麼」變成可執行的斷言，
    /// 而不是只存在於 commit message 裡。
    ///
    /// 這比「暫時把程式改回舊版跑一次」更好：那種驗證做完就消失，
    /// 這個會一直留著，日後若有人把判斷改回短路寫法，這支測試會directly轉紅。
    #[test]
    fn old_short_circuit_let_it_through_new_one_blocks() {
        let me = Uuid::new_v4();
        let someone_else = Uuid::new_v4();
        let has_manage = false;
        let has_manage_own = true; // EXPERIMENT_STAFF / INTERN / EQUIPMENT_MAINTENANCE 的實際情形

        // 舊條件逐字重現：!has_manage && !has_manage_own && record.user_id != current_user.id
        let old_would_block = !has_manage && !has_manage_own && someone_else != me;
        assert!(
            !old_would_block,
            "舊條件在 has_manage_own = true 時短路成 false，等同一律放行——這就是被修掉的洞"
        );

        let new_blocks = !can_read_training_record(has_manage, someone_else, me);
        assert!(new_blocks, "新判斷必須擋下「非 manage 者讀他人紀錄」");
    }
}
