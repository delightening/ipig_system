// 動物管理服務模組
//
// 將原 animal.rs 拆分為以下子模組：
// - core: 動物 CRUD、列表、批次分配
// - blood_test: 血液檢測模板與面板管理
// - medical: 病歷紀錄、疫苗、獸醫建議
// - surgery: 手術紀錄
// - observation: 觀察紀錄
// - weight: 體重紀錄
// - source: 動物來源管理
// - import_export: 動物資料匯入匯出

mod blood_test;
pub mod care_record;
mod core;
pub(crate) mod field_correction;
mod import_export;
pub(crate) mod medical;
pub(crate) mod observation;
mod source;
pub(crate) mod surgery;
mod transfer;
pub mod utils;
pub(crate) mod weight;

pub use blood_test::AnimalBloodTestService;

pub use import_export::AnimalImportExportService;
pub use medical::AnimalMedicalService;
pub use observation::AnimalObservationService;
pub use source::AnimalSourceService;
pub use surgery::AnimalSurgeryService;
pub use transfer::AnimalTransferService;
pub use weight::AnimalWeightService;

pub struct AnimalService;
