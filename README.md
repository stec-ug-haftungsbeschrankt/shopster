# Shopster

Database Layer for shop system with tenant support.

## Overview

Shopster is a Rust-based database abstraction layer designed for e-commerce systems with multi-tenant support. It provides a clean, type-safe API for managing customers, products, orders, baskets, and shop settings using PostgreSQL as the backend database.

## Features

- **Multi-tenant Support**: Built-in tenant isolation for managing multiple shops
- **Comprehensive E-commerce Models**: Support for:
    - Customers and customer management
    - Products with tags and images
    - Shopping baskets
    - Order processing
    - Shop settings and configuration
- **PostgreSQL Backend**: Uses Diesel ORM for type-safe database interactions
- **Type-safe**: Leverages Rust's type system for compile-time guarantees

## Quick Start

### Basic Example

```rust
use shopster::{Shopster, DatabaseSelector};
use stec_tenet::Tenet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tenant management
    let tenet = Tenet::new("postgres://localhost/tenet_db".to_string());

    // Create a database selector
    let mut db_selector = DatabaseSelector::new(tenet);

    // Add default database
    let tenant_id = db_selector.add_default(
        "postgres://localhost/shopster_db".to_string()
    )?;

    // Initialize Shopster
    let shopster = Shopster::new(database_selector);

    // Access modules
    let customers = shopster.customers(tenant_id)?;
    let products = shopster.products(tenant_id)?;
    let baskets = shopster.baskets(tenant_id)?;
    
    Ok(())
}
```

### Customer Management

```rust
use shopster::customers::Customer;
use stec_tenet::encryption_modes::EncryptionModes;
use uuid::Uuid;
use std::str::FromStr;

let customers = shopster.customers(tenant_id)?;

// Create a new customer
let customer = Customer {
    id: Uuid::new_v4(),
    email: "user@example.com".to_string(),
    email_verified: false,
    encryption_mode: EncryptionModes::from_str("Argon2")?,
    password: "secure_password".to_string(),
    full_name: "John Doe".to_string(),
    created_at: chrono::Utc::now().naive_utc(),
    updated_at: None,
};

let created = customers.insert(&customer)?;

// Authenticate
let authenticated = customers.verify_email_password(
    "user@example.com".to_string(),
    "secure_password"
)?;

// Search
let results = customers.search_customers("John")?;
```

### Shopping Cart Operations

```rust
let baskets = shopster.baskets(tenant_id)?;

// Create a basket
let basket_id = baskets.add_basket()?;

// Add products to basket
baskets.add_product_to_basket(basket_id, product_id, quantity)?;

// Get basket with full details
let products_detail = baskets.get_products_with_details(basket_id)?;
for item in products_detail {
    println!("{}: {}", item.product.title, item.quantity);
}

// Calculate total
let (total_cents, currency) = baskets.calculate_basket_total(basket_id)?;
println!("Total: {}{}", currency, total_cents as f64 / 100.0);
```

### Product Management

```rust
use shopster::products::{Product, Price};

let products = shopster.products(tenant_id)?;

// Create a product
let product = Product {
    id: 0,
    article_number: "SKU-001".to_string(),
    gtin: "1234567890123".to_string(),
    title: "Laptop".to_string(),
    short_description: "High-performance laptop".to_string(),
    description: "Full spec laptop".to_string(),
    tags: vec!["electronics".to_string(), "computers".to_string()],
    image_url: "https://example.com/laptop.jpg".to_string(),
    additional_images: vec![],
    price: Some(Price {
        amount: 99900, // $999.00
        currency: "USD".to_string(),
    }),
    weight: 2000,
    created_at: chrono::Utc::now().naive_utc(),
    updated_at: None,
};

let created = products.insert(&product)?;

// Get all products
let all = products.get_all()?;

// Get specific product
let specific = products.get(product_id)?;

// Update product
let mut updated = specific;
updated.price = Some(Price { amount: 89900, currency: "USD".to_string() });
products.update(&updated)?;
```

### Inventory Management

