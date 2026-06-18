//! # Shopster: E-commerce Database Layer
//!
//! Shopster is a comprehensive Rust database abstraction layer for multi-tenant e-commerce systems.
//! It provides type-safe, ergonomic APIs for managing customers, products, orders, baskets, and shop
//! settings using PostgreSQL as the backend.
//!
//! ## Features
//!
//! - **Multi-tenant Support**: Built-in tenant isolation for managing multiple shops
//! - **E-commerce Models**: Customers, Products, Shopping Baskets, Orders, Warehouse inventory
//! - **Type Safety**: Leverages Rust's type system for compile-time guarantees
//! - **PostgreSQL Backend**: Uses Diesel ORM for type-safe database interactions
//! - **Connection Pooling**: Efficient connection management with r2d2
//!
//! ## Quick Start
//!
//! ```ignore
//! use shopster::{Shopster, DatabaseSelector};
//! use stec_tenet::Tenet;
//! use uuid::Uuid;
//!
//! // Initialize tenant management
//! let tenet = Tenet::new("postgres://localhost/tenet_db".to_string());
//!
//! // Create a database selector
//! let mut db_selector = DatabaseSelector::new(tenet);
//!
//! // Add default database and get tenant ID
//! let tenant_id = db_selector.add_default(
//!     "postgres://localhost/shopster_db".to_string()
//! )?;
//!
//! // Initialize Shopster with the database selector
//! let shopster = Shopster::new(db_selector);
//!
//! // Access modules
//! let customers = shopster.customers(tenant_id)?;
//! let products = shopster.products(tenant_id)?;
//! let baskets = shopster.baskets(tenant_id)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Architecture Overview
//!
//! Shopster implements a layered architecture:
//!
//! - **Public API Layer** (`Shopster` struct): Entry point for business logic
//! - **Domain Modules**: Each module (customers, products, etc.) encapsulates domain logic
//! - **Database Layer** (`postgresql/`): Low-level Diesel-based database operations
//! - **Schema Layer** (`schema.rs`): Diesel table! macros and database schema definitions
//!
//! All operations are tenant-aware through a unique `Uuid` tenant_id.
//!
//! ## Modules
//!
//! - [`baskets`]: Shopping cart functionality for customers
//! - [`customers`]: Customer management, authentication, and user profiles
//! - [`products`]: Product catalog management with images and pricing
//! - [`orders`]: Order processing and status tracking
//! - [`settings`]: Shop configuration and settings
//! - [`warehouse`]: Inventory and warehouse management
//! - [`error`]: Error types and handling

extern crate diesel;
#[macro_use] extern crate diesel_migrations;

mod postgresql;
mod schema;
pub mod error;
pub mod baskets;
pub mod customers;
pub mod products;
pub mod orders;
pub mod settings;
pub mod warehouse;
pub use orders::OrderStatus;

// Export DbOrderStatus for tests only - hidden from public documentation
#[doc(hidden)]
pub use postgresql::dborder::DbOrderStatus;

use diesel::PgConnection;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::EmbeddedMigrations;

use error::ShopsterError;
use log::warn;
use crate::diesel_migrations::MigrationHarness;
use crate::postgresql::DatabaseHelper;
use log::info;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::Mutex;
use stec_tenet::Tenet;
use uuid::Uuid;

use baskets::Baskets;
use customers::Customers;
use products::Products;
use orders::Orders;
use settings::Settings;
use warehouse::Warehouse;


type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
/// A pooled connection to PostgreSQL managed by r2d2.
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

const DATABASE_ACQUISITION_ERROR: &str = "Unable to acquire Database";

/// Manages database connections for multiple tenants in a multi-tenant system.
///
/// `DatabaseSelector` handles tenant-to-database mapping, connection pooling, and
/// automatic schema migrations. It maintains a cache of connection pools, creating
/// new pools on-demand when a tenant is first accessed.
///
/// # Multi-tenancy Model
///
/// Each tenant can have one or more database storages. The current implementation
/// uses the first available storage for each tenant. This could be extended to
/// support storage selection strategies.
#[derive(Debug)]
pub struct DatabaseSelector {
    /// Tenet service for tenant metadata resolution
    tenants: Tenet,
    /// Cache of per-tenant connection pools
    database_cache: HashMap<Uuid, Pool>
}

