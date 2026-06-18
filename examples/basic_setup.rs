//! Basic setup example showing how to initialize Shopster.

use stec_shopster::{Shopster, DatabaseSelector};
use stec_tenet::Tenet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_env().ok();

    let tenet_db = "postgres://postgres:postgres@localhost/tenet_db";
    let shopster_db = "postgres://postgres:postgres@localhost/shopster_db";

    let tenet = Tenet::new(tenet_db.to_string());
    let mut db_selector = DatabaseSelector::new(tenet);

    let default_tenant_id = db_selector.add_default(shopster_db.to_string()).await?;
    println!("Default tenant created with ID: {}", default_tenant_id);

    let shopster = Shopster::new(db_selector);
    println!("Shopster initialized");

    let customers = shopster.customers(default_tenant_id)?;
    let count = customers.count_customers().await?;
    println!("Total customers: {}", count);

    let products = shopster.products(default_tenant_id)?;
    let products_list = products.get_all().await?;
    println!("Total products: {}", products_list.len());

    Ok(())
}
