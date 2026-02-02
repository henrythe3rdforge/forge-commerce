use axum::extract::{Path, Query, State};
use axum::response::Html;
use axum_extra::extract::CookieJar;
use crate::db::{self, Db};
use crate::auth;
use crate::models::SearchQuery;
use tera::Tera;
use std::sync::Arc;

pub async fn index(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
) -> (CookieJar, Html<String>) {
    let featured = db::get_featured_products(&db);
    let categories = db::get_categories(&db);
    let user = auth::get_current_user(&db, &jar);
    let (cart_token, new_jar) = auth::ensure_cart_token(jar);
    let cart_count = db::get_cart_count(&db, &cart_token);

    let mut ctx = tera::Context::new();
    ctx.insert("featured_products", &featured);
    ctx.insert("categories", &categories);
    ctx.insert("user", &user);
    ctx.insert("cart_count", &cart_count);
    (new_jar, Html(tera.render("index.html", &ctx).unwrap()))
}

pub async fn product_list(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Query(query): Query<SearchQuery>,
) -> (CookieJar, Html<String>) {
    let products = db::search_products(&db, &query);
    let categories = db::get_categories(&db);
    let user = auth::get_current_user(&db, &jar);
    let (cart_token, new_jar) = auth::ensure_cart_token(jar);
    let cart_count = db::get_cart_count(&db, &cart_token);

    let mut ctx = tera::Context::new();
    ctx.insert("products", &products);
    ctx.insert("categories", &categories);
    ctx.insert("user", &user);
    ctx.insert("cart_count", &cart_count);
    ctx.insert("query", &query.q.unwrap_or_default());
    ctx.insert("current_category", &query.category.unwrap_or_default());
    ctx.insert("current_sort", &query.sort.unwrap_or_default());
    (new_jar, Html(tera.render("product_list.html", &ctx).unwrap()))
}

pub async fn product_list_partial(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Query(query): Query<SearchQuery>,
) -> (CookieJar, Html<String>) {
    let products = db::search_products(&db, &query);
    let (_, new_jar) = auth::ensure_cart_token(jar);

    let mut html = String::new();
    for p in &products {
        html.push_str(&format!(
            r##"<div class="product-card">
                <a href="/products/{id}">
                    <div class="product-image"><img src="{img}" alt="{name}" loading="lazy"></div>
                    <div class="product-info">
                        <span class="product-category">{cat}</span>
                        <h3>{name}</h3>
                        <p class="product-price">${price:.2}</p>
                    </div>
                </a>
                <button class="btn btn-primary btn-add-cart"
                    hx-post="/api/cart/add"
                    hx-vals='{{"product_id":"{id}","quantity":"1"}}'
                    hx-target="#cart-count"
                    hx-swap="innerHTML"
                    hx-indicator=".htmx-indicator">
                    Add to Cart
                </button>
            </div>"##,
            id = p.id, img = p.image_url, name = p.name, cat = p.category, price = p.price
        ));
    }
    if products.is_empty() {
        html.push_str(r#"<div class="no-results"><p>No products found. Try adjusting your search.</p></div>"#);
    }
    (new_jar, Html(html))
}

pub async fn product_detail(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> (CookieJar, Html<String>) {
    let user = auth::get_current_user(&db, &jar);
    let (cart_token, new_jar) = auth::ensure_cart_token(jar);
    let cart_count = db::get_cart_count(&db, &cart_token);

    match db::get_product(&db, &id) {
        Some(product) => {
            let related = db::get_related_products(&db, &product);
            let mut ctx = tera::Context::new();
            ctx.insert("product", &product);
            ctx.insert("related_products", &related);
            ctx.insert("user", &user);
            ctx.insert("cart_count", &cart_count);
            (new_jar, Html(tera.render("product_detail.html", &ctx).unwrap()))
        }
        None => (new_jar, Html("<h1>Product not found</h1>".to_string())),
    }
}
