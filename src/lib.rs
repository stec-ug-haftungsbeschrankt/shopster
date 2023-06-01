extern crate diesel;
#[macro_use] extern crate diesel_migrations;

mod error;

use diesel::PgConnection;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::EmbeddedMigrations;
use error::ShopsterError;
use crate::diesel_migrations::MigrationHarness;
use log::info;
use std::collections::HashMap;
use tenet::Tenet;
use uuid::Uuid;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();


struct DatabaseSelector {
    tenants: Tenet,
    database_cache: HashMap<Uuid, DbConnection>
}

impl DatabaseSelector {
    fn new(tenant_database_url: String) -> Self {      
        DatabaseSelector {
            tenants: Tenet::new(tenant_database_url),
            database_cache: HashMap::new()
        }
    }

    fn get_storage_for_tenant(&mut self, tenant_id: Uuid) -> Result<&DbConnection, ShopsterError> {
        info!("Initializing Database");
    
        if !self.database_cache.contains_key(&tenant_id) {
            let tenant = self.tenants.get_tenant_by_id(tenant_id).ok_or(ShopsterError::TenantNotFoundError)?;
            let storages = tenant.get_storages();
            let storage = &storages[0];
            let connection_string = storage.connection_string.clone().unwrap(); // Option unwrap

            let manager = ConnectionManager::<PgConnection>::new(connection_string);
            let pool = Pool::new(manager)?;
        
            let mut database_connection = pool.get()?;
            database_connection.run_pending_migrations(MIGRATIONS).unwrap();

            self.database_cache.insert(tenant.id, database_connection);
        }

        Ok(self.database_cache.get(&tenant_id).unwrap())
    }
}


#[cfg(test)]
mod tests {
    use crate::DatabaseSelector;
    use crate::Uuid;
    
    #[test]
    fn tenant_not_found_test() {
        let mut database_selector = DatabaseSelector::new("TestString".to_string());
        database_selector.get_storage_for_tenant(Uuid::new_v4());
    }
    
}

