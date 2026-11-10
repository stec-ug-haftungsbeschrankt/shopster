//! Tenant database initialisation: failures must be reported, not swallowed, and a failed
//! initialisation must not be cached.

mod common;

use diesel::prelude::*;
use diesel::sql_query;
use stec_shopster::DatabaseSelector;
use stec_shopster::error::ShopsterError;
use stec_tenet::{Storage, Tenet};

use crate::common::test_harness;

fn server_url(database_url: &str) -> String {
    database_url.rsplit_once('/').unwrap().0.to_string()
}

fn database_name(database_url: &str) -> String {
    database_url.rsplit_once('/').unwrap().1.to_string()
}

#[tokio::test]
async fn failed_migration_is_an_error_and_is_retried_on_next_access() {
    test_harness(|tenet_cs, shopster_cs| async move {
        // A database that already contains a conflicting `settings` table makes the very
        // first shopster migration fail.
        let mut admin = PgConnection::establish(&server_url(&shopster_cs)).unwrap();
        sql_query(format!("CREATE DATABASE \"{}\"", database_name(&shopster_cs))).execute(&mut admin).unwrap();
        let mut tenant_db = PgConnection::establish(&shopster_cs).unwrap();
        sql_query("CREATE TABLE settings (id INT)").execute(&mut tenant_db).unwrap();

        let tenet = Tenet::new(tenet_cs);
        let tenant = tenet.create_tenant("tenant_init_failed_migration".to_string()).unwrap();
        tenant.add_storage(&Storage::new_postgresql_database(shopster_cs.clone(), tenant.id)).unwrap();
        let selector = DatabaseSelector::new(tenet);

        let first = selector.get_storage_for_tenant(tenant.id).await;
        assert!(matches!(first, Err(ShopsterError::DatabaseMigrationError(_))), "got {:?}", first.err());

        // Still failing: nothing broken was cached in the meantime.
        assert!(selector.get_storage_for_tenant(tenant.id).await.is_err());

        // Fix the database; the next access must retry initialisation and succeed.
        sql_query("DROP TABLE settings").execute(&mut tenant_db).unwrap();
        assert!(selector.get_storage_for_tenant(tenant.id).await.is_ok());
    }).await;
}

#[tokio::test]
async fn unreachable_server_is_an_error_not_a_missing_database() {
    test_harness(|tenet_cs, _shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant = tenet.create_tenant("tenant_init_unreachable".to_string()).unwrap();
        // Port 1 on localhost: connection refused.
        tenant.add_storage(&Storage::new_postgresql_database("postgres://postgres:postgres@127.0.0.1:1/some_db".to_string(), tenant.id)).unwrap();
        let selector = DatabaseSelector::new(tenet);

        let result = selector.get_storage_for_tenant(tenant.id).await;
        assert!(matches!(result, Err(ShopsterError::DatabaseConnectionError(_))), "got {:?}", result.err());
    }).await;
}

#[tokio::test]
async fn invalid_database_name_is_rejected_without_running_sql() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant = tenet.create_tenant("tenant_init_bad_name".to_string()).unwrap();
        let bad_url = format!("{}/x\"; DROP DATABASE postgres; --", server_url(&shopster_cs));
        tenant.add_storage(&Storage::new_postgresql_database(bad_url, tenant.id)).unwrap();
        let selector = DatabaseSelector::new(tenet);

        let result = selector.get_storage_for_tenant(tenant.id).await;
        assert!(result.is_err());

        // The server (and its default database) is still there.
        assert!(PgConnection::establish(&format!("{}/postgres", server_url(&shopster_cs))).is_ok());
    }).await;
}

#[tokio::test]
async fn missing_database_is_created_on_first_access() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant = tenet.create_tenant("tenant_init_creates_db".to_string()).unwrap();
        tenant.add_storage(&Storage::new_postgresql_database(format!("{}?connect_timeout=10", shopster_cs), tenant.id)).unwrap();
        let selector = DatabaseSelector::new(tenet);

        assert!(selector.get_storage_for_tenant(tenant.id).await.is_ok());
        assert!(PgConnection::establish(&shopster_cs).is_ok());
    }).await;
}
