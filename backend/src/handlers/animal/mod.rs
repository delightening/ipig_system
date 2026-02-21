// 動物管理 Handlers
// 拆分自原始 animal.rs

mod animal_core;
mod source;
mod observation;
mod surgery;
mod weight_vaccination;
mod sacrifice_pathology;
mod vet_recommendation;
mod import_export;
mod blood_test;
mod dashboard;
mod sudden_death;
mod transfer;
pub mod care_record;

pub use animal_core::*;
pub use source::*;
pub use observation::*;
pub use surgery::*;
pub use weight_vaccination::*;
pub use sacrifice_pathology::*;
pub use vet_recommendation::*;
pub use import_export::*;
pub use blood_test::*;
pub use dashboard::*;
pub use sudden_death::*;
pub use transfer::*;
pub use care_record::*;

// 通知用輔助函式（供 vet_recommendation 子模組使用）
use sqlx::PgPool;
use uuid::Uuid;

/// 從觀察紀錄 ID 取得動物資訊（用於發送通知）
pub(crate) async fn get_animal_info_from_observation(
    pool: &PgPool,
    observation_id: Uuid,
) -> std::result::Result<Option<(Uuid, String, Option<Uuid>)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, Option<Uuid>)>(
        r#"
        SELECT p.id, p.ear_tag, pr.id as protocol_id
        FROM animal_observations po
        JOIN animals p ON po.animal_id = p.id
        LEFT JOIN protocols pr ON p.iacuc_no = pr.iacuc_no
        WHERE po.id = $1
        "#
    )
    .bind(observation_id)
    .fetch_optional(pool)
    .await
}

/// 從手術紀錄 ID 取得動物資訊（用於發送通知）
pub(crate) async fn get_animal_info_from_surgery(
    pool: &PgPool,
    surgery_id: Uuid,
) -> std::result::Result<Option<(Uuid, String, Option<Uuid>)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, Option<Uuid>)>(
        r#"
        SELECT p.id, p.ear_tag, pr.id as protocol_id
        FROM animal_surgeries ps
        JOIN animals p ON ps.animal_id = p.id
        LEFT JOIN protocols pr ON p.iacuc_no = pr.iacuc_no
        WHERE ps.id = $1
        "#
    )
    .bind(surgery_id)
    .fetch_optional(pool)
    .await
}
