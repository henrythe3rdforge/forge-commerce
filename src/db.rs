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
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            location TEXT NOT NULL DEFAULT '',
            avatar_url TEXT NOT NULL DEFAULT '',
            payment_info TEXT NOT NULL DEFAULT '',
            bio TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS listings (
            id TEXT PRIMARY KEY,
            seller_id TEXT NOT NULL REFERENCES users(id),
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            price REAL NOT NULL,
            category TEXT NOT NULL,
            condition TEXT NOT NULL DEFAULT 'Good',
            location TEXT NOT NULL DEFAULT '',
            image_url TEXT NOT NULL DEFAULT '/static/images/placeholder.svg',
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            listing_id TEXT NOT NULL REFERENCES listings(id),
            buyer_id TEXT NOT NULL REFERENCES users(id),
            seller_id TEXT NOT NULL REFERENCES users(id),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(listing_id, buyer_id)
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL REFERENCES conversations(id),
            sender_id TEXT NOT NULL REFERENCES users(id),
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS offers (
            id TEXT PRIMARY KEY,
            listing_id TEXT NOT NULL REFERENCES listings(id),
            conversation_id TEXT NOT NULL REFERENCES conversations(id),
            buyer_id TEXT NOT NULL REFERENCES users(id),
            amount REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS message_reads (
            user_id TEXT NOT NULL REFERENCES users(id),
            conversation_id TEXT NOT NULL REFERENCES conversations(id),
            last_read_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (user_id, conversation_id)
        );

        CREATE INDEX IF NOT EXISTS idx_listings_seller ON listings(seller_id);
        CREATE INDEX IF NOT EXISTS idx_listings_category ON listings(category);
        CREATE INDEX IF NOT EXISTS idx_listings_status ON listings(status);
        CREATE INDEX IF NOT EXISTS idx_conversations_buyer ON conversations(buyer_id);
        CREATE INDEX IF NOT EXISTS idx_conversations_seller ON conversations(seller_id);
        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);
        CREATE INDEX IF NOT EXISTS idx_offers_listing ON offers(listing_id);
    ").expect("Failed to run migrations");
}

