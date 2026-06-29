mod common;

use std::convert::TryFrom;
use chrono::Utc;
use uuid::Uuid;
use stec_tenet::{Storage, Tenet};
use stec_shopster::{DatabaseSelector, DbOrderStatus, Shopster};
use stec_shopster::customers::Customer;
use stec_shopster::orders::{Order, OrderItemSnapshot, OrderItemPrice, OrderStatus};
use stec_shopster::products::{Price, Product};
use stec_shopster::warehouse::WarehouseItem;
use stec_tenet::encryption_modes::EncryptionModes;
use crate::common::test_harness;

fn make_order(status: OrderStatus) -> Order {
    Order {
        id: 0,
        customer_id: None,
        status,
        delivery_address: "Test Street 1, 12345 Testcity".to_string(),
        billing_address: "Test Street 1, 12345 Testcity".to_string(),
        items: Vec::new(),
        created_at: Utc::now().naive_utc(),
        updated_at: None,
    }
}

fn make_product_with_price(article: &str, gtin: &str, price: i64) -> Product {
    Product {
        id: 0,
        article_number: article.to_string(),
        title: "Order Test Product".to_string(),
        gtin: gtin.to_string(),
        short_description: "Short".to_string(),
        description: "Desc".to_string(),
        image_url: "/images/test.png".to_string(),
        additional_images: Vec::new(),
        price: Some(Price { amount: price, currency: "EUR".to_string() }),
        weight: 100,
        tags: Vec::new(),
        created_at: Utc::now().naive_utc(),
        updated_at: None,
    }
}

#[tokio::test]
async fn order_insert_and_get_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_insert_get".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let orders = shopster.orders(tenant.id).unwrap();
        let inserted = orders.insert(&make_order(OrderStatus::New)).await.unwrap();

        assert_eq!(OrderStatus::New, inserted.status);
        assert_eq!("Test Street 1, 12345 Testcity", inserted.delivery_address);

        let fetched = orders.get_by_id(inserted.id).await.unwrap();
        assert_eq!(inserted.id, fetched.id);
        assert_eq!(OrderStatus::New, fetched.status);

        let all = orders.get_all().await.unwrap();
        assert_eq!(1, all.len());
    }).await;
}

#[tokio::test]
async fn order_full_lifecycle_transition_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_full_lifecycle".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let orders = shopster.orders(tenant.id).unwrap();
        let order = orders.insert(&make_order(OrderStatus::New)).await.unwrap();

        let transitions = [
            OrderStatus::InProgress,
            OrderStatus::ReadyToShip,
            OrderStatus::Shipping,
            OrderStatus::Done,
        ];

        let mut current = order;
        for next_status in transitions {
            current.status = next_status;
            current = orders.update(&current).await.unwrap();
            assert_eq!(next_status, current.status);
        }
    }).await;
}

#[tokio::test]
async fn order_invalid_transition_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_invalid_transition".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let orders = shopster.orders(tenant.id).unwrap();
        let order = orders.insert(&make_order(OrderStatus::New)).await.unwrap();

        let mut bad_order = order;
        bad_order.status = OrderStatus::Done;
        let result = orders.update(&bad_order).await;

        assert!(result.is_err(), "Expected error for invalid transition New -> Done");
    }).await;
}

#[tokio::test]
async fn order_inventory_reservation_on_insert_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_inventory_reservation".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product_with_price("ART-ORD-001", "1111111111111", 500)).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 100,
            reserved: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        }).await.unwrap();

        // Insert a New order with items — New is a reserving status
        let order = Order {
            id: 0,
            customer_id: None,
            status: OrderStatus::New,
            delivery_address: "Test Street 1, 12345 Testcity".to_string(),
            billing_address: "Test Street 1, 12345 Testcity".to_string(),
            items: vec![OrderItemSnapshot {
                id: 0,
                product_id: product.id,
                quantity: 3,
                article_number: product.article_number.clone(),
                gtin: product.gtin.clone(),
                title: product.title.clone(),
                short_description: product.short_description.clone(),
                description: product.description.clone(),
                tags: vec![],
                title_image: product.image_url.clone(),
                additional_images: vec![],
                price: OrderItemPrice { amount: 500, currency: "EUR".to_string() },
                weight: product.weight,
            }],
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        };

        let orders = shopster.orders(tenant.id).unwrap();
        orders.insert(&order).await.unwrap();

        let wh_item = warehouse.get_by_product_id(product.id).await.unwrap();
        assert_eq!(3, wh_item.reserved, "Inventory should be reserved for New order");
        assert_eq!(97, wh_item.available());
    }).await;
}

