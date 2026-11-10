//! Several `Shopster` instances can live in one process, each with its own selector.

mod common;

use stec_shopster::error::ShopsterError;
use stec_shopster::{DatabaseSelector, Shopster};
use stec_tenet::{Storage, Tenet};

use crate::common::test_harness_two_tenants;

#[tokio::test]
async fn instances_do_not_share_tenants() {
    test_harness_two_tenants(|tenet_cs, shop_cs1, shop_cs2| async move {
        let tenet_a = Tenet::new(tenet_cs.clone());
        let tenant_a = tenet_a.create_tenant("multi_instance_a".to_string()).unwrap();
        tenant_a.add_storage(&Storage::new_postgresql_database(shop_cs1, tenant_a.id)).unwrap();

        let tenet_b = Tenet::new(tenet_cs);
        let tenant_b = tenet_b.create_tenant("multi_instance_b".to_string()).unwrap();
        tenant_b.add_storage(&Storage::new_postgresql_database(shop_cs2, tenant_b.id)).unwrap();

        let a = Shopster::new(DatabaseSelector::new(tenet_a));
        let b = Shopster::new(DatabaseSelector::new(tenet_b));

        // Each instance serves its own tenant ...
        a.coupons(tenant_a.id).unwrap().insert("A10".into(), Some(10), None, None, None, None, None).await.unwrap();
        b.coupons(tenant_b.id).unwrap().insert("B10".into(), Some(10), None, None, None, None, None).await.unwrap();

        assert!(a.coupons(tenant_a.id).unwrap().find_by_code("A10").await.unwrap().is_some());
        assert!(b.coupons(tenant_b.id).unwrap().find_by_code("A10").await.unwrap().is_none());

        // ... and the second instance no longer silently reuses the first one's selector.
        // (Tenet resolves tenants globally here, so use an unknown tenant id to prove the error path.)
        let unknown = uuid::Uuid::new_v4();
        let err = a.coupons(unknown).unwrap().find_by_code("A10").await.unwrap_err();
        assert!(matches!(err, ShopsterError::TenantNotFoundError), "got {:?}", err);
    }).await;
}
