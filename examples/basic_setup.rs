//! Basic setup example showing how to initialize Shopster.
//!
//! This example demonstrates:
//! - Creating a tenant management system
//! - Initializing a database selector
//! - Setting up the Shopster instance

use shopster::{Shopster, DatabaseSelector};
use stec_tenet::Tenet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    simple_logger::init_by_env().ok();

    // Connection strings (in production, load from environment)
    let tenet_db = "postgres://postgres:postgres@localhost/tenet_db";
    let shopster_db = "postgres://postgres:postgres@localhost/shopster_db";

    // Create tenant management service
    let tenet = Tenet::new(tenet_db.to_string());

    // Create database selector
    let mut db_selector = DatabaseSelector::new(tenet);

    // Add default database
    let default_tenant_id = db_selector.add_default(shopster_db.to_string())?;
    println!("✓ Default tenant created with ID: {}", default_tenant_id);

    // Initialize Shopster
    let shopster = Shopster::new(db_selector);
    println!("✓ Shopster initialized");

    // Now you can use shopster to access domain handlers
    let customers = shopster.customers(default_tenant_id)?;
    let count = customers.count_customers()?;
    println!("✓ Total customers: {}", count);

    let products = shopster.products(default_tenant_id)?;
    let products_list = products.get_all()?;
    println!("✓ Total products: {}", products_list.len());

    Ok(())
}

