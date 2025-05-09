use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Serialize;

// Home page template
#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub user: Option<UserContext>,
    pub current_year: i32,
}

// Login page template
#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate {
    pub user: Option<UserContext>,
    pub current_year: i32,
    pub error: Option<String>,
    pub pgp_challenge: Option<String>,
}

// Register page template
#[derive(Template)]
#[template(path = "pages/register.html")]
pub struct RegisterTemplate {
    pub user: Option<UserContext>,
    pub current_year: i32,
    pub error: Option<String>,
}

// Products page template
#[derive(Template)]
#[template(path = "pages/products.html")]
pub struct ProductsTemplate {
    pub user: Option<UserContext>,
    pub current_year: i32,
    pub products: Vec<ProductContext>,
    pub categories: Vec<CategoryContext>,
    pub currency: String,
    pub page: i32,
    pub total_pages: i32,
}

// Product detail page template
#[derive(Template)]
#[template(path = "pages/product_detail.html")]
pub struct ProductDetailTemplate {
    pub user: Option<UserContext>,
    pub current_year: i32,
    pub product: ProductDetailContext,
    pub currency: String,
}

// Context structs for templates

#[derive(Serialize, Clone)]
pub struct UserContext {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub pgp_public_key: Option<String>,
    pub pgp_added_date: Option<String>,
    pub reputation: Option<f64>,
    pub review_count: Option<i32>,
    pub created_at: String,
    pub is_vendor: bool,
    pub is_admin: bool,
    pub is_moderator: bool,
}

#[derive(Serialize, Clone)]
pub struct CategoryContext {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
}

#[derive(Serialize, Clone)]
pub struct ProductContext {
    pub id: i32,
    pub title: String,
    pub vendor_id: i32,
    pub vendor_name: String,
    pub price_btc: Option<String>,
    pub price_xmr: Option<String>,
    pub rating: f64,
    pub review_count: i32,
    pub primary_image: Option<i32>,
    pub stock: i32,
    pub sales_count: Option<i32>,
}

#[derive(Serialize, Clone)]
pub struct ProductDetailContext {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub vendor: VendorContext,
    pub category: CategoryContext,
    pub price_btc: Option<String>,
    pub price_xmr: Option<String>,
    pub stock: i32,
    pub rating: f64,
    pub review_count: i32,
    pub primary_image_id: Option<i32>,
    pub images: Vec<ProductImageContext>,
    pub variants: Vec<ProductVariantContext>,
    pub shipping_options: Vec<ShippingOptionContext>,
    pub reviews: Vec<ReviewContext>,
}

#[derive(Serialize, Clone)]
pub struct ProductImageContext {
    pub id: i32,
    pub is_primary: bool,
}

#[derive(Serialize, Clone)]
pub struct ProductVariantContext {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub price_btc: Option<String>,
    pub price_xmr: Option<String>,
    pub stock: i32,
}

#[derive(Serialize, Clone)]
pub struct ShippingOptionContext {
    pub name: String,
    pub description: String,
    pub price_btc: String,
    pub price_xmr: String,
}

#[derive(Serialize, Clone)]
pub struct VendorContext {
    pub id: i32,
    pub username: String,
    pub is_verified: bool,
    pub rating: f64,
    pub total_sales: i32,
    pub response_time: String,
    pub created_at: String,
    pub status: String,
}

#[derive(Serialize, Clone)]
pub struct VendorStatsContext {
    pub total_sales: i32,
    pub sales_change: f64,
    pub revenue_btc: String,
    pub revenue_xmr: String,
    pub rating: f64,
    pub review_count: i32,
    pub active_products: i32,
    pub total_products: i32,
}

#[derive(Serialize, Clone)]
pub struct VendorBondContext {
    pub amount_btc: String,
    pub payment_address: String,
}

#[derive(Serialize, Clone)]
pub struct WalletsContext {
    pub btc: WalletContext,
    pub xmr: WalletContext,
}

#[derive(Serialize, Clone)]
pub struct WalletContext {
    pub balance: String,
    pub available: String,
    pub pending: String,
    pub in_escrow: String,
}

#[derive(Serialize, Clone)]
pub struct TransactionContext {
    pub id: i32,
    pub wallet_type: String,
    pub transaction_type: String,
    pub amount: String,
    pub fee: String,
    pub tx_hash: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Serialize, Clone)]
pub struct ConversationContext {
    pub id: i32,
    pub other_username: String,
    pub last_message: String,
    pub last_message_time: String,
    pub unread_count: i32,
}

#[derive(Serialize, Clone)]
pub struct ConversationDetailContext {
    pub id: i32,
    pub other_username: String,
    pub other_user_is_vendor: bool,
    pub other_user_pgp_key: Option<String>,
    pub messages: Vec<MessageContext>,
    pub related_order: Option<RelatedOrderContext>,
}

#[derive(Serialize, Clone)]
pub struct MessageContext {
    pub id: i32,
    pub content: String,
    pub is_from_me: bool,
    pub created_at: String,
}

#[derive(Serialize, Clone)]
pub struct RelatedOrderContext {
    pub id: i32,
}

#[derive(Serialize, Clone)]
pub struct OrderContext {
    pub id: i32,
    pub buyer_id: i32,
    pub buyer_username: Option<String>,
    pub vendor_id: i32,
    pub vendor_name: String,
    pub status: String,
    pub currency: String,
    pub total_amount: String,
    pub item_count: i32,
    pub created_at: String,
    pub shipping_method: String,
}

#[derive(Serialize, Clone)]
pub struct OrderDetailContext {
    pub id: i32,
    pub status: String,
    pub currency: String,
    pub currency_name: String,
    pub subtotal: String,
    pub shipping_cost: String,
    pub escrow_fee: String,
    pub total_amount: String,
    pub payment_status: String,
    pub escrow_address: String,
    pub encrypted_shipping_address: String,
    pub shipping_method: String,
    pub estimated_delivery: String,
    pub created_at: String,
    pub vendor: VendorContext,
    pub items: Vec<OrderItemContext>,
    pub status_history: Vec<OrderStatusHistoryContext>,
    pub review: Option<ReviewContext>,
    pub has_review: bool,
}

#[derive(Serialize, Clone)]
pub struct OrderItemContext {
    pub product_id: i32,
    pub product_title: String,
    pub variant_id: Option<i32>,
    pub variant_title: Option<String>,
    pub quantity: i32,
    pub price_per_unit: String,
    pub total_price: String,
}

#[derive(Serialize, Clone)]
pub struct OrderStatusHistoryContext {
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Clone)]
pub struct ReviewContext {
    pub id: i32,
    pub order_id: i32,
    pub product_id: i32,
    pub product_title: String,
    pub reviewer_id: i32,
    pub reviewer_username: String,
    pub rating: i32,
    pub comment: String,
    pub created_at: String,
}

#[derive(Serialize, Clone)]
pub struct LoginHistoryContext {
    pub timestamp: String,
    pub ip_address: String,
    pub success: bool,
}

// Helper function to get the current year for templates
fn current_year() -> i32 {
    chrono::Utc::now().year()
}

// Implement IntoResponse for all template structs
impl<T> IntoResponse for T
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                eprintln!("Template error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
