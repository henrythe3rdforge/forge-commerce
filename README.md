# 🔨 Forge Commerce

E-commerce application built with Rust, HTMX, and SQLite. No JavaScript frameworks, no bloat.

## Stack

- **Axum 0.8** — async web framework
- **HTMX 2.0** — hypermedia-driven interactivity
- **Tera** — server-side templates
- **SQLite** — single-file database
- **Plain CSS** — no preprocessors, no frameworks
- **argon2** — password hashing

## Quick Start

```bash
chmod +x start.sh stop.sh status.sh
./start.sh
```

Opens on `http://localhost:3000` (or next free port).

## Default Admin

- **Email:** admin@forge.com
- **Password:** admin123

## Features

- Product catalog with search, filtering, sorting
- Shopping cart with HTMX partial updates
- Multi-step checkout flow
- User authentication with session cookies
- Admin panel with product CRUD and order management
- Dark/light mode (CSS-only, no JS)
- Mobile-responsive design
- Toast notifications for cart actions

## Scripts

| Script | Purpose |
|--------|---------|
| `start.sh` | Build release, download HTMX, find free port, start |
| `stop.sh` | Graceful shutdown |
| `status.sh` | Check if running |

## API Routes

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Home page |
| GET | `/products` | Product catalog |
| GET | `/products/search` | HTMX search endpoint |
| GET | `/products/{id}` | Product detail |
| GET | `/cart` | View cart |
| POST | `/api/cart/add` | Add to cart |
| PUT | `/api/cart/{id}` | Update cart item |
| DELETE | `/api/cart/{id}` | Remove from cart |
| GET | `/api/cart/count` | Cart count (polling) |
| POST | `/login` | Login |
| POST | `/register` | Register |
| GET | `/checkout` | Checkout flow |
| GET | `/admin` | Admin dashboard |
| GET | `/health` | Health check |

## Development

```bash
source ~/.cargo/env
cargo run
```

## Testing

```bash
cargo test
```
