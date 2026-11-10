mod common;

use chrono::{Duration, Utc};
use stec_tenet::{Storage, Tenet};
use stec_shopster::{DatabaseSelector, Shopster};
use crate::common::test_harness;

#[tokio::test]
async fn coupon_insert_and_find_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("coupon_insert_find".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let coupons = shopster.coupons(tenant.id).unwrap();

        let coupon = coupons.insert(
            "SAVE10".to_string(),
            Some(10),
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();

        assert_eq!("SAVE10", coupon.code);
        assert_eq!(Some(10), coupon.percent_off);
        assert_eq!(None, coupon.amount_off);
        assert_eq!(0, coupon.use_count);

        let found = coupons.find_by_code("SAVE10").await.unwrap();
        assert!(found.is_some());
        assert_eq!(coupon.id, found.unwrap().id);

        let missing = coupons.find_by_code("DOESNOTEXIST").await.unwrap();
        assert!(missing.is_none());
    }).await;
}

#[tokio::test]
async fn coupon_get_all_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("coupon_get_all".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let coupons = shopster.coupons(tenant.id).unwrap();

        let empty = coupons.get_all().await.unwrap();
        assert_eq!(0, empty.len());

        coupons.insert("CODE1".to_string(), Some(5), None, None, None, None, None).await.unwrap();
        coupons.insert("CODE2".to_string(), None, Some(500), Some("EUR".to_string()), None, None, None).await.unwrap();

        let all = coupons.get_all().await.unwrap();
        assert_eq!(2, all.len());
    }).await;
}

#[tokio::test]
async fn coupon_validate_valid_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("coupon_validate_valid".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let coupons = shopster.coupons(tenant.id).unwrap();

        let past = Utc::now().naive_utc() - Duration::days(1);
        let future = Utc::now().naive_utc() + Duration::days(30);

        coupons.insert("VALID".to_string(), Some(15), None, None, Some(past), Some(future), Some(100)).await.unwrap();

        let result = coupons.validate("VALID").await.unwrap();
        assert!(result.is_some());
        assert_eq!("VALID", result.unwrap().code);
    }).await;
}

#[tokio::test]
async fn coupon_validate_expired_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("coupon_validate_expired".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let coupons = shopster.coupons(tenant.id).unwrap();

        let past_start = Utc::now().naive_utc() - Duration::days(10);
        let past_end = Utc::now().naive_utc() - Duration::days(1);

        coupons.insert("EXPIRED".to_string(), Some(10), None, None, Some(past_start), Some(past_end), None).await.unwrap();

        let result = coupons.validate("EXPIRED").await.unwrap();
        assert!(result.is_none(), "Expired coupon should not validate");
    }).await;
}

#[tokio::test]
async fn coupon_validate_max_uses_reached_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("coupon_validate_maxuses".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let coupons = shopster.coupons(tenant.id).unwrap();

        let coupon = coupons.insert("ONCE".to_string(), Some(50), None, None, None, None, Some(1)).await.unwrap();
        coupons.increment_use_count(coupon.id).await.unwrap();

        let result = coupons.validate("ONCE").await.unwrap();
        assert!(result.is_none(), "Exhausted coupon should not validate");
    }).await;
}

#[tokio::test]
async fn coupon_increment_use_count_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("coupon_use_count".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let coupons = shopster.coupons(tenant.id).unwrap();

        let coupon = coupons.insert("MULTI".to_string(), Some(5), None, None, None, None, Some(10)).await.unwrap();
        assert_eq!(0, coupon.use_count);

        let updated = coupons.increment_use_count(coupon.id).await.unwrap();
        assert_eq!(1, updated.use_count);

        let updated2 = coupons.increment_use_count(coupon.id).await.unwrap();
        assert_eq!(2, updated2.use_count);
    }).await;
}

#[tokio::test]
async fn coupon_delete_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);
        let tenant = tenet.create_tenant("coupon_delete".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);
        let coupons = shopster.coupons(tenant.id).unwrap();

        let coupon = coupons.insert("DEL".to_string(), Some(1), None, None, None, None, None).await.unwrap();
        let deleted = coupons.delete(coupon.id).await.unwrap();
        assert!(deleted);

        let found = coupons.find_by_code("DEL").await.unwrap();
        assert!(found.is_none());

        let deleted_again = coupons.delete(coupon.id).await.unwrap();
        assert!(!deleted_again);
    }).await;
}
