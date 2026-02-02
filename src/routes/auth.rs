use axum::extract::State;
use axum::response::{Html, Redirect, IntoResponse, Response};
use axum::Form;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use crate::db::{self, Db};
use crate::auth as auth_service;
use crate::models::{LoginForm, RegisterForm, ProfileForm};
use tera::Tera;
use std::sync::Arc;

type AppState = (Db, Arc<Tera>);

pub async fn login_page(
    State((_db, tera)): State<AppState>,
    jar: CookieJar,
) -> Html<String> {
    let mut ctx = tera::Context::new();
    ctx.insert("error", &"");
    ctx.insert("user", &None::<crate::models::User>);
    ctx.insert("unread_count", &0i64);
    Html(tera.render("login.html", &ctx).unwrap())
}

pub async fn login(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    let user = db::get_user_by_email(&db, &form.email);
    match user {
        Some(u) if auth_service::verify_password(&form.password, &u.password_hash) => {
            let session_id = db::create_session(&db, &u.id);
            let cookie = Cookie::build((auth_service::SESSION_COOKIE, session_id))
                .path("/")
                .http_only(true)
                .max_age(time::Duration::days(7))
                .build();
            (jar.add(cookie), Redirect::to("/")).into_response()
        }
        _ => {
            let mut ctx = tera::Context::new();
            ctx.insert("error", &"Invalid email or password");
            ctx.insert("user", &None::<crate::models::User>);
            ctx.insert("unread_count", &0i64);
            Html(tera.render("login.html", &ctx).unwrap()).into_response()
        }
    }
}

pub async fn register_page(
    State((_db, tera)): State<AppState>,
    jar: CookieJar,
) -> Html<String> {
    let mut ctx = tera::Context::new();
    ctx.insert("error", &"");
    ctx.insert("user", &None::<crate::models::User>);
    ctx.insert("unread_count", &0i64);
    Html(tera.render("register.html", &ctx).unwrap())
}

pub async fn register(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> Response {
    if form.password != form.password_confirm {
        let mut ctx = tera::Context::new();
        ctx.insert("error", &"Passwords do not match");
        ctx.insert("user", &None::<crate::models::User>);
        ctx.insert("unread_count", &0i64);
        return Html(tera.render("register.html", &ctx).unwrap()).into_response();
    }
    if form.password.len() < 8 {
        let mut ctx = tera::Context::new();
        ctx.insert("error", &"Password must be at least 8 characters");
        ctx.insert("user", &None::<crate::models::User>);
        ctx.insert("unread_count", &0i64);
        return Html(tera.render("register.html", &ctx).unwrap()).into_response();
    }

    let hash = auth_service::hash_password(&form.password);
    match db::create_user(&db, &form.name, &form.email, &hash) {
        Ok(user_id) => {
            let session_id = db::create_session(&db, &user_id);
            let cookie = Cookie::build((auth_service::SESSION_COOKIE, session_id))
                .path("/")
                .http_only(true)
                .max_age(time::Duration::days(7))
                .build();
            (jar.add(cookie), Redirect::to("/")).into_response()
        }
        Err(e) => {
            let mut ctx = tera::Context::new();
            ctx.insert("error", &e);
            ctx.insert("user", &None::<crate::models::User>);
            ctx.insert("unread_count", &0i64);
            Html(tera.render("register.html", &ctx).unwrap()).into_response()
        }
    }
}

pub async fn logout(jar: CookieJar) -> (CookieJar, Redirect) {
    let jar = jar.remove(Cookie::from(auth_service::SESSION_COOKIE));
    (jar, Redirect::to("/"))
}

pub async fn profile(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
) -> Response {
    let user = match auth_service::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let listings = db::get_user_listings(&db, &user.id);
    let unread = db::get_unread_count(&db, &user.id);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("listings", &listings);
    ctx.insert("unread_count", &unread);
    ctx.insert("error", &"");
    ctx.insert("success", &"");
    Html(tera.render("profile.html", &ctx).unwrap()).into_response()
}

pub async fn update_profile(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
    Form(form): Form<ProfileForm>,
) -> Response {
    let user = match auth_service::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    db::update_user_profile(&db, &user.id, &form);
    // Reload user
    let user = db::get_user_by_id(&db, &user.id).unwrap();
    let listings = db::get_user_listings(&db, &user.id);
    let unread = db::get_unread_count(&db, &user.id);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("listings", &listings);
    ctx.insert("unread_count", &unread);
    ctx.insert("error", &"");
    ctx.insert("success", &"Profile updated!");
    Html(tera.render("profile.html", &ctx).unwrap()).into_response()
}
