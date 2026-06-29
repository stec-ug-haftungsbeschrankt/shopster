mod common;

use chrono::Utc;
use stec_tenet::{Storage, Tenet};
use uuid::Uuid;
use stec_shopster::{DatabaseSelector, Shopster};
use stec_shopster::products::{Price, Product};
use crate::common::test_harness;

#[tokio::test]
async fn basket_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("basket_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let new_product = Product {
            id: 0,
            article_number: "ART-11111".to_string(),
            title: "Test Product".to_string(),
            gtin: "1234567890".to_string(),
            short_description: "Short Description".to_string(),
            description: "Description".to_string(),
            image_url: "/images/ART-1111/IMG_1101.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price {
                amount: 129,
                currency: "EUR".to_string()
            }),
            weight: 88,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };
        let product = products.insert(&new_product).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();
        let _basket = baskets.get_basket(basket_id).await.unwrap();

        baskets.add_product_to_basket(basket_id, product.id, 2).await.unwrap();
        let products = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(1, products.len());
    }).await;
}

#[tokio::test]
async fn update_product_quantity_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("update_product_quantity_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let new_product = Product {
            id: 0,
            article_number: "ART-22222".to_string(),
            title: "Test Product for Quantity Update".to_string(),
            gtin: "9876543210".to_string(),
            short_description: "Short Description".to_string(),
            description: "Description".to_string(),
            image_url: "/images/ART-2222/IMG_2202.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price {
                amount: 199,
                currency: "EUR".to_string()
            }),
            weight: 100,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };
        let product = products.insert(&new_product).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();

        let basket_product_id = baskets.add_product_to_basket(basket_id, product.id, 2).await.unwrap();

        let updated_product = baskets.update_product_quantity(basket_id, basket_product_id, 5).await.unwrap();
        assert_eq!(5, updated_product.quantity);

        let basket_products = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(1, basket_products.len());
        assert_eq!(5, basket_products[0].quantity);
    }).await;
}

#[tokio::test]
async fn remove_product_from_basket_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("remove_product_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let new_product = Product {
            id: 0,
            article_number: "ART-33333".to_string(),
            title: "Test Product for Removal".to_string(),
            gtin: "5432109876".to_string(),
            short_description: "Short Description".to_string(),
            description: "Description".to_string(),
            image_url: "/images/ART-3333/IMG_3303.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price {
                amount: 299,
                currency: "EUR".to_string()
            }),
            weight: 150,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };
        let product = products.insert(&new_product).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();

        let basket_product_id = baskets.add_product_to_basket(basket_id, product.id, 3).await.unwrap();

        let removal_success = baskets.remove_product_from_basket(basket_id, basket_product_id).await.unwrap();
        assert_eq!(true, removal_success);

        let basket_products = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(0, basket_products.len());
    }).await;
}

#[tokio::test]
async fn get_all_baskets_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("get_all_baskets_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let baskets = shopster.baskets(tenant.id).unwrap();

        let initial_baskets = baskets.get_all_baskets().await.unwrap();
        assert_eq!(0, initial_baskets.len());

        let basket_id1 = baskets.add_basket().await.unwrap();
        let basket_id2 = baskets.add_basket().await.unwrap();
        let basket_id3 = baskets.add_basket().await.unwrap();

        let all_baskets = baskets.get_all_baskets().await.unwrap();
        assert_eq!(3, all_baskets.len());

        let basket_ids: Vec<Uuid> = all_baskets.iter().map(|b| b.id).collect();
        assert!(basket_ids.contains(&basket_id1));
        assert!(basket_ids.contains(&basket_id2));
        assert!(basket_ids.contains(&basket_id3));
    }).await;
}

