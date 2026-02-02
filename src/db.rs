use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use crate::models::*;

pub type Db = Arc<Mutex<Connection>>;

pub fn init_db() -> Db {
    let conn = Connection::open("forge-commerce.db").expect("Failed to open database");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap();
    let db = Arc::new(Mutex::new(conn));
    run_migrations(&db);
    seed_data(&db);
    db
}

pub fn init_db_with_path(path: &str) -> Db {
    let conn = Connection::open(path).expect("Failed to open database");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap();
    let db = Arc::new(Mutex::new(conn));
    run_migrations(&db);
    db
}

pub fn run_migrations(db: &Db) {
    let conn = db.lock().unwrap();
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS products (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            price REAL NOT NULL,
            category TEXT NOT NULL,
            image_url TEXT NOT NULL DEFAULT '/static/images/placeholder.svg',
            featured INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            is_admin INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            expires_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS cart_items (
            id TEXT PRIMARY KEY,
            session_token TEXT NOT NULL,
            product_id TEXT NOT NULL REFERENCES products(id),
            quantity INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS orders (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            status TEXT NOT NULL DEFAULT 'pending',
            total REAL NOT NULL,
            shipping_name TEXT NOT NULL DEFAULT '',
            shipping_address TEXT NOT NULL DEFAULT '',
            shipping_city TEXT NOT NULL DEFAULT '',
            shipping_zip TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS order_items (
            id TEXT PRIMARY KEY,
            order_id TEXT NOT NULL REFERENCES orders(id),
            product_id TEXT NOT NULL,
            product_name TEXT NOT NULL,
            price REAL NOT NULL,
            quantity INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS addresses (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            city TEXT NOT NULL,
            zip TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);
        CREATE INDEX IF NOT EXISTS idx_cart_session ON cart_items(session_token);
        CREATE INDEX IF NOT EXISTS idx_orders_user ON orders(user_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
    ").expect("Failed to run migrations");
}

fn seed_data(db: &Db) {
    let conn = db.lock().unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0)).unwrap();
    if count > 0 { return; }

    let products = vec![
        ("Artisan Ceramic Mug", "Hand-thrown stoneware mug with a reactive glaze finish. Holds 12oz comfortably. Microwave and dishwasher safe.", 34.99, "Home & Kitchen", true),
        ("Walnut Cutting Board", "End-grain walnut cutting board, 18x12 inches. Finished with food-safe mineral oil. Built to last generations.", 89.00, "Home & Kitchen", true),
        ("Cast Iron Skillet 12\"", "Pre-seasoned cast iron skillet. Even heat distribution, oven safe to 500°F. The only pan you'll ever need.", 45.00, "Home & Kitchen", false),
        ("Linen Tea Towels (Set of 3)", "Stonewashed Belgian linen tea towels in neutral tones. Absorbent, quick-drying, and gets softer with every wash.", 28.00, "Home & Kitchen", false),
        ("Copper Pour-Over Kettle", "Gooseneck copper kettle with precision spout. 1.2L capacity with built-in thermometer. Pour with control.", 72.00, "Home & Kitchen", true),
        ("Merino Wool Beanie", "100% extra-fine merino wool. Breathable, temperature-regulating, and itch-free. One size fits most.", 38.00, "Apparel", true),
        ("Heavyweight Cotton Tee", "8oz organic cotton, garment-dyed for a lived-in feel. Relaxed fit, reinforced collar. Made in Portugal.", 42.00, "Apparel", false),
        ("Waxed Canvas Tote", "Water-resistant waxed cotton canvas with leather handles. 18L capacity. Ages beautifully with patina.", 78.00, "Apparel", true),
        ("Selvedge Denim Jeans", "14oz Japanese selvedge denim, raw indigo. Slim-straight fit. Will mold to your shape over time.", 148.00, "Apparel", false),
        ("Alpaca Wool Scarf", "Baby alpaca wool in a herringbone weave. Incredibly soft, lightweight, and warm. Handwoven in Peru.", 65.00, "Apparel", false),
        ("Brass Desk Lamp", "Adjustable solid brass lamp with a linen shade. Warm, focused light for reading or working. E26 bulb included.", 120.00, "Lighting", true),
        ("Concrete Pendant Light", "Minimalist concrete pendant with a smooth interior finish. 8\" diameter. Includes 4ft adjustable cord.", 95.00, "Lighting", false),
        ("Beeswax Taper Candles (Pair)", "Hand-dipped pure beeswax tapers, 10\" tall. Burns clean with a subtle honey scent. Approximately 8 hours each.", 22.00, "Lighting", false),
        ("Japanese Paper Lantern", "Handmade washi paper lantern, 16\" diameter. Creates soft, diffused ambient light. Collapsible for storage.", 55.00, "Lighting", true),
        ("Soy Candle — Cedar & Sage", "Hand-poured soy wax with essential oils. 8oz tin, 45+ hour burn time. Clean-burning cotton wick.", 28.00, "Lighting", false),
        ("Leather Journal A5", "Full-grain vegetable-tanned leather cover with 240 pages of 100gsm cream paper. Lay-flat binding.", 52.00, "Stationery", true),
        ("Fountain Pen — Matte Black", "Brass body with matte black finish. Medium nib, smooth ink flow. Uses standard international cartridges.", 85.00, "Stationery", true),
        ("Letterpress Notebook Set", "Set of 3 pocket notebooks with letterpress covers. 48 pages each, dot grid. Perfect for quick notes.", 18.00, "Stationery", false),
        ("Wooden Pen Tray", "Solid oak desk tray with three compartments. Oil-finished with felt-lined base. 10\" x 4\" x 1.5\".", 35.00, "Stationery", false),
        ("Wax Seal Kit", "Brass seal stamp with your choice of initial, plus a stick of forest green sealing wax. Makes any letter special.", 44.00, "Stationery", false),
        ("Steel Water Bottle", "Double-wall vacuum insulated, 750ml. Keeps drinks cold 24hrs or hot 12hrs. Powder-coated matte finish.", 32.00, "Home & Kitchen", false),
        ("Olive Wood Salad Servers", "Hand-carved olive wood serving set. Each piece is unique with natural grain patterns. 12\" long.", 38.00, "Home & Kitchen", false),
    ];

    for (name, desc, price, category, featured) in products {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO products (id, name, description, price, category, image_url, featured) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, name, desc, price, category, "/static/images/placeholder.svg", featured as i32],
        ).unwrap();
    }

    // Create admin user (password: admin123)
    let admin_id = uuid::Uuid::new_v4().to_string();
    let admin_hash = crate::auth::hash_password("admin123");
    conn.execute(
        "INSERT INTO users (id, email, name, password_hash, is_admin) VALUES (?1, ?2, ?3, ?4, 1)",
        params![admin_id, "admin@forge.com", "Admin", admin_hash],
    ).unwrap();
}

// Product queries
pub fn get_all_products(db: &Db) -> Vec<Product> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, description, price, category, image_url, featured, created_at FROM products ORDER BY created_at DESC").unwrap();
    stmt.query_map([], |row| {
        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            price: row.get(3)?,
            category: row.get(4)?,
            image_url: row.get(5)?,
            featured: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_featured_products(db: &Db) -> Vec<Product> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, description, price, category, image_url, featured, created_at FROM products WHERE featured = 1 LIMIT 8").unwrap();
    stmt.query_map([], |row| {
        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            price: row.get(3)?,
            category: row.get(4)?,
            image_url: row.get(5)?,
            featured: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_product(db: &Db, id: &str) -> Option<Product> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, name, description, price, category, image_url, featured, created_at FROM products WHERE id = ?1",
        params![id],
        |row| Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            price: row.get(3)?,
            category: row.get(4)?,
            image_url: row.get(5)?,
            featured: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
        })
    ).ok()
}

pub fn search_products(db: &Db, query: &SearchQuery) -> Vec<Product> {
    let conn = db.lock().unwrap();
    let mut sql = String::from("SELECT id, name, description, price, category, image_url, featured, created_at FROM products WHERE 1=1");
    let mut param_values: Vec<String> = Vec::new();

    if let Some(q) = &query.q {
        if !q.is_empty() {
            sql.push_str(&format!(" AND (name LIKE '%' || ?{} || '%' OR description LIKE '%' || ?{} || '%')", param_values.len() + 1, param_values.len() + 1));
            param_values.push(q.clone());
        }
    }
    if let Some(cat) = &query.category {
        if !cat.is_empty() {
            sql.push_str(&format!(" AND category = ?{}", param_values.len() + 1));
            param_values.push(cat.clone());
        }
    }

    let order = match query.sort.as_deref() {
        Some("price_asc") => "price ASC",
        Some("price_desc") => "price DESC",
        Some("name") => "name ASC",
        _ => "created_at DESC",
    };
    sql.push_str(&format!(" ORDER BY {}", order));

    let mut stmt = conn.prepare(&sql).unwrap();
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    stmt.query_map(params_refs.as_slice(), |row| {
        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            price: row.get(3)?,
            category: row.get(4)?,
            image_url: row.get(5)?,
            featured: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_related_products(db: &Db, product: &Product) -> Vec<Product> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, description, price, category, image_url, featured, created_at FROM products WHERE category = ?1 AND id != ?2 LIMIT 4").unwrap();
    stmt.query_map(params![product.category, product.id], |row| {
        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            price: row.get(3)?,
            category: row.get(4)?,
            image_url: row.get(5)?,
            featured: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_categories(db: &Db) -> Vec<Category> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT category, COUNT(*) FROM products GROUP BY category ORDER BY category").unwrap();
    stmt.query_map([], |row| {
        Ok(Category { name: row.get(0)?, count: row.get(1)? })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn create_product(db: &Db, form: &ProductForm, image_url: &str) -> String {
    let conn = db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let price: f64 = form.price.parse().unwrap_or(0.0);
    let featured = form.featured.is_some();
    conn.execute(
        "INSERT INTO products (id, name, description, price, category, image_url, featured) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, form.name, form.description, price, form.category, image_url, featured as i32],
    ).unwrap();
    id
}

pub fn update_product(db: &Db, id: &str, form: &ProductForm, image_url: Option<&str>) {
    let conn = db.lock().unwrap();
    let price: f64 = form.price.parse().unwrap_or(0.0);
    let featured = form.featured.is_some();
    if let Some(url) = image_url {
        conn.execute(
            "UPDATE products SET name=?1, description=?2, price=?3, category=?4, image_url=?5, featured=?6 WHERE id=?7",
            params![form.name, form.description, price, form.category, url, featured as i32, id],
        ).unwrap();
    } else {
        conn.execute(
            "UPDATE products SET name=?1, description=?2, price=?3, category=?4, featured=?5 WHERE id=?6",
            params![form.name, form.description, price, form.category, featured as i32, id],
        ).unwrap();
    }
}

pub fn delete_product(db: &Db, id: &str) {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM products WHERE id = ?1", params![id]).unwrap();
}

// Cart queries
pub fn get_cart_items(db: &Db, session_token: &str) -> Vec<CartItem> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT c.product_id, p.name, p.price, c.quantity, p.image_url
         FROM cart_items c JOIN products p ON c.product_id = p.id
         WHERE c.session_token = ?1"
    ).unwrap();
    stmt.query_map(params![session_token], |row| {
        Ok(CartItem {
            product_id: row.get(0)?,
            product_name: row.get(1)?,
            price: row.get(2)?,
            quantity: row.get(3)?,
            image_url: row.get(4)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn add_to_cart(db: &Db, session_token: &str, product_id: &str, qty: i32) {
    let conn = db.lock().unwrap();
    let existing: Option<(String, i32)> = conn.query_row(
        "SELECT id, quantity FROM cart_items WHERE session_token = ?1 AND product_id = ?2",
        params![session_token, product_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).ok();

    if let Some((id, current_qty)) = existing {
        conn.execute("UPDATE cart_items SET quantity = ?1 WHERE id = ?2", params![current_qty + qty, id]).unwrap();
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO cart_items (id, session_token, product_id, quantity) VALUES (?1, ?2, ?3, ?4)",
            params![id, session_token, product_id, qty],
        ).unwrap();
    }
}

pub fn update_cart_item(db: &Db, session_token: &str, product_id: &str, qty: i32) {
    let conn = db.lock().unwrap();
    if qty <= 0 {
        conn.execute("DELETE FROM cart_items WHERE session_token = ?1 AND product_id = ?2", params![session_token, product_id]).unwrap();
    } else {
        conn.execute(
            "UPDATE cart_items SET quantity = ?1 WHERE session_token = ?2 AND product_id = ?3",
            params![qty, session_token, product_id],
        ).unwrap();
    }
}

pub fn remove_from_cart(db: &Db, session_token: &str, product_id: &str) {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM cart_items WHERE session_token = ?1 AND product_id = ?2", params![session_token, product_id]).unwrap();
}

pub fn clear_cart(db: &Db, session_token: &str) {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM cart_items WHERE session_token = ?1", params![session_token]).unwrap();
}

pub fn get_cart_count(db: &Db, session_token: &str) -> i32 {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT COALESCE(SUM(quantity), 0) FROM cart_items WHERE session_token = ?1",
        params![session_token],
        |row| row.get(0)
    ).unwrap_or(0)
}

// User queries
pub fn create_user(db: &Db, name: &str, email: &str, password_hash: &str) -> Result<String, String> {
    let conn = db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO users (id, email, name, password_hash) VALUES (?1, ?2, ?3, ?4)",
        params![id, email, name, password_hash],
    ).map_err(|e| {
        if e.to_string().contains("UNIQUE") { "Email already registered".to_string() }
        else { e.to_string() }
    })?;
    Ok(id)
}

pub fn get_user_by_email(db: &Db, email: &str) -> Option<User> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, email, name, password_hash, is_admin, created_at FROM users WHERE email = ?1",
        params![email],
        |row| Ok(User {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            password_hash: row.get(3)?,
            is_admin: row.get::<_, i32>(4)? != 0,
            created_at: row.get(5)?,
        })
    ).ok()
}

pub fn get_user_by_id(db: &Db, id: &str) -> Option<User> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, email, name, password_hash, is_admin, created_at FROM users WHERE id = ?1",
        params![id],
        |row| Ok(User {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            password_hash: row.get(3)?,
            is_admin: row.get::<_, i32>(4)? != 0,
            created_at: row.get(5)?,
        })
    ).ok()
}

// Session queries
pub fn create_session(db: &Db, user_id: &str) -> String {
    let conn = db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES (?1, ?2, datetime('now', '+7 days'))",
        params![id, user_id],
    ).unwrap();
    id
}

pub fn get_session_user(db: &Db, session_id: &str) -> Option<User> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT u.id, u.email, u.name, u.password_hash, u.is_admin, u.created_at
         FROM sessions s JOIN users u ON s.user_id = u.id
         WHERE s.id = ?1 AND s.expires_at > datetime('now')",
        params![session_id],
        |row| Ok(User {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            password_hash: row.get(3)?,
            is_admin: row.get::<_, i32>(4)? != 0,
            created_at: row.get(5)?,
        })
    ).ok()
}

pub fn delete_session(db: &Db, session_id: &str) {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id]).unwrap();
}

