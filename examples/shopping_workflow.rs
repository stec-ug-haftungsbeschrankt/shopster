//! Shopping cart and basket workflow example.

use stec_shopster::{Shopster, DatabaseSelector, products::Product, products::Price};
use stec_tenet::Tenet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_env().ok();

    let tenet_db = "postgres://postgres:postgres@localhost/tenet_db";
    let shopster_db = "postgres://postgres:postgres@localhost/shopster_db";

    let tenet = Tenet::new(tenet_db.to_string());
    let mut db_selector = DatabaseSelector::new(tenet);
    let tenant_id = db_selector.add_default(shopster_db.to_string()).await?;

    let shopster = Shopster::new(db_selector);
    let baskets = shopster.baskets(tenant_id)?;
    let products = shopster.products(tenant_id)?;

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
            amount: 99900,
            currency: "USD".to_string(),
        }),
        weight: 2000,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
    };

    let created_product = products.insert(&product).await?;
    println!("Product created: {} (ID: {})", created_product.title, created_product.id);

    println!("\n=== Creating Shopping Basket ===");
    let basket_id = baskets.add_basket().await?;
    println!("Basket created: {}", basket_id);

    println!("\n=== Adding Products to Basket ===");
    let _item_id = baskets.add_product_to_basket(basket_id, created_product.id, 1).await?;
    println!("Added 1x {} to basket", created_product.title);

    let _ = baskets.add_product_to_basket(basket_id, created_product.id, 2).await?;
    println!("Updated quantity to 2");

    println!("\n=== Basket Contents ===");
    let basket = baskets.get_basket(basket_id).await?;
    println!("Basket ID: {}", basket.id);
    println!("Items in basket: {}", basket.products.len());

    println!("\n=== Products with Details ===");
    let basket_with_details = baskets.get_products_with_details(basket_id).await?;
    for item in &basket_with_details {
        println!("  - {} x{}: ${}",
            item.product.title,
            item.quantity,
            item.product.price.as_ref().map(|p| p.amount as f64 / 100.0).unwrap_or(0.0)
        );
    }

    println!("\n=== Calculate Total ===");
    let (total_cents, currency) = baskets.calculate_basket_total(basket_id).await?;
    let total_dollars = total_cents as f64 / 100.0;
    println!("Total: {}{}", currency, total_dollars);

    println!("\n=== Merging Baskets ===");
    let basket2_id = baskets.add_basket().await?;
    let _ = baskets.add_product_to_basket(basket2_id, created_product.id, 1).await?;
    println!("Created second basket with 1 item");

    baskets.merge_baskets(basket2_id, basket_id).await?;
    println!("Merged basket {} into {}", basket2_id, basket_id);

    let merged_basket = baskets.get_basket(basket_id).await?;
    let (total_merged, _) = baskets.calculate_basket_total(basket_id).await?;
    println!("Merged basket total items: {}", merged_basket.products.len());
    println!("Merged basket total: ${}", total_merged as f64 / 100.0);

    Ok(())
}