#[tokio::test]
async fn get_products_with_details_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("product_details_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let new_product = Product {
            id: 0,
            article_number: "ART-44444".to_string(),
            title: "Test Product with Details".to_string(),
            gtin: "1122334455".to_string(),
            short_description: "Product Details Test".to_string(),
            description: "Full Description for Testing".to_string(),
            image_url: "/images/ART-4444/IMG_4404.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price {
                amount: 499,
                currency: "EUR".to_string()
            }),
            weight: 200,
            tags: vec!["test".to_string(), "details".to_string()],
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };
        let product = products.insert(&new_product).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();

        baskets.add_product_to_basket(basket_id, product.id, 2).await.unwrap();

        let products_with_details = baskets.get_products_with_details(basket_id).await.unwrap();

        assert_eq!(1, products_with_details.len());
        let product_with_details = &products_with_details[0];
        assert_eq!(2, product_with_details.quantity);
        assert_eq!("Test Product with Details", product_with_details.product.title);
        assert_eq!("ART-44444", product_with_details.product.article_number);
        assert_eq!(499, product_with_details.product.price.as_ref().unwrap().amount);
        assert_eq!("EUR", product_with_details.product.price.as_ref().unwrap().currency);
    }).await;
}

#[tokio::test]
async fn calculate_basket_total_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("basket_total_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();

        let product1 = Product {
            id: 0,
            article_number: "ART-55555".to_string(),
            title: "Product 1".to_string(),
            gtin: "9988776655".to_string(),
            short_description: "Product 1 Description".to_string(),
            description: "Full Description for Product 1".to_string(),
            image_url: "/images/ART-5555/IMG_5505.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price {
                amount: 100,
                currency: "EUR".to_string()
            }),
            weight: 100,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };

        let product2 = Product {
            id: 0,
            article_number: "ART-66666".to_string(),
            title: "Product 2".to_string(),
            gtin: "5566778899".to_string(),
            short_description: "Product 2 Description".to_string(),
            description: "Full Description for Product 2".to_string(),
            image_url: "/images/ART-6666/IMG_6606.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price {
                amount: 200,
                currency: "EUR".to_string()
            }),
            weight: 200,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };

        let product1 = products.insert(&product1).await.unwrap();
        let product2 = products.insert(&product2).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();

        baskets.add_product_to_basket(basket_id, product1.id, 3).await.unwrap();
        baskets.add_product_to_basket(basket_id, product2.id, 2).await.unwrap();

        let (total, currency) = baskets.calculate_basket_total(basket_id).await.unwrap();

        assert_eq!(700, total);
        assert_eq!("EUR", currency);
    }).await;
}

#[tokio::test]
async fn merge_baskets_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("merge_baskets_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();

        let product1 = Product {
            id: 0,
            article_number: "ART-77777".to_string(),
            title: "Product A".to_string(),
            gtin: "1111222233".to_string(),
            short_description: "Product A Description".to_string(),
            description: "Full Description for Product A".to_string(),
            image_url: "/images/ART-7777/IMG_7707.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price { amount: 100, currency: "EUR".to_string() }),
            weight: 100,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };

        let product2 = Product {
            id: 0,
            article_number: "ART-88888".to_string(),
            title: "Product B".to_string(),
            gtin: "4444555566".to_string(),
            short_description: "Product B Description".to_string(),
            description: "Full Description for Product B".to_string(),
            image_url: "/images/ART-8888/IMG_8808.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price { amount: 200, currency: "EUR".to_string() }),
            weight: 200,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };

        let product3 = Product {
            id: 0,
            article_number: "ART-99999".to_string(),
            title: "Product C".to_string(),
            gtin: "7777888899".to_string(),
            short_description: "Product C Description".to_string(),
            description: "Full Description for Product C".to_string(),
            image_url: "/images/ART-9999/IMG_9909.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price { amount: 300, currency: "EUR".to_string() }),
            weight: 300,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };

        let product1 = products.insert(&product1).await.unwrap();
        let product2 = products.insert(&product2).await.unwrap();
        let product3 = products.insert(&product3).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let source_basket_id = baskets.add_basket().await.unwrap();
        let target_basket_id = baskets.add_basket().await.unwrap();

        baskets.add_product_to_basket(source_basket_id, product1.id, 2).await.unwrap();
        baskets.add_product_to_basket(source_basket_id, product2.id, 1).await.unwrap();

        baskets.add_product_to_basket(target_basket_id, product1.id, 1).await.unwrap();
        baskets.add_product_to_basket(target_basket_id, product3.id, 3).await.unwrap();

        baskets.merge_baskets(source_basket_id, target_basket_id).await.unwrap();

        let source_basket_result = baskets.get_basket(source_basket_id).await;
        assert!(source_basket_result.is_err());

        let target_products = baskets.get_products_from_basket(target_basket_id).await.unwrap();

        let mut sorted_products = target_products.clone();
        sorted_products.sort_by_key(|p| p.product_id);

        assert_eq!(3, sorted_products.len());

        assert_eq!(product1.id, sorted_products[0].product_id);
        assert_eq!(3, sorted_products[0].quantity);

        assert_eq!(product2.id, sorted_products[1].product_id);
        assert_eq!(1, sorted_products[1].quantity);

        assert_eq!(product3.id, sorted_products[2].product_id);
        assert_eq!(3, sorted_products[2].quantity);
    }).await;
}

