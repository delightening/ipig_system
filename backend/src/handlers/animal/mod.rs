// 動物管理 Handlers
// 拆分自原始 animal.rs

mod animal_core;
mod blood_test;
mod byproduct_sample;
pub mod care_record;
mod dashboard;
mod field_correction;
mod import_export;
mod observation;
pub(crate) mod pdf_export;
mod sacrifice_pathology;
mod source;
mod sudden_death;
mod surgery;
mod transfer;
mod vet_advice;
mod vet_patrol;
mod weight_vaccination;

pub use animal_core::*;
pub use blood_test::*;
pub use byproduct_sample::*;
pub use care_record::*;
pub use dashboard::*;
pub use field_correction::*;
pub use import_export::*;
pub use observation::*;
pub use pdf_export::*;
pub use sacrifice_pathology::*;
pub use source::*;
pub use sudden_death::*;
pub use surgery::*;
pub use transfer::*;
pub use vet_advice::*;
pub use vet_patrol::*;
pub use weight_vaccination::*;
