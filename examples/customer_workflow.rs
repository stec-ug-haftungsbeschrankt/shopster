//! Customer management workflow example.
//!
//! This example demonstrates:
//! - Creating customers
//! - Authenticating customers
//! - Updating customer information
//! - Password management
//! - Email verification

use stec_shopster::{Shopster, DatabaseSelector, customers::Customer};
use stec_tenet::{Tenet, encryption_modes::EncryptionModes};
use uuid::Uuid;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_env().ok();

    let tenet_db = "postgres://postgres:postgres@localhost/tenet_db";
    let shopster_db = "postgres://postgres:postgres@localhost/shopster_db";

    let tenet = Tenet::new(tenet_db.to_string());
    let mut db_selector = DatabaseSelector::new(tenet);
    let tenant_id = db_selector.add_default(shopster_db.to_string())?;

    let shopster = Shopster::new(db_selector);
    let customers = shopster.customers(tenant_id)?;

    // Create a new customer
    println!("\n=== Creating Customer ===");
    let new_customer = Customer {
        id: Uuid::new_v4(),
        email: "john.doe@example.com".to_string(),
        email_verified: false,
        encryption_mode: EncryptionModes::from_str("Argon2")?,
        password: "securepassword123".to_string(),
        full_name: "John Doe".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
    };

    let created = customers.insert(&new_customer)?;
    println!("✓ Customer created: {} ({})", created.full_name, created.email);

    // Authenticate customer
    println!("\n=== Customer Authentication ===");
    match customers.verify_email_password("john.doe@example.com".to_string(), "securepassword123") {
        Ok(customer) => println!("✓ Authentication successful: {}", customer.email),
        Err(e) => println!("✗ Authentication failed: {}", e),
    }

    // Get customer count
    println!("\n=== Customer Statistics ===");
    let count = customers.count_customers()?;
    println!("✓ Total customers: {}", count);

    // Search customers
    println!("\n=== Search Customers ===");
    let search_results = customers.search_customers("John")?;
    println!("✓ Found {} customers matching 'John'", search_results.len());

    // Verify email
    println!("\n=== Email Verification ===");
    let verified = customers.verify_email(created.id)?;
    println!("✓ Email verified: {}", verified.email_verified);

    // Get all customers
    println!("\n=== List All Customers ===");
    let all = customers.get_all()?;
    for customer in all {
        println!("  - {}: {} (verified: {})",
            customer.full_name,
            customer.email,
            customer.email_verified
        );
    }

    Ok(())
}

