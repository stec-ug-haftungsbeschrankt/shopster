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
//! - **Connection Pooling**: Efficient async connection management with bb8
//!
//! ## Quick Start
//!
//! ```ignore
//! use shopster::{Shopster, DatabaseSelector};
//! use stec_tenet::Tenet;
//! use uuid::Uuid;
//!
//! let tenet = Tenet::new("postgres://localhost/tenet_db".to_string());
//! let mut db_selector = DatabaseSelector::new(tenet);
//! let tenant_id = db_selector.add_default(
//!     "postgres://localhost/shopster_db".to_string()
//! ).await?;
//! let shopster = Shopster::new(db_selector);
//! let customers = shopster.customers(tenant_id)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Architecture Overview
//!
//! Shopster implements a layered architecture:
//!
//! - **Public API Layer** (`Shopster` struct): Entry point for business logic
//! - **Domain Modules**: Each module (customers, products, etc.) encapsulates domain logic
//! - **Database Layer** (`postgresql/`): Diesel-based async database operations
//! - **Schema Layer** (`schema.rs`): Diesel table! macros and database schema definitions
//!
//! All operations are tenant-aware through a unique `Uuid` tenant_id.

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
pub use orders::PaymentStatus;

#[doc(hidden)]
pub use postgresql::dborder::DbOrderStatus;
#[doc(hidden)]
pub use postgresql::dborder::DbPaymentStatus;

use diesel::PgConnection;
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_migrations::EmbeddedMigrations;

use error::ShopsterError;
use log::warn;
use crate::diesel_migrations::MigrationHarness;
use crate::postgresql::DatabaseHelper;
use log::info;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use stec_tenet::Tenet;
use uuid::Uuid;

use baskets::Baskets;
use customers::Customers;
use products::Products;
use orders::Orders;
use settings::Settings;
use warehouse::Warehouse;


pub(crate) type DbPool = Pool<AsyncPgConnection>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

const DATABASE_ACQUISITION_ERROR: &str = "Unable to acquire Database";

/// Manages database connections for multiple tenants in a multi-tenant system.
///
/// `DatabaseSelector` handles tenant-to-database mapping, connection pooling, and
/// automatic schema migrations. It maintains a cache of connection pools, creating
/// new pools on-demand when a tenant is first accessed.
#[derive(Debug)]
pub struct DatabaseSelector {
    tenants: Tenet,
    database_cache: HashMap<Uuid, DbPool>
}

impl DatabaseSelector {
    /// Creates a new `DatabaseSelector` with the given tenant service.
    pub fn new(tenet: Tenet) -> Self {
        DatabaseSelector {
            tenants: tenet,
            database_cache: HashMap::new()
        }
    }

    /// Registers and initializes the default database.
    ///
    /// Runs migrations synchronously on a temporary connection, then builds
    /// the async connection pool. Returns the generated tenant ID.
    pub async fn add_default(&mut self, connection_string: String) -> Result<Uuid, ShopsterError> {
        info!("Initializing default Database");

        {
            let mut sync_conn = PgConnection::establish(&connection_string)
                .map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
            sync_conn.run_pending_migrations(MIGRATIONS)
                .map_err(|e| ShopsterError::DatabaseMigrationError(e.to_string()))?;
        }

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&connection_string);
        let pool = Pool::builder()
            .build(manager)
            .await
            .map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let tenant_id = Uuid::new_v4();
        self.database_cache.insert(tenant_id, pool);

        Ok(tenant_id)
    }

    /// Retrieves or creates a connection pool for the given tenant.
    ///
    /// Caches pools per tenant. On first access: fetches tenant metadata,
    /// creates the database if missing, runs migrations, and caches the pool.
    pub async fn get_storage_for_tenant(&mut self, tenant_id: Uuid) -> Result<DbPool, ShopsterError> {
        if !self.database_cache.contains_key(&tenant_id) {
            info!("Initializing Database");

            let tenant = self.tenants.get_tenant_by_id(tenant_id).ok_or(ShopsterError::TenantNotFoundError)?;
            let storages = tenant.get_storages();

            if storages.is_empty() {
                return Err(ShopsterError::TenantStorageNotFound);
            }

            let storage = &storages[0];
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

            {
                let mut sync_conn = PgConnection::establish(&connection_string)
                    .map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
                match sync_conn.run_pending_migrations(MIGRATIONS) {
                    Ok(_) => info!("Shopster Database migrations successfully executed."),
                    Err(e) => warn!("Migrations failed: {:?}", e)
                }
            }

            let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&connection_string);
            let pool = Pool::builder()
                .build(manager)
                .await
                .map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

            self.database_cache.insert(tenant.id, pool);
        }

        Ok(self.database_cache.get(&tenant_id).unwrap().clone())
    }
}

static DATABASE_SELECTOR: OnceLock<Mutex<DatabaseSelector>> = OnceLock::new();

/// Acquires a connection pool for the given tenant from the global selector.
pub(crate) async fn aquire_pool(tenant_id: Uuid) -> Result<DbPool, ShopsterError> {
    let mut database_selector = DATABASE_SELECTOR.get()
        .expect(DATABASE_ACQUISITION_ERROR)
        .lock()
        .await;
    database_selector.get_storage_for_tenant(tenant_id).await
}


/// The main entry point for accessing Shopster functionality.
///
/// `Shopster` provides a unified interface to access domain modules for a specific tenant.
///
/// # Example
///
/// ```ignore
/// let shopster = Shopster::new(database_selector);
/// let customers = shopster.customers(tenant_id)?;
/// let customer = customers.get(customer_uuid).await?;
/// ```
#[derive(Debug, Clone)]
pub struct Shopster { }

impl Shopster {
    /// Creates a new `Shopster` instance and initializes the global database selector.
    pub fn new(database_selector: DatabaseSelector) -> Self {
        if let Err(e) = DATABASE_SELECTOR.set(Mutex::new(database_selector)) {
            warn!("{:?}", e);
        }
        Shopster { }
    }

    /// Gets a `Baskets` handler for managing shopping carts.
    pub fn baskets(&self, tenant_id: Uuid) -> Result<Baskets, ShopsterError> {
        Ok(Baskets::new(tenant_id))
    }

    /// Gets a `Customers` handler for customer management.
    pub fn customers(&self, tenant_id: Uuid) -> Result<Customers, ShopsterError> {
        Ok(Customers::new(tenant_id))
    }

    /// Gets a `Products` handler for product catalog management.
    pub fn products(&self, tenant_id: Uuid) -> Result<Products, ShopsterError> {
        Ok(Products::new(tenant_id))
    }

    /// Gets an `Orders` handler for order management and processing.
    pub fn orders(&self, tenant_id: Uuid) -> Result<Orders, ShopsterError> {
        Ok(Orders::new(tenant_id))
    }

    /// Gets a `Settings` handler for shop configuration.
    pub fn settings(&self, tenant_id: Uuid) -> Result<Settings, ShopsterError> {
        Ok(Settings::new(tenant_id))
    }

    /// Gets a `Warehouse` handler for inventory management.
    pub fn warehouse(&self, tenant_id: Uuid) -> Result<Warehouse, ShopsterError> {
        Ok(Warehouse::new(tenant_id))
    }
}
