//! Integration tests that require Docker
//!
//! These tests require Docker to be running. Run with:
//! `docker ps` to verify Docker is available
//! `cargo test --test integration_test` to run these tests

use stec_shopster::{Shopster, DatabaseSelector, orders::Order, orders::OrderStatus};
use stec_tenet::{Tenet, Storage};
use uuid::Uuid;

use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::SyncRunner;

fn test_harness(test_code: impl Fn(String, String)) {
    let tenet_node = Postgres::default().start().expect("Unable to create tenet container");
    let tenet_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", tenet_node.get_host_port_ipv4(5432).unwrap());

    let shopster_node = Postgres::default().start().expect("Unable to create shopster container");
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
            customer_id: None,
            status: OrderStatus::New,
            delivery_address: "Duffy Duck, Duck road 22, 44444 Duckhousen".to_string(),
            billing_address: "Duffy Duck, Duck road 22, 44444 Duckhousen".to_string(),
            items: Vec::new(),
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

