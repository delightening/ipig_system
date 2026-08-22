//! species 主檔 ↔ animals.breed enum 的換算（過渡期集中點）
//!
//! 自 P1 起 `animals.species_id` 是動物種類的真相源，`animals.breed`（`animal_breed` enum）
//! 降為尚未退役的相容欄位——它是 NOT NULL 且被 18 個測試檔與 print-pdf 契約依賴，
//! 無法在同一階段移除（移除排程見 docs/design/self-service-animal-species/DESIGN.md P3）。
//!
//! 兩者的對應規則只寫在這裡一處，避免再度散落成多份硬編對照表。

use uuid::Uuid;

use crate::{models::AnimalBreed, services::animal::utils::AnimalUtils, AppError, Result};

pub struct SpeciesLink;

impl SpeciesLink {
    /// `species.code` → `breed` enum。
    ///
    /// 對得上 enum label 的物種沿用原值，其餘（admin 自助新增的物種，如「山羊」）一律歸
    /// `Other`。這是通用規則：新增物種不需要改這段程式碼。
    pub fn breed_for_code(code: &str) -> AnimalBreed {
        match code {
            "miniature" => AnimalBreed::Minipig,
            "white" => AnimalBreed::White,
            "LYD" => AnimalBreed::LYD,
            _ => AnimalBreed::Other,
        }
    }

    /// 由建立請求解析出 `(species_id, breed)`，兩者都保證有值。
    ///
    /// - 有 `species_id`：以它為準推導 `breed`；同時送來的 `breed` 會被忽略
    ///   （species 是真相源，不讓呼叫端用矛盾的兩個值寫出不一致的列）。
    /// - 只有 `breed`（舊 client、Excel 匯入）：反查對應的 species 列補上 `species_id`，
    ///   讓新建的動物一律帶 species_id，P3 要改 NOT NULL 時不會有新的漏網列。
    /// - 兩者都沒有：擋下。
    ///
    /// 吃交易連線而非連線池：呼叫端會在同一個交易裡接著 INSERT 動物，
    /// 驗證與寫入之間不能有讓物種被停用的空窗。
    pub async fn resolve(
        conn: &mut sqlx::PgConnection,
        species_id: Option<Uuid>,
        breed: Option<AnimalBreed>,
    ) -> Result<(Uuid, AnimalBreed)> {
        if let Some(species_id) = species_id {
            return Self::resolve_from_species(conn, species_id).await;
        }
        let Some(breed) = breed else {
            return Err(AppError::Validation("請指定動物物種".to_string()));
        };
        Self::resolve_from_breed(conn, breed).await
    }

    /// 呼叫端明確指定物種：必須存在、啟用中，且是可指派的葉節點。
    ///
    /// `pub(crate)`：除了建立流程的 `resolve`，`core::update` 也直接呼叫這個方法
    /// ——`PUT /animals/:id` 允許改 `species_id`，同樣需要這組驗證，不能只靠
    /// `COALESCE` 寫入未經檢查的值（R86-5）。
    pub(crate) async fn resolve_from_species(
        conn: &mut sqlx::PgConnection,
        species_id: Uuid,
    ) -> Result<(Uuid, AnimalBreed)> {
        // FOR UPDATE 鎖住這一列，讓「驗證通過」到「INSERT 完成」之間，
        // 別的交易不能把它停用。
        let row: Option<(String, bool)> =
            sqlx::query_as("SELECT code, is_active FROM species WHERE id = $1 FOR UPDATE")
                .bind(species_id)
                .fetch_optional(&mut *conn)
                .await?;
        let Some((code, is_active)) = row else {
            return Err(AppError::Validation("指定的物種不存在".to_string()));
        };
        if !is_active {
            return Err(AppError::Validation(
                "指定的物種已停用，請改選啟用中的物種".to_string(),
            ));
        }

        // 只有葉節點能指派給動物：頂層的「豬」是分類用的父層，指給牠會寫出語意不完整的資料。
        // 前端下拉本來就只列葉節點，這裡是後端自己的邊界——不把前端當安全邊界。
        let has_children: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM species WHERE parent_id = $1)")
                .bind(species_id)
                .fetch_one(&mut *conn)
                .await?;
        if has_children.0 {
            return Err(AppError::Validation(
                "此物種底下還有子物種，請改選更明確的品種".to_string(),
            ));
        }

        Ok((species_id, Self::breed_for_code(&code)))
    }

    /// 相容路徑：由固定的 enum 值反查物種列。
    ///
    /// 刻意不套用 `is_active` 與葉節點檢查——這裡的物種不是呼叫端挑的，而是從 enum 推出來的：
    /// LYD 在主檔是停用狀態（從未有動物使用），舊 client 真的送了 lyd 仍該建得起來；
    /// 日後若有人在 `other` 底下加子物種，也不該讓既有匯入檔整批失敗。
    async fn resolve_from_breed(
        conn: &mut sqlx::PgConnection,
        breed: AnimalBreed,
    ) -> Result<(Uuid, AnimalBreed)> {
        let db_value = AnimalUtils::breed_to_db_value(&breed);
        let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM species WHERE code = $1")
            .bind(db_value)
            .fetch_optional(&mut *conn)
            .await?;
        let Some((species_id,)) = row else {
            return Err(AppError::Validation(format!(
                "品種「{}」在物種主檔中沒有對應項目，請先於物種管理建立",
                db_value
            )));
        };
        Ok((species_id, breed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_codes_map_to_their_enum() {
        assert_eq!(
            SpeciesLink::breed_for_code("miniature"),
            AnimalBreed::Minipig
        );
        assert_eq!(SpeciesLink::breed_for_code("white"), AnimalBreed::White);
        assert_eq!(SpeciesLink::breed_for_code("LYD"), AnimalBreed::LYD);
        assert_eq!(SpeciesLink::breed_for_code("other"), AnimalBreed::Other);
    }

    #[test]
    fn self_service_species_fall_back_to_other() {
        // admin 自助新增的物種不需要改 code 就能建立動物
        assert_eq!(SpeciesLink::breed_for_code("goat"), AnimalBreed::Other);
        assert_eq!(SpeciesLink::breed_for_code("山羊"), AnimalBreed::Other);
    }

    #[test]
    fn code_match_is_case_sensitive_like_the_enum_labels() {
        // DB enum label 是 'LYD'（大寫）；小寫 'lyd' 不是有效的 species code，歸 Other
        assert_eq!(SpeciesLink::breed_for_code("lyd"), AnimalBreed::Other);
        assert_eq!(SpeciesLink::breed_for_code("Miniature"), AnimalBreed::Other);
    }
}
