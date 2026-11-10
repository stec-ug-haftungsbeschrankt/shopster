mod common;

use chrono::Utc;
use stec_tenet::{Storage, Tenet};
use stec_shopster::{DatabaseSelector, Shopster};
use stec_shopster::products::{Price, Product};
use stec_shopster::warehouse::WarehouseItem;
use crate::common::test_harness;

fn make_product(article_number: &str, gtin: &str) -> Product {
    Product {
        id: 0,
        article_number: article_number.to_string(),
        title: "Warehouse Test Product".to_string(),
        gtin: gtin.to_string(),
        short_description: "Short".to_string(),
        description: "Description".to_string(),
        image_url: "/images/test.png".to_string(),
        additional_images: Vec::new(),
        price: Some(Price { amount: 100, currency: "EUR".to_string() }),
        weight: 500,
        tags: Vec::new(),
        created_at: Utc::now().naive_utc(),
        updated_at: None,
    }
}

#[tokio::test]
async fn warehouse_get_all_empty_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_get_all_empty".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        let items = warehouse.get_all().await.unwrap();

        assert_eq!(0, items.len());
    }).await;
}

#[tokio::test]
async fn warehouse_insert_and_get_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_insert_get".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product("ART-WH-001", "1234567890123")).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        let new_item = WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 50,
            reserved: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
            low_stock_threshold: None,
        };
        let created = warehouse.insert(&new_item).await.unwrap();

        assert_eq!(product.id, created.product_id);
        assert_eq!(50, created.in_stock);
        assert_eq!(0, created.reserved);
        assert_eq!(50, created.available());

        let all = warehouse.get_all().await.unwrap();
        assert_eq!(1, all.len());
    }).await;
}

#[tokio::test]
async fn warehouse_get_by_product_id_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_get_by_product".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product("ART-WH-002", "2345678901234")).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 20,
            reserved: 5,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
            low_stock_threshold: None,
        }).await.unwrap();

        let item = warehouse.get_by_product_id(product.id).await.unwrap();
        assert_eq!(product.id, item.product_id);
        assert_eq!(20, item.in_stock);
        assert_eq!(5, item.reserved);
        assert_eq!(15, item.available());
    }).await;
}

#[tokio::test]
async fn warehouse_apply_reserved_delta_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_reserved_delta".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product("ART-WH-003", "3456789012345")).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 100,
            reserved: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
            low_stock_threshold: None,
        }).await.unwrap();

        // Reserve 10 units
        let after_reserve = warehouse.apply_reserved_delta(product.id, 10).await.unwrap();
        assert_eq!(10, after_reserve.reserved);
        assert_eq!(90, after_reserve.available());

        // Reserve 5 more
        let after_reserve2 = warehouse.apply_reserved_delta(product.id, 5).await.unwrap();
        assert_eq!(15, after_reserve2.reserved);
        assert_eq!(85, after_reserve2.available());

        // Release 8 units
        let after_release = warehouse.apply_reserved_delta(product.id, -8).await.unwrap();
        assert_eq!(7, after_release.reserved);
        assert_eq!(93, after_release.available());
    }).await;
}

#[tokio::test]
async fn warehouse_update_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_update".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product("ART-WH-004", "4567890123456")).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        let created = warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 30,
            reserved: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
            low_stock_threshold: None,
        }).await.unwrap();

        let updated_item = WarehouseItem {
            in_stock: 60,
            ..created
        };
        let updated = warehouse.update_by_product_id(product.id, &updated_item).await.unwrap();

        assert_eq!(60, updated.in_stock);
        assert_eq!(0, updated.reserved);
    }).await;
}

#[tokio::test]
async fn warehouse_remove_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_remove".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product("ART-WH-005", "5678901234567")).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 10,
            reserved: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
            low_stock_threshold: None,
        }).await.unwrap();

        let removed = warehouse.remove_by_product_id(product.id).await.unwrap();
        assert!(removed);

        let all = warehouse.get_all().await.unwrap();
        assert_eq!(0, all.len());
    }).await;
}

#[tokio::test]
async fn warehouse_get_all_with_details_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_get_all_details".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let product = products.insert(&make_product("ART-WH-006", "6789012345678")).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();
        warehouse.insert(&WarehouseItem {
            id: 0,
            product_id: product.id,
            in_stock: 25,
            reserved: 5,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
            low_stock_threshold: None,
        }).await.unwrap();

        let details = warehouse.get_all_with_details().await.unwrap();
        assert_eq!(1, details.len());

        let detail = &details[0];
        assert_eq!(product.id, detail.product_id);
        assert_eq!("ART-WH-006", detail.article_number);
        assert_eq!("Warehouse Test Product", detail.title);
        assert_eq!(25, detail.in_stock);
        assert_eq!(5, detail.reserved);
        assert_eq!(20, detail.available);
    }).await;
}

#[tokio::test]
async fn warehouse_get_low_stock_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_low_stock".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let p_low = products.insert(&make_product("ART-LS-001", "1000000000001")).await.unwrap();
        let p_ok = products.insert(&make_product("ART-LS-002", "1000000000002")).await.unwrap();
        let p_custom = products.insert(&make_product("ART-LS-003", "1000000000003")).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();

        // p_low: 3 available, default threshold is 10 → should appear
        warehouse.insert(&WarehouseItem {
            id: 0, product_id: p_low.id,
            in_stock: 5, reserved: 2,
            created_at: Utc::now().naive_utc(), updated_at: None,
            low_stock_threshold: None,
        }).await.unwrap();

        // p_ok: 15 available, default threshold is 10 → should NOT appear
        warehouse.insert(&WarehouseItem {
            id: 0, product_id: p_ok.id,
            in_stock: 20, reserved: 5,
            created_at: Utc::now().naive_utc(), updated_at: None,
            low_stock_threshold: None,
        }).await.unwrap();

        // p_custom: 8 available, custom threshold 5 → should NOT appear
        warehouse.insert(&WarehouseItem {
            id: 0, product_id: p_custom.id,
            in_stock: 10, reserved: 2,
            created_at: Utc::now().naive_utc(), updated_at: None,
            low_stock_threshold: Some(5),
        }).await.unwrap();

        let low_stock = warehouse.get_low_stock(10).await.unwrap();
        assert_eq!(1, low_stock.len());
        assert_eq!(p_low.id, low_stock[0].product_id);
    }).await;
}

#[tokio::test]
async fn warehouse_low_stock_threshold_overrides_default_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("warehouse_low_stock_custom".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let p = products.insert(&make_product("ART-LST-001", "2000000000001")).await.unwrap();

        let warehouse = shopster.warehouse(tenant.id).unwrap();

        // available = 8, custom threshold = 20 → low stock even though default threshold would be 10
        warehouse.insert(&WarehouseItem {
            id: 0, product_id: p.id,
            in_stock: 10, reserved: 2,
            created_at: Utc::now().naive_utc(), updated_at: None,
            low_stock_threshold: Some(20),
        }).await.unwrap();

        let low = warehouse.get_low_stock(5).await.unwrap();
        assert_eq!(1, low.len());
        assert_eq!(p.id, low[0].product_id);

        let not_low = warehouse.get_low_stock(5).await.unwrap();
        assert_eq!(1, not_low.len()); // still 1, confirming custom threshold beats default
    }).await;
}
