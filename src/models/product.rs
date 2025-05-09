use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{categories, product_images, product_variants, products};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = products)]
#[diesel(belongs_to(Category))]
pub struct Product {
    pub id: i32,
    pub vendor_id: i32,
    pub title: String,
    pub description: String,
    pub category_id: Option<i32>,
    pub price_btc: Option<BigDecimal>,
    pub price_xmr: Option<BigDecimal>,
    pub stock: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub vendor_id: i32,
    pub title: String,
    pub description: String,
    pub category_id: Option<i32>,
    pub price_btc: Option<BigDecimal>,
    pub price_xmr: Option<BigDecimal>,
    pub stock: i32,
    pub is_active: bool,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = products)]
pub struct UpdateProduct {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub price_btc: Option<BigDecimal>,
    pub price_xmr: Option<BigDecimal>,
    pub stock: Option<i32>,
    pub is_active: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = product_variants)]
#[diesel(belongs_to(Product))]
pub struct ProductVariant {
    pub id: i32,
    pub product_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub price_btc: Option<BigDecimal>,
    pub price_xmr: Option<BigDecimal>,
    pub stock: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = product_variants)]
pub struct NewProductVariant {
    pub product_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub price_btc: Option<BigDecimal>,
    pub price_xmr: Option<BigDecimal>,
    pub stock: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = product_images)]
#[diesel(belongs_to(Product))]
pub struct ProductImage {
    pub id: i32,
    pub product_id: i32,
    pub image_data: Vec<u8>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = product_images)]
pub struct NewProductImage {
    pub product_id: i32,
    pub image_data: Vec<u8>,
    pub is_primary: bool,
}

// For serialization in API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductWithDetails {
    #[serde(flatten)]
    pub product: Product,
    pub category: Option<Category>,
    pub variants: Vec<ProductVariant>,
    pub primary_image_id: Option<i32>,
}

use diesel::sql_types::Numeric;
use bigdecimal::BigDecimal;
