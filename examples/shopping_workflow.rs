//! Shopping cart and basket workflow example.
//!
//! This example demonstrates:
//! - Creating baskets
//! - Adding products to baskets
//! - Calculating totals
//! - Merging baskets
//! - Checkout operations

use stec_shopster::{Shopster, DatabaseSelector, products::Product, products::Price};
use stec_tenet::Tenet;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_env().ok();

    let tenet_db = "postgres://postgres:postgres@localhost/tenet_db";
    let shopster_db = "postgres://postgres:postgres@localhost/shopster_db";

    let tenet = Tenet::new(tenet_db.to_string());
    let mut db_selector = DatabaseSelector::new(tenet);
    let tenant_id = db_selector.add_default(shopster_db.to_string())?;

    let shopster = Shopster::new(db_selector);
    let baskets = shopster.baskets(tenant_id)?;
    let products = shopster.products(tenant_id)?;

    // Create sample products if needed
    println!("\n=== Creating Sample Products ===");
    let product = Product {
        id: 0,
        article_number: "SKU-001".to_string(),
        gtin: "1234567890123".to_string(),
        title: "Laptop".to_string(),
        short_description: "High-performance laptop".to_string(),
        description: "A powerful laptop for professionals".to_string(),
        tags: vec!["electronics".to_string(), "computers".to_string()],
        image_url: "https://example.com/laptop.jpg".to_string(),
        additional_images: vec![],
        price: Some(Price {
            amount: 99900, // $999.00
            currency: "USD".to_string(),
        }),
        weight: 2000, // 2kg
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
    };

    let created_product = products.insert(&product)?;
    println!("✓ Product created: {} (ID: {})", created_product.title, created_product.id);

    // Create a basket (anonymous shopping)
    println!("\n=== Creating Shopping Basket ===");
    let basket_id = baskets.add_basket()?;
    println!("✓ Basket created: {}", basket_id);

    // Add product to basket
    println!("\n=== Adding Products to Basket ===");
    let item_id = baskets.add_product_to_basket(basket_id, created_product.id, 1)?;
    println!("✓ Added 1x {} to basket", created_product.title);

    // Add another quantity
    let _ = baskets.add_product_to_basket(basket_id, created_product.id, 2)?;
    println!("✓ Updated quantity to 2");

    // Get basket details
    println!("\n=== Basket Contents ===");
    let basket = baskets.get_basket(basket_id)?;
    println!("Basket ID: {}", basket.id);
    println!("Items in basket: {}", basket.products.len());

    // Get full product details
    println!("\n=== Products with Details ===");
    let basket_with_details = baskets.get_products_with_details(basket_id)?;
    for item in &basket_with_details {
        println!("  - {} x{}: ${}",
            item.product.title,
            item.quantity,
            item.product.price.as_ref().map(|p| p.amount as f64 / 100.0).unwrap_or(0.0)
        );
    }

    // Calculate total
    println!("\n=== Calculate Total ===");
    let (total_cents, currency) = baskets.calculate_basket_total(basket_id)?;
    let total_dollars = total_cents as f64 / 100.0;
    println!("✓ Total: {}{}", currency, total_dollars);

    // Create second basket for merge demo
    println!("\n=== Merging Baskets ===");
    let basket2_id = baskets.add_basket()?;
    let _ = baskets.add_product_to_basket(basket2_id, created_product.id, 1)?;
    println!("✓ Created second basket with 1 item");

    // Merge baskets
    baskets.merge_baskets(basket2_id, basket_id)?;
    println!("✓ Merged basket {} into {}", basket2_id, basket_id);

    // Verify merge
    let merged_basket = baskets.get_basket(basket_id)?;
    let (total_merged, _) = baskets.calculate_basket_total(basket_id)?;
    println!("✓ Merged basket total items: {}", merged_basket.products.len());
    println!("✓ Merged basket total: ${}", total_merged as f64 / 100.0);

    Ok(())
}

