pub mod user;
pub mod product;
pub mod order;
pub mod message;
pub mod payment;
pub mod vendor;

pub fn register_custom_types() {
    // Register custom types for Diesel
    order::register_order_status_type();
    payment::register_payment_types();
}
