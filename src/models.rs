use serde::{Deserialize, Serialize};

// === Domain Models ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub location: String,
    pub avatar_url: String,
    pub payment_info: String,
    pub bio: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: String,
    pub seller_id: String,
    pub seller_name: String,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub condition: String,
    pub location: String,
    pub image_url: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub listing_id: String,
    pub listing_title: String,
    pub listing_image: String,
    pub buyer_id: String,
    pub buyer_name: String,
    pub seller_id: String,
    pub seller_name: String,
    pub last_message: String,
    pub last_message_at: String,
    pub unread_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub is_offer: bool,
    pub offer_amount: Option<f64>,
    pub offer_status: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub id: String,
    pub listing_id: String,
    pub conversation_id: String,
    pub buyer_id: String,
    pub amount: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub count: i64,
}

// === Form structs ===

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
pub struct ListingForm {
    pub title: String,
    pub description: String,
    pub price: String,
    pub category: String,
    pub condition: String,
    pub location: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub condition: Option<String>,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageForm {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct MakeOfferForm {
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct ProfileForm {
    pub name: String,
    pub location: String,
    pub bio: String,
    pub payment_info: String,
}

pub fn format_price(price: f64) -> String {
    format!("${:.2}", price)
}

pub fn time_ago(created_at: &str) -> String {
    // Simple relative time - parse ISO datetime
    let now = chrono::Utc::now();
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S") {
        let created = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc);
        let diff = now - created;
        let mins = diff.num_minutes();
        if mins < 1 { return "just now".to_string(); }
        if mins < 60 { return format!("{}m ago", mins); }
        let hours = diff.num_hours();
        if hours < 24 { return format!("{}h ago", hours); }
        let days = diff.num_days();
        if days < 7 { return format!("{}d ago", days); }
        if days < 30 { return format!("{}w ago", days / 7); }
        return format!("{}mo ago", days / 30);
    }
    created_at.to_string()
}
