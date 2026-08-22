pub mod accounting;
pub mod ai;
pub mod ai_review;
mod amendment;
mod animal;
mod application_notice;
mod audit;
pub mod audit_diff;
mod calendar;
mod document;
mod equipment;
mod euthanasia;
mod facility;
pub mod glp_compliance;
mod hr;
mod invitation;
mod notification;
mod partner;
pub mod pdf_artifact;
mod planned_experiment;
mod product;
mod protocol;
mod protocol_template_versions;
mod qa_plan;
mod role;
mod sku;
mod stock;
mod storage_location;
mod training;
mod treatment_drug;
mod user;
pub mod user_preferences;
mod warehouse;

pub use amendment::*;
pub use animal::*;
pub use application_notice::*;
pub use audit::*;
pub use calendar::*;
pub use document::*;
pub use equipment::*;
pub use euthanasia::*;
pub use facility::*;
pub use hr::*;
pub use invitation::*;
pub use notification::*;
pub use partner::*;
pub use planned_experiment::*;
pub use product::*;
pub use protocol::*;
pub use protocol_template_versions::*;
pub use qa_plan::*;
pub use role::*;
pub use sku::*;
pub use stock::*;
pub use storage_location::*;
pub use training::*;
pub use treatment_drug::*;
pub use user::*;
pub use warehouse::*;

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

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    20
}

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
                // SEC: 使用 saturating_mul 防止極端 page 值溢位
                let offset = (page.max(1) - 1).saturating_mul(per_page);
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
        assert_eq!(
            resp.total_pages, 6,
            "101 筆 / 每頁 20 = 6 頁（最後一頁不滿）"
        );
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
        let p = PaginationParams {
            page: None,
            per_page: None,
        };
        assert_eq!(p.sql_suffix(), "");
    }

    #[test]
    fn test_pagination_params_partial_params() {
        let p = PaginationParams {
            page: Some(2),
            per_page: None,
        };
        assert_eq!(p.sql_suffix(), "");
    }

    #[test]
    fn test_pagination_params_basic() {
        let p = PaginationParams {
            page: Some(1),
            per_page: Some(20),
        };
        assert_eq!(p.sql_suffix(), " LIMIT 20 OFFSET 0");
    }

    #[test]
    fn test_pagination_params_page_2() {
        let p = PaginationParams {
            page: Some(2),
            per_page: Some(10),
        };
        assert_eq!(p.sql_suffix(), " LIMIT 10 OFFSET 10");
    }

    #[test]
    fn test_pagination_params_clamp_per_page() {
        let p = PaginationParams {
            page: Some(1),
            per_page: Some(999),
        };
        assert_eq!(p.sql_suffix(), " LIMIT 100 OFFSET 0", "per_page 上限為 100");

        let p = PaginationParams {
            page: Some(1),
            per_page: Some(0),
        };
        assert_eq!(p.sql_suffix(), " LIMIT 1 OFFSET 0", "per_page 下限為 1");
    }

    #[test]
    fn test_pagination_params_page_floor() {
        let p = PaginationParams {
            page: Some(0),
            per_page: Some(10),
        };
        assert_eq!(p.sql_suffix(), " LIMIT 10 OFFSET 0", "page < 1 視為第 1 頁");

        let p = PaginationParams {
            page: Some(-5),
            per_page: Some(10),
        };
        assert_eq!(
            p.sql_suffix(),
            " LIMIT 10 OFFSET 0",
            "負數 page 視為第 1 頁"
        );
    }
}
