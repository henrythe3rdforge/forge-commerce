use axum::extract::{Multipart, Path, Query, State};
use axum::response::{Html, Redirect, IntoResponse, Response};
use axum_extra::extract::CookieJar;
use crate::db::{self, Db};
use crate::auth;
use crate::models::{SearchQuery, time_ago};
use tera::Tera;
use std::sync::Arc;

type AppState = (Db, Arc<Tera>);

pub async fn feed(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    let listings = db::get_listings(&db, &query);
    let categories = db::get_categories(&db);
    let user = auth::get_current_user(&db, &jar);
    let unread = user.as_ref().map(|u| db::get_unread_count(&db, &u.id)).unwrap_or(0);

    let mut ctx = tera::Context::new();
    ctx.insert("listings", &listings);
    ctx.insert("categories", &categories);
    ctx.insert("user", &user);
    ctx.insert("unread_count", &unread);
    ctx.insert("query", &query.q.clone().unwrap_or_default());
    ctx.insert("current_category", &query.category.clone().unwrap_or_default());
    ctx.insert("current_condition", &query.condition.clone().unwrap_or_default());
    ctx.insert("current_sort", &query.sort.clone().unwrap_or_default());
    // Add time_ago for each listing
    let listings_with_time: Vec<(crate::models::Listing, String)> = listings.iter()
        .map(|l| (l.clone(), time_ago(&l.created_at)))
        .collect();
    ctx.insert("listings_with_time", &listings_with_time);
    Html(tera.render("feed.html", &ctx).unwrap())
}

pub async fn feed_partial(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    let listings = db::get_listings(&db, &query);
    let user = auth::get_current_user(&db, &jar);

    let mut html = String::new();
    if listings.is_empty() {
        html.push_str(r#"<div class="no-results"><p>No listings found. Try a different search.</p></div>"#);
    }
    for l in &listings {
        let ago = time_ago(&l.created_at);
        html.push_str(&format!(
            r##"<a href="/listing/{id}" class="listing-card">
                <div class="listing-image"><img src="{img}" alt="{title}" loading="lazy"></div>
                <div class="listing-info">
                    <p class="listing-price">${price:.0}</p>
                    <h3 class="listing-title">{title}</h3>
                    <div class="listing-meta">
                        <span class="listing-location">📍 {location}</span>
                        <span class="listing-time">{ago}</span>
                    </div>
                </div>
            </a>"##,
            id = l.id, img = l.image_url, title = tera::escape_html(&l.title),
            price = l.price, location = tera::escape_html(&l.location), ago = ago,
        ));
    }
    Html(html)
}

pub async fn listing_detail(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Response {
    let user = auth::get_current_user(&db, &jar);
    let unread = user.as_ref().map(|u| db::get_unread_count(&db, &u.id)).unwrap_or(0);

    match db::get_listing(&db, &id) {
        Some(listing) => {
            let seller = db::get_user_by_id(&db, &listing.seller_id);
            let seller_listings = db::get_seller_listings(&db, &listing.seller_id, &listing.id);
            let ago = time_ago(&listing.created_at);
            let is_owner = user.as_ref().map(|u| u.id == listing.seller_id).unwrap_or(false);

            // Check if there's an existing conversation
            let existing_convo = user.as_ref().and_then(|u| {
                if !is_owner {
                    let convos = db::get_user_conversations(&db, &u.id);
                    convos.into_iter().find(|c| c.listing_id == listing.id)
                } else {
                    None
                }
            });

            let mut ctx = tera::Context::new();
            ctx.insert("listing", &listing);
            ctx.insert("seller", &seller);
            ctx.insert("seller_listings", &seller_listings);
            ctx.insert("user", &user);
            ctx.insert("unread_count", &unread);
            ctx.insert("time_ago", &ago);
            ctx.insert("is_owner", &is_owner);
            ctx.insert("existing_convo", &existing_convo);
            Html(tera.render("listing_detail.html", &ctx).unwrap()).into_response()
        }
        None => Html("<h1>Listing not found</h1>".to_string()).into_response(),
    }
}

pub async fn new_listing_page(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let unread = db::get_unread_count(&db, &user.id);
    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("unread_count", &unread);
    ctx.insert("listing", &None::<crate::models::Listing>);
    ctx.insert("editing", &false);
    ctx.insert("error", &"");
    Html(tera.render("listing_form.html", &ctx).unwrap()).into_response()
}

pub async fn create_listing(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    let mut title = String::new();
    let mut description = String::new();
    let mut price = String::new();
    let mut category = String::new();
    let mut condition = String::from("Good");
    let mut location = user.location.clone();
    let mut image_url = "/static/images/placeholder.svg".to_string();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "title" => title = field.text().await.unwrap_or_default(),
            "description" => description = field.text().await.unwrap_or_default(),
            "price" => price = field.text().await.unwrap_or_default(),
            "category" => category = field.text().await.unwrap_or_default(),
            "condition" => condition = field.text().await.unwrap_or_default(),
            "location" => location = field.text().await.unwrap_or_default(),
            "image" => {
                let filename = field.file_name().unwrap_or("").to_string();
                if !filename.is_empty() {
                    let data = field.bytes().await.unwrap_or_default();
                    if !data.is_empty() {
                        let _ = std::fs::create_dir_all("static/images");
                        let ext = filename.rsplit('.').next().unwrap_or("jpg");
                        let save_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
                        let path = format!("static/images/{}", save_name);
                        std::fs::write(&path, &data).ok();
                        image_url = format!("/static/images/{}", save_name);
                    }
                }
            }
            _ => {}
        }
    }

    let form = crate::models::ListingForm { title, description, price, category, condition, location };
    let id = db::create_listing(&db, &user.id, &form, &image_url);
    Redirect::to(&format!("/listing/{}", id)).into_response()
}

