//! Product management and catalog operations example.
//!
//! This example demonstrates:
//! - Creating products
//! - Managing inventory
//! - Product catalogs
//! - Stock management

use stec_shopster::{Shopster, DatabaseSelector, products::Product, products::Price};
use stec_tenet::Tenet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_env().ok();

    let tenet_db = "postgres://postgres:postgres@localhost/tenet_db";
    let shopster_db = "postgres://postgres:postgres@localhost/shopster_db";

    let tenet = Tenet::new(tenet_db.to_string());
    let mut db_selector = DatabaseSelector::new(tenet);
    let tenant_id = db_selector.add_default(shopster_db.to_string())?;

    let shopster = Shopster::new(db_selector);
    let products = shopster.products(tenant_id)?;
    let warehouse = shopster.warehouse(tenant_id)?;

    // Create products
    println!("\n=== Creating Products ===");
    let laptop = Product {
        id: 0,
        article_number: "LAPTOP-001".to_string(),
        gtin: "1234567890123".to_string(),
        title: "Professional Laptop".to_string(),
        short_description: "High-performance laptop".to_string(),
        description: "Latest generation laptop with 16GB RAM and 512GB SSD".to_string(),
        tags: vec!["electronics".to_string(), "computers".to_string(), "portable".to_string()],
        image_url: "https://example.com/laptop.jpg".to_string(),
        additional_images: vec!["https://example.com/laptop-side.jpg".to_string()],
        price: Some(Price {
            amount: 129900, // $1299.00
            currency: "USD".to_string(),
        }),
        weight: 1800,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
    };

    let created_laptop = products.insert(&laptop)?;
    println!("✓ Created product: {} (ID: {})", created_laptop.title, created_laptop.id);

    // Create wireless mouse
    let mouse = Product {
        id: 0,
        article_number: "MOUSE-001".to_string(),
        gtin: "9876543210987".to_string(),
        title: "Wireless Mouse".to_string(),
        short_description: "Ergonomic wireless mouse".to_string(),
        description: "Comfortable wireless mouse with long battery life".to_string(),
        tags: vec!["electronics".to_string(), "accessories".to_string()],
        image_url: "https://example.com/mouse.jpg".to_string(),
        additional_images: vec![],
        price: Some(Price {
            amount: 2999, // $29.99
            currency: "USD".to_string(),
        }),
        weight: 100,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
    };

    let created_mouse = products.insert(&mouse)?;
    println!("✓ Created product: {} (ID: {})", created_mouse.title, created_mouse.id);

    // List all products
    println!("\n=== Catalog ===");
    let all_products = products.get_all()?;
    println!("✓ Total products: {}", all_products.len());
    for product in &all_products {
        if let Some(price) = &product.price {
            println!("  - {} ({}): ${}",
                product.title, product.article_number,
                price.amount as f64 / 100.0
            );
        }
    }

    // Get specific product
    println!("\n=== Product Details ===");
    let product_detail = products.get(created_laptop.id)?;
    println!("Title: {}", product_detail.title);
    println!("Description: {}", product_detail.description);
    println!("Tags: {}", product_detail.tags.join(", "));
    if let Some(price) = product_detail.price {
        println!("Price: {} {}", price.amount as f64 / 100.0, price.currency);
    }

    // Update product price
    println!("\n=== Update Product ===");
    let mut updated_laptop = created_laptop.clone();
    updated_laptop.price = Some(Price {
        amount: 119900, // Sale price: $1199.00
        currency: "USD".to_string(),
    });
    updated_laptop.short_description = "Limited time sale - high-performance laptop".to_string();

    let price_updated = products.update(&updated_laptop)?;
    println!("✓ Product updated:");
    if let Some(price) = price_updated.price {
        println!("  New price: ${}", price.amount as f64 / 100.0);
    }

    // Warehouse management
    println!("\n=== Warehouse Management ===");
    use stec_shopster::warehouse::WarehouseItem;

    let laptop_stock = WarehouseItem {
        id: 0,
        product_id: created_laptop.id,
        in_stock: 50,
        reserved: 5,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
    };

    let warehouse_item = warehouse.insert(&laptop_stock)?;
    println!("✓ Added to warehouse: {} units (ID: {})", warehouse_item.in_stock, warehouse_item.id);
    println!("  In stock: {}", warehouse_item.in_stock);
    println!("  Reserved: {}", warehouse_item.reserved);
    println!("  Available: {}", warehouse_item.available());

    // Display all inventory
    println!("\n=== Full Inventory ===");
    let inventory = warehouse.get_all()?;
    for item in inventory {
        if let Ok(product) = products.get(item.product_id) {
            println!("  - {}: {} available (in stock: {}, reserved: {})",
                product.title,
                item.available(),
                item.in_stock,
                item.reserved
            );
        }
    }

    Ok(())
}

