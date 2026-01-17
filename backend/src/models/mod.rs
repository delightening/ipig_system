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
mod pig;
mod notification;
mod hr;
mod facility;
mod calendar;

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
pub use pig::*;
pub use notification::*;
pub use hr::*;
pub use facility::*;
pub use calendar::*;

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