fn seed_data(db: &Db) {
    let conn = db.lock().unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0)).unwrap();
    if count > 0 { return; }

    // Create demo users
    let users = vec![
        ("Alice", "alice@example.com", "Austin, TX", "Venmo: @alice-sells", "Love vintage finds and handmade goods."),
        ("Bob", "bob@example.com", "Portland, OR", "PayPal: bob@email.com", "Woodworker and tool collector."),
        ("Clara", "clara@example.com", "Denver, CO", "Zelle: 555-0123", "Minimalist. Selling things I no longer need."),
    ];

    let mut user_ids = Vec::new();
    for (name, email, location, payment, bio) in &users {
        let id = uuid::Uuid::new_v4().to_string();
        let hash = crate::auth::hash_password("password123");
        conn.execute(
            "INSERT INTO users (id, email, name, password_hash, location, payment_info, bio) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, email, name, hash, location, payment, bio],
        ).unwrap();
        user_ids.push(id);
    }

    // Create demo listings
    let listings = vec![
        (&user_ids[0], "Vintage Le Creuset Dutch Oven", "5.5 qt in flame orange. Some patina on the exterior but the enamel interior is perfect. These last forever.", 85.00, "Home & Kitchen", "Good", "Austin, TX"),
        (&user_ids[0], "Handmade Ceramic Mug Set (4)", "Hand-thrown stoneware mugs with a reactive glaze. Each one is unique. Holds about 12oz.", 48.00, "Home & Kitchen", "Like New", "Austin, TX"),
        (&user_ids[1], "Lie-Nielsen No. 4 Smoothing Plane", "Bronze body, A2 steel blade. Used on maybe 3 projects. Incredible tool, just downsizing the shop.", 295.00, "Tools", "Like New", "Portland, OR"),
        (&user_ids[1], "Japanese Pull Saw Set", "Ryoba and Dozuki pair. Razor sharp, barely used. Great for fine joinery work.", 65.00, "Tools", "Like New", "Portland, OR"),
        (&user_ids[1], "Waxed Canvas Shop Apron", "Heavy-duty waxed cotton with leather straps. Has character — some wax wear and a few stains.", 35.00, "Apparel", "Good", "Portland, OR"),
        (&user_ids[2], "Herman Miller Aeron Chair", "Size B, fully loaded with PostureFit. Some wear on the mesh but fully functional. Pickup only.", 450.00, "Furniture", "Good", "Denver, CO"),
        (&user_ids[2], "Wool Pendleton Blanket", "Queen size, Rob Roy tartan pattern. 100% virgin wool. Washed once, stored in cedar chest.", 120.00, "Home & Kitchen", "Like New", "Denver, CO"),
        (&user_ids[0], "1960s Brass Desk Lamp", "Adjustable arm, original patina. Rewired with a new cloth cord for safety. Works perfectly.", 75.00, "Lighting", "Good", "Austin, TX"),
        (&user_ids[2], "Leuchtturm1917 Notebooks (5 pack)", "A5 dot grid, assorted colors. Bought too many. Still sealed in plastic.", 55.00, "Stationery", "New", "Denver, CO"),
        (&user_ids[1], "Cast Iron Skillet 12\" — Lodge", "Freshly re-seasoned. Smooth cooking surface from years of use. Better than anything new.", 40.00, "Home & Kitchen", "Good", "Portland, OR"),
        (&user_ids[0], "Merino Wool Beanie — Hand Knit", "100% merino, charcoal gray. Made this myself. Fits most heads comfortably.", 28.00, "Apparel", "New", "Austin, TX"),
        (&user_ids[2], "Mid-Century Teak Bookshelf", "Danish modern style, 5 shelves. Some minor scratches on top. Solid teak, not veneer.", 280.00, "Furniture", "Good", "Denver, CO"),
    ];

    for (seller_id, title, desc, price, category, condition, location) in listings {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO listings (id, seller_id, title, description, price, category, condition, location, image_url) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, seller_id, title, desc, price, category, condition, location, "/static/images/placeholder.svg"],
        ).unwrap();
    }
}

// === Listing queries ===

