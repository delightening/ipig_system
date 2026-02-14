#![allow(dead_code)]

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


use serde::{Deserialize, Serialize};

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 { 1 }
fn default_per_page() -> i64 { 20 }

/// Paginated response wrapper
#[derive(Debug, Serialize)]
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
}

