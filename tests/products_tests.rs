mod common;

use chrono::Utc;
use stec_tenet::{Storage, Tenet};
use stec_shopster::{DatabaseSelector, Shopster};
use stec_shopster::products::{Price, Product, ProductSearchParams, ProductSort};
use crate::common::test_harness;

fn make_product(article: &str, gtin: &str, title: &str, tags: Vec<&str>) -> Product {
    Product {
        id: 0,
        article_number: article.to_string(),
        gtin: gtin.to_string(),
        title: title.to_string(),
        short_description: format!("Short description for {}", title),
        description: format!("Full description for {}", title),
        image_url: "/images/test.png".to_string(),
        additional_images: Vec::new(),
        price: Some(Price { amount: 1000, currency: "EUR".to_string() }),
        weight: 500,
        tags: tags.into_iter().map(String::from).collect(),
        created_at: Utc::now().naive_utc(),
        updated_at: None,
    }
}

#[tokio::test]
async fn product_search_by_title_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("product_search_title".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let products = shopster.products(tenant.id).unwrap();

        products.insert(&make_product("ART-S-001", "1111111111111", "Red Widget", vec![])).await.unwrap();
        products.insert(&make_product("ART-S-002", "2222222222222", "Blue Widget", vec![])).await.unwrap();
        products.insert(&make_product("ART-S-003", "3333333333333", "Green Gadget", vec![])).await.unwrap();

        let result = products.search(ProductSearchParams {
            query: Some("Widget".to_string()),
            ..Default::default()
        }).await.unwrap();

        assert_eq!(2, result.total);
        assert_eq!(2, result.items.len());
        assert!(result.items.iter().any(|p| p.title == "Red Widget"));
        assert!(result.items.iter().any(|p| p.title == "Blue Widget"));
    }).await;
}

#[tokio::test]
async fn product_search_by_tag_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("product_search_tag".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let products = shopster.products(tenant.id).unwrap();

        products.insert(&make_product("ART-T-001", "4111111111111", "Tagged A", vec!["electronics", "sale"])).await.unwrap();
        products.insert(&make_product("ART-T-002", "4222222222222", "Tagged B", vec!["electronics"])).await.unwrap();
        products.insert(&make_product("ART-T-003", "4333333333333", "Untagged", vec![])).await.unwrap();

        let electronics = products.search(ProductSearchParams {
            tags: vec!["electronics".to_string()],
            ..Default::default()
        }).await.unwrap();
        assert_eq!(2, electronics.total);

        let sale = products.search(ProductSearchParams {
            tags: vec!["sale".to_string()],
            ..Default::default()
        }).await.unwrap();
        assert_eq!(1, sale.total);
        assert_eq!("Tagged A", sale.items[0].title);
    }).await;
}

#[tokio::test]
async fn product_search_no_query_returns_all_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("product_search_all".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let products = shopster.products(tenant.id).unwrap();

        products.insert(&make_product("ART-A-001", "5111111111111", "Alpha", vec![])).await.unwrap();
        products.insert(&make_product("ART-A-002", "5222222222222", "Beta", vec![])).await.unwrap();
        products.insert(&make_product("ART-A-003", "5333333333333", "Gamma", vec![])).await.unwrap();

        let result = products.search(ProductSearchParams {
            limit: 10,
            ..Default::default()
        }).await.unwrap();
        assert_eq!(3, result.total);
        assert_eq!(3, result.items.len());
    }).await;
}

#[tokio::test]
async fn product_search_pagination_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("product_search_pagination".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let products = shopster.products(tenant.id).unwrap();

        for i in 1..=5 {
            products.insert(&make_product(
                &format!("ART-P-{:03}", i),
                &format!("6{:012}", i),
                &format!("Paged Product {}", i),
                vec![],
            )).await.unwrap();
        }

        let page1 = products.search(ProductSearchParams {
            sort: ProductSort::TitleAsc,
            limit: 2,
            offset: 0,
            ..Default::default()
        }).await.unwrap();
        assert_eq!(5, page1.total);
        assert_eq!(2, page1.items.len());

        let page2 = products.search(ProductSearchParams {
            sort: ProductSort::TitleAsc,
            limit: 2,
            offset: 2,
            ..Default::default()
        }).await.unwrap();
        assert_eq!(5, page2.total);
        assert_eq!(2, page2.items.len());
        assert_ne!(page1.items[0].id, page2.items[0].id);
    }).await;
}
