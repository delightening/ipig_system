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
mod hr;
mod facility;
mod calendar;
mod euthanasia;
mod amendment;
mod storage_location;
pub mod user_preferences;
mod treatment_drug;
mod training;
mod equipment;

pub use user::*;
pub use role::*;
pub use warehouse::*;
pub use product::*;
pub use partner::*;
pub use document::*;
pub use stock::*;
pub use audit::*;
pub use sku::*;
pub use protocol::*;
pub use animal::*;
pub use notification::*;
pub use hr::*;
pub use facility::*;
pub use calendar::*;
pub use euthanasia::*;
pub use amendment::*;
pub use storage_location::*;
pub use treatment_drug::*;
pub use training::*;
pub use equipment::*;


use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Pagination query parameters
#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 { 1 }
fn default_per_page() -> i64 { 20 }

/// Optional pagination parameters — backward compatible.
/// When both `page` and `per_page` are provided, LIMIT/OFFSET is applied.
/// When absent, all records are returned.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl PaginationParams {
    pub fn sql_suffix(&self) -> String {
        match (self.page, self.per_page) {
            (Some(page), Some(per_page)) => {
                let per_page = per_page.clamp(1, 100);
                let offset = (page.max(1) - 1) * per_page;
                format!(" LIMIT {} OFFSET {}", per_page, offset)
            }
            _ => String::new(),
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_response_total_pages() {
        let resp = PaginatedResponse::<i32>::new(vec![], 100, 1, 20);
        assert_eq!(resp.total_pages, 5);
    }

    #[test]
    fn test_paginated_response_partial_last_page() {
        let resp = PaginatedResponse::<i32>::new(vec![], 101, 1, 20);
        assert_eq!(resp.total_pages, 6, "101 筆 / 每頁 20 = 6 頁（最後一頁不滿）");
    }

    #[test]
    fn test_paginated_response_single_item() {
        let resp = PaginatedResponse::<i32>::new(vec![1], 1, 1, 20);
        assert_eq!(resp.total_pages, 1);
        assert_eq!(resp.total, 1);
    }

    #[test]
    fn test_paginated_response_empty() {
        let resp = PaginatedResponse::<i32>::new(vec![], 0, 1, 20);
        assert_eq!(resp.total_pages, 0);
    }

    #[test]
    fn test_default_pagination_values() {
        assert_eq!(default_page(), 1);
        assert_eq!(default_per_page(), 20);
    }

    #[test]
    fn test_pagination_params_no_params() {
        let p = PaginationParams { page: None, per_page: None };
        assert_eq!(p.sql_suffix(), "");
    }

    #[test]
    fn test_pagination_params_partial_params() {
        let p = PaginationParams { page: Some(2), per_page: None };
        assert_eq!(p.sql_suffix(), "");
    }

    #[test]
    fn test_pagination_params_basic() {
        let p = PaginationParams { page: Some(1), per_page: Some(20) };
        assert_eq!(p.sql_suffix(), " LIMIT 20 OFFSET 0");
    }

    #[test]
    fn test_pagination_params_page_2() {
        let p = PaginationParams { page: Some(2), per_page: Some(10) };
        assert_eq!(p.sql_suffix(), " LIMIT 10 OFFSET 10");
    }

    #[test]
    fn test_pagination_params_clamp_per_page() {
        let p = PaginationParams { page: Some(1), per_page: Some(999) };
        assert_eq!(p.sql_suffix(), " LIMIT 100 OFFSET 0", "per_page 上限為 100");

        let p = PaginationParams { page: Some(1), per_page: Some(0) };
        assert_eq!(p.sql_suffix(), " LIMIT 1 OFFSET 0", "per_page 下限為 1");
    }

    #[test]
    fn test_pagination_params_page_floor() {
        let p = PaginationParams { page: Some(0), per_page: Some(10) };
        assert_eq!(p.sql_suffix(), " LIMIT 10 OFFSET 0", "page < 1 視為第 1 頁");

        let p = PaginationParams { page: Some(-5), per_page: Some(10) };
        assert_eq!(p.sql_suffix(), " LIMIT 10 OFFSET 0", "負數 page 視為第 1 頁");
    }
}