#[tokio::test]
async fn order_reservation_released_on_shipping_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_reservation_release".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product_with_price("ART-ORD-002", "2222222222222", 200)).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 50,
            reserved: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        }).await.unwrap();

        let order_with_item = Order {
            id: 0,
            customer_id: None,
            status: OrderStatus::New,
            delivery_address: "Test Street 1, 12345 Testcity".to_string(),
            billing_address: "Test Street 1, 12345 Testcity".to_string(),
            items: vec![OrderItemSnapshot {
                id: 0,
                product_id: product.id,
                quantity: 5,
                article_number: product.article_number.clone(),
                gtin: product.gtin.clone(),
                title: product.title.clone(),
                short_description: product.short_description.clone(),
                description: product.description.clone(),
                tags: vec![],
                title_image: product.image_url.clone(),
                additional_images: vec![],
                price: OrderItemPrice { amount: 200, currency: "EUR".to_string() },
                weight: product.weight,
            }],
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        };

        let orders = shopster.orders(tenant.id).unwrap();
        let mut order = orders.insert(&order_with_item).await.unwrap();

        // Verify reserved after New
        let wh_after_new = warehouse.get_by_product_id(product.id).await.unwrap();
        assert_eq!(5, wh_after_new.reserved);

        // Transition to Shipping (non-reserving) — reservation should be released
        order.status = OrderStatus::InProgress;
        order = orders.update(&order).await.unwrap();
        order.status = OrderStatus::ReadyToShip;
        order = orders.update(&order).await.unwrap();
        order.status = OrderStatus::Shipping;
        orders.update(&order).await.unwrap();

        let wh_after_shipping = warehouse.get_by_product_id(product.id).await.unwrap();
        assert_eq!(0, wh_after_shipping.reserved, "Reservation should be released when order reaches Shipping");
    }).await;
}

#[tokio::test]
async fn order_create_from_basket_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_from_basket".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product_with_price("ART-ORD-003", "3333333333333", 999)).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 10,
            reserved: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        }).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();
        baskets.add_product_to_basket(basket_id, product.id, 2).await.unwrap();

        let orders = shopster.orders(tenant.id).unwrap();
        let order = orders.create_from_basket(
            basket_id,
            "Delivery Street 5, 99999 Deliverytown".to_string(),
            "Billing Street 5, 99999 Billingtown".to_string(),
        ).await.unwrap();

        assert_eq!(OrderStatus::New, order.status);
        assert_eq!(1, order.items.len());
        assert_eq!(product.id, order.items[0].product_id);
        assert_eq!(2, order.items[0].quantity);
        assert_eq!(999, order.items[0].price.amount);
    }).await;
}

#[tokio::test]
async fn order_missing_address_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_missing_address".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let orders = shopster.orders(tenant.id).unwrap();

        let empty_delivery = Order {
            delivery_address: "".to_string(),
            ..make_order(OrderStatus::New)
        };
        assert!(orders.insert(&empty_delivery).await.is_err());

        let empty_billing = Order {
            billing_address: "   ".to_string(),
            ..make_order(OrderStatus::New)
        };
        assert!(orders.insert(&empty_billing).await.is_err());
    }).await;
}

#[tokio::test]
async fn order_get_by_customer_id_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_get_by_customer".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();
        let customer = customers.insert(&Customer {
            id: Default::default(),
            email: "customer@test.de".to_string(),
            email_verified: false,
            encryption_mode: EncryptionModes::Argon2,
            password: "password123".to_string(),
            full_name: "Test Customer".to_string(),
            created_at: Default::default(),
            updated_at: None,
        }).await.unwrap();

        let orders = shopster.orders(tenant.id).unwrap();
        let customer_id = customer.id;

        let order_with_customer = Order {
            customer_id: Some(customer_id),
            ..make_order(OrderStatus::New)
        };
        let order_no_customer = make_order(OrderStatus::New);

        orders.insert(&order_with_customer).await.unwrap();
        orders.insert(&order_no_customer).await.unwrap();

        let customer_orders = orders.get_by_customer_id(customer_id).await.unwrap();
        assert_eq!(1, customer_orders.len());
        assert_eq!(Some(customer_id), customer_orders[0].customer_id);

        let anonymous_orders = orders.get_without_customer_id().await.unwrap();
        assert_eq!(1, anonymous_orders.len());
        assert!(anonymous_orders[0].customer_id.is_none());
    }).await;
}

