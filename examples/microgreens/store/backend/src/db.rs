use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

pub async fn init_pool(database_url: &str) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("failed to connect to sqlite database");

    sqlx::query(SCHEMA)
        .execute(&pool)
        .await
        .expect("failed to apply schema");

    seed_discounts(&pool).await;

    pool
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'customer',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS batches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    variety TEXT NOT NULL,
    sow_date TEXT NOT NULL,
    quantity_trays INTEGER NOT NULL,
    price_per_tray REAL NOT NULL,
    height_cm REAL,
    mold INTEGER,
    color_uniform INTEGER,
    stem_ok INTEGER,
    root_mat_ok INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS cart_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    batch_id INTEGER NOT NULL REFERENCES batches(id),
    quantity INTEGER NOT NULL,
    UNIQUE(user_id, batch_id)
);

CREATE TABLE IF NOT EXISTS discounts (
    code TEXT PRIMARY KEY,
    percent_off REAL NOT NULL,
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    discount_code TEXT,
    subtotal REAL NOT NULL,
    total REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL REFERENCES orders(id),
    batch_id INTEGER NOT NULL REFERENCES batches(id),
    variety TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price REAL NOT NULL
);
"#;

async fn seed_discounts(pool: &SqlitePool) {
    sqlx::query(
        "INSERT OR IGNORE INTO discounts (code, percent_off, active) VALUES ('FRESH10', 10.0, 1)",
    )
    .execute(pool)
    .await
    .ok();
}
