# ⚒️ Forge Market

Peer-to-peer marketplace built with Rust, HTMX, and SQLite. No JavaScript frameworks, no middlemen, no bloat.

Think Facebook Marketplace — but built the right way.

## Stack

- **Axum 0.8** — async web framework
- **HTMX 2.0** — hypermedia-driven interactivity (live search, message polling)
- **Tera** — server-side templates
- **SQLite** — single-file database
- **Plain CSS** — no preprocessors, no Tailwind, no frameworks
- **argon2** — password hashing

## Quick Start

```bash
cargo build --release
PORT=8000 ./target/release/forge-commerce
```

Opens on `http://localhost:8000`.

## Demo Accounts

All passwords: `password123`

| Account | Email | Location |
|---------|-------|----------|
| Alice | alice@example.com | Austin, TX |
| Bob | bob@example.com | Portland, OR |
| Clara | clara@example.com | Denver, CO |

## Features

### Marketplace
- Browse listings with category/condition filters
- Live HTMX search (no page reload)
- Sort by price, date
- Condition tags (New, Like New, Good, Fair)
- Location-based listings

### Selling
- Any user can list items for sale
- Photo upload, category, condition, location
- Edit/delete your own listings
- Mark items as sold

### Messaging
- Direct buyer-seller chat per listing
- Real-time message polling (HTMX, 2s interval)
- Unread message badges
- Message inbox with conversation list

### Offers & Payments
- Buyers submit price offers in-chat
- Sellers accept/reject offers
- On acceptance: seller's payment info revealed to buyer
- Payment info configurable in profile (Venmo, PayPal, Zelle, etc.)
- Privacy: payment details hidden until offer accepted

### Auth
- Session-based authentication
- Argon2 password hashing
- Editable user profiles (name, location, bio, payment info)

## Routes

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Marketplace feed |
| GET | `/search` | HTMX search partial |
| GET | `/listing/{id}` | Listing detail |
| GET/POST | `/sell` | Create listing |
| POST | `/listing/{id}/edit` | Edit listing |
| POST | `/listing/{id}/sold` | Mark as sold |
| GET | `/listing/{id}/contact` | Start conversation |
| GET | `/messages` | Message inbox |
| GET | `/messages/{id}` | Conversation view |
| POST | `/messages/{id}/send` | Send message |
| POST | `/messages/{id}/offer` | Make offer |
| GET | `/messages/{id}/poll` | HTMX message polling |
| GET/POST | `/login` | Login |
| GET/POST | `/register` | Register |
| GET/POST | `/profile` | Profile |
| GET | `/health` | Health check |

## Development

```bash
cargo run
```

## Philosophy

Every line earns its place. No runtime CDNs, no React, no node_modules. Server renders HTML, HTMX handles interactivity, CSS handles styling. The way it should be.
