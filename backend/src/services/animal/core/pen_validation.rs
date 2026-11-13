use sqlx::PgPool;
use uuid::Uuid;

use super::super::utils::AnimalUtils;
use super::super::AnimalService;
use crate::{repositories::pen::PenRepository, AppError, Result};

/// 本次請求最後指向的欄位（id 與正規代碼）。
///
/// 寫入端以 `pen_id` 為準；`code` 用來同步寫回 `animals.pen_location` 文字欄，
/// 讓兩欄在寫入當下就一致（B 方案第 2 階段，讀取端仍有人讀文字欄）。
#[derive(Debug, Clone)]
pub(crate) struct ResolvedPen {
    pub id: Uuid,
    pub code: String,
}

/// 欄位允許的狀態（active 或 empty 才能收容動物）
const PEN_ALLOWED_STATUSES: &[&str] = &["active", "empty"];

impl AnimalService {
    /// 解出本次請求指向的欄位。
    ///
    /// - 只給 `pen_location`（前端現況）：以代碼找唯一一個啟用欄位，轉成 `pen_id`。
    ///   找不到、或同一代碼對到多個啟用欄位（無法判定）→ 400，不猜。
    /// - 只給 `pen_id`：確認欄位存在，並取回代碼。
    /// - 兩者都給：必須指向同一欄，否則 400（兩個來源互相矛盾，不挑一個蓋掉另一個）。
    /// - 都沒給（或 `pen_location` 為空白）：`None`，呼叫端維持原值。
    pub(crate) async fn resolve_pen_target(
        pool: &PgPool,
        pen_id: Option<Uuid>,
        pen_location: Option<&str>,
    ) -> Result<Option<ResolvedPen>> {
        let by_code = match pen_location.map(str::trim).filter(|s| !s.is_empty()) {
            Some(location) => Some(Self::find_unique_pen_by_code(pool, location).await?),
            None => None,
        };

        match (pen_id, by_code) {
            (None, None) => Ok(None),
            (None, Some(pen)) => Ok(Some(pen)),
            (Some(id), None) => {
                let pen = PenRepository::find_pen_by_id(pool, id)
                    .await?
                    .ok_or_else(|| AppError::NotFound(format!("欄位 (id={}) 不存在", id)))?;
                Ok(Some(ResolvedPen {
                    id: pen.id,
                    code: pen.code,
                }))
            }
            (Some(id), Some(pen)) => {
                if pen.id == id {
                    Ok(Some(pen))
                } else {
                    Err(AppError::BadRequest(
                        "pen_id 與 pen_location 指向不同欄位，請只指定其中一個".to_string(),
                    ))
                }
            }
        }
    }

    /// 依代碼找唯一一個啟用欄位；輸入先過 `format_pen_location`（補零、去連字號）再正規化，
    /// 與 DB 端的正規化規則一致。
    async fn find_unique_pen_by_code(pool: &PgPool, location: &str) -> Result<ResolvedPen> {
        let normalized = AnimalUtils::format_pen_location(location)
            .trim()
            .to_uppercase()
            .replace('-', "");
        let mut pens =
            PenRepository::find_active_pens_by_normalized_code(pool, &normalized).await?;
        match pens.len() {
            0 => Err(AppError::BadRequest(format!(
                "欄位「{}」不存在或已停用",
                location.trim()
            ))),
            1 => {
                let pen = pens.remove(0);
                Ok(ResolvedPen {
                    id: pen.id,
                    code: pen.code,
                })
            }
            _ => Err(AppError::BadRequest(format!(
                "欄位代碼「{}」對應到多個啟用欄位，無法判定，請聯絡管理員",
                location.trim()
            ))),
        }
    }

    /// 驗證欄位是否可收容動物
    ///
    /// 檢查項目：
    /// 1. 欄位存在且 is_active = true
    /// 2. 欄位狀態為 active 或 empty
    /// 3. 容量限制：capacity > 0 時，current_count < capacity（capacity = 0 表示無限制）
    pub(crate) async fn validate_pen_for_assignment(pool: &PgPool, pen_id: Uuid) -> Result<()> {
        let pen = PenRepository::find_pen_by_id(pool, pen_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("欄位 (id={}) 不存在", pen_id)))?;

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
            let current_count = PenRepository::count_active_animals_in_pen(pool, pen_id).await?;
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
