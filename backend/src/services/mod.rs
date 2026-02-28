#![allow(dead_code)]

mod auth;
mod user;
mod role;
mod warehouse;
mod product;
mod partner;
mod document;
mod stock;
mod audit;
mod sku;
mod protocol;
mod animal;
mod notification;
mod file;
mod hr;
mod facility;
mod calendar;
mod pdf;
pub mod google_calendar;
mod login_tracker;
mod session_manager;
pub mod scheduler;
pub mod report;
pub mod email;
mod signature;
mod euthanasia;
mod amendment;
mod storage_location;
mod treatment_drug;
pub mod system_settings;

pub use auth::AuthService;
pub use user::UserService;
pub use role::RoleService;
pub use warehouse::WarehouseService;
pub use product::ProductService;
pub use partner::PartnerService;
pub use document::DocumentService;
pub use stock::StockService;
pub use audit::AuditService;
pub use sku::SkuService;
pub use protocol::ProtocolService;
pub use animal::AnimalService;
pub use animal::care_record::{CareRecordService, CareRecord, CreateCareRecordRequest, UpdateCareRecordRequest};
pub use email::EmailService;
pub use notification::NotificationService;
pub use file::{FileService, FileCategory, UploadResult};
pub use hr::HrService;
pub use facility::FacilityService;
pub use calendar::CalendarService;
pub use pdf::PdfService;
pub use signature::{SignatureService, AnnotationService, SignatureType, AnnotationType};
pub use euthanasia::EuthanasiaService;
pub use amendment::AmendmentService;
pub use storage_location::StorageLocationService;
pub use treatment_drug::TreatmentDrugService;
pub use system_settings::SystemSettingsService;
pub use login_tracker::LoginTracker;
pub use session_manager::SessionManager;

pub mod geoip;
pub use geoip::GeoIpService;



mod balance_expiration;
pub use balance_expiration::BalanceExpirationJob;

mod partition_maintenance;
pub use partition_maintenance::PartitionMaintenanceJob;
