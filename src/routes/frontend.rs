use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::templates::{
    HomeTemplate, LoginTemplate, ProductDetailTemplate, ProductsTemplate, RegisterTemplate,
    UserContext,
};

pub fn create_route() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/products", get(products))
        .route("/products/:id", get(product_detail))
        .route("/api/products/:id/image/:image_id", get(product_image))
}

async fn home() -> impl IntoResponse {
    let template = HomeTemplate {
        user: None,
        current_year: chrono::Utc::now().year(),
    };
    template
}

async fn login() -> impl IntoResponse {
    let template = LoginTemplate {
        user: None,
        current_year: chrono::Utc::now().year(),
        error: None,
        pgp_challenge: None,
    };
    template
}

async fn register() -> impl IntoResponse {
    let template = RegisterTemplate {
        user: None,
        current_year: chrono::Utc::now().year(),
        error: None,
    };
    template
}

#[derive(Deserialize)]
struct ProductsQuery {
    page: Option<i32>,
    category: Option<i32>,
    search: Option<String>,
    currency: Option<String>,
}

async fn products(Query(params): Query<ProductsQuery>) -> impl IntoResponse {
    // In a real implementation, we would fetch products from the database
    // For now, we'll just return a template with mock data
    let page = params.page.unwrap_or(1);
    let currency = params.currency.unwrap_or_else(|| "btc".to_string());

    let template = ProductsTemplate {
        user: None,
        current_year: chrono::Utc::now().year(),
        products: vec![
            mock_product(1, "Product 1", 4.5, 10),
            mock_product(2, "Product 2", 5.0, 25),
            mock_product(3, "Product 3", 3.5, 5),
            mock_product(4, "Product 4", 4.0, 15),
            mock_product(5, "Product 5", 4.8, 30),
            mock_product(6, "Product 6", 3.0, 8),
        ],
        categories: vec![
            mock_category(1, "Electronics"),
            mock_category(2, "Clothing"),
            mock_category(3, "Books"),
            mock_category(4, "Home & Garden"),
            mock_category(5, "Toys"),
        ],
        currency,
        page,
        total_pages: 3,
    };
    template
}

async fn product_detail(Path(id): Path<i32>) -> impl IntoResponse {
    // In a real implementation, we would fetch the product from the database
    // For now, we'll just return a template with mock data
    let template = ProductDetailTemplate {
        user: None,
        current_year: chrono::Utc::now().year(),
        product: mock_product_detail(id),
        currency: "btc".to_string(),
    };
    template
}

async fn product_image(Path((product_id, image_id)): Path<(i32, i32)>) -> impl IntoResponse {
    // In a real implementation, we would fetch the image from the database
    // For now, we'll just redirect to a placeholder image
    Redirect::to("/static/images/placeholder.svg")
}

// Mock data functions
fn mock_product(id: i32, title: &str, rating: f64, review_count: i32) -> crate::templates::ProductContext {
    crate::templates::ProductContext {
        id,
        title: title.to_string(),
        vendor_id: 1,
        vendor_name: "VendorName".to_string(),
        price_btc: Some("0.001".to_string()),
        price_xmr: Some("0.1".to_string()),
        rating,
        review_count,
        primary_image: Some(1),
        stock: 100,
        sales_count: Some(50),
    }
}

fn mock_category(id: i32, name: &str) -> crate::templates::CategoryContext {
    crate::templates::CategoryContext {
        id,
        name: name.to_string(),
        description: Some(format!("Description for {}", name)),
        parent_id: None,
    }
}

fn mock_product_detail(id: i32) -> crate::templates::ProductDetailContext {
    crate::templates::ProductDetailContext {
        id,
        title: format!("Product {}", id),
        description: "This is a detailed description of the product. It includes information about the features, specifications, and usage instructions.".to_string(),
        vendor: crate::templates::VendorContext {
            id: 1,
            username: "VendorName".to_string(),
            is_verified: true,
            rating: 4.5,
            total_sales: 100,
            response_time: "2 hours".to_string(),
            created_at: "2023-01-01".to_string(),
            status: "verified".to_string(),
        },
        category: mock_category(1, "Electronics"),
        price_btc: Some("0.001".to_string()),
        price_xmr: Some("0.1".to_string()),
        stock: 100,
        rating: 4.5,
        review_count: 10,
        primary_image_id: Some(1),
        images: vec![
            crate::templates::ProductImageContext {
                id: 1,
                is_primary: true,
            },
            crate::templates::ProductImageContext {
                id: 2,
                is_primary: false,
            },
        ],
        variants: vec![
            crate::templates::ProductVariantContext {
                id: 1,
                title: "Variant 1".to_string(),
                description: Some("Description for Variant 1".to_string()),
                price_btc: Some("0.001".to_string()),
                price_xmr: Some("0.1".to_string()),
                stock: 50,
            },
            crate::templates::ProductVariantContext {
                id: 2,
                title: "Variant 2".to_string(),
                description: Some("Description for Variant 2".to_string()),
                price_btc: Some("0.0012".to_string()),
                price_xmr: Some("0.12".to_string()),
                stock: 50,
            },
        ],
        shipping_options: vec![
            crate::templates::ShippingOptionContext {
                name: "Standard Shipping".to_string(),
                description: "7-10 business days".to_string(),
                price_btc: "0.0001".to_string(),
                price_xmr: "0.01".to_string(),
            },
            crate::templates::ShippingOptionContext {
                name: "Express Shipping".to_string(),
                description: "3-5 business days".to_string(),
                price_btc: "0.0002".to_string(),
                price_xmr: "0.02".to_string(),
            },
        ],
        reviews: vec![
            crate::templates::ReviewContext {
                id: 1,
                order_id: 1,
                product_id: id,
                product_title: format!("Product {}", id),
                reviewer_id: 2,
                reviewer_username: "Buyer1".to_string(),
                rating: 5,
                comment: "Great product, exactly as described!".to_string(),
                created_at: "2023-05-15".to_string(),
            },
            crate::templates::ReviewContext {
                id: 2,
                order_id: 2,
                product_id: id,
                product_title: format!("Product {}", id),
                reviewer_id: 3,
                reviewer_username: "Buyer2".to_string(),
                rating: 4,
                comment: "Good product, but shipping was a bit slow.".to_string(),
                created_at: "2023-05-10".to_string(),
            },
        ],
    }
}
