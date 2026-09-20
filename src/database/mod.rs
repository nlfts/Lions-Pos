pub mod seeder;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::time::Duration;

pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    // Enable WAL mode for high performance SQLite concurrency
    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    run_migrations(&pool).await?;
    seeder::seed_initial_data(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 1. Users & RBAC
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            fullname TEXT NOT NULL,
            phone TEXT,
            avatar TEXT,
            branch_id TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS roles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            slug TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS permissions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            module TEXT NOT NULL,
            action TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS role_permissions (
            role_id TEXT NOT NULL,
            permission_id TEXT NOT NULL,
            PRIMARY KEY (role_id, permission_id),
            FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
            FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS user_roles (
            user_id TEXT NOT NULL,
            role_id TEXT NOT NULL,
            PRIMARY KEY (user_id, role_id),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS tokens (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            token_type TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // 2. Catalog (Categories, Products, Photos)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            icon TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS products (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            sku TEXT UNIQUE,
            barcode TEXT,
            base_price REAL NOT NULL DEFAULT 0.0,
            cost_price REAL,
            category_id TEXT,
            track_stock INTEGER NOT NULL DEFAULT 1,
            is_active INTEGER NOT NULL DEFAULT 1,
            image_url TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS product_photos (
            id TEXT PRIMARY KEY,
            product_id TEXT NOT NULL,
            url TEXT NOT NULL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // 3. Inventory & Branches
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS branches (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            code TEXT NOT NULL UNIQUE,
            address TEXT,
            phone TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS warehouses (
            id TEXT PRIMARY KEY,
            branch_id TEXT,
            name TEXT NOT NULL,
            code TEXT NOT NULL UNIQUE,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS stock_balances (
            id TEXT PRIMARY KEY,
            branch_id TEXT NOT NULL,
            product_id TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 0,
            min_stock INTEGER NOT NULL DEFAULT 5,
            updated_at TEXT NOT NULL,
            UNIQUE (branch_id, product_id),
            FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS stock_mutations (
            id TEXT PRIMARY KEY,
            branch_id TEXT NOT NULL,
            product_id TEXT NOT NULL,
            mutation_type TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            balance_before INTEGER NOT NULL,
            balance_after INTEGER NOT NULL,
            reference_no TEXT,
            notes TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // 4. Vouchers
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vouchers (
            id TEXT PRIMARY KEY,
            code TEXT NOT NULL UNIQUE,
            description TEXT,
            discount_type TEXT NOT NULL, -- PERCENTAGE, FIXED
            discount_value REAL NOT NULL DEFAULT 0.0,
            min_spend REAL,
            max_discount REAL,
            quota INTEGER,
            used_count INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            start_date TEXT,
            end_date TEXT,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    // 5. Cashier Shifts, Orders, Order Items, Payments
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS cashier_shifts (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            branch_id TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT,
            starting_cash REAL NOT NULL DEFAULT 0.0,
            actual_cash REAL,
            expected_cash REAL,
            difference REAL,
            notes TEXT,
            status TEXT NOT NULL DEFAULT 'OPEN', -- OPEN, CLOSED
            created_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (branch_id) REFERENCES branches(id)
        );

        CREATE TABLE IF NOT EXISTS orders (
            id TEXT PRIMARY KEY,
            order_number TEXT NOT NULL UNIQUE,
            branch_id TEXT NOT NULL,
            user_id TEXT,
            shift_id TEXT,
            voucher_id TEXT,
            buyer_name TEXT,
            subtotal REAL NOT NULL DEFAULT 0.0,
            discount_amount REAL NOT NULL DEFAULT 0.0,
            tax_amount REAL NOT NULL DEFAULT 0.0,
            total REAL NOT NULL DEFAULT 0.0,
            status TEXT NOT NULL DEFAULT 'PAID', -- PAID, PENDING, CANCELLED
            notes TEXT,
            manual_discount_type TEXT,
            manual_discount_value REAL,
            manual_discount_note TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (branch_id) REFERENCES branches(id),
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (shift_id) REFERENCES cashier_shifts(id),
            FOREIGN KEY (voucher_id) REFERENCES vouchers(id)
        );

        CREATE TABLE IF NOT EXISTS order_items (
            id TEXT PRIMARY KEY,
            order_id TEXT NOT NULL,
            product_id TEXT NOT NULL,
            product_name TEXT NOT NULL,
            product_sku TEXT,
            unit_price REAL NOT NULL DEFAULT 0.0,
            quantity INTEGER NOT NULL DEFAULT 1,
            discount_type TEXT,
            discount_value REAL,
            discount_amount REAL NOT NULL DEFAULT 0.0,
            subtotal REAL NOT NULL DEFAULT 0.0,
            note TEXT,
            FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(id)
        );

        CREATE TABLE IF NOT EXISTS payments (
            id TEXT PRIMARY KEY,
            order_id TEXT NOT NULL,
            method TEXT NOT NULL, -- CASH, TRANSFER, QRIS, CARD
            amount REAL NOT NULL,
            cash_tendered REAL,
            change_due REAL,
            bank_name TEXT,
            reference_no TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
