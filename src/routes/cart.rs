use axum::extract::State;
use axum::response::Html;
use axum::Form;
use axum_extra::extract::CookieJar;
use crate::db::{self, Db};
use crate::auth;
use crate::models::{self, AddToCartForm, UpdateCartForm};
use tera::Tera;
use std::sync::Arc;

pub async fn view_cart(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
) -> (CookieJar, Html<String>) {
    let (cart_token, new_jar) = auth::ensure_cart_token(jar);
    let items = db::get_cart_items(&db, &cart_token);
    let user = auth::get_current_user(&db, &new_jar);
    let cart_count = db::get_cart_count(&db, &cart_token);
    let total = models::cart_total(&items);

    let mut ctx = tera::Context::new();
    ctx.insert("items", &items);
    ctx.insert("total", &total);
    ctx.insert("user", &user);
    ctx.insert("cart_count", &cart_count);
    (new_jar, Html(tera.render("cart.html", &ctx).unwrap()))
}

pub async fn add_to_cart(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Form(form): Form<AddToCartForm>,
) -> (CookieJar, Html<String>) {
    let (cart_token, new_jar) = auth::ensure_cart_token(jar);
    let qty = form.quantity.unwrap_or(1);
    db::add_to_cart(&db, &cart_token, &form.product_id, qty);
    let count = db::get_cart_count(&db, &cart_token);
    (new_jar, Html(format!(
        r##"{count}<div id="toast" class="toast show" hx-swap-oob="true">Added to cart!</div>"##,
        count = count
    )))
}

fn render_cart_fragment(items: &[models::CartItem], total: f64, count: i32) -> String {
    let mut html = String::from(r#"<div id="cart-items">"#);
    for item in items {
        html.push_str(&format!(
            r##"<div class="cart-item">
                <img src="{img}" alt="{name}" class="cart-item-image">
                <div class="cart-item-details">
                    <h3>{name}</h3>
                    <p class="cart-item-price">${price:.2}</p>
                </div>
                <div class="cart-item-quantity">
                    <button class="btn btn-sm"
                        hx-put="/api/cart/{pid}"
                        hx-vals='{{"quantity":{minus}}}'
                        hx-target="#cart-content"
                        hx-swap="innerHTML">−</button>
                    <span>{qty}</span>
                    <button class="btn btn-sm"
                        hx-put="/api/cart/{pid}"
                        hx-vals='{{"quantity":{plus}}}'
                        hx-target="#cart-content"
                        hx-swap="innerHTML">+</button>
                </div>
                <div class="cart-item-total">${line_total:.2}</div>
                <button class="btn btn-danger btn-sm"
                    hx-delete="/api/cart/{pid}"
                    hx-target="#cart-content"
                    hx-swap="innerHTML">✕</button>
            </div>"##,
            img = item.image_url,
            name = item.product_name,
            price = item.price,
            pid = item.product_id,
            minus = item.quantity - 1,
            qty = item.quantity,
            plus = item.quantity + 1,
            line_total = item.price * item.quantity as f64,
        ));
    }
    if items.is_empty() {
        html.push_str(r##"<div class="empty-cart"><p>Your cart is empty.</p><a href="/products" class="btn btn-primary">Browse Products</a></div>"##);
    }
    html.push_str("</div>");
    html.push_str(&format!(
        r##"<div id="cart-summary"><div class="cart-total"><span>Total:</span><span>${total:.2}</span></div>"##,
        total = total
    ));
    if !items.is_empty() {
        html.push_str(r##"<a href="/checkout" class="btn btn-primary btn-block">Proceed to Checkout</a>"##);
    }
    html.push_str("</div>");
    html.push_str(&format!(
        r##"<span id="cart-count" hx-swap-oob="true">{count}</span>"##,
        count = count
    ));
    html
}

pub async fn update_cart(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    axum::extract::Path(product_id): axum::extract::Path<String>,
    Form(form): Form<UpdateCartForm>,
) -> (CookieJar, Html<String>) {
    let (cart_token, new_jar) = auth::ensure_cart_token(jar);
    db::update_cart_item(&db, &cart_token, &product_id, form.quantity);
    let items = db::get_cart_items(&db, &cart_token);
    let total = models::cart_total(&items);
    let count = models::cart_count(&items);
    (new_jar, Html(render_cart_fragment(&items, total, count)))
}

pub async fn remove_from_cart(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    axum::extract::Path(product_id): axum::extract::Path<String>,
) -> (CookieJar, Html<String>) {
    let (cart_token, new_jar) = auth::ensure_cart_token(jar);
    db::remove_from_cart(&db, &cart_token, &product_id);
    let items = db::get_cart_items(&db, &cart_token);
    let total = models::cart_total(&items);
    let count = models::cart_count(&items);
    (new_jar, Html(render_cart_fragment(&items, total, count)))
}

pub async fn cart_count(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
) -> (CookieJar, Html<String>) {
    let (cart_token, new_jar) = auth::ensure_cart_token(jar);
    let count = db::get_cart_count(&db, &cart_token);
    (new_jar, Html(format!("{}", count)))
}
