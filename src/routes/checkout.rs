use axum::extract::State;
use axum::response::{Html, Redirect, IntoResponse, Response};
use axum::Form;
use axum_extra::extract::CookieJar;
use crate::db::{self, Db};
use crate::auth as auth_service;
use crate::models::{self, ShippingForm};
use tera::Tera;
use std::sync::Arc;

pub async fn checkout_page(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
) -> Response {
    let user = match auth_service::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let items = db::get_cart_items(&db, &cart_token);
    if items.is_empty() {
        return (new_jar, Redirect::to("/cart")).into_response();
    }
    let total = models::cart_total(&items);
    let cart_count = models::cart_count(&items);
    let addresses = db::get_user_addresses(&db, &user.id);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &cart_count);
    ctx.insert("items", &items);
    ctx.insert("total", &total);
    ctx.insert("addresses", &addresses);
    ctx.insert("step", &"shipping");
    ctx.insert("error", &"");
    (new_jar, Html(tera.render("checkout.html", &ctx).unwrap())).into_response()
}

pub async fn checkout_shipping(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Form(form): Form<ShippingForm>,
) -> Response {
    let user = match auth_service::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let items = db::get_cart_items(&db, &cart_token);
    let total = models::cart_total(&items);
    let cart_count = models::cart_count(&items);

    if form.name.is_empty() || form.address.is_empty() || form.city.is_empty() || form.zip.is_empty() {
        let addresses = db::get_user_addresses(&db, &user.id);
        let mut ctx = tera::Context::new();
        ctx.insert("user", &Some(&user));
        ctx.insert("cart_count", &cart_count);
        ctx.insert("items", &items);
        ctx.insert("total", &total);
        ctx.insert("addresses", &addresses);
        ctx.insert("step", &"shipping");
        ctx.insert("error", &"All shipping fields are required");
        return (new_jar, Html(tera.render("checkout.html", &ctx).unwrap())).into_response();
    }

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &cart_count);
    ctx.insert("items", &items);
    ctx.insert("total", &total);
    ctx.insert("step", &"payment");
    ctx.insert("shipping", &serde_json::json!({
        "name": form.name,
        "address": form.address,
        "city": form.city,
        "zip": form.zip,
    }));
    ctx.insert("error", &"");
    (new_jar, Html(tera.render("checkout.html", &ctx).unwrap())).into_response()
}

pub async fn checkout_payment(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Form(form): Form<CheckoutPaymentForm>,
) -> Response {
    let user = match auth_service::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let items = db::get_cart_items(&db, &cart_token);
    let total = models::cart_total(&items);
    let cart_count = models::cart_count(&items);

    if form.card_number.len() < 13 || form.expiry.is_empty() || form.cvv.len() < 3 {
        let mut ctx = tera::Context::new();
        ctx.insert("user", &Some(&user));
        ctx.insert("cart_count", &cart_count);
        ctx.insert("items", &items);
        ctx.insert("total", &total);
        ctx.insert("step", &"payment");
        ctx.insert("shipping", &serde_json::json!({
            "name": form.shipping_name,
            "address": form.shipping_address,
            "city": form.shipping_city,
            "zip": form.shipping_zip,
        }));
        ctx.insert("error", &"Please enter valid payment details");
        return (new_jar, Html(tera.render("checkout.html", &ctx).unwrap())).into_response();
    }

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &cart_count);
    ctx.insert("items", &items);
    ctx.insert("total", &total);
    ctx.insert("step", &"review");
    ctx.insert("shipping", &serde_json::json!({
        "name": form.shipping_name,
        "address": form.shipping_address,
        "city": form.shipping_city,
        "zip": form.shipping_zip,
    }));
    ctx.insert("card_last_four", &form.card_number.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>());
    ctx.insert("error", &"");
    (new_jar, Html(tera.render("checkout.html", &ctx).unwrap())).into_response()
}

pub async fn checkout_confirm(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Form(form): Form<ConfirmForm>,
) -> Response {
    let user = match auth_service::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let items = db::get_cart_items(&db, &cart_token);
    if items.is_empty() {
        return (new_jar, Redirect::to("/cart")).into_response();
    }

    let shipping = ShippingForm {
        name: form.shipping_name.clone(),
        address: form.shipping_address.clone(),
        city: form.shipping_city.clone(),
        zip: form.shipping_zip.clone(),
    };

    let order_id = db::create_order(&db, &user.id, &items, &shipping);
    db::save_address(&db, &user.id, &shipping);
    db::clear_cart(&db, &cart_token);

    let order = db::get_order(&db, &order_id);
    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &0);
    ctx.insert("step", &"confirmation");
    ctx.insert("order", &order);
    ctx.insert("error", &"");
    (new_jar, Html(tera.render("checkout.html", &ctx).unwrap())).into_response()
}

#[derive(serde::Deserialize)]
pub struct CheckoutPaymentForm {
    pub card_number: String,
    pub expiry: String,
    pub cvv: String,
    pub shipping_name: String,
    pub shipping_address: String,
    pub shipping_city: String,
    pub shipping_zip: String,
}

#[derive(serde::Deserialize)]
pub struct ConfirmForm {
    pub shipping_name: String,
    pub shipping_address: String,
    pub shipping_city: String,
    pub shipping_zip: String,
}
