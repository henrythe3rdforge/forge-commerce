use std::sync::Arc;
use tera::Tera;

// Helper to build the app for testing
fn setup() -> (forge_commerce::db::Db, Arc<Tera>) {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);
    let tera = Arc::new(Tera::new("templates/**/*.html").expect("Failed to load templates"));
    (db, tera)
}

fn seed_test_db(db: &forge_commerce::db::Db) {
    let conn = db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO products (id, name, description, price, category, image_url, featured) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, "Test Product", "A test product", 29.99, "TestCat", "/static/images/placeholder.svg", 1],
    ).unwrap();

    // Create test user
    let user_id = uuid::Uuid::new_v4().to_string();
    let hash = forge_commerce::auth::hash_password("testpass123");
    conn.execute(
        "INSERT INTO users (id, email, name, password_hash, is_admin) VALUES (?1, ?2, ?3, ?4, 0)",
        rusqlite::params![user_id, "test@test.com", "Test User", hash],
    ).unwrap();

    // Create admin user
    let admin_id = uuid::Uuid::new_v4().to_string();
    let admin_hash = forge_commerce::auth::hash_password("admin123");
    conn.execute(
        "INSERT INTO users (id, email, name, password_hash, is_admin) VALUES (?1, ?2, ?3, ?4, 1)",
        rusqlite::params![admin_id, "admin@forge.com", "Admin", admin_hash],
    ).unwrap();
}

// === Database Tests ===

#[test]
fn test_migrations_run_without_error() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);
    // Run again to test idempotency
    forge_commerce::db::run_migrations(&db);
}

#[test]
fn test_create_and_get_product() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "Test Widget".to_string(),
        description: "A test widget".to_string(),
        price: "19.99".to_string(),
        category: "Widgets".to_string(),
        featured: None,
    };
    let id = forge_commerce::db::create_product(&db, &form, "/static/images/placeholder.svg");
    let product = forge_commerce::db::get_product(&db, &id).unwrap();
    assert_eq!(product.name, "Test Widget");
    assert!((product.price - 19.99).abs() < 0.01);
}

#[test]
fn test_update_product() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "Original".to_string(),
        description: "Desc".to_string(),
        price: "10.00".to_string(),
        category: "Cat".to_string(),
        featured: None,
    };
    let id = forge_commerce::db::create_product(&db, &form, "/img.jpg");

    let update = forge_commerce::models::ProductForm {
        name: "Updated".to_string(),
        description: "New desc".to_string(),
        price: "20.00".to_string(),
        category: "Cat".to_string(),
        featured: Some("on".to_string()),
    };
    forge_commerce::db::update_product(&db, &id, &update, None);
    let p = forge_commerce::db::get_product(&db, &id).unwrap();
    assert_eq!(p.name, "Updated");
    assert!(p.featured);
}

#[test]
fn test_delete_product() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "ToDelete".to_string(),
        description: "D".to_string(),
        price: "5.00".to_string(),
        category: "C".to_string(),
        featured: None,
    };
    let id = forge_commerce::db::create_product(&db, &form, "/img.jpg");
    forge_commerce::db::delete_product(&db, &id);
    assert!(forge_commerce::db::get_product(&db, &id).is_none());
}

#[test]
fn test_search_products_by_name() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "UniqueSearchable".to_string(),
        description: "A thing".to_string(),
        price: "15.00".to_string(),
        category: "Cat".to_string(),
        featured: None,
    };
    forge_commerce::db::create_product(&db, &form, "/img.jpg");

    let query = forge_commerce::models::SearchQuery {
        q: Some("UniqueSearchable".to_string()),
        category: None,
        sort: None,
    };
    let results = forge_commerce::db::search_products(&db, &query);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "UniqueSearchable");
}

