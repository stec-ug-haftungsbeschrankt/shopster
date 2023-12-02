extern crate diesel;
#[macro_use] extern crate diesel_migrations;

mod postgresql;
mod error;
mod schema;
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
use crate::diesel_migrations::MigrationHarness;
use log::info;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::Mutex;
use tenet::Tenet;
use uuid::Uuid;

use baskets::Baskets;
use customers::Customers;
use products::Products;
use orders::Orders;
use settings::Settings;


type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

const DATABASE_AQUISITION_ERROR: &str = "Unable to aquire Database";

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
            let connection_string = storage.connection_string.clone().unwrap(); // Option unwrap

            let manager = ConnectionManager::<PgConnection>::new(connection_string);
            let pool = Pool::new(manager)?;
        
            let mut database_connection = pool.get()?;
            database_connection.run_pending_migrations(MIGRATIONS).unwrap();

            self.database_cache.insert(tenant.id, pool);
        }

        let cached_pool = self.database_cache.get(&tenant_id).unwrap();
        Ok(cached_pool.clone())
    }
}

static DATABASE_SELECTOR: OnceLock<Mutex<DatabaseSelector>> = OnceLock::new();

fn aquire_database(tenant_id: Uuid) -> Result<DbConnection, ShopsterError> {
    let mut database_selector = DATABASE_SELECTOR.get().expect(DATABASE_AQUISITION_ERROR).lock().unwrap();
    let pool = database_selector.get_storage_for_tenant(tenant_id)?;
    let connection = pool.get()?;
    Ok(connection)
}


pub struct Shopster { }

impl Shopster {
    pub fn new(database_selector: DatabaseSelector) -> Self {
        DATABASE_SELECTOR.set(Mutex::new(database_selector)).unwrap();
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
    use tenet::Storage;

    use super::*;

    use crate::DatabaseSelector;
    use crate::Uuid;
    
    static TEST_TENET_DATABASE_URL: &str = "postgres://postgres:@localhost/stec_tenet_test";
    static TEST_SHOPSTER_DATABASE_URL: &str = "postgres://postgres:@localhost/stec_shopster_test";

    #[test]
    fn tenant_not_found_test() {
        let tenant_database_url = TEST_TENET_DATABASE_URL.to_string();
        let tenet = Tenet::new(tenant_database_url);
        let mut database_selector = DatabaseSelector::new(tenet);
        let tenant = database_selector.get_storage_for_tenant(Uuid::new_v4());
        
        assert!(tenant.is_err());
    }

    #[test]
    fn settings_get_all() {
        let tenant_database_url = TEST_TENET_DATABASE_URL.to_string();
        let mut tenet = Tenet::new(tenant_database_url);
        
        let tenant = tenet.create_tenant("settings_get_all_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(TEST_SHOPSTER_DATABASE_URL.to_string(), tenant.id);
        tenant.add_storage(&storage).unwrap();
        
        let database_selector = DatabaseSelector::new(tenet);
        
        let shopster = Shopster::new(database_selector);
        let settings = shopster.settings(tenant.id).unwrap().get_all();
        
        assert!(settings.is_ok());
        assert_eq!(12, settings.unwrap().len());
    }
}

