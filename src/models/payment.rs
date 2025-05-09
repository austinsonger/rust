use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};

use crate::schema::{transactions, wallets};
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::models::payment::PaymentCurrencyMapping"]
pub enum PaymentCurrency {
    BTC,
    XMR,
}

#[derive(Debug, Clone, Copy, QueryId, SqlType)]
#[diesel(postgres_type(name = "payment_currency"))]
pub struct PaymentCurrencyMapping;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::models::payment::WalletTypeMapping"]
pub enum WalletType {
    BTC,
    XMR,
}

#[derive(Debug, Clone, Copy, QueryId, SqlType)]
#[diesel(postgres_type(name = "wallet_type"))]
pub struct WalletTypeMapping;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::models::payment::TransactionTypeMapping"]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    EscrowLock,
    EscrowRelease,
    Fee,
}

#[derive(Debug, Clone, Copy, QueryId, SqlType)]
#[diesel(postgres_type(name = "transaction_type"))]
pub struct TransactionTypeMapping;

pub fn register_payment_types() {
    // Register the custom types with Diesel
    // This is called from models/mod.rs
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = wallets)]
pub struct Wallet {
    pub id: i32,
    pub user_id: i32,
    pub wallet_type: WalletType,
    pub encrypted_private_key: String,
    pub public_address: String,
    pub balance: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = wallets)]
pub struct NewWallet {
    pub user_id: i32,
    pub wallet_type: WalletType,
    pub encrypted_private_key: String,
    pub public_address: String,
    pub balance: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = transactions)]
#[diesel(belongs_to(Wallet))]
pub struct Transaction {
    pub id: i32,
    pub wallet_id: i32,
    pub transaction_type: TransactionType,
    pub amount: BigDecimal,
    pub fee: BigDecimal,
    pub tx_hash: Option<String>,
    pub order_id: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub wallet_id: i32,
    pub transaction_type: TransactionType,
    pub amount: BigDecimal,
    pub fee: BigDecimal,
    pub tx_hash: Option<String>,
    pub order_id: Option<i32>,
    pub status: String,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = transactions)]
pub struct UpdateTransaction {
    pub tx_hash: Option<String>,
    pub status: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}
