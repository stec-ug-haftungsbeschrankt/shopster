//! Settings tests, focused on secret settings (encrypted at rest).
//!
//! Every test in this binary registers a `DatabaseSelector` that carries the same secret
//! key: `Shopster::new` keeps only the first selector per process, so mixing keyed and
//! un-keyed selectors in one binary would be order dependent.

mod common;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use stec_shopster::error::ShopsterError;
use stec_shopster::secrets::SecretKey;
use stec_shopster::{DatabaseSelector, Shopster};
use stec_tenet::{Storage, Tenet};
use uuid::Uuid;

use crate::common::test_harness;

#[derive(QueryableByName)]
struct RawValue {
    #[diesel(sql_type = Text)]
    value: String,
}

fn raw_value(connection_string: &str, title: &str) -> String {
    let mut conn = PgConnection::establish(connection_string).unwrap();
    sql_query("SELECT value FROM settings WHERE title = $1")
        .bind::<Text, _>(title)
        .get_result::<RawValue>(&mut conn)
        .unwrap()
        .value
}

fn set_raw_value(connection_string: &str, title: &str, value: &str) {
    let mut conn = PgConnection::establish(connection_string).unwrap();
    sql_query("UPDATE settings SET value = $1 WHERE title = $2")
        .bind::<Text, _>(value)
        .bind::<Text, _>(title)
        .execute(&mut conn)
        .unwrap();
}

fn shopster_with_key(tenet: Tenet) -> Shopster {
    let key = SecretKey::from_bytes([42u8; 32]);
    Shopster::new(DatabaseSelector::new(tenet).with_secret_key(key))
}

fn new_tenant(tenet: &Tenet, name: &str, connection_string: String) -> Uuid {
    let tenant = tenet.create_tenant(name.to_string()).unwrap();
    let storage = Storage::new_postgresql_database(connection_string, tenant.id);
    tenant.add_storage(&storage).unwrap();
    tenant.id
}

#[tokio::test]
async fn get_all_lists_only_non_secret_settings() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant_id = new_tenant(&tenet, "settings_get_all_secrets", shopster_cs);
        let shopster = shopster_with_key(tenet);

        let settings = shopster.settings(tenant_id).unwrap().get_all().await.unwrap();

        assert_eq!(11, settings.len());
        for hidden in ["stripe_secret_api_key", "stripe_webhook_secret", "sendgrid_api_key"] {
            assert!(settings.iter().all(|s| s.title != hidden), "{hidden} must not be listed");
        }
        assert!(settings.iter().all(|s| !s.is_secret));
    }).await;
}

#[tokio::test]
async fn secret_round_trip_is_encrypted_at_rest() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant_id = new_tenant(&tenet, "settings_secret_round_trip", shopster_cs.clone());
        let shopster = shopster_with_key(tenet);
        let settings = shopster.settings(tenant_id).unwrap();

        assert_eq!("", settings.get_secret("stripe_webhook_secret").await.unwrap());

        settings.set_secret("stripe_webhook_secret", "whsec_super_secret").await.unwrap();

        assert_eq!("whsec_super_secret", settings.get_secret("stripe_webhook_secret").await.unwrap());

        let stored = raw_value(&shopster_cs, "stripe_webhook_secret");
        assert!(stored.starts_with("enc:v1:"), "stored value: {stored}");
        assert!(!stored.contains("whsec_super_secret"));

        // Overwriting works and does not create a second row.
        settings.set_secret("stripe_webhook_secret", "whsec_rotated").await.unwrap();
        assert_eq!("whsec_rotated", settings.get_secret("stripe_webhook_secret").await.unwrap());
    }).await;
}

