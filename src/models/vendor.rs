use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{reviews, vendor_bonds};
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = vendor_bonds)]
pub struct VendorBond {
    pub id: i32,
    pub vendor_id: i32,
    pub amount_btc: BigDecimal,
    pub status: String,
    pub transaction_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = vendor_bonds)]
pub struct NewVendorBond {
    pub vendor_id: i32,
    pub amount_btc: BigDecimal,
    pub status: String,
    pub transaction_id: Option<i32>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = vendor_bonds)]
pub struct UpdateVendorBond {
    pub status: Option<String>,
    pub transaction_id: Option<i32>,
    pub updated_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = reviews)]
pub struct Review {
    pub id: i32,
    pub order_id: i32,
    pub reviewer_id: i32,
    pub vendor_id: i32,
    pub product_id: i32,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = reviews)]
pub struct NewReview {
    pub order_id: i32,
    pub reviewer_id: i32,
    pub vendor_id: i32,
    pub product_id: i32,
    pub rating: i32,
    pub comment: Option<String>,
}

// For API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct VendorWithStats {
    pub id: i32,
    pub username: String,
    pub pgp_public_key: Option<String>,
    pub reputation: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub total_sales: i64,
    pub average_rating: Option<f64>,
    pub is_verified: bool,
}
