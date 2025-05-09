use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};

use crate::schema::{order_items, order_status_history, orders};
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::models::order::OrderStatusMapping"]
pub enum OrderStatus {
    Pending,
    Paid,
    Processing,
    Shipped,
    Delivered,
    Disputed,
    Cancelled,
    Completed,
}

// This is needed for Diesel to map the enum to the database
#[derive(Debug, Clone, Copy, QueryId, SqlType)]
#[diesel(postgres_type(name = "order_status"))]
pub struct OrderStatusMapping;

pub fn register_order_status_type() {
    // Register the custom type with Diesel
    // This is called from models/mod.rs
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: i32,
    pub buyer_id: i32,
    pub vendor_id: i32,
    pub status: OrderStatus,
    pub currency: crate::models::payment::PaymentCurrency,
    pub total_amount: BigDecimal,
    pub escrow_address: Option<String>,
    pub encrypted_shipping_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub buyer_id: i32,
    pub vendor_id: i32,
    pub status: OrderStatus,
    pub currency: crate::models::payment::PaymentCurrency,
    pub total_amount: BigDecimal,
    pub escrow_address: Option<String>,
    pub encrypted_shipping_address: String,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = orders)]
pub struct UpdateOrder {
    pub status: Option<OrderStatus>,
    pub escrow_address: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = order_items)]
#[diesel(belongs_to(Order))]
pub struct OrderItem {
    pub id: i32,
    pub order_id: i32,
    pub product_id: i32,
    pub variant_id: Option<i32>,
    pub quantity: i32,
    pub price_per_unit: BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = order_items)]
pub struct NewOrderItem {
    pub order_id: i32,
    pub product_id: i32,
    pub variant_id: Option<i32>,
    pub quantity: i32,
    pub price_per_unit: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = order_status_history)]
#[diesel(belongs_to(Order))]
pub struct OrderStatusHistory {
    pub id: i32,
    pub order_id: i32,
    pub status: OrderStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = order_status_history)]
pub struct NewOrderStatusHistory {
    pub order_id: i32,
    pub status: OrderStatus,
    pub notes: Option<String>,
}

// For API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderWithItems {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItem>,
    pub status_history: Vec<OrderStatusHistory>,
}