impl DatabaseSelector {
    /// Creates a new `DatabaseSelector` with the given tenant service.
    ///
    /// # Arguments
    ///
    /// * `tenet` - The tenant service providing tenant metadata
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tenet = Tenet::new("postgres://localhost/tenet_db".to_string());
    /// let selector = DatabaseSelector::new(tenet);
    /// ```
    pub fn new(tenet: Tenet) -> Self {
        DatabaseSelector {
            tenants: tenet,
            database_cache: HashMap::new()
        }
    }

    /// Registers and initializes the default database.
    ///
    /// This creates a connection pool for a default tenant (with a new UUID), runs
    /// all pending migrations, and returns the tenant ID.
    ///
    /// # Arguments
    ///
    /// * `connection_string` - PostgreSQL connection string (e.g., `postgres://user:pass@host/db`)
    ///
    /// # Returns
    ///
    /// `Ok(Uuid)` - The generated tenant ID for the default database
    /// `Err(ShopsterError)` - If connection or migration fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut selector = DatabaseSelector::new(tenet);
    /// let tenant_id = selector.add_default(
    ///     "postgres://postgres:postgres@localhost/shopster_db".to_string()
    /// )?;
    /// ```
    pub fn add_default(&mut self, connection_string: String) -> Result<Uuid, ShopsterError> {
        info!("Initializing default Database");
        let manager = ConnectionManager::<PgConnection>::new(connection_string);
        let pool = Pool::new(manager)?;

        let mut database_connection = pool.get()?;
        database_connection.run_pending_migrations(MIGRATIONS)
            .map_err(|e| ShopsterError::DatabaseMigrationError(e.to_string()))?;

        let tenant_id = Uuid::new_v4();
        self.database_cache.insert(tenant_id, pool);

        Ok(tenant_id)
    }

    /// Retrieves or creates a connection pool for the given tenant.
    ///
    /// This method caches pools per tenant. On first access for a tenant:
    /// 1. Fetches tenant metadata from the Tenet service
    /// 2. Creates the database if it doesn't exist
    /// 3. Runs pending migrations
    /// 4. Caches the pool for future use
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The UUID of the tenant
    ///
    /// # Returns
    ///
    /// `Ok(Pool)` - A cloned connection pool for the tenant
    /// `Err(ShopsterError)` - If tenant not found, storage not configured, or database operations fail
    pub fn get_storage_for_tenant(&mut self, tenant_id: Uuid) -> Result<Pool, ShopsterError> {
        if !self.database_cache.contains_key(&tenant_id) {
            info!("Initializing Database");

            let tenant = self.tenants.get_tenant_by_id(tenant_id).ok_or(ShopsterError::TenantNotFoundError)?;
            let storages = tenant.get_storages();
            
            if storages.is_empty() {
                return Err(ShopsterError::TenantStorageNotFound);
            }
            
            let storage = &storages[0]; // FIXME What if we have multiple storages? Choose by storage type? 
            let connection_string = match storage.connection_string.clone() {
                Some(cs) => cs,
                None => return Err(ShopsterError::TenantStorageNotFound)
            };

            info!("Database connection string: {}", connection_string);

            if !DatabaseHelper::is_database_exists(&connection_string) {
                info!("Database does not exit, creating it...");
                if let Err(e) = DatabaseHelper::create_database(&connection_string) {
                    warn!("{:?}", e);
                }
            }

            let manager = ConnectionManager::<PgConnection>::new(connection_string);
            let pool = Pool::new(manager)?;

            let mut database_connection = pool.get()?;

            match database_connection.run_pending_migrations(MIGRATIONS) {
                Ok(_) => info!("Shopster Database migrations successfully executed."),
                Err(e) => warn!("Migrations failed: {:?}", e)
            }

            self.database_cache.insert(tenant.id, pool);
        }

        let cached_pool = self.database_cache.get(&tenant_id).unwrap();
        Ok(cached_pool.clone())
    }
}

/// Global static database selector stored in `OnceLock` for thread-safe access.
static DATABASE_SELECTOR: OnceLock<Mutex<DatabaseSelector>> = OnceLock::new();