pub async fn edit_listing_page(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let listing = match db::get_listing(&db, &id) {
        Some(l) if l.seller_id == user.id => l,
        _ => return Redirect::to("/").into_response(),
    };
    let unread = db::get_unread_count(&db, &user.id);
    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("unread_count", &unread);
    ctx.insert("listing", &Some(&listing));
    ctx.insert("editing", &true);
    ctx.insert("error", &"");
    Html(tera.render("listing_form.html", &ctx).unwrap()).into_response()
}

pub async fn update_listing(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    let mut title = String::new();
    let mut description = String::new();
    let mut price = String::new();
    let mut category = String::new();
    let mut condition = String::from("Good");
    let mut location = String::new();
    let mut image_url: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "title" => title = field.text().await.unwrap_or_default(),
            "description" => description = field.text().await.unwrap_or_default(),
            "price" => price = field.text().await.unwrap_or_default(),
            "category" => category = field.text().await.unwrap_or_default(),
            "condition" => condition = field.text().await.unwrap_or_default(),
            "location" => location = field.text().await.unwrap_or_default(),
            "image" => {
                let filename = field.file_name().unwrap_or("").to_string();
                if !filename.is_empty() {
                    let data = field.bytes().await.unwrap_or_default();
                    if !data.is_empty() {
                        let _ = std::fs::create_dir_all("static/images");
                        let ext = filename.rsplit('.').next().unwrap_or("jpg");
                        let save_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
                        let path = format!("static/images/{}", save_name);
                        std::fs::write(&path, &data).ok();
                        image_url = Some(format!("/static/images/{}", save_name));
                    }
                }
            }
            _ => {}
        }
    }

    let form = crate::models::ListingForm { title, description, price, category, condition, location };
    db::update_listing(&db, &id, &user.id, &form, image_url.as_deref());
    Redirect::to(&format!("/listing/{}", id)).into_response()
}

pub async fn mark_sold(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    db::update_listing_status(&db, &id, &user.id, "sold");
    Redirect::to(&format!("/listing/{}", id)).into_response()
}

pub async fn delete_listing(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    db::delete_listing(&db, &id, &user.id);
    Redirect::to("/profile").into_response()
}
