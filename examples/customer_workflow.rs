//! Customer management workflow example.

use stec_shopster::{Shopster, DatabaseSelector, customers::Customer};
use stec_tenet::{Tenet, encryption_modes::EncryptionModes};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_env().ok();

    let tenet_db = "postgres://postgres:postgres@localhost/tenet_db";
    let shopster_db = "postgres://postgres:postgres@localhost/shopster_db";

    let tenet = Tenet::new(tenet_db.to_string());
    let mut db_selector = DatabaseSelector::new(tenet);
    let tenant_id = db_selector.add_default(shopster_db.to_string()).await?;

    let shopster = Shopster::new(db_selector);
    let customers = shopster.customers(tenant_id)?;

    println!("\n=== Creating Customer ===");
    let new_customer = Customer {
        id: Uuid::new_v4(),
        email: "john.doe@example.com".to_string(),
        email_verified: false,
        encryption_mode: EncryptionModes::Argon2,
        password: "securepassword123".to_string(),
        full_name: "John Doe".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
    };

    let created = customers.insert(&new_customer).await?;
    println!("Customer created: {} ({})", created.full_name, created.email);

    println!("\n=== Customer Authentication ===");
    match customers.verify_email_password("john.doe@example.com".to_string(), "securepassword123").await {
        Ok(customer) => println!("Authentication successful: {}", customer.email),
        Err(e) => println!("Authentication failed: {}", e),
    }

    println!("\n=== Customer Statistics ===");
    let count = customers.count_customers().await?;
    println!("Total customers: {}", count);

    println!("\n=== Search Customers ===");
    let search_results = customers.search_customers("John").await?;
    println!("Found {} customers matching 'John'", search_results.len());

    println!("\n=== Email Verification ===");
    let verified = customers.verify_email(created.id).await?;
    println!("Email verified: {}", verified.email_verified);

    println!("\n=== List All Customers ===");
    let all = customers.get_all().await?;
    for customer in all {
        println!("  - {}: {} (verified: {})", customer.full_name, customer.email, customer.email_verified);
    }

    Ok(())
}
