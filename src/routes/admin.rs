use axum::extract::{Multipart, Path, State};
use axum::response::{Html, Redirect, IntoResponse, Response};
use axum::Form;
use axum_extra::extract::CookieJar;
use crate::db::{self, Db};
use crate::auth as auth_service;
use crate::models::ProductForm;
use tera::Tera;
use std::sync::Arc;

fn require_admin(db: &Db, jar: &CookieJar) -> Result<crate::models::User, Redirect> {
    match auth_service::get_current_user(db, jar) {
        Some(u) if u.is_admin => Ok(u),
        _ => Err(Redirect::to("/login")),
    }
}

pub async fn dashboard(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
) -> Response {
    let user = match require_admin(&db, &jar) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let (product_count, order_count, revenue, user_count) = db::get_admin_stats(&db);
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let cart_count = db::get_cart_count(&db, &cart_token);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &cart_count);
    ctx.insert("product_count", &product_count);
    ctx.insert("order_count", &order_count);
    ctx.insert("revenue", &revenue);
    ctx.insert("user_count", &user_count);
    (new_jar, Html(tera.render("admin_dashboard.html", &ctx).unwrap())).into_response()
}

pub async fn product_list(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
) -> Response {
    let user = match require_admin(&db, &jar) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let products = db::get_all_products(&db);
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let cart_count = db::get_cart_count(&db, &cart_token);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &cart_count);
    ctx.insert("products", &products);
    (new_jar, Html(tera.render("admin_products.html", &ctx).unwrap())).into_response()
}

pub async fn product_new_page(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
) -> Response {
    let user = match require_admin(&db, &jar) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let categories = db::get_categories(&db);
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let cart_count = db::get_cart_count(&db, &cart_token);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &cart_count);
    ctx.insert("categories", &categories);
    ctx.insert("product", &None::<crate::models::Product>);
    ctx.insert("editing", &false);
    (new_jar, Html(tera.render("admin_product_form.html", &ctx).unwrap())).into_response()
}

pub async fn product_create(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Response {
    if require_admin(&db, &jar).is_err() {
        return Redirect::to("/login").into_response();
    }

    let mut name = String::new();
    let mut description = String::new();
    let mut price = String::new();
    let mut category = String::new();
    let mut featured = None;
    let mut image_url = "/static/images/placeholder.svg".to_string();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "name" => name = field.text().await.unwrap_or_default(),
            "description" => description = field.text().await.unwrap_or_default(),
            "price" => price = field.text().await.unwrap_or_default(),
            "category" => category = field.text().await.unwrap_or_default(),
            "featured" => featured = Some(field.text().await.unwrap_or_default()),
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

    let form = ProductForm { name, description, price, category, featured };
    db::create_product(&db, &form, &image_url);
    (jar, Redirect::to("/admin/products")).into_response()
}

pub async fn product_edit_page(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Response {
    let user = match require_admin(&db, &jar) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let product = db::get_product(&db, &id);
    let categories = db::get_categories(&db);
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let cart_count = db::get_cart_count(&db, &cart_token);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &cart_count);
    ctx.insert("categories", &categories);
    ctx.insert("product", &product);
    ctx.insert("editing", &true);
    (new_jar, Html(tera.render("admin_product_form.html", &ctx).unwrap())).into_response()
}

pub async fn product_update(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Response {
    if require_admin(&db, &jar).is_err() {
        return Redirect::to("/login").into_response();
    }

    let mut name = String::new();
    let mut description = String::new();
    let mut price = String::new();
    let mut category = String::new();
    let mut featured = None;
    let mut image_url: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "name" => name = field.text().await.unwrap_or_default(),
            "description" => description = field.text().await.unwrap_or_default(),
            "price" => price = field.text().await.unwrap_or_default(),
            "category" => category = field.text().await.unwrap_or_default(),
            "featured" => featured = Some(field.text().await.unwrap_or_default()),
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

    let form = ProductForm { name, description, price, category, featured };
    db::update_product(&db, &id, &form, image_url.as_deref());
    (jar, Redirect::to("/admin/products")).into_response()
}

pub async fn product_delete(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Response {
    if require_admin(&db, &jar).is_err() {
        return Redirect::to("/login").into_response();
    }
    db::delete_product(&db, &id);
    (jar, Redirect::to("/admin/products")).into_response()
}

pub async fn order_list(
    State((db, tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
) -> Response {
    let user = match require_admin(&db, &jar) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let orders = db::get_all_orders(&db);
    let (cart_token, new_jar) = auth_service::ensure_cart_token(jar);
    let cart_count = db::get_cart_count(&db, &cart_token);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("cart_count", &cart_count);
    ctx.insert("orders", &orders);
    (new_jar, Html(tera.render("admin_orders.html", &ctx).unwrap())).into_response()
}

#[derive(serde::Deserialize)]
pub struct StatusForm {
    pub status: String,
}

pub async fn order_update_status(
    State((db, _tera)): State<(Db, Arc<Tera>)>,
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<StatusForm>,
) -> Response {
    if require_admin(&db, &jar).is_err() {
        return Redirect::to("/login").into_response();
    }
    db::update_order_status(&db, &id, &form.status);
    (jar, Html(format!(r#"<span class="order-status status-{}">{}</span>"#, form.status, form.status))).into_response()
}