#[tokio::test]
async fn order_remove_releases_reservation_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("order_remove_reservation".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product_with_price("ART-ORD-004", "4444444444444", 100)).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 20,
            reserved: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        }).await.unwrap();

        let order_with_item = Order {
            id: 0,
            customer_id: None,
            status: OrderStatus::New,
            delivery_address: "Test Street 1, 12345 Testcity".to_string(),
            billing_address: "Test Street 1, 12345 Testcity".to_string(),
            items: vec![OrderItemSnapshot {
                id: 0,
                product_id: product.id,
                quantity: 4,
                article_number: product.article_number.clone(),
                gtin: product.gtin.clone(),
                title: product.title.clone(),
                short_description: product.short_description.clone(),
                description: product.description.clone(),
                tags: vec![],
                title_image: product.image_url.clone(),
                additional_images: vec![],
                price: OrderItemPrice { amount: 100, currency: "EUR".to_string() },
                weight: product.weight,
            }],
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        };

        let orders = shopster.orders(tenant.id).unwrap();
        let order = orders.insert(&order_with_item).await.unwrap();

        let wh_after_insert = warehouse.get_by_product_id(product.id).await.unwrap();
        assert_eq!(4, wh_after_insert.reserved);

        let removed = orders.remove(order.id).await.unwrap();
        assert!(removed);

        let wh_after_remove = warehouse.get_by_product_id(product.id).await.unwrap();
        assert_eq!(0, wh_after_remove.reserved, "Reservation should be released when reserving order is deleted");
    }).await;
}


/// Test successful conversions from valid i32 values to DbOrderStatus
#[test]
fn test_valid_order_status_conversions() {
    // Test all valid status values
    assert_eq!(DbOrderStatus::try_from(0).unwrap(), DbOrderStatus::New);
    assert_eq!(DbOrderStatus::try_from(1).unwrap(), DbOrderStatus::InProgress);
    assert_eq!(DbOrderStatus::try_from(2).unwrap(), DbOrderStatus::ReadyToShip);
    assert_eq!(DbOrderStatus::try_from(3).unwrap(), DbOrderStatus::Shipping);
    assert_eq!(DbOrderStatus::try_from(4).unwrap(), DbOrderStatus::Done);
}

/// Test that invalid i32 values return errors instead of panicking
#[test]
fn test_invalid_order_status_conversions() {
    // Test negative values
    assert!(DbOrderStatus::try_from(-1).is_err());
    assert!(DbOrderStatus::try_from(-100).is_err());

    // Test out-of-range positive values
    assert!(DbOrderStatus::try_from(5).is_err());
    assert!(DbOrderStatus::try_from(10).is_err());
    assert!(DbOrderStatus::try_from(100).is_err());
    assert!(DbOrderStatus::try_from(i32::MAX).is_err());
    assert!(DbOrderStatus::try_from(i32::MIN).is_err());
}

/// Test that error messages are informative
#[test]
fn test_order_status_error_messages() {
    let invalid_values = vec![-1, 5, 10, 99];

    for val in invalid_values {
        let result = DbOrderStatus::try_from(val);
        assert!(result.is_err(), "Expected error for value: {}", val);

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains(&val.to_string()),
            "Error message should contain the invalid value: {}",
            val
        );
        assert!(
            error_msg.contains("Unknown order status"),
            "Error message should describe the issue"
        );
    }
}

/// Test round-trip conversion: DbOrderStatus -> i32 -> DbOrderStatus
#[test]
fn test_round_trip_conversion() {
    let statuses = vec![
        DbOrderStatus::New,
        DbOrderStatus::InProgress,
        DbOrderStatus::ReadyToShip,
        DbOrderStatus::Shipping,
        DbOrderStatus::Done,
    ];

    for original_status in statuses {
        // Convert to i32
        let status_ref = &original_status;
        let as_i32 = i32::from(status_ref);

        // Convert back to DbOrderStatus
        let converted_back = DbOrderStatus::try_from(as_i32)
            .expect("Should successfully convert valid i32 back to DbOrderStatus");

        // Verify we got the same status
        assert_eq!(
            converted_back, original_status,
            "Round-trip conversion failed for status: {:?}",
            original_status
        );
    }
}

/// Test that the conversion is safe and doesn't panic even with extreme values
#[test]
fn test_extreme_values_dont_panic() {
    // These should not panic, they should return errors
    let extreme_values = vec![
        i32::MIN,
        i32::MIN + 1,
        -1000000,
        -1,
        5,
        1000,
        i32::MAX - 1,
        i32::MAX,
    ];

    for val in extreme_values {
        let result = DbOrderStatus::try_from(val);
        // All of these should produce an error, not a panic
        assert!(
            result.is_err(),
            "Should return error for extreme value: {}",
            val
        );
    }
}

/// Verify that valid statuses match expected numeric values
#[test]
fn test_status_numeric_mapping() {
    let mappings = vec![
        (0, DbOrderStatus::New),
        (1, DbOrderStatus::InProgress),
        (2, DbOrderStatus::ReadyToShip),
        (3, DbOrderStatus::Shipping),
        (4, DbOrderStatus::Done),
    ];

    for (expected_num, status) in mappings {
        let num_from_status = i32::from(&status);
        assert_eq!(
            num_from_status, expected_num,
            "Status {:?} should map to {}",
            status, expected_num
        );

        let status_from_num = DbOrderStatus::try_from(expected_num)
            .expect("Valid status number should convert");
        assert_eq!(status, status_from_num, "Round-trip failed for value: {}", expected_num);
    }
}