#[tokio::test]
async fn delete_basket_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("delete_basket_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let baskets = shopster.baskets(tenant.id).unwrap();

        let basket_id = baskets.add_basket().await.unwrap();

        let basket = baskets.get_basket(basket_id).await;
        assert!(basket.is_ok());

        let delete_result = baskets.delete_basket(basket_id).await.unwrap();
        assert_eq!(true, delete_result);

        let deleted_basket = baskets.get_basket(basket_id).await;
        assert!(deleted_basket.is_err());
    }).await;
}

#[tokio::test]
async fn clear_basket_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("clear_basket_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let new_product = Product {
            id: 0,
            article_number: "ART-CLEAR".to_string(),
            title: "Test Product for Clear".to_string(),
            gtin: "9876543210".to_string(),
            short_description: "Short Description".to_string(),
            description: "Description".to_string(),
            image_url: "/images/ART-CLEAR/IMG_CLEAR.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price { amount: 199, currency: "EUR".to_string() }),
            weight: 100,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };
        let product = products.insert(&new_product).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();

        baskets.add_product_to_basket(basket_id, product.id, 3).await.unwrap();

        let products_before = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(1, products_before.len());

        let clear_result = baskets.clear_basket(basket_id).await.unwrap();
        assert_eq!(true, clear_result);

        let products_after = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(0, products_after.len());

        let basket = baskets.get_basket(basket_id).await;
        assert!(basket.is_ok());
    }).await;
}

#[tokio::test]
async fn add_existing_product_to_basket_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("add_existing_product_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let new_product = Product {
            id: 0,
            article_number: "ART-EXISTING".to_string(),
            title: "Test Product for Existing".to_string(),
            gtin: "1122334455".to_string(),
            short_description: "Short Description".to_string(),
            description: "Description".to_string(),
            image_url: "/images/ART-EXISTING/IMG_EXIST.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price { amount: 299, currency: "EUR".to_string() }),
            weight: 150,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        };
        let product = products.insert(&new_product).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();

        let basket_product_id1 = baskets.add_product_to_basket(basket_id, product.id, 2).await.unwrap();

        let products_before = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(1, products_before.len());
        assert_eq!(2, products_before[0].quantity);

        let basket_product_id2 = baskets.add_product_to_basket(basket_id, product.id, 3).await.unwrap();

        assert_eq!(basket_product_id1, basket_product_id2);

        let products_after = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(1, products_after.len());
        assert_eq!(3, products_after[0].quantity);
    }).await;
}

#[tokio::test]
async fn non_existent_basket_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("non_existent_basket_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let baskets = shopster.baskets(tenant.id).unwrap();

        let non_existent_id = Uuid::new_v4();

        let get_result = baskets.get_basket(non_existent_id).await;
        assert!(get_result.is_err());

        let add_result = baskets.add_product_to_basket(non_existent_id, 1, 1).await;
        assert!(add_result.is_err());

        let clear_result = baskets.clear_basket(non_existent_id).await;
        assert!(clear_result.is_err());

        let delete_result = baskets.delete_basket(non_existent_id).await;
        assert!(delete_result.is_err());
    }).await;
}

