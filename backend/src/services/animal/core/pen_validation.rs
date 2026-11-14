use sqlx::PgConnection;
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
    /// 請求指定的欄位是否就是動物「目前所在的欄位」（沒有要換）。
    ///
    /// 「整份表單原樣送出」型的編輯（前端一律帶 `pen_location`）即使沒動欄位也會帶原值。
    /// 沒換欄位就不該重新解析、也不該重驗：若動物所在的欄位後來被停用，重新解析會回 400，
    /// 使用者連這隻動物的其他欄位都改不了——那不是使用者想改的東西。
    ///
    /// - 動物目前沒有 `pen_id`（舊資料只有文字）→ 一律當作「要解析」，才補得齊 `pen_id`。
    /// - 請求的 `pen_id` 與目前不同 → 換欄位。
    /// - 兩個欄位（`pen_id` 與代碼）都給 → 一律要解析：舊資料的文字欄可能與 `pen_id` 不一致，
    ///   只拿代碼比文字欄會漏掉「兩個欄位其實指向不同欄位」的矛盾請求，要交給解析去擋（400）。
    /// - 請求沒給代碼或只有空白 → 沒要換。
    /// - 只給代碼，且經正規化（補零、大小寫、連字號）後與目前的文字欄相同 → 沒換。
    pub(crate) fn pen_request_is_unchanged(
        req_pen_id: Option<Uuid>,
        req_location: Option<&str>,
        current_pen_id: Option<Uuid>,
        current_location: Option<&str>,
    ) -> bool {
        let Some(current_id) = current_pen_id else {
            return false;
        };
        if req_pen_id.is_some_and(|id| id != current_id) {
            return false;
        }
        match req_location.map(str::trim).filter(|s| !s.is_empty()) {
            None => true,
            Some(location) => {
                req_pen_id.is_none()
                    && current_location.is_some_and(|current| {
                        AnimalUtils::canonical_pen_code(current)
                            == AnimalUtils::canonical_pen_code(location)
                    })
            }
        }
    }

    /// 解出本次請求指向的欄位。
    ///
    /// - 只給 `pen_location`（前端現況）：以代碼找唯一一個啟用欄位，轉成 `pen_id`。
    ///   找不到、或同一代碼對到多個啟用欄位（無法判定）→ 400，不猜。
    /// - 只給 `pen_id`：確認欄位存在，並取回代碼。
    /// - 兩者都給：必須指向同一欄，否則 400（兩個來源互相矛盾，不挑一個蓋掉另一個）。
    /// - 都沒給（或 `pen_location` 為空白）：`None`，呼叫端維持原值。
    ///
    /// 吃連線而不是連線池：更新流程已持有交易與動物列鎖，再向連線池要第二條連線，
    /// 高併發時可能所有連線都卡在等第二條而逾時。
    pub(crate) async fn resolve_pen_target(
        conn: &mut PgConnection,
        pen_id: Option<Uuid>,
        pen_location: Option<&str>,
    ) -> Result<Option<ResolvedPen>> {
        let by_code = match pen_location.map(str::trim).filter(|s| !s.is_empty()) {
            Some(location) => Some(Self::find_unique_pen_by_code(&mut *conn, location).await?),
            None => None,
        };

        match (pen_id, by_code) {
            (None, None) => Ok(None),
            (None, Some(pen)) => Ok(Some(pen)),
            (Some(id), None) => {
                let pen = PenRepository::find_pen_by_id(&mut *conn, id)
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

    /// 依代碼找唯一一個啟用欄位。輸入與資料庫端用同一套正規化（補零、大小寫、去連字號），
    /// 見 `AnimalUtils::canonical_pen_code` 與 `repositories::pen::canonical_code_expr`。
    async fn find_unique_pen_by_code(
        conn: &mut PgConnection,
        location: &str,
    ) -> Result<ResolvedPen> {
        let canonical = AnimalUtils::canonical_pen_code(location);
        let mut pens = PenRepository::find_active_pens_by_canonical_code(conn, &canonical).await?;
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
    pub(crate) async fn validate_pen_for_assignment(
        conn: &mut PgConnection,
        pen_id: Uuid,
    ) -> Result<()> {
        let pen = PenRepository::find_pen_by_id(&mut *conn, pen_id)
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
            let current_count =
                PenRepository::count_active_animals_in_pen(&mut *conn, pen_id).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn unchanged_when_request_echoes_the_current_pen() {
        // 前端整份表單原樣送出：代碼格式不同（A-1 / A01）但是同一個欄位
        assert!(AnimalService::pen_request_is_unchanged(
            None,
            Some("A-1"),
            Some(id(1)),
            Some("A01")
        ));
        assert!(AnimalService::pen_request_is_unchanged(
            Some(id(1)),
            None,
            Some(id(1)),
            Some("A01")
        ));
    }

    #[test]
    fn both_pen_id_and_code_given_always_resolves() {
        // 兩個欄位都給：即使與目前的文字欄相同也要解析，確認 pen_id 與代碼指向同一欄
        assert!(!AnimalService::pen_request_is_unchanged(
            Some(id(1)),
            Some("a01"),
            Some(id(1)),
            Some("A01")
        ));
        // 舊資料文字欄（B01）與 pen_id 不一致：請求帶 pen_id 與同樣的 B01，不能被當成沒更動
        assert!(!AnimalService::pen_request_is_unchanged(
            Some(id(1)),
            Some("B01"),
            Some(id(1)),
            Some("B01")
        ));
    }

    #[test]
    fn blank_location_is_not_a_change() {
        assert!(AnimalService::pen_request_is_unchanged(
            None,
            Some("   "),
            Some(id(1)),
            Some("A01")
        ));
        assert!(AnimalService::pen_request_is_unchanged(
            None,
            None,
            Some(id(1)),
            None
        ));
    }

    #[test]
    fn changed_when_pen_or_code_differs() {
        assert!(!AnimalService::pen_request_is_unchanged(
            Some(id(2)),
            None,
            Some(id(1)),
            Some("A01")
        ));
        assert!(!AnimalService::pen_request_is_unchanged(
            None,
            Some("A02"),
            Some(id(1)),
            Some("A01")
        ));
        // 目前沒有文字欄、請求帶了代碼 → 要解析並補上
        assert!(!AnimalService::pen_request_is_unchanged(
            None,
            Some("A01"),
            Some(id(1)),
            None
        ));
    }

    #[test]
    fn legacy_animal_without_pen_id_always_resolves() {
        // 舊資料只有文字沒有 pen_id：即使文字相同也要解析，才補得齊 pen_id
        assert!(!AnimalService::pen_request_is_unchanged(
            None,
            Some("A01"),
            None,
            Some("A01")
        ));
        assert!(!AnimalService::pen_request_is_unchanged(
            None, None, None, None
        ));
    }
}
