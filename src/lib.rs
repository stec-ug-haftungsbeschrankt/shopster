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


type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

const DATABASE_ACQUISITION_ERROR: &str = "Unable to acquire Database";

#[derive(Debug)]
pub struct DatabaseSelector {
    tenants: Tenet,
    database_cache: HashMap<Uuid, Pool>
}

impl DatabaseSelector {
    pub fn new(tenet: Tenet) -> Self {      
        DatabaseSelector {
            tenants: tenet,
            database_cache: HashMap::new()
        }
    }

    pub fn add_default(&mut self, connection_string: String) -> Result<Uuid, ShopsterError> {
        info!("Initializing default Database");
        let manager = ConnectionManager::<PgConnection>::new(connection_string);
        let pool = Pool::new(manager)?;

        let mut database_connection = pool.get()?;
        database_connection.run_pending_migrations(MIGRATIONS).unwrap();

        let tenant_id = Uuid::new_v4();
        self.database_cache.insert(tenant_id, pool);

        Ok(tenant_id)
    }

    fn get_storage_for_tenant(&mut self, tenant_id: Uuid) -> Result<Pool, ShopsterError> {
        info!("Initializing Database");
    
        if !self.database_cache.contains_key(&tenant_id) {
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
                Err(e) => warn!("{:?}", e)
            }

            self.database_cache.insert(tenant.id, pool);
        }

        let cached_pool = self.database_cache.get(&tenant_id).unwrap();
        Ok(cached_pool.clone())
    }
}

static DATABASE_SELECTOR: OnceLock<Mutex<DatabaseSelector>> = OnceLock::new();

fn aquire_database(tenant_id: Uuid) -> Result<DbConnection, ShopsterError> {
    let mut database_selector = DATABASE_SELECTOR.get().expect(DATABASE_ACQUISITION_ERROR).lock().unwrap();
    let pool = database_selector.get_storage_for_tenant(tenant_id)?;
    let connection = pool.get()?;
    Ok(connection)
}


#[derive(Debug, Clone)]
pub struct Shopster { }

impl Shopster {
    pub fn new(database_selector: DatabaseSelector) -> Self {
        if let Err(e) = DATABASE_SELECTOR.set(Mutex::new(database_selector)) {
            warn!("{:?}", e);
        }
        Shopster { }
    }
    
    pub fn baskets(&self, tenant_id: Uuid) -> Result<Baskets, ShopsterError> {
        Ok(Baskets::new(tenant_id))
    }

    pub fn customers(&self, tenant_id: Uuid) -> Result<Customers, ShopsterError> {
        Ok(Customers::new(tenant_id))
    }

    pub fn products(&self, tenant_id: Uuid) -> Result<Products, ShopsterError> {     
        Ok(Products::new(tenant_id))
    }

    pub fn orders(&self, tenant_id: Uuid) -> Result<Orders, ShopsterError> {
        Ok(Orders::new(tenant_id))
    }

    pub fn settings(&self, tenant_id: Uuid) -> Result<Settings, ShopsterError> {
        Ok(Settings::new(tenant_id))
    } 
}



#[cfg(test)]
mod tests {
    use stec_tenet::Storage;

    use super::*;

    use crate::DatabaseSelector;
    use crate::Uuid;

    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::SyncRunner;
    
    use crate::orders::Order;
    use crate::postgresql::dborder::OrderStatus;

    fn test_harness(test_code: impl Fn(String, String)) {
        let tenet_node = Postgres::default().start().expect("Unable to create to tenet container");
        let tenet_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", tenet_node.get_host_port_ipv4(5432).unwrap());

        let shopster_node = Postgres::default().start().expect("Unable to create to shopster container");
        let shopster_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/stec_shopster_test", shopster_node.get_host_port_ipv4(5432).unwrap());

        test_code(tenet_connection_string, shopster_connection_string);

        shopster_node.stop().expect("Failed to stop shopster");
        tenet_node.stop().expect("Failed to stop tenet");
    }

    #[test]
    fn tenant_not_found_test() {
        test_harness(|tenet_connection_string, _shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);
            let mut database_selector = DatabaseSelector::new(tenet);
            let tenant = database_selector.get_storage_for_tenant(Uuid::new_v4());
            
            assert!(tenant.is_err());
        });
    }

    #[test]
    fn settings_get_all() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("settings_get_all_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);

            let shopster = Shopster::new(database_selector);
            let settings = shopster.settings(tenant.id).unwrap().get_all();

            assert!(settings.is_ok());
            assert_eq!(13, settings.unwrap().len());
        });
    }

    #[test]
    fn order_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("basket_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            let orders = shopster.orders(tenant.id).unwrap();
            let new_order = Order {
                id: 0,
                status: OrderStatus::New,
                delivery_address: "Duffy Duck, Duck road 22, 44444 Duckhousen".to_string(),
                billing_address: "Duffy Duck, Duck road 22, 44444 Duckhousen".to_string(),
                created_at: Default::default(),
                updated_at: None,
            };

            let order = orders.insert(&new_order).unwrap();

            let mut all_orders = orders.get_all().unwrap();
            assert_eq!(1, all_orders.len());

            let inserted_order = all_orders.first().unwrap();
            assert_eq!(new_order.status, inserted_order.status);
            assert_eq!(new_order.billing_address, inserted_order.billing_address);
            assert_eq!(new_order.delivery_address, inserted_order.delivery_address);

            let updated_order = all_orders.get_mut(0).unwrap();
            updated_order.status = OrderStatus::ReadyToShip;
            updated_order.delivery_address = "Bugs Bunny, Bunny road 44, 55555 Bunnycity".to_string();

            orders.update(updated_order).unwrap();

            let success = orders.remove(all_orders.first().unwrap().id).unwrap();
            assert_eq!(true, success);
        });
    }
}

