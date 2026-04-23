// NOTE: 原本的 `#![allow(dead_code)]` (CRIT-04) 在 PR #3 拿掉；但仍有 11 處
// 零星死碼（API 型別、預留常數、早期 utility 等）分散在子 module 裡，每個需要
// 個別判斷（真死 vs API shape / OpenAPI spec 引用）。為避免 PR #3 scope 膨脹，
// 本輪只處理「確定死碼」（見 `protocol/history.rs::get_next_version_no` 已刪除），
// 其餘以 per-item `#[allow(dead_code)]` + 理由標記，整批清理排入 R26-7。
//
// **此 crate-level blanket allow 已不存在** — 代表新死碼再也進不來（未標註就
// clippy 紅燈）。這比舊版的「整個 services/ 樹免疫死碼警告」更嚴格。

pub mod access;
pub mod alert_threshold;
mod ai;
pub mod mcp;
pub mod accounting;
mod amendment;
mod animal;
mod audit;
pub mod audit_chain_verify;
mod auth;
mod calendar;
mod document;
pub mod email;
mod equipment;
mod euthanasia;
mod facility;
mod file;
pub mod google_calendar;
mod hr;
mod login_tracker;
mod notification;
mod partner;
mod gotenberg;
mod image_processor;
mod pdf;
mod pdf_service_client;
mod product;
pub(crate) mod product_parser;
mod template;
mod protocol;
mod qau;
mod qa_plan;
pub mod report;
pub mod retry;
mod role;
pub mod scheduler;
pub mod ip_blocklist;
pub mod security_notifier;
mod session_manager;
mod signature;
mod sku;
mod stock;
mod storage_location;
pub mod system_settings;
mod training;
mod treatment_drug;
mod user;
mod warehouse;

pub use accounting::AccountingService;
pub use ai::AiService;
pub use amendment::AmendmentService;
pub use animal::care_record::{
    CareRecord, CareRecordService, CareVetRecordType, CreateCareRecordRequest,
    UpdateCareRecordRequest,
};
pub use animal::{
    field_correction::AnimalFieldCorrectionService, AnimalBloodTestService,
    AnimalImportExportService, AnimalMedicalService, AnimalObservationService, AnimalService,
    AnimalSourceService, AnimalSurgeryService, AnimalTransferService, AnimalVetAdviceService,
    AnimalWeightService, VetPatrolReportService,
};
pub use animal::vet_advice::{
    AnimalVetAdvice, UpsertVetAdviceRequest,
    VetAdviceRecord, VetAdviceRecordService,
    CreateVetAdviceRecordRequest, UpdateVetAdviceRecordRequest,
};
pub use animal::vet_patrol::{
    VetPatrolReport, VetPatrolReportWithEntries,
    CreateVetPatrolReportRequest, UpdateVetPatrolReportRequest,
};
pub use alert_threshold::AlertThresholdService;
pub use audit::AuditService;
pub use auth::AuthService;
pub use calendar::CalendarService;
pub use document::DocumentService;
pub use email::EmailService;
pub use equipment::EquipmentService;
pub use euthanasia::EuthanasiaService;
pub use facility::FacilityService;
pub use file::{FileCategory, FileService, UploadResult};
pub use hr::HrService;
pub use hr::overtime::{OvertimeLimitCheck, WeekdayOvertimeTiers, WorkHoursValidation};
pub use hr::balance::{calculate_annual_leave_days, seniority_months};
pub use login_tracker::LoginTracker;
pub use notification::NotificationService;
pub use partner::PartnerService;
pub use gotenberg::GotenbergClient;
pub use image_processor::ImageProcessorClient;
pub use pdf::PdfService;
pub use pdf_service_client::PdfServiceClient;
pub use product::ProductService;
pub use template::TemplateService;
pub use protocol::ai_review::AiReviewService;
pub use protocol::ai_review::validate_only as validate_protocol_content;
pub use protocol::ProtocolService;
pub use qau::{QauDashboard, QauService};
pub use qa_plan::QaPlanService;
pub use role::RoleService;
pub use ip_blocklist::{IpBlocklistEntry, IpBlocklistService};
pub use security_notifier::{SecurityNotifier, SecurityNotification};
pub use session_manager::SessionManager;
pub use signature::{
    AnnotationService, AnnotationType, ElectronicSignature, SignatureInfoDto, SignatureService,
    SignatureType,
};
pub use sku::SkuService;
pub use stock::StockService;
pub use storage_location::StorageLocationService;
pub use system_settings::SystemSettingsService;
pub use training::TrainingService;
pub use treatment_drug::TreatmentDrugService;
pub use user::UserService;
pub use invitation::InvitationService;
pub use warehouse::WarehouseService;

pub mod geoip;
pub use geoip::GeoIpService;

mod balance_expiration;
pub use balance_expiration::BalanceExpirationJob;

mod invitation;
mod glp_compliance;
pub use glp_compliance::GlpComplianceService;
mod data_export;
mod data_import;
mod schema_mapping;
pub use data_export::{export_full_database, get_schema_version, ExportFormat, ExportParams};
pub use data_import::{import_idxf, ImportMode, ImportResult};

mod partition_maintenance;
pub use partition_maintenance::PartitionMaintenanceJob;