```rust
use shopster::warehouse::WarehouseItem;

let warehouse = shopster.warehouse(tenant_id)?;

// Add inventory
let item = WarehouseItem {
    id: 0,
    product_id: 42,
    in_stock: 100,
    reserved: 10,
    created_at: chrono::Utc::now().naive_utc(),
    updated_at: None,
};

let created = warehouse.insert(&item)?;
println!("Available: {}", created.available()); // Will print 90

// Check inventory
let all_items = warehouse.get_all()?;
for item in all_items {
    println!("Product {}: {} available", item.product_id, item.available());
}
```

## Usage Examples

See the `examples/` directory for complete working examples:

- `basic_setup.rs` - Initialization and setup
- `customer_workflow.rs` - Customer lifecycle management
- `shopping_workflow.rs` - Shopping cart operations
- `product_management.rs` - Product catalog management

Run examples with:
```bash
cargo run --example basic_setup
cargo run --example customer_workflow
cargo run --example shopping_workflow
cargo run --example product_management
```

## Project Structure

- `src/lib.rs` - Main entry point and Shopster struct
- `src/postgresql/` - PostgreSQL/Diesel-specific database implementations
- `src/baskets.rs` - Shopping basket domain logic
- `src/customers.rs` - Customer domain logic
- `src/orders.rs` - Order processing logic
- `src/products.rs` - Product catalog logic
- `src/settings.rs` - Configuration and settings
- `src/warehouse.rs` - Inventory management
- `src/schema.rs` - Database schema definitions
- `src/error.rs` - Error types
- `migrations/` - Database migrations
- `tests/` - Integration tests

## Architecture

Shopster implements a layered, multi-tenant architecture:

1. **Public API Layer** (`Shopster` and domain handlers) - Business logic interface
2. **Domain Layer** (customers, products, orders, etc.) - Core domain logic and data transformations
3. **Database Layer** (`postgresql/`) - Low-level Diesel operations
4. **Schema Layer** (`schema.rs`) - Diesel table definitions

**Multi-tenancy**: Every operation is tenant-aware through a `Uuid` tenant_id parameter, ensuring complete data isolation.

See `docs/ARCHITECTURE.md` for detailed architecture documentation.

## Prerequisites

- Rust (latest stable version)
- Docker (for running tests)
- PostgreSQL 12+

## Installation

Add Shopster to your `Cargo.toml`:

```toml
[dependencies]
stec_shopster = "0.2.19"
```

## Configuration

Create a `.env` file in the project root:

```
DATABASE_URL=postgres://user:password@localhost/shopster_db
TENET_DATABASE_URL=postgres://user:password@localhost/tenet_db
RUST_LOG=info
```

## Database Setup

Run migrations using Diesel CLI:

```bash
cargo install diesel_cli --no-default-features --features postgres
diesel migration run
```

## Testing

Tests use Docker containers for PostgreSQL. Requirements:

- Docker running
- `cargo-nextest` installed

Install test tools:
```bash
cargo install cargo-nextest --locked
```

Run tests:
```bash
cargo nextest run
```

Generate test coverage:
```bash
cargo install cargo-llvm-cov
cargo llvm-cov nextest
```

## API Documentation

Generate and view complete API documentation:

```bash
cargo doc --open
```

## Common Patterns

### Tenant-aware Operations

All Shopster operations require a `tenant_id`:

```rust
let customers = shopster.customers(tenant_id)?;
let baskets = shopster.baskets(tenant_id)?;
let orders = shopster.orders(tenant_id)?;
```

### Error Handling

Shopster uses `ShopsterError` enum for all operations:

```rust
match customers.get(customer_id) {
    Ok(customer) => println!("Found: {}", customer.email),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Connection Pooling

Shopster automatically manages connection pooling through r2d2. Connections are acquired per operation and returned to the pool automatically.

## CI/CD

This project uses GitHub Actions. Workflows are defined in `.github/workflows/` and run automatically on push and pull requests.

## Contributing

Contributions are welcome! Please ensure:

- All tests pass: `cargo nextest run`
- Code is formatted: `cargo fmt`
- No clippy warnings: `cargo clippy`
- Documentation is updated for public APIs

## License

GPL-3.0-or-later
