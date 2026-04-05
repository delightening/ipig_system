use sqlx::PgPool;
use uuid::Uuid;

use super::super::AnimalService;
use crate::{repositories::pen::PenRepository, AppError, Result};

/// 欄位允許的狀態（active 或 empty 才能收容動物）
const PEN_ALLOWED_STATUSES: &[&str] = &["active", "empty"];

impl AnimalService {
    /// 驗證欄位是否可收容動物
    ///
    /// 檢查項目：
    /// 1. 欄位存在且 is_active = true
    /// 2. 欄位狀態為 active 或 empty
    /// 3. 容量限制：capacity > 0 時，current_count < capacity（capacity = 0 表示無限制）
    pub(crate) async fn validate_pen_for_assignment(
        pool: &PgPool,
        pen_id: Uuid,
    ) -> Result<()> {
        let pen = PenRepository::find_pen_by_id(pool, pen_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("欄位 (id={}) 不存在", pen_id))
            })?;

        if !pen.is_active {
            return Err(AppError::BusinessRule(format!(
                "欄位「{}」已停用，無法分配動物",
                pen.code
            )));
        }

        if !PEN_ALLOWED_STATUSES.contains(&pen.status.as_str()) {
            return Err(AppError::BusinessRule(format!(
                "欄位「{}」狀態為「{}」，僅允許 active 或 empty 狀態的欄位收容動物",
                pen.code, pen.status
            )));
        }

        // capacity = 0 表示無限制
        if pen.capacity > 0 {
            let current_count =
                PenRepository::count_active_animals_in_pen(pool, pen_id).await?;
            if current_count >= pen.capacity as i64 {
                return Err(AppError::BusinessRule(format!(
                    "欄位「{}」已滿（容量 {}，目前 {} 隻），無法再分配動物",
                    pen.code, pen.capacity, current_count
                )));
            }
        }

        Ok(())
    }
}