#[test]
fn test_search_products_by_category() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    for i in 0..3 {
        let form = forge_commerce::models::ProductForm {
            name: format!("Item {}", i),
            description: "D".to_string(),
            price: "10.00".to_string(),
            category: "SpecialCat".to_string(),
            featured: None,
        };
        forge_commerce::db::create_product(&db, &form, "/img.jpg");
    }

    let query = forge_commerce::models::SearchQuery {
        q: None,
        category: Some("SpecialCat".to_string()),
        sort: None,
    };
    let results = forge_commerce::db::search_products(&db, &query);
    assert_eq!(results.len(), 3);
}

#[test]
fn test_search_products_sort_price() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    for price in &["50.00", "10.00", "30.00"] {
        let form = forge_commerce::models::ProductForm {
            name: format!("P{}", price),
            description: "D".to_string(),
            price: price.to_string(),
            category: "SortCat".to_string(),
            featured: None,
        };
        forge_commerce::db::create_product(&db, &form, "/img.jpg");
    }

    let query = forge_commerce::models::SearchQuery {
        q: None,
        category: Some("SortCat".to_string()),
        sort: Some("price_asc".to_string()),
    };
    let results = forge_commerce::db::search_products(&db, &query);
    assert!(results[0].price <= results[1].price);
    assert!(results[1].price <= results[2].price);
}

// === Cart Tests ===

#[test]
fn test_add_to_cart() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "CartItem".to_string(),
        description: "D".to_string(),
        price: "25.00".to_string(),
        category: "C".to_string(),
        featured: None,
    };
    let pid = forge_commerce::db::create_product(&db, &form, "/img.jpg");

    forge_commerce::db::add_to_cart(&db, "session123", &pid, 2);
    let items = forge_commerce::db::get_cart_items(&db, "session123");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].quantity, 2);
}

#[test]
fn test_add_to_cart_merges_quantities() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "MergeItem".to_string(),
        description: "D".to_string(),
        price: "10.00".to_string(),
        category: "C".to_string(),
        featured: None,
    };
    let pid = forge_commerce::db::create_product(&db, &form, "/img.jpg");

    forge_commerce::db::add_to_cart(&db, "s1", &pid, 1);
    forge_commerce::db::add_to_cart(&db, "s1", &pid, 3);
    let items = forge_commerce::db::get_cart_items(&db, "s1");
    assert_eq!(items[0].quantity, 4);
}

#[test]
fn test_update_cart_item() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "UpdateCartItem".to_string(),
        description: "D".to_string(),
        price: "10.00".to_string(),
        category: "C".to_string(),
        featured: None,
    };
    let pid = forge_commerce::db::create_product(&db, &form, "/img.jpg");

    forge_commerce::db::add_to_cart(&db, "s2", &pid, 3);
    forge_commerce::db::update_cart_item(&db, "s2", &pid, 5);
    let items = forge_commerce::db::get_cart_items(&db, "s2");
    assert_eq!(items[0].quantity, 5);
}

#[test]
fn test_update_cart_item_zero_removes() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "RemoveItem".to_string(),
        description: "D".to_string(),
        price: "10.00".to_string(),
        category: "C".to_string(),
        featured: None,
    };
    let pid = forge_commerce::db::create_product(&db, &form, "/img.jpg");

    forge_commerce::db::add_to_cart(&db, "s3", &pid, 2);
    forge_commerce::db::update_cart_item(&db, "s3", &pid, 0);
    let items = forge_commerce::db::get_cart_items(&db, "s3");
    assert_eq!(items.len(), 0);
}

#[test]
fn test_remove_from_cart() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let form = forge_commerce::models::ProductForm {
        name: "RmItem".to_string(),
        description: "D".to_string(),
        price: "10.00".to_string(),
        category: "C".to_string(),
        featured: None,
    };
    let pid = forge_commerce::db::create_product(&db, &form, "/img.jpg");

    forge_commerce::db::add_to_cart(&db, "s4", &pid, 1);
    forge_commerce::db::remove_from_cart(&db, "s4", &pid);
    assert_eq!(forge_commerce::db::get_cart_count(&db, "s4"), 0);
}