/// Acquires a database connection for the given tenant.
///
/// This function retrieves a connection from the global database selector.
/// Used internally by domain modules.
///
/// # Arguments
///
/// * `tenant_id` - The UUID of the tenant
///
/// # Returns
///
/// `Ok(DbConnection)` - A pooled connection ready for use
/// `Err(ShopsterError)` - If tenant not found or connection acquisition fails
fn aquire_database(tenant_id: Uuid) -> Result<DbConnection, ShopsterError> {
    let mut database_selector = DATABASE_SELECTOR.get().expect(DATABASE_ACQUISITION_ERROR)
        .lock().map_err(|_| ShopsterError::InternalError("Database selector mutex poisoned".to_string()))?;
    let pool = database_selector.get_storage_for_tenant(tenant_id)?;
    let connection = pool.get()?;
    Ok(connection)
}


/// The main entry point for accessing Shopster functionality.
///
/// `Shopster` provides a unified interface to access domain modules (customers, products, orders, etc.)
/// for a specific tenant. It manages the global database selector and routes operations to the
/// appropriate domain handler.
///
/// # Example
///
/// ```ignore
/// let shopster = Shopster::new(database_selector);
/// let customers = shopster.customers(tenant_id)?;
/// let customer = customers.get(customer_uuid)?;
/// ```
#[derive(Debug, Clone)]
pub struct Shopster { }

impl Shopster {
    /// Creates a new `Shopster` instance and initializes the global database selector.
    ///
    /// # Arguments
    ///
    /// * `database_selector` - The database selector to use for all operations
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut db_selector = DatabaseSelector::new(tenet);
    /// db_selector.add_default(connection_string)?;
    /// let shopster = Shopster::new(db_selector);
    /// ```
    pub fn new(database_selector: DatabaseSelector) -> Self {
        if let Err(e) = DATABASE_SELECTOR.set(Mutex::new(database_selector)) {
            warn!("{:?}", e);
        }
        Shopster { }
    }
    
    /// Gets a `Baskets` handler for managing shopping carts.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Baskets)` - A baskets handler for the tenant
    /// `Err(ShopsterError)` - If operations fail
    ///
    /// # Example
    ///
    /// ```ignore
    /// let baskets = shopster.baskets(tenant_id)?;
    /// let basket_id = baskets.add_basket()?;
    /// ```
    pub fn baskets(&self, tenant_id: Uuid) -> Result<Baskets, ShopsterError> {
        Ok(Baskets::new(tenant_id))
    }

    /// Gets a `Customers` handler for customer management.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Customers)` - A customers handler for the tenant
    /// `Err(ShopsterError)` - If operations fail
    ///
    /// # Example
    ///
    /// ```ignore
    /// let customers = shopster.customers(tenant_id)?;
    /// let all_customers = customers.get_all()?;
    /// ```
    pub fn customers(&self, tenant_id: Uuid) -> Result<Customers, ShopsterError> {
        Ok(Customers::new(tenant_id))
    }

    /// Gets a `Products` handler for product catalog management.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Products)` - A products handler for the tenant
    /// `Err(ShopsterError)` - If operations fail
    ///
    /// # Example
    ///
    /// ```ignore
    /// let products = shopster.products(tenant_id)?;
    /// let product = products.get(product_id)?;
    /// ```
    pub fn products(&self, tenant_id: Uuid) -> Result<Products, ShopsterError> {
        Ok(Products::new(tenant_id))
    }

    /// Gets an `Orders` handler for order management and processing.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Orders)` - An orders handler for the tenant
    /// `Err(ShopsterError)` - If operations fail
    ///
    /// # Example
    ///
    /// ```ignore
    /// let orders = shopster.orders(tenant_id)?;
    /// let all_orders = orders.get_all()?;
    /// ```
    pub fn orders(&self, tenant_id: Uuid) -> Result<Orders, ShopsterError> {
        Ok(Orders::new(tenant_id))
    }

    /// Gets a `Settings` handler for shop configuration.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Settings)` - A settings handler for the tenant
    /// `Err(ShopsterError)` - If operations fail
    pub fn settings(&self, tenant_id: Uuid) -> Result<Settings, ShopsterError> {
        Ok(Settings::new(tenant_id))
    } 

    /// Gets a `Warehouse` handler for inventory management.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Warehouse)` - A warehouse handler for the tenant
    /// `Err(ShopsterError)` - If operations fail
    pub fn warehouse(&self, tenant_id: Uuid) -> Result<Warehouse, ShopsterError> {
        Ok(Warehouse::new(tenant_id))
    }
}



