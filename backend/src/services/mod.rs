#![allow(dead_code)]

pub mod access;
mod ai;
pub mod accounting;
mod amendment;
mod animal;
mod audit;
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
    AnimalSourceService, AnimalSurgeryService, AnimalTransferService, AnimalWeightService,
};
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
pub use product::ProductService;
pub use template::TemplateService;
pub use protocol::ai_review::AiReviewService;
pub use protocol::ai_review::validate_only as validate_protocol_content;
pub use protocol::ProtocolService;
pub use qau::{QauDashboard, QauService};
pub use qa_plan::QaPlanService;
pub use role::RoleService;
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
