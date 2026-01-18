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
mod pig;
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
pub use pig::PigService;
pub use email::EmailService;
pub use notification::NotificationService;
pub use file::{FileService, FileCategory, UploadResult};
pub use hr::HrService;
pub use facility::FacilityService;
pub use calendar::CalendarService;
pub use pdf::PdfService;

mod balance_expiration;
pub use balance_expiration::BalanceExpirationJob;


