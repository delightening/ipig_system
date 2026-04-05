// Repository 層：封裝重複使用的 SQL 查詢（≥2 處出現）
// 依賴方向：Services → Repositories → Models

pub mod ai;
pub mod equipment;

pub use ai::AiRepository;
pub mod notification;
pub mod product;
pub mod role;
pub mod sku;
pub mod user;
pub mod user_preference;
pub mod accounting;
pub mod warehouse;
pub mod qa_plan;
pub mod glp_compliance;
pub mod pen;
