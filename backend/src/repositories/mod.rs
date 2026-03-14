// Repository 層：封裝重複使用的 SQL 查詢（≥2 處出現）
// 依賴方向：Services → Repositories → Models

pub mod equipment;
pub mod product;
pub mod role;
pub mod sku;
pub mod user;
pub mod warehouse;
