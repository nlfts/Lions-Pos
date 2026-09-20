use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn seed_initial_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users;")
        .fetch_one(pool)
        .await?;

    if user_count > 0 {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();

    // 1. Seed Permissions
    let modules = [
        "user", "produk", "category", "stock-mutation", "partner",
        "location", "voucher", "role", "permission", "module",
        "supplier", "order", "purchase-order", "transfer-request", "stock-opname", "log"
    ];
    let actions = ["index", "show", "store", "update", "delete"];

    let mut perm_ids = Vec::new();
    for module in modules.iter() {
        for action in actions.iter() {
            let perm_id = format!("perm-{}-{}", module, action);
            let name = format!("{} {}", action, module);
            let slug = format!("{}.{}", module, action);

            sqlx::query(
                "INSERT OR IGNORE INTO permissions (id, name, slug, module, action) VALUES (?, ?, ?, ?, ?);"
            )
            .bind(&perm_id)
            .bind(&name)
            .bind(&slug)
            .bind(module)
            .bind(action)
            .execute(pool)
            .await?;

            perm_ids.push(perm_id);
        }
    }

    // 2. Seed Roles
    let admin_role_id = "role-admin".to_string();
    let cashier_role_id = "role-cashier".to_string();

    sqlx::query(
        "INSERT OR IGNORE INTO roles (id, name, slug, description, created_at) VALUES (?, ?, ?, ?, ?);"
    )
    .bind(&admin_role_id)
    .bind("ADMIN")
    .bind("admin")
    .bind("Administrator with full access")
    .bind(&now)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO roles (id, name, slug, description, created_at) VALUES (?, ?, ?, ?, ?);"
    )
    .bind(&cashier_role_id)
    .bind("CASHIER")
    .bind("cashier")
    .bind("Cashier with POS checkout access")
    .bind(&now)
    .execute(pool)
    .await?;

    // Link all permissions to admin role
    for pid in &perm_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO role_permissions (role_id, permission_id) VALUES (?, ?);"
        )
        .bind(&admin_role_id)
        .bind(pid)
        .execute(pool)
        .await?;
    }

    // 3. Seed Default Branch & Warehouse
    let branch_id = "branch-main".to_string();
    sqlx::query(
        "INSERT OR IGNORE INTO branches (id, name, code, address, phone, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, ?, ?);"
    )
    .bind(&branch_id)
    .bind("Cabang Utama POS")
    .bind("BR-001")
    .bind("Jl. Sudirman No. 1, Jakarta Pusat")
    .bind("081234567890")
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    let warehouse_id = "wh-main".to_string();
    sqlx::query(
        "INSERT OR IGNORE INTO warehouses (id, branch_id, name, code, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, 1, ?, ?);"
    )
    .bind(&warehouse_id)
    .bind(&branch_id)
    .bind("Gudang Pusat")
    .bind("WH-001")
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    // 4. Seed Admin User
    // Default password: password123
    let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST)
        .map_err(|e| sqlx::Error::Protocol(format!("Bcrypt hash error: {}", e)))?;

    let admin_user_id = "user-admin".to_string();
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO users (id, username, password_hash, fullname, phone, avatar, branch_id, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?, ?);
        "#
    )
    .bind(&admin_user_id)
    .bind("admin")
    .bind(&password_hash)
    .bind("Super Admin Lion POS")
    .bind("081234567890")
    .bind("https://api.dicebear.com/7.x/avataaars/svg?seed=admin")
    .bind(&branch_id)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?, ?);"
    )
    .bind(&admin_user_id)
    .bind(&admin_role_id)
    .execute(pool)
    .await?;

    // 5. Seed Sample Categories
    let categories = [
        ("cat-food", "Makanan", "makanan", "Utensils"),
        ("cat-drink", "Minuman", "minuman", "Coffee"),
        ("cat-snack", "Snack & Cemilan", "snack", "Cookie"),
        ("cat-dessert", "Dessert", "dessert", "Cake"),
    ];

    for (cid, name, slug, icon) in categories.iter() {
        sqlx::query(
            "INSERT OR IGNORE INTO categories (id, name, slug, icon, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, 1, ?, ?);"
        )
        .bind(cid)
        .bind(name)
        .bind(slug)
        .bind(icon)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    // 6. Seed Sample Products & Stock
    let sample_products = [
        ("prod-1", "Kopi Susu Gula Aren", "KOP-001", 18000.0, 8000.0, "cat-drink", "https://images.unsplash.com/photo-1541167760496-1628856ab772?w=400&q=80", 50),
        ("prod-2", "Americano Double Shot", "KOP-002", 15000.0, 5000.0, "cat-drink", "https://images.unsplash.com/photo-1514432324607-a09d9b4aefdd?w=400&q=80", 40),
        ("prod-3", "Nasi Goreng Spesial Lion", "NAS-001", 28000.0, 14000.0, "cat-food", "https://images.unsplash.com/photo-1603133872878-684f208fb84b?w=400&q=80", 30),
        ("prod-4", "Mie Ayam Bakso Jamur", "MIE-001", 24000.0, 11000.0, "cat-food", "https://images.unsplash.com/photo-1569718212165-3a8278d5f624?w=400&q=80", 25),
        ("prod-5", "Kentang Goreng Crispy", "SNK-001", 16000.0, 7000.0, "cat-snack", "https://images.unsplash.com/photo-1573080496219-bb080dd4f877?w=400&q=80", 60),
        ("prod-6", "Croissant Butter French", "DST-001", 22000.0, 10000.0, "cat-dessert", "https://images.unsplash.com/photo-1555507036-ab1f4038808a?w=400&q=80", 20),
    ];

    for (pid, name, sku, price, cost, cat_id, img_url, stock_qty) in sample_products.iter() {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO products (id, name, sku, barcode, base_price, cost_price, category_id, track_stock, is_active, image_url, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, 1, 1, ?, ?, ?);
            "#
        )
        .bind(pid)
        .bind(name)
        .bind(sku)
        .bind(sku)
        .bind(price)
        .bind(cost)
        .bind(cat_id)
        .bind(img_url)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        // Primary Photo
        sqlx::query(
            "INSERT OR IGNORE INTO product_photos (id, product_id, url, is_primary, sort_order, created_at) VALUES (?, ?, ?, 1, 0, ?);"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(pid)
        .bind(img_url)
        .bind(&now)
        .execute(pool)
        .await?;

        // Stock in main branch
        sqlx::query(
            "INSERT OR IGNORE INTO stock_balances (id, branch_id, product_id, quantity, min_stock, updated_at) VALUES (?, ?, ?, ?, 5, ?);"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&branch_id)
        .bind(pid)
        .bind(stock_qty)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    // 7. Seed Sample Voucher
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO vouchers (id, code, description, discount_type, discount_value, min_spend, max_discount, quota, used_count, is_active, start_date, end_date, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, 1, NULL, NULL, ?);
        "#
    )
    .bind("vouc-1")
    .bind("DISCOUNT10")
    .bind("Diskon 10% Spesial Pembukaan")
    .bind("PERCENTAGE")
    .bind(10.0)
    .bind(30000.0)
    .bind(15000.0)
    .bind(100)
    .bind(&now)
    .execute(pool)
    .await?;

    tracing::info!("Database initial seeding completed successfully!");
    Ok(())
}
