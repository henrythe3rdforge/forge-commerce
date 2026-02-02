use axum::extract::{Path, Query, State};
use axum::response::{Html, Redirect, IntoResponse, Response};
use axum::Form;
use axum_extra::extract::CookieJar;
use crate::db::{self, Db};
use crate::auth;
use crate::models::{SendMessageForm, MakeOfferForm, time_ago};
use tera::Tera;
use std::sync::Arc;

type AppState = (Db, Arc<Tera>);

pub async fn inbox(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let conversations = db::get_user_conversations(&db, &user.id);
    let unread = db::get_unread_count(&db, &user.id);

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("conversations", &conversations);
    ctx.insert("unread_count", &unread);
    // Add time_ago for conversations
    let convos_with_time: Vec<(&crate::models::Conversation, String)> = conversations.iter()
        .map(|c| (c, time_ago(&c.last_message_at)))
        .collect();
    ctx.insert("convos_with_time", &convos_with_time);
    Html(tera.render("messages.html", &ctx).unwrap()).into_response()
}

pub async fn conversation(
    State((db, tera)): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let convo = match db::get_conversation(&db, &id) {
        Some(c) if c.buyer_id == user.id || c.seller_id == user.id => c,
        _ => return Redirect::to("/messages").into_response(),
    };

    db::mark_conversation_read(&db, &user.id, &id);
    let messages = db::get_messages(&db, &id);
    let listing = db::get_listing(&db, &convo.listing_id);
    let pending_offer = db::get_pending_offer(&db, &id);
    let is_seller = user.id == convo.seller_id;
    let unread = db::get_unread_count(&db, &user.id);

    // If there's an accepted offer, get the seller's payment info
    let payment_info = if !is_seller {
        // Check for any accepted offer
        let conn_check = db.lock().unwrap();
        let accepted: Option<String> = conn_check.query_row(
            "SELECT u.payment_info FROM offers o JOIN listings l ON o.listing_id = l.id JOIN users u ON l.seller_id = u.id WHERE o.conversation_id = ?1 AND o.status = 'accepted' LIMIT 1",
            rusqlite::params![id],
            |row| row.get(0),
        ).ok();
        drop(conn_check);
        accepted.filter(|s| !s.is_empty())
    } else {
        None
    };

    let other_name = if is_seller { &convo.buyer_name } else { &convo.seller_name };

    let mut ctx = tera::Context::new();
    ctx.insert("user", &Some(&user));
    ctx.insert("conversation", &convo);
    ctx.insert("messages", &messages);
    ctx.insert("listing", &listing);
    ctx.insert("pending_offer", &pending_offer);
    ctx.insert("is_seller", &is_seller);
    ctx.insert("unread_count", &unread);
    ctx.insert("payment_info", &payment_info);
    ctx.insert("other_name", other_name);
    // Last message ID for polling
    let last_id = messages.last().map(|m| m.id.as_str()).unwrap_or("");
    ctx.insert("last_message_id", last_id);
    Html(tera.render("conversation.html", &ctx).unwrap()).into_response()
}

// Start a conversation from a listing page
pub async fn start_conversation(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Path(listing_id): Path<String>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let listing = match db::get_listing(&db, &listing_id) {
        Some(l) => l,
        None => return Redirect::to("/").into_response(),
    };
    if listing.seller_id == user.id {
        return Redirect::to(&format!("/listing/{}", listing_id)).into_response();
    }
    let convo_id = db::get_or_create_conversation(&db, &listing_id, &user.id, &listing.seller_id);
    Redirect::to(&format!("/messages/{}", convo_id)).into_response()
}

pub async fn send_message(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<SendMessageForm>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let convo = match db::get_conversation(&db, &id) {
        Some(c) if c.buyer_id == user.id || c.seller_id == user.id => c,
        _ => return Redirect::to("/messages").into_response(),
    };
    if !form.content.trim().is_empty() {
        db::send_message(&db, &id, &user.id, form.content.trim());
    }
    Redirect::to(&format!("/messages/{}", id)).into_response()
}

pub async fn make_offer(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Path(convo_id): Path<String>,
    Form(form): Form<MakeOfferForm>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let convo = match db::get_conversation(&db, &convo_id) {
        Some(c) if c.buyer_id == user.id => c,
        _ => return Redirect::to("/messages").into_response(),
    };
    let amount: f64 = form.amount.replace('$', "").replace(',', "").parse().unwrap_or(0.0);
    if amount > 0.0 {
        db::create_offer(&db, &convo.listing_id, &convo_id, &user.id, amount);
        let msg = format!("💰 Offer: ${:.2}", amount);
        db::send_message(&db, &convo_id, &user.id, &msg);
    }
    Redirect::to(&format!("/messages/{}", convo_id)).into_response()
}

pub async fn respond_offer(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Path((convo_id, offer_id)): Path<(String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let accept = params.get("accept").map(|v| v == "true").unwrap_or(false);
    if db::respond_to_offer(&db, &offer_id, &user.id, accept) {
        let msg = if accept {
            "✅ Offer accepted! Check below for payment details."
        } else {
            "❌ Offer declined."
        };
        db::send_message(&db, &convo_id, &user.id, msg);
    }
    Redirect::to(&format!("/messages/{}", convo_id)).into_response()
}

// HTMX polling endpoint — returns new messages as HTML fragments
#[derive(serde::Deserialize)]
pub struct PollQuery {
    pub after: Option<String>,
}

pub async fn poll_messages(
    State((db, _tera)): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
    Query(query): Query<PollQuery>,
) -> Response {
    let user = match auth::get_current_user(&db, &jar) {
        Some(u) => u,
        None => return Html(String::new()).into_response(),
    };
    let convo = match db::get_conversation(&db, &id) {
        Some(c) if c.buyer_id == user.id || c.seller_id == user.id => c,
        _ => return Html(String::new()).into_response(),
    };

    let after_id = query.after.unwrap_or_default();
    if after_id.is_empty() {
        return Html(String::new()).into_response();
    }

    let new_msgs = db::get_messages_after(&db, &id, &after_id);
    if new_msgs.is_empty() {
        return Html(String::new()).into_response();
    }

    db::mark_conversation_read(&db, &user.id, &id);

    let mut html = String::new();
    let last_id = new_msgs.last().map(|m| m.id.clone()).unwrap_or(after_id);
    for msg in &new_msgs {
        let is_mine = msg.sender_id == user.id;
        let cls = if is_mine { "message-bubble mine" } else { "message-bubble theirs" };
        let time = time_ago(&msg.created_at);
        html.push_str(&format!(
            r##"<div class="{cls}">
                <div class="message-content">{content}</div>
                <span class="message-time">{time}</span>
            </div>"##,
            cls = cls, content = tera::escape_html(&msg.content), time = time,
        ));
    }
    // Update the polling URL with new last_id via OOB swap
    html.push_str(&format!(
        r##"<div id="message-poller" hx-get="/messages/{convo_id}/poll?after={last_id}" hx-trigger="every 2s" hx-target="#new-messages" hx-swap="beforeend" hx-swap-oob="true"></div>"##,
        convo_id = id, last_id = last_id,
    ));
    Html(html).into_response()
}
