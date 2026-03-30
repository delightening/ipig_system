// utils/ 只包含純函式工具，不依賴任何業務模組或 AppState
// 注意：存取權限檢查（check_resource_access）已移至 services/access.rs

pub mod http;
pub mod jsonb_validation;
pub mod pdf_pages;
