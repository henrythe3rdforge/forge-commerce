use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub image_url: String,
    pub featured: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub product_name: String,
    pub price: f64,
    pub quantity: i32,
    pub image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub status: String,
    pub total: f64,
    pub shipping_name: String,
    pub shipping_address: String,
    pub shipping_city: String,
    pub shipping_zip: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: String,
    pub order_id: String,
    pub product_id: String,
    pub product_name: String,
    pub price: f64,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub address: String,
    pub city: String,
    pub zip: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub count: i64,
}

// Form structs
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Debug, Deserialize)]
pub struct AddToCartForm {
    pub product_id: String,
    pub quantity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCartForm {
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct ShippingForm {
    pub name: String,
    pub address: String,
    pub city: String,
    pub zip: String,
}

#[derive(Debug, Deserialize)]
pub struct PaymentForm {
    pub card_number: String,
    pub expiry: String,
    pub cvv: String,
}

#[derive(Debug, Deserialize)]
pub struct ProductForm {
    pub name: String,
    pub description: String,
    pub price: String,
    pub category: String,
    pub featured: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub sort: Option<String>,
}

pub fn format_price(price: f64) -> String {
    format!("${:.2}", price)
}

pub fn cart_total(items: &[CartItem]) -> f64 {
    items.iter().map(|i| i.price * i.quantity as f64).sum()
}

pub fn cart_count(items: &[CartItem]) -> i32 {
    items.iter().map(|i| i.quantity).sum()
}
