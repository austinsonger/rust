use axum::extract::Query;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u32 {
    20
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationInfo {
    pub total: u64,
    pub page: u64,
    pub limit: u32,
    pub pages: u64,
}

pub fn calculate_offset(page: u64, limit: u32) -> u64 {
    (page - 1) * limit as u64
}