#[tokio::test]
async fn multiple_products_in_basket_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("multiple_products_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();

        let product1 = products.insert(&Product {
            id: 0,
            article_number: "ART-MULTI-1".to_string(),
            title: "Product Multi 1".to_string(),
            gtin: "1111222233".to_string(),
            short_description: "Multi Product 1".to_string(),
            description: "Description 1".to_string(),
            image_url: "/images/ART-MULTI-1/IMG_M1.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price { amount: 100, currency: "EUR".to_string() }),
            weight: 100,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        }).await.unwrap();

        let product2 = products.insert(&Product {
            id: 0,
            article_number: "ART-MULTI-2".to_string(),
            title: "Product Multi 2".to_string(),
            gtin: "4444555566".to_string(),
            short_description: "Multi Product 2".to_string(),
            description: "Description 2".to_string(),
            image_url: "/images/ART-MULTI-2/IMG_M2.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price { amount: 200, currency: "EUR".to_string() }),
            weight: 200,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        }).await.unwrap();

        let product3 = products.insert(&Product {
            id: 0,
            article_number: "ART-MULTI-3".to_string(),
            title: "Product Multi 3".to_string(),
            gtin: "7777888899".to_string(),
            short_description: "Multi Product 3".to_string(),
            description: "Description 3".to_string(),
            image_url: "/images/ART-MULTI-3/IMG_M3.png".to_string(),
            additional_images: Vec::new(),
            price: Some(Price { amount: 300, currency: "EUR".to_string() }),
            weight: 300,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc().to_owned(),
            updated_at: None
        }).await.unwrap();

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();

        baskets.add_product_to_basket(basket_id, product1.id, 2).await.unwrap();
        baskets.add_product_to_basket(basket_id, product2.id, 1).await.unwrap();
        baskets.add_product_to_basket(basket_id, product3.id, 3).await.unwrap();

        let basket_products = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(3, basket_products.len());

        let mut sorted_products = basket_products.clone();
        sorted_products.sort_by_key(|p| p.product_id);

        assert_eq!(product1.id, sorted_products[0].product_id);
        assert_eq!(2, sorted_products[0].quantity);

        assert_eq!(product2.id, sorted_products[1].product_id);
        assert_eq!(1, sorted_products[1].quantity);

        assert_eq!(product3.id, sorted_products[2].product_id);
        assert_eq!(3, sorted_products[2].quantity);

        let remove_result = baskets.remove_product_from_basket(basket_id, sorted_products[1].id).await.unwrap();
        assert_eq!(true, remove_result);

        let updated_products = baskets.get_products_from_basket(basket_id).await.unwrap();
        assert_eq!(2, updated_products.len());

        let product_ids: Vec<i64> = updated_products.iter().map(|p| p.product_id).collect();
        assert!(product_ids.contains(&product1.id));
        assert!(!product_ids.contains(&product2.id));
        assert!(product_ids.contains(&product3.id));
    }).await;
}

#[tokio::test]
async fn empty_basket_total_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("empty_basket_total_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let baskets = shopster.baskets(tenant.id).unwrap();
        let basket_id = baskets.add_basket().await.unwrap();

        let (total, currency) = baskets.calculate_basket_total(basket_id).await.unwrap();

        assert_eq!(0, total, "Empty basket total should be 0");
        assert_eq!("EUR", currency, "Empty basket total should default to EUR");
    }).await;
}

#[tokio::test]
async fn insert_product_without_price_is_rejected_test() {
    // Products require a price at the DB layer (DbProduct::try_from enforces this),
    // so a priceless product can never enter a basket. This test documents that guarantee.
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("basket_total_no_price_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let products = shopster.products(tenant.id).unwrap();
        let result = products.insert(&Product {
            id: 0,
            article_number: "ART-NOPRICE".to_string(),
            title: "Product Without Price".to_string(),
            gtin: "0000000000001".to_string(),
            short_description: "No price product".to_string(),
            description: "This product has no price".to_string(),
            image_url: "/images/noprice.png".to_string(),
            additional_images: Vec::new(),
            price: None,
            weight: 100,
            tags: Vec::new(),
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        }).await;

        assert!(result.is_err(), "Inserting a product without a price should be rejected");
    }).await;
}
