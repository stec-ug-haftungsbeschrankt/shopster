//! Integration tests that require Docker
//!
//! These tests require Docker to be running. Run with:
//! `docker ps` to verify Docker is available
//! `cargo test --test integration_test` to run these tests

use stec_shopster::{Shopster, DatabaseSelector, orders::Order, orders::OrderStatus, orders::PaymentStatus};
use stec_tenet::{Tenet, Storage};
use uuid::Uuid;

use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

async fn test_harness<F, Fut>(test_code: F)
where
    F: FnOnce(String, String) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let tenet_node = Postgres::default().start().await.expect("Unable to create tenet container");
    let tenet_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", tenet_node.get_host_port_ipv4(5432).await.unwrap());

    let shopster_node = Postgres::default().start().await.expect("Unable to create shopster container");
    let shopster_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/stec_shopster_test", shopster_node.get_host_port_ipv4(5432).await.unwrap());

    test_code(tenet_connection_string, shopster_connection_string).await;

    shopster_node.stop().await.expect("Failed to stop shopster");
    tenet_node.stop().await.expect("Failed to stop tenet");
}

#[tokio::test]
async fn tenant_not_found_test() {
    test_harness(|tenet_connection_string, _shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let mut database_selector = DatabaseSelector::new(tenet);
        let tenant = database_selector.get_storage_for_tenant(Uuid::new_v4()).await;

        assert!(tenant.is_err());
    }).await;
}

#[tokio::test]
async fn settings_get_all() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("settings_get_all_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let settings = shopster.settings(tenant.id).unwrap().get_all().await;

        assert!(settings.is_ok());
        assert_eq!(14, settings.unwrap().len());
    }).await;
}

#[tokio::test]
async fn order_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("order_test".to_string()).unwrap();
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
            payment_reference: None,
            payment_status: PaymentStatus::Pending,
        };

        let _ = orders.insert(&new_order).await.unwrap();

        let mut all_orders = orders.get_all().await.unwrap();
        assert_eq!(1, all_orders.len());

        let inserted_order = all_orders.first().unwrap();
        assert_eq!(new_order.status, inserted_order.status);
        assert_eq!(new_order.billing_address, inserted_order.billing_address);
        assert_eq!(new_order.delivery_address, inserted_order.delivery_address);

        let updated_order = all_orders.get_mut(0).unwrap();
        updated_order.status = OrderStatus::InProgress;
        updated_order.delivery_address = "Bugs Bunny, Bunny road 44, 55555 Bunnycity".to_string();

        orders.update(updated_order).await.unwrap();

        let mut all_orders = orders.get_all().await.unwrap();
        let updated_order = all_orders.get_mut(0).unwrap();
        updated_order.status = OrderStatus::ReadyToShip;
        orders.update(updated_order).await.unwrap();

        let success = orders.remove(all_orders.first().unwrap().id).await.unwrap();
        assert_eq!(true, success);
    }).await;
}