#[test]
fn test_clear_cart() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    for i in 0..3 {
        let form = forge_commerce::models::ProductForm {
            name: format!("ClearItem{}", i),
            description: "D".to_string(),
            price: "10.00".to_string(),
            category: "C".to_string(),
            featured: None,
        };
        let pid = forge_commerce::db::create_product(&db, &form, "/img.jpg");
        forge_commerce::db::add_to_cart(&db, "s5", &pid, 1);
    }

    forge_commerce::db::clear_cart(&db, "s5");
    assert_eq!(forge_commerce::db::get_cart_count(&db, "s5"), 0);
}

// === Price Calculation Tests ===

#[test]
fn test_cart_total() {
    let items = vec![
        forge_commerce::models::CartItem { product_id: "1".into(), product_name: "A".into(), price: 10.0, quantity: 2, image_url: "".into() },
        forge_commerce::models::CartItem { product_id: "2".into(), product_name: "B".into(), price: 25.50, quantity: 1, image_url: "".into() },
    ];
    let total = forge_commerce::models::cart_total(&items);
    assert!((total - 45.50).abs() < 0.01);
}

#[test]
fn test_cart_count() {
    let items = vec![
        forge_commerce::models::CartItem { product_id: "1".into(), product_name: "A".into(), price: 10.0, quantity: 2, image_url: "".into() },
        forge_commerce::models::CartItem { product_id: "2".into(), product_name: "B".into(), price: 25.50, quantity: 3, image_url: "".into() },
    ];
    assert_eq!(forge_commerce::models::cart_count(&items), 5);
}

#[test]
fn test_format_price() {
    assert_eq!(forge_commerce::models::format_price(29.9), "$29.90");
    assert_eq!(forge_commerce::models::format_price(100.0), "$100.00");
    assert_eq!(forge_commerce::models::format_price(0.5), "$0.50");
}

// === Auth Tests ===

#[test]
fn test_password_hash_and_verify() {
    let hash = forge_commerce::auth::hash_password("mysecretpass");
    assert!(forge_commerce::auth::verify_password("mysecretpass", &hash));
    assert!(!forge_commerce::auth::verify_password("wrongpass", &hash));
}

#[test]
fn test_password_hash_different_salts() {
    let h1 = forge_commerce::auth::hash_password("same_password");
    let h2 = forge_commerce::auth::hash_password("same_password");
    assert_ne!(h1, h2); // Different salts
    assert!(forge_commerce::auth::verify_password("same_password", &h1));
    assert!(forge_commerce::auth::verify_password("same_password", &h2));
}

// === User & Session Tests ===

#[test]
fn test_create_user_and_login() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let hash = forge_commerce::auth::hash_password("password123");
    let user_id = forge_commerce::db::create_user(&db, "Alice", "alice@test.com", &hash).unwrap();

    let user = forge_commerce::db::get_user_by_email(&db, "alice@test.com").unwrap();
    assert_eq!(user.name, "Alice");
    assert_eq!(user.id, user_id);
}

#[test]
fn test_duplicate_email_rejected() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let hash = forge_commerce::auth::hash_password("pass");
    forge_commerce::db::create_user(&db, "A", "dup@test.com", &hash).unwrap();
    let result = forge_commerce::db::create_user(&db, "B", "dup@test.com", &hash);
    assert!(result.is_err());
}

#[test]
fn test_session_creation_and_lookup() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let hash = forge_commerce::auth::hash_password("pass");
    let user_id = forge_commerce::db::create_user(&db, "Bob", "bob@test.com", &hash).unwrap();
    let session_id = forge_commerce::db::create_session(&db, &user_id);

    let user = forge_commerce::db::get_session_user(&db, &session_id).unwrap();
    assert_eq!(user.name, "Bob");
}

#[test]
fn test_session_delete() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let hash = forge_commerce::auth::hash_password("pass");
    let user_id = forge_commerce::db::create_user(&db, "Carol", "carol@test.com", &hash).unwrap();
    let session_id = forge_commerce::db::create_session(&db, &user_id);
    forge_commerce::db::delete_session(&db, &session_id);

    assert!(forge_commerce::db::get_session_user(&db, &session_id).is_none());
}

// === Order Tests ===

