// R26-7 完成：原本的 crate-level `#![allow(dead_code)]` 在 PR #3 移除後，剩餘
// 零星死碼已逐一審視——真死碼已刪除（PR #173），serde 被動欄位改為 `_` 前綴
// 命名 + `#[serde(rename = ...)]`。本模組樹零 `#[allow(dead_code)]`；新死碼
// 進不來（未標註即 clippy 紅燈）。

pub mod access;
pub mod accounting;
mod ai;
mod alert_lock;
pub mod alert_threshold;
mod amendment;
mod animal;
mod animal_medical_report;
mod application_notice;
pub mod audit;
pub mod audit_chain_verify;
mod auth;
mod calendar;
pub mod csp_report;
mod document;
pub mod email;
mod equipment;
mod euthanasia;
mod facility;
mod file;
pub mod google_calendar;
pub mod holiday;
mod hr;
pub mod ip_blocklist;
mod login_tracker;
pub mod mcp;
mod messaging;
mod notification;
pub mod outbox;
mod partner;
pub mod pdf_artifact;
pub(crate) mod pdf_service_client;
/// 待處理人解析（「這一關卡在誰手上」）：跨模組共用，故為 pub。
pub mod pending_owner;
/// 權限常數 codegen：產生器 bin 與 CI 守衛測試共用，故為 pub。
pub mod permission_codegen;
mod planned_experiment;
mod product;
pub(crate) mod product_parser;
mod protocol;
mod protocol_template_versions;
mod qa_plan;
mod qau;
pub mod report;
pub mod retry;
mod role;
pub mod scheduler;
pub mod security_notifier;
mod session_manager;
mod signature;
pub mod signature_bridge;
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
pub use alert_lock::AlertLockService;
pub use alert_threshold::AlertThresholdService;
pub use amendment::AmendmentService;
pub use animal::care_record::{
    CareRecord, CareRecordService, CareVetRecordType, CreateCareRecordRequest,
    UpdateCareRecordRequest,
};
pub use animal::vet_advice::{
    AnimalVetAdvice, UpsertVetAdviceRequest, VetAdviceRecord, VetAdviceRecordService,
};
pub use animal::vet_patrol::{
    CreateVetPatrolReportRequest, UpdateVetPatrolReportRequest, VetPatrolEntryPhoto,
    VetPatrolListFilter, VetPatrolPhoto, VetPatrolReport, VetPatrolReportWithEntries,
};
pub use animal::{
    excel_export as animal_excel_export, field_correction::AnimalFieldCorrectionService,
    AnimalBloodTestService, AnimalImportExportService, AnimalMedicalService,
    AnimalObservationService, AnimalService, AnimalSourceService, AnimalSurgeryService,
    AnimalTransferService, AnimalVetAdviceService, AnimalWeightService, ByproductMonthlyRow,
    ByproductSample, ByproductSampleService, CreateByproductSampleRequest,
    UpdateByproductSampleRequest, VetPatrolReportService,
};
pub use animal_medical_report::{
    AnimalMedicalReportService, MedicalTimelineEvent, WeeklyMedicalReportFilter,
};
pub use application_notice::ApplicationNoticeService;
pub use audit::AuditService;
pub use auth::AuthService;
pub use calendar::CalendarService;
pub use document::DocumentService;
pub use email::{EmailService, RenderedEmail};
pub use equipment::{EquipmentService, MAINTENANCE_RESIGN_SUPERSEDE_REASON};
pub use euthanasia::EuthanasiaService;
pub use facility::FacilityService;
pub use file::{FileCategory, FileService, UploadResult};
pub use holiday::HolidayService;
pub use hr::balance::{
    calculate_annual_leave_days, seniority_months, ExpiryChange, MissingHireDateUser,
    RecomputeExpiryReport,
};
pub use hr::overtime::{OvertimeLimitCheck, WeekdayOvertimeTiers, WorkHoursValidation};
pub use hr::CancelLeaveOutcome;
pub use hr::HrService;
pub use hr::{BackfillLeavePinsReport, BackfilledLeavePin};
pub use invitation::InvitationService;
pub use ip_blocklist::{IpBlocklistEntry, IpBlocklistService};
pub use login_tracker::LoginTracker;
pub use messaging::{
    CreateThreadRequest, Message, MessageAttachment, MessageThread, MessageWithSender,
    MessagingCategory, MessagingService, RecipientSummary, SendMessageRequest,
    ThreadParticipantSummary, ThreadSummary, ThreadWithMessages,
};
pub use notification::{
    init_app_url as init_notification_app_url, EventContext, NotificationPayload,
    NotificationService, OrphanPinnedRow, ReconcileReport,
};
pub use outbox::{ChannelAdapter, ChannelRegistry, EmailAdapter, OutboxEvent, OutboxService};
pub use partner::PartnerService;
pub use pdf_service_client::PdfServiceClient;
pub use planned_experiment::PlannedExperimentService;
pub use product::ProductService;
pub use protocol::ai_review::validate_only as validate_protocol_content;
pub use protocol::ai_review::AiReviewService;
pub use protocol::ProtocolService;
pub use protocol_template_versions::ProtocolTemplateVersionService;
pub use qa_plan::QaPlanService;
pub use qau::{QauDashboard, QauService};
pub use role::RoleService;
pub use security_notifier::{SecurityNotification, SecurityNotifier};
pub use session_manager::SessionManager;
pub use signature::{
    AnnotationService, AnnotationType, ElectronicSignature, SignatureInfoDto, SignatureService,
    SignatureType,
};
pub use signature_bridge::SignatureBridgeService;
pub use sku::SkuService;
pub use stock::StockService;
pub use storage_location::StorageLocationService;
pub use system_settings::SystemSettingsService;
pub use training::TrainingService;
pub use treatment_drug::TreatmentDrugService;
pub use user::UserService;
pub use warehouse::WarehouseService;

pub mod geoip;
pub use geoip::GeoIpService;

mod balance_expiration;
pub use balance_expiration::BalanceExpirationJob;

mod glp_compliance;
mod invitation;
pub use glp_compliance::GlpComplianceService;
mod data_export;
mod data_import;
mod schema_mapping;
pub use data_export::{export_full_database, get_schema_version, ExportFormat, ExportParams};
pub use data_import::{import_idxf, ImportMode, ImportResult};

mod partition_maintenance;
pub use partition_maintenance::PartitionMaintenanceJob;

pub mod retention_enforcer;
pub use retention_enforcer::{RetentionEnforcer, RetentionRunReport};