#[tokio::test]
async fn secret_value_is_redacted_in_regular_getters() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant_id = new_tenant(&tenet, "settings_secret_redacted", shopster_cs);
        let shopster = shopster_with_key(tenet);
        let settings = shopster.settings(tenant_id).unwrap();

        settings.set_secret("stripe_secret_api_key", "sk_live_abc").await.unwrap();

        let by_title = settings.get_by_title("stripe_secret_api_key".to_string()).await.unwrap();
        assert!(by_title.is_secret);
        assert_eq!("", by_title.value);
        assert!(!format!("{:?}", by_title).contains("sk_live_abc"));

        let by_id = settings.get_by_id(by_title.id).await.unwrap();
        assert_eq!("", by_id.value);
    }).await;
}

#[tokio::test]
async fn update_by_id_refuses_secret_settings() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant_id = new_tenant(&tenet, "settings_update_refuses_secret", shopster_cs.clone());
        let shopster = shopster_with_key(tenet);
        let settings = shopster.settings(tenant_id).unwrap();

        let secret = settings.get_by_title("sendgrid_api_key".to_string()).await.unwrap();
        let result = settings.update_by_id(secret.id, "SG.plaintext".to_string()).await;

        assert!(matches!(result, Err(ShopsterError::InvalidOperationError(_))));
        assert_eq!("", raw_value(&shopster_cs, "sendgrid_api_key"));
    }).await;
}

#[tokio::test]
async fn secret_api_refuses_non_secret_settings_and_creates_new_secrets() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant_id = new_tenant(&tenet, "settings_secret_scope", shopster_cs);
        let shopster = shopster_with_key(tenet);
        let settings = shopster.settings(tenant_id).unwrap();

        assert!(matches!(settings.get_secret("shop_title").await, Err(ShopsterError::InvalidOperationError(_))));
        assert!(matches!(settings.set_secret("shop_title", "x").await, Err(ShopsterError::InvalidOperationError(_))));
        assert_eq!("Unnamed Shop", settings.get_by_title("shop_title".to_string()).await.unwrap().value);

        settings.set_secret("custom_token", "abc").await.unwrap();
        let created = settings.get_by_title("custom_token".to_string()).await.unwrap();
        assert!(created.is_secret);
        assert_eq!("abc", settings.get_secret("custom_token").await.unwrap());
    }).await;
}

#[tokio::test]
async fn legacy_plaintext_secrets_can_be_read_and_then_encrypted() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant_id = new_tenant(&tenet, "settings_legacy_secrets", shopster_cs.clone());
        let shopster = shopster_with_key(tenet);
        let settings = shopster.settings(tenant_id).unwrap();

        // Make sure the schema exists, then simulate a value written before encryption existed.
        settings.get_all().await.unwrap();
        set_raw_value(&shopster_cs, "stripe_secret_api_key", "sk_legacy_plain");

        assert_eq!("sk_legacy_plain", settings.get_secret("stripe_secret_api_key").await.unwrap());

        assert_eq!(1, settings.encrypt_legacy_secrets().await.unwrap());
        assert!(raw_value(&shopster_cs, "stripe_secret_api_key").starts_with("enc:v1:"));
        assert_eq!("sk_legacy_plain", settings.get_secret("stripe_secret_api_key").await.unwrap());

        // Idempotent.
        assert_eq!(0, settings.encrypt_legacy_secrets().await.unwrap());
    }).await;
}

#[tokio::test]
async fn a_ciphertext_copied_to_another_setting_does_not_decrypt() {
    test_harness(|tenet_cs, shopster_cs| async move {
        let tenet = Tenet::new(tenet_cs);
        let tenant_id = new_tenant(&tenet, "settings_secret_swap", shopster_cs.clone());
        let shopster = shopster_with_key(tenet);
        let settings = shopster.settings(tenant_id).unwrap();

        settings.set_secret("stripe_secret_api_key", "sk_live_abc").await.unwrap();
        let ciphertext = raw_value(&shopster_cs, "stripe_secret_api_key");
        set_raw_value(&shopster_cs, "sendgrid_api_key", &ciphertext);

        assert!(settings.get_secret("sendgrid_api_key").await.is_err());
    }).await;
}