#[test]
fn test_create_order() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let hash = forge_commerce::auth::hash_password("pass");
    let user_id = forge_commerce::db::create_user(&db, "Dave", "dave@test.com", &hash).unwrap();

    let items = vec![
        forge_commerce::models::CartItem { product_id: "p1".into(), product_name: "Widget".into(), price: 15.0, quantity: 2, image_url: "".into() },
    ];
    let shipping = forge_commerce::models::ShippingForm {
        name: "Dave".into(), address: "123 Main".into(), city: "Anytown".into(), zip: "12345".into(),
    };

    let order_id = forge_commerce::db::create_order(&db, &user_id, &items, &shipping);
    let order = forge_commerce::db::get_order(&db, &order_id).unwrap();
    assert!((order.total - 30.0).abs() < 0.01);
    assert_eq!(order.status, "pending");

    let order_items = forge_commerce::db::get_order_items(&db, &order_id);
    assert_eq!(order_items.len(), 1);
    assert_eq!(order_items[0].quantity, 2);
}

#[test]
fn test_update_order_status() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    let hash = forge_commerce::auth::hash_password("pass");
    let uid = forge_commerce::db::create_user(&db, "Eve", "eve@test.com", &hash).unwrap();
    let items = vec![
        forge_commerce::models::CartItem { product_id: "p1".into(), product_name: "W".into(), price: 10.0, quantity: 1, image_url: "".into() },
    ];
    let shipping = forge_commerce::models::ShippingForm {
        name: "Eve".into(), address: "456 Oak".into(), city: "Town".into(), zip: "54321".into(),
    };
    let oid = forge_commerce::db::create_order(&db, &uid, &items, &shipping);
    forge_commerce::db::update_order_status(&db, &oid, "shipped");
    let order = forge_commerce::db::get_order(&db, &oid).unwrap();
    assert_eq!(order.status, "shipped");
}

// === Categories Test ===

#[test]
fn test_get_categories() {
    let db = forge_commerce::db::init_db_with_path(":memory:");
    forge_commerce::db::run_migrations(&db);

    for (name, cat) in &[("A", "Alpha"), ("B", "Alpha"), ("C", "Beta")] {
        let form = forge_commerce::models::ProductForm {
            name: name.to_string(), description: "D".to_string(), price: "10.00".to_string(),
            category: cat.to_string(), featured: None,
        };
        forge_commerce::db::create_product(&db, &form, "/img.jpg");
    }

    let cats = forge_commerce::db::get_categories(&db);
    assert_eq!(cats.len(), 2);
    let alpha = cats.iter().find(|c| c.name == "Alpha").unwrap();
    assert_eq!(alpha.count, 2);
}

// === Integration: HTTP Tests ===

#[tokio::test]
async fn test_health_endpoint() {
    let (db, tera) = setup();
    seed_test_db(&db);
    let app = forge_commerce::build_router((db, tera));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });

    let resp = reqwest::get(format!("http://{}/health", addr)).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "OK");
}

#[tokio::test]
async fn test_home_page() {
    let (db, tera) = setup();
    seed_test_db(&db);
    let app = forge_commerce::build_router((db, tera));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });

    let resp = reqwest::get(format!("http://{}/", addr)).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("Forge"));
}

#[tokio::test]
async fn test_products_page() {
    let (db, tera) = setup();
    seed_test_db(&db);
    let app = forge_commerce::build_router((db, tera));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });

    let resp = reqwest::get(format!("http://{}/products", addr)).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("Products"));
}

#[tokio::test]
async fn test_login_page() {
    let (db, tera) = setup();
    seed_test_db(&db);
    let app = forge_commerce::build_router((db, tera));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });

    let resp = reqwest::get(format!("http://{}/login", addr)).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("Login"));
}

#[tokio::test]
async fn test_cart_page() {
    let (db, tera) = setup();
    seed_test_db(&db);
    let app = forge_commerce::build_router((db, tera));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });

    let resp = reqwest::get(format!("http://{}/cart", addr)).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("Cart"));
}
