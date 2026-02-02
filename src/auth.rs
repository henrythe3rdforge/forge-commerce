use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use rand::rngs::OsRng;
use crate::db::{self, Db};
use crate::models::User;

pub const SESSION_COOKIE: &str = "forge_session";
pub const CART_COOKIE: &str = "forge_cart";

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

pub fn get_current_user(db: &Db, jar: &CookieJar) -> Option<User> {
    let session_id = jar.get(SESSION_COOKIE)?.value().to_string();
    db::get_session_user(db, &session_id)
}

pub fn get_cart_token(jar: &CookieJar) -> (String, Option<CookieJar>) {
    if let Some(c) = jar.get(CART_COOKIE) {
        (c.value().to_string(), None)
    } else {
        let token = uuid::Uuid::new_v4().to_string();
        let cookie = Cookie::build((CART_COOKIE, token.clone()))
            .path("/")
            .http_only(true)
            .max_age(time::Duration::days(30))
            .build();
        (token, Some(jar.clone().add(cookie)))
    }
}

pub fn ensure_cart_token(jar: CookieJar) -> (String, CookieJar) {
    if let Some(c) = jar.get(CART_COOKIE) {
        (c.value().to_string(), jar)
    } else {
        let token = uuid::Uuid::new_v4().to_string();
        let cookie = Cookie::build((CART_COOKIE, token.clone()))
            .path("/")
            .http_only(true)
            .max_age(time::Duration::days(30))
            .build();
        (token, jar.add(cookie))
    }
}