pub fn get_listings(db: &Db, query: &SearchQuery) -> Vec<Listing> {
    let conn = db.lock().unwrap();
    let mut sql = String::from(
        "SELECT l.id, l.seller_id, u.name, l.title, l.description, l.price, l.category, l.condition, l.location, l.image_url, l.status, l.created_at
         FROM listings l JOIN users u ON l.seller_id = u.id WHERE l.status = 'active'"
    );
    let mut param_values: Vec<String> = Vec::new();

    if let Some(q) = &query.q {
        if !q.is_empty() {
            let idx = param_values.len() + 1;
            sql.push_str(&format!(" AND (l.title LIKE '%' || ?{} || '%' OR l.description LIKE '%' || ?{} || '%')", idx, idx));
            param_values.push(q.clone());
        }
    }
    if let Some(cat) = &query.category {
        if !cat.is_empty() {
            let idx = param_values.len() + 1;
            sql.push_str(&format!(" AND l.category = ?{}", idx));
            param_values.push(cat.clone());
        }
    }
    if let Some(cond) = &query.condition {
        if !cond.is_empty() {
            let idx = param_values.len() + 1;
            sql.push_str(&format!(" AND l.condition = ?{}", idx));
            param_values.push(cond.clone());
        }
    }
    if let Some(min) = &query.min_price {
        if let Ok(v) = min.parse::<f64>() {
            let idx = param_values.len() + 1;
            sql.push_str(&format!(" AND l.price >= ?{}", idx));
            param_values.push(v.to_string());
        }
    }
    if let Some(max) = &query.max_price {
        if let Ok(v) = max.parse::<f64>() {
            let idx = param_values.len() + 1;
            sql.push_str(&format!(" AND l.price <= ?{}", idx));
            param_values.push(v.to_string());
        }
    }

    let order = match query.sort.as_deref() {
        Some("price_asc") => "l.price ASC",
        Some("price_desc") => "l.price DESC",
        Some("oldest") => "l.created_at ASC",
        _ => "l.created_at DESC",
    };
    sql.push_str(&format!(" ORDER BY {}", order));
    sql.push_str(" LIMIT 50");

    let mut stmt = conn.prepare(&sql).unwrap();
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    stmt.query_map(params_refs.as_slice(), |row| {
        Ok(Listing {
            id: row.get(0)?, seller_id: row.get(1)?, seller_name: row.get(2)?,
            title: row.get(3)?, description: row.get(4)?, price: row.get(5)?,
            category: row.get(6)?, condition: row.get(7)?, location: row.get(8)?,
            image_url: row.get(9)?, status: row.get(10)?, created_at: row.get(11)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_listing(db: &Db, id: &str) -> Option<Listing> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT l.id, l.seller_id, u.name, l.title, l.description, l.price, l.category, l.condition, l.location, l.image_url, l.status, l.created_at
         FROM listings l JOIN users u ON l.seller_id = u.id WHERE l.id = ?1",
        params![id],
        |row| Ok(Listing {
            id: row.get(0)?, seller_id: row.get(1)?, seller_name: row.get(2)?,
            title: row.get(3)?, description: row.get(4)?, price: row.get(5)?,
            category: row.get(6)?, condition: row.get(7)?, location: row.get(8)?,
            image_url: row.get(9)?, status: row.get(10)?, created_at: row.get(11)?,
        })
    ).ok()
}

pub fn get_user_listings(db: &Db, user_id: &str) -> Vec<Listing> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT l.id, l.seller_id, u.name, l.title, l.description, l.price, l.category, l.condition, l.location, l.image_url, l.status, l.created_at
         FROM listings l JOIN users u ON l.seller_id = u.id WHERE l.seller_id = ?1 ORDER BY l.created_at DESC"
    ).unwrap();
    stmt.query_map(params![user_id], |row| {
        Ok(Listing {
            id: row.get(0)?, seller_id: row.get(1)?, seller_name: row.get(2)?,
            title: row.get(3)?, description: row.get(4)?, price: row.get(5)?,
            category: row.get(6)?, condition: row.get(7)?, location: row.get(8)?,
            image_url: row.get(9)?, status: row.get(10)?, created_at: row.get(11)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn create_listing(db: &Db, seller_id: &str, form: &ListingForm, image_url: &str) -> String {
    let conn = db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let price: f64 = form.price.parse().unwrap_or(0.0);
    conn.execute(
        "INSERT INTO listings (id, seller_id, title, description, price, category, condition, location, image_url) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, seller_id, form.title, form.description, price, form.category, form.condition, form.location, image_url],
    ).unwrap();
    id
}

pub fn update_listing(db: &Db, id: &str, seller_id: &str, form: &ListingForm, image_url: Option<&str>) -> bool {
    let conn = db.lock().unwrap();
    let price: f64 = form.price.parse().unwrap_or(0.0);
    let rows = if let Some(url) = image_url {
        conn.execute(
            "UPDATE listings SET title=?1, description=?2, price=?3, category=?4, condition=?5, location=?6, image_url=?7 WHERE id=?8 AND seller_id=?9",
            params![form.title, form.description, price, form.category, form.condition, form.location, url, id, seller_id],
        ).unwrap_or(0)
    } else {
        conn.execute(
            "UPDATE listings SET title=?1, description=?2, price=?3, category=?4, condition=?5, location=?6 WHERE id=?7 AND seller_id=?8",
            params![form.title, form.description, price, form.category, form.condition, form.location, id, seller_id],
        ).unwrap_or(0)
    };
    rows > 0
}

pub fn update_listing_status(db: &Db, id: &str, seller_id: &str, status: &str) -> bool {
    let conn = db.lock().unwrap();
    let rows = conn.execute(
        "UPDATE listings SET status = ?1 WHERE id = ?2 AND seller_id = ?3",
        params![status, id, seller_id],
    ).unwrap_or(0);
    rows > 0
}

pub fn delete_listing(db: &Db, id: &str, seller_id: &str) -> bool {
    let conn = db.lock().unwrap();
    let rows = conn.execute("DELETE FROM listings WHERE id = ?1 AND seller_id = ?2", params![id, seller_id]).unwrap_or(0);
    rows > 0
}

pub fn get_categories(db: &Db) -> Vec<Category> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT category, COUNT(*) FROM listings WHERE status = 'active' GROUP BY category ORDER BY category").unwrap();
    stmt.query_map([], |row| {
        Ok(Category { name: row.get(0)?, count: row.get(1)? })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_seller_listings(db: &Db, seller_id: &str, exclude_id: &str) -> Vec<Listing> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT l.id, l.seller_id, u.name, l.title, l.description, l.price, l.category, l.condition, l.location, l.image_url, l.status, l.created_at
         FROM listings l JOIN users u ON l.seller_id = u.id WHERE l.seller_id = ?1 AND l.id != ?2 AND l.status = 'active' LIMIT 4"
    ).unwrap();
    stmt.query_map(params![seller_id, exclude_id], |row| {
        Ok(Listing {
            id: row.get(0)?, seller_id: row.get(1)?, seller_name: row.get(2)?,
            title: row.get(3)?, description: row.get(4)?, price: row.get(5)?,
            category: row.get(6)?, condition: row.get(7)?, location: row.get(8)?,
            image_url: row.get(9)?, status: row.get(10)?, created_at: row.get(11)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

// === Conversation queries ===

pub fn get_or_create_conversation(db: &Db, listing_id: &str, buyer_id: &str, seller_id: &str) -> String {
    let conn = db.lock().unwrap();
    // Check if conversation exists
    if let Ok(id) = conn.query_row(
        "SELECT id FROM conversations WHERE listing_id = ?1 AND buyer_id = ?2",
        params![listing_id, buyer_id],
        |row| row.get::<_, String>(0),
    ) {
        return id;
    }
    // Create new
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO conversations (id, listing_id, buyer_id, seller_id) VALUES (?1, ?2, ?3, ?4)",
        params![id, listing_id, buyer_id, seller_id],
    ).unwrap();
    id
}

pub fn get_user_conversations(db: &Db, user_id: &str) -> Vec<Conversation> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT c.id, c.listing_id, l.title, l.image_url, c.buyer_id, bu.name, c.seller_id, su.name,
                COALESCE((SELECT content FROM messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1), ''),
                COALESCE((SELECT created_at FROM messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1), c.created_at),
                COALESCE((SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id
                    AND m.created_at > COALESCE((SELECT last_read_at FROM message_reads WHERE user_id = ?1 AND conversation_id = c.id), '1970-01-01')
                    AND m.sender_id != ?1), 0)
         FROM conversations c
         JOIN listings l ON c.listing_id = l.id
         JOIN users bu ON c.buyer_id = bu.id
         JOIN users su ON c.seller_id = su.id
         WHERE c.buyer_id = ?1 OR c.seller_id = ?1
         ORDER BY 10 DESC"
    ).unwrap();
    stmt.query_map(params![user_id], |row| {
        Ok(Conversation {
            id: row.get(0)?, listing_id: row.get(1)?, listing_title: row.get(2)?,
            listing_image: row.get(3)?, buyer_id: row.get(4)?, buyer_name: row.get(5)?,
            seller_id: row.get(6)?, seller_name: row.get(7)?, last_message: row.get(8)?,
            last_message_at: row.get(9)?, unread_count: row.get(10)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_conversation(db: &Db, id: &str) -> Option<Conversation> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT c.id, c.listing_id, l.title, l.image_url, c.buyer_id, bu.name, c.seller_id, su.name,
                COALESCE((SELECT content FROM messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1), ''),
                COALESCE((SELECT created_at FROM messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1), c.created_at),
                0
         FROM conversations c
         JOIN listings l ON c.listing_id = l.id
         JOIN users bu ON c.buyer_id = bu.id
         JOIN users su ON c.seller_id = su.id
         WHERE c.id = ?1",
        params![id],
        |row| Ok(Conversation {
            id: row.get(0)?, listing_id: row.get(1)?, listing_title: row.get(2)?,
            listing_image: row.get(3)?, buyer_id: row.get(4)?, buyer_name: row.get(5)?,
            seller_id: row.get(6)?, seller_name: row.get(7)?, last_message: row.get(8)?,
            last_message_at: row.get(9)?, unread_count: row.get(10)?,
        })
    ).ok()
}

// === Message queries ===

pub fn get_messages(db: &Db, conversation_id: &str) -> Vec<Message> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT m.id, m.conversation_id, m.sender_id, u.name, m.content, m.created_at,
                COALESCE(o.id, '') as offer_id, COALESCE(o.amount, 0) as offer_amount, COALESCE(o.status, '') as offer_status
         FROM messages m
         JOIN users u ON m.sender_id = u.id
         LEFT JOIN offers o ON o.conversation_id = m.conversation_id
            AND m.content LIKE '%offer%' AND o.amount = CAST(
                REPLACE(REPLACE(SUBSTR(m.content, INSTR(m.content, '$') + 1), ',', ''), ' ', '') AS REAL
            )
         WHERE m.conversation_id = ?1
         ORDER BY m.created_at ASC"
    ).unwrap();
    // Simpler approach — just get messages, we'll handle offers separately
    drop(stmt);

    let mut stmt = conn.prepare(
        "SELECT m.id, m.conversation_id, m.sender_id, u.name, m.content, m.created_at
         FROM messages m JOIN users u ON m.sender_id = u.id
         WHERE m.conversation_id = ?1 ORDER BY m.created_at ASC"
    ).unwrap();
    stmt.query_map(params![conversation_id], |row| {
        Ok(Message {
            id: row.get(0)?, conversation_id: row.get(1)?, sender_id: row.get(2)?,
            sender_name: row.get(3)?, content: row.get(4)?, is_offer: false,
            offer_amount: None, offer_status: None, created_at: row.get(5)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_messages_after(db: &Db, conversation_id: &str, after_id: &str) -> Vec<Message> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT m.id, m.conversation_id, m.sender_id, u.name, m.content, m.created_at
         FROM messages m JOIN users u ON m.sender_id = u.id
         WHERE m.conversation_id = ?1 AND m.created_at > (SELECT created_at FROM messages WHERE id = ?2)
         ORDER BY m.created_at ASC"
    ).unwrap();
    stmt.query_map(params![conversation_id, after_id], |row| {
        Ok(Message {
            id: row.get(0)?, conversation_id: row.get(1)?, sender_id: row.get(2)?,
            sender_name: row.get(3)?, content: row.get(4)?, is_offer: false,
            offer_amount: None, offer_status: None, created_at: row.get(5)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn send_message(db: &Db, conversation_id: &str, sender_id: &str, content: &str) -> String {
    let conn = db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO messages (id, conversation_id, sender_id, content) VALUES (?1, ?2, ?3, ?4)",
        params![id, conversation_id, sender_id, content],
    ).unwrap();
    id
}

pub fn mark_conversation_read(db: &Db, user_id: &str, conversation_id: &str) {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO message_reads (user_id, conversation_id, last_read_at) VALUES (?1, ?2, datetime('now'))",
        params![user_id, conversation_id],
    ).unwrap();
}

pub fn get_unread_count(db: &Db, user_id: &str) -> i64 {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT COALESCE(SUM(cnt), 0) FROM (
            SELECT COUNT(*) as cnt FROM messages m
            JOIN conversations c ON m.conversation_id = c.id
            WHERE (c.buyer_id = ?1 OR c.seller_id = ?1)
            AND m.sender_id != ?1
            AND m.created_at > COALESCE(
                (SELECT last_read_at FROM message_reads WHERE user_id = ?1 AND conversation_id = c.id),
                '1970-01-01'
            )
            GROUP BY c.id
        )",
        params![user_id],
        |row| row.get(0),
    ).unwrap_or(0)
}

// === Offer queries ===

pub fn create_offer(db: &Db, listing_id: &str, conversation_id: &str, buyer_id: &str, amount: f64) -> String {
    let conn = db.lock().unwrap();
    // Cancel any previous pending offers for this listing+buyer
    conn.execute(
        "UPDATE offers SET status = 'cancelled' WHERE listing_id = ?1 AND buyer_id = ?2 AND status = 'pending'",
        params![listing_id, buyer_id],
    ).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO offers (id, listing_id, conversation_id, buyer_id, amount) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, listing_id, conversation_id, buyer_id, amount],
    ).unwrap();
    id
}

pub fn get_pending_offer(db: &Db, conversation_id: &str) -> Option<Offer> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, listing_id, conversation_id, buyer_id, amount, status, created_at FROM offers WHERE conversation_id = ?1 AND status = 'pending' ORDER BY created_at DESC LIMIT 1",
        params![conversation_id],
        |row| Ok(Offer {
            id: row.get(0)?, listing_id: row.get(1)?, conversation_id: row.get(2)?,
            buyer_id: row.get(3)?, amount: row.get(4)?, status: row.get(5)?, created_at: row.get(6)?,
        })
    ).ok()
}

pub fn respond_to_offer(db: &Db, offer_id: &str, seller_id: &str, accept: bool) -> bool {
    let conn = db.lock().unwrap();
    let status = if accept { "accepted" } else { "rejected" };
    // Verify the seller owns the listing
    let rows = conn.execute(
        "UPDATE offers SET status = ?1 WHERE id = ?2 AND listing_id IN (SELECT id FROM listings WHERE seller_id = ?3)",
        params![status, offer_id, seller_id],
    ).unwrap_or(0);
    rows > 0
}

pub fn get_offer(db: &Db, id: &str) -> Option<Offer> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, listing_id, conversation_id, buyer_id, amount, status, created_at FROM offers WHERE id = ?1",
        params![id],
        |row| Ok(Offer {
            id: row.get(0)?, listing_id: row.get(1)?, conversation_id: row.get(2)?,
            buyer_id: row.get(3)?, amount: row.get(4)?, status: row.get(5)?, created_at: row.get(6)?,
        })
    ).ok()
}

// === User queries ===

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
        "SELECT id, email, name, password_hash, location, avatar_url, payment_info, bio, created_at FROM users WHERE email = ?1",
        params![email],
        |row| Ok(User {
            id: row.get(0)?, email: row.get(1)?, name: row.get(2)?, password_hash: row.get(3)?,
            location: row.get(4)?, avatar_url: row.get(5)?, payment_info: row.get(6)?,
            bio: row.get(7)?, created_at: row.get(8)?,
        })
    ).ok()
}

pub fn get_user_by_id(db: &Db, id: &str) -> Option<User> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, email, name, password_hash, location, avatar_url, payment_info, bio, created_at FROM users WHERE id = ?1",
        params![id],
        |row| Ok(User {
            id: row.get(0)?, email: row.get(1)?, name: row.get(2)?, password_hash: row.get(3)?,
            location: row.get(4)?, avatar_url: row.get(5)?, payment_info: row.get(6)?,
            bio: row.get(7)?, created_at: row.get(8)?,
        })
    ).ok()
}

pub fn update_user_profile(db: &Db, id: &str, form: &ProfileForm) -> bool {
    let conn = db.lock().unwrap();
    let rows = conn.execute(
        "UPDATE users SET name = ?1, location = ?2, bio = ?3, payment_info = ?4 WHERE id = ?5",
        params![form.name, form.location, form.bio, form.payment_info, id],
    ).unwrap_or(0);
    rows > 0
}

// === Session queries ===

pub fn create_session(db: &Db, user_id: &str) -> String {
    let conn = db.lock().unwrap();
    // We'll reuse the sessions table concept but inline it
    // Actually, we need a sessions table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            expires_at TEXT NOT NULL
        );
    ").unwrap();
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
        "SELECT u.id, u.email, u.name, u.password_hash, u.location, u.avatar_url, u.payment_info, u.bio, u.created_at
         FROM sessions s JOIN users u ON s.user_id = u.id
         WHERE s.id = ?1 AND s.expires_at > datetime('now')",
        params![session_id],
        |row| Ok(User {
            id: row.get(0)?, email: row.get(1)?, name: row.get(2)?, password_hash: row.get(3)?,
            location: row.get(4)?, avatar_url: row.get(5)?, payment_info: row.get(6)?,
            bio: row.get(7)?, created_at: row.get(8)?,
        })
    ).ok()
}

pub fn delete_session(db: &Db, session_id: &str) {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id]).unwrap();
}