// Order queries
pub fn create_order(db: &Db, user_id: &str, items: &[CartItem], shipping: &ShippingForm) -> String {
    let conn = db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    conn.execute(
        "INSERT INTO orders (id, user_id, total, shipping_name, shipping_address, shipping_city, shipping_zip) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, user_id, total, shipping.name, shipping.address, shipping.city, shipping.zip],
    ).unwrap();

    for item in items {
        let item_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO order_items (id, order_id, product_id, product_name, price, quantity) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![item_id, id, item.product_id, item.product_name, item.price, item.quantity],
        ).unwrap();
    }
    id
}

pub fn get_user_orders(db: &Db, user_id: &str) -> Vec<Order> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, user_id, status, total, shipping_name, shipping_address, shipping_city, shipping_zip, created_at FROM orders WHERE user_id = ?1 ORDER BY created_at DESC"
    ).unwrap();
    stmt.query_map(params![user_id], |row| {
        Ok(Order {
            id: row.get(0)?, user_id: row.get(1)?, status: row.get(2)?, total: row.get(3)?,
            shipping_name: row.get(4)?, shipping_address: row.get(5)?, shipping_city: row.get(6)?,
            shipping_zip: row.get(7)?, created_at: row.get(8)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_all_orders(db: &Db) -> Vec<Order> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, user_id, status, total, shipping_name, shipping_address, shipping_city, shipping_zip, created_at FROM orders ORDER BY created_at DESC"
    ).unwrap();
    stmt.query_map([], |row| {
        Ok(Order {
            id: row.get(0)?, user_id: row.get(1)?, status: row.get(2)?, total: row.get(3)?,
            shipping_name: row.get(4)?, shipping_address: row.get(5)?, shipping_city: row.get(6)?,
            shipping_zip: row.get(7)?, created_at: row.get(8)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_order(db: &Db, id: &str) -> Option<Order> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, user_id, status, total, shipping_name, shipping_address, shipping_city, shipping_zip, created_at FROM orders WHERE id = ?1",
        params![id],
        |row| Ok(Order {
            id: row.get(0)?, user_id: row.get(1)?, status: row.get(2)?, total: row.get(3)?,
            shipping_name: row.get(4)?, shipping_address: row.get(5)?, shipping_city: row.get(6)?,
            shipping_zip: row.get(7)?, created_at: row.get(8)?,
        })
    ).ok()
}

pub fn get_order_items(db: &Db, order_id: &str) -> Vec<OrderItem> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, order_id, product_id, product_name, price, quantity FROM order_items WHERE order_id = ?1"
    ).unwrap();
    stmt.query_map(params![order_id], |row| {
        Ok(OrderItem {
            id: row.get(0)?, order_id: row.get(1)?, product_id: row.get(2)?,
            product_name: row.get(3)?, price: row.get(4)?, quantity: row.get(5)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn update_order_status(db: &Db, id: &str, status: &str) {
    let conn = db.lock().unwrap();
    conn.execute("UPDATE orders SET status = ?1 WHERE id = ?2", params![status, id]).unwrap();
}

// Admin stats
pub fn get_admin_stats(db: &Db) -> (i64, i64, f64, i64) {
    let conn = db.lock().unwrap();
    let product_count: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0)).unwrap_or(0);
    let order_count: i64 = conn.query_row("SELECT COUNT(*) FROM orders", [], |r| r.get(0)).unwrap_or(0);
    let revenue: f64 = conn.query_row("SELECT COALESCE(SUM(total), 0) FROM orders", [], |r| r.get(0)).unwrap_or(0.0);
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0)).unwrap_or(0);
    (product_count, order_count, revenue, user_count)
}

// Address queries
pub fn get_user_addresses(db: &Db, user_id: &str) -> Vec<Address> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, user_id, name, address, city, zip, is_default FROM addresses WHERE user_id = ?1"
    ).unwrap();
    stmt.query_map(params![user_id], |row| {
        Ok(Address {
            id: row.get(0)?, user_id: row.get(1)?, name: row.get(2)?, address: row.get(3)?,
            city: row.get(4)?, zip: row.get(5)?, is_default: row.get::<_, i32>(6)? != 0,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn save_address(db: &Db, user_id: &str, shipping: &ShippingForm) {
    let conn = db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO addresses (id, user_id, name, address, city, zip) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, user_id, shipping.name, shipping.address, shipping.city, shipping.zip],
    ).unwrap();
}
