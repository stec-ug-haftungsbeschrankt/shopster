use diesel::Connection;
use diesel::PgConnection;
use std::collections::HashMap;
use tenet::Tenet;
use uuid::Uuid;


struct DatabaseSelector {
    tenants: Tenet,
    database_cache: HashMap<Uuid, PgConnection>
}

impl DatabaseSelector {
    fn new(tenant_database_url: String) -> Self {      
        DatabaseSelector {
            tenants: Tenet::new(tenant_database_url),
            database_cache: HashMap::new()
        }
    }

    fn get_storage_for_tenant(&mut self, tenant_id: Uuid) -> Option<&PgConnection> {
        if !self.database_cache.contains_key(&tenant_id) {
            let tenant = self.tenants.get_tenant_by_id(tenant_id)?; // Tenet Error
            let storages = tenant.get_storages();
            let storage = &storages[0];
            let connection_string = storage.connection_string.clone()?; // Option unwrap

            let database_connection = PgConnection::establish(&connection_string).ok()?; // diesel Error
            self.database_cache.insert(tenant.id, database_connection);
        }

        self.database_cache.get(&tenant_id)
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

