mod common;

use stec_tenet::{Storage, Tenet};
use stec_tenet::encryption_modes::EncryptionModes;
use stec_shopster::{DatabaseSelector, Shopster};
use stec_shopster::customers::{Customer, CustomerProfile};
use uuid::Uuid;

use crate::common::{test_harness, test_harness_two_tenants};

#[tokio::test]
async fn customer_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("basket_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();
        let new_customer = Customer {
            id: Default::default(),
            email: "test@stecug.de".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "1234567890".to_string(),
            full_name: "Dummy Testuser".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let customer = customers.insert(&new_customer).await.unwrap();

        let mut all_customers = customers.get_all().await.unwrap();
        assert_eq!(1, all_customers.len());

        let inserted_customer = all_customers.first().unwrap();
        assert_eq!(new_customer.email, inserted_customer.email);
        assert_eq!(new_customer.email_verified, inserted_customer.email_verified);
        assert_eq!(new_customer.encryption_mode, inserted_customer.encryption_mode);
        assert_eq!(new_customer.full_name, inserted_customer.full_name);

        let inserted_customer = all_customers.first().unwrap();
        customers.update(inserted_customer.id, &CustomerProfile {
            email: "dummy@stecug.de".to_string(),
            email_verified: false,
            full_name: inserted_customer.full_name.clone(),
        }).await.unwrap();

        let success = customers.remove(all_customers.first().unwrap().id).await.unwrap();
        assert_eq!(true, success);

        let _ = customer;
    }).await;
}

#[tokio::test]
async fn find_customer_by_email_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("find_by_email_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let test_email = "find_test@example.com".to_string();
        let new_customer = Customer {
            id: Default::default(),
            email: test_email.clone(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "SecurePassword123".to_string(),
            full_name: "Find Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        customers.insert(&new_customer).await.unwrap();

        let found_customer = customers.find_by_email(test_email).await.unwrap();

        assert_eq!(new_customer.email, found_customer.email);
        assert_eq!(new_customer.full_name, found_customer.full_name);
        assert_eq!(new_customer.email_verified, found_customer.email_verified);

        customers.remove(found_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn verify_password_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("verify_password_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let test_password = "CorrectPassword123";
        let new_customer = Customer {
            id: Default::default(),
            email: "password_test@example.com".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: test_password.to_string(),
            full_name: "Password Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();

        let is_valid = customers.verify_password(created_customer.id, test_password).await.unwrap();
        assert_eq!(true, is_valid);

        let is_valid = customers.verify_password(created_customer.id, "WrongPassword123").await.unwrap();
        assert_eq!(false, is_valid);

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn verify_email_password_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("verify_email_password_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let test_email = "email_password_test@example.com";
        let test_password = "SecureEmailPassword123";
        let new_customer = Customer {
            id: Default::default(),
            email: test_email.to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: test_password.to_string(),
            full_name: "Email Password Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        customers.insert(&new_customer).await.unwrap();

        let auth_customer = customers.verify_email_password(test_email.to_string(), test_password).await.unwrap();
        assert_eq!(new_customer.email, auth_customer.email);
        assert_eq!(new_customer.full_name, auth_customer.full_name);

        let auth_result = customers.verify_email_password(test_email.to_string(), "WrongPassword123").await;
        assert!(auth_result.is_err());

        let auth_result = customers.verify_email_password("wrong@example.com".to_string(), test_password).await;
        assert!(auth_result.is_err());

        customers.remove(auth_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn change_password_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("change_password_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let old_password = "OldPassword123";
        let new_password = "NewPassword456";
        let new_customer = Customer {
            id: Default::default(),
            email: "change_password@example.com".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: old_password.to_string(),
            full_name: "Change Password Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();

        let is_valid = customers.verify_password(created_customer.id, old_password).await.unwrap();
        assert_eq!(true, is_valid);

        let change_result = customers.change_password(created_customer.id, old_password, new_password).await.unwrap();
        assert_eq!(true, change_result);

        let is_valid = customers.verify_password(created_customer.id, old_password).await.unwrap();
        assert_eq!(false, is_valid);

        let is_valid = customers.verify_password(created_customer.id, new_password).await.unwrap();
        assert_eq!(true, is_valid);

        let change_result = customers.change_password(created_customer.id, "WrongCurrentPassword", "AnotherNewPassword").await;
        assert!(change_result.is_err());

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn reset_password_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("reset_password_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let old_password = "OriginalPassword123";
        let reset_password = "ResetPassword789";
        let test_email = "reset_password@example.com";
        let new_customer = Customer {
            id: Default::default(),
            email: test_email.to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: old_password.to_string(),
            full_name: "Reset Password Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();

        let is_valid = customers.verify_password(created_customer.id, old_password).await.unwrap();
        assert_eq!(true, is_valid);

        let reset_result = customers.reset_password(test_email.to_string(), reset_password).await.unwrap();
        assert_eq!(true, reset_result);

        let is_valid = customers.verify_password(created_customer.id, old_password).await.unwrap();
        assert_eq!(false, is_valid);

        let is_valid = customers.verify_password(created_customer.id, reset_password).await.unwrap();
        assert_eq!(true, is_valid);

        let reset_result = customers.reset_password("nonexistent@example.com".to_string(), "AnyPassword").await;
        assert!(reset_result.is_err());

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn request_password_reset_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("request_reset_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let test_email = "request_reset@example.com";
        let new_customer = Customer {
            id: Default::default(),
            email: test_email.to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "RequestResetPassword".to_string(),
            full_name: "Request Reset Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();

        let request_result = customers.request_password_reset(test_email.to_string()).await.unwrap();
        assert_eq!(true, request_result);

        let request_result = customers.request_password_reset("nonexistent@example.com".to_string()).await;
        assert!(request_result.is_err());

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn verify_email_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("verify_email_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let new_customer = Customer {
            id: Default::default(),
            email: "verify_email@example.com".to_string(),
            email_verified: false,
            encryption_mode: EncryptionModes::Argon2,
            password: "VerifyEmailPassword".to_string(),
            full_name: "Verify Email Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();
        assert_eq!(false, created_customer.email_verified);

        let verified_customer = customers.verify_email(created_customer.id).await.unwrap();
        assert_eq!(true, verified_customer.email_verified);

        let retrieved_customer = customers.get(created_customer.id).await.unwrap();
        assert_eq!(true, retrieved_customer.email_verified);

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn count_customers_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("count_customers_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let initial_count = customers.count_customers().await.unwrap();
        assert_eq!(0, initial_count);

        for i in 1..=3 {
            let new_customer = Customer {
                id: Default::default(),
                email: format!("count_test{}@example.com", i),
                email_verified: true,
                encryption_mode: EncryptionModes::Argon2,
                password: format!("CountTestPassword{}", i),
                full_name: format!("Count Test User {}", i),
                created_at: Default::default(),
                updated_at: None,
            };
            customers.insert(&new_customer).await.unwrap();
        }

        let count_after_insert = customers.count_customers().await.unwrap();
        assert_eq!(3, count_after_insert);

        let all_customers = customers.get_all().await.unwrap();
        customers.remove(all_customers[0].id).await.unwrap();

        let count_after_delete = customers.count_customers().await.unwrap();
        assert_eq!(2, count_after_delete);

        let remaining_customers = customers.get_all().await.unwrap();
        for customer in remaining_customers {
            customers.remove(customer.id).await.unwrap();
        }
    }).await;
}

#[tokio::test]
async fn search_customers_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("search_customers_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let test_customers = vec![
            Customer {
                id: Default::default(),
                email: "john.doe@example.com".to_string(),
                email_verified: true,
                encryption_mode: EncryptionModes::Argon2,
                password: "JohnPassword".to_string(),
                full_name: "John Doe".to_string(),
                created_at: Default::default(),
                updated_at: None,
            },
            Customer {
                id: Default::default(),
                email: "jane.smith@example.com".to_string(),
                email_verified: true,
                encryption_mode: EncryptionModes::Argon2,
                password: "JanePassword".to_string(),
                full_name: "Jane Smith".to_string(),
                created_at: Default::default(),
                updated_at: None,
            },
            Customer {
                id: Default::default(),
                email: "robert.jahnson@example.com".to_string(),
                email_verified: true,
                encryption_mode: EncryptionModes::Argon2,
                password: "RobertPassword".to_string(),
                full_name: "Robert Jahnson".to_string(),
                created_at: Default::default(),
                updated_at: None,
            },
        ];

        for customer in &test_customers {
            customers.insert(customer).await.unwrap();
        }

        let john_results = customers.search_customers("john").await.unwrap();
        assert_eq!(1, john_results.len());
        assert_eq!("John Doe", john_results[0].full_name);

        let smith_results = customers.search_customers("smith").await.unwrap();
        assert_eq!(1, smith_results.len());
        assert_eq!("Jane Smith", smith_results[0].full_name);

        let example_results = customers.search_customers("@example.com").await.unwrap();
        assert_eq!(3, example_results.len());

        let no_results = customers.search_customers("nonexistent").await.unwrap();
        assert_eq!(0, no_results.len());

        let all_customers = customers.get_all().await.unwrap();
        for customer in all_customers {
            customers.remove(customer.id).await.unwrap();
        }
    }).await;
}

#[tokio::test]
async fn pagination_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("pagination_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        for i in 1..=10 {
            let new_customer = Customer {
                id: Default::default(),
                email: format!("page_test{}@example.com", i),
                email_verified: true,
                encryption_mode: EncryptionModes::Argon2,
                password: format!("PageTestPassword{}", i),
                full_name: format!("Page Test User {}", i),
                created_at: Default::default(),
                updated_at: None,
            };
            customers.insert(&new_customer).await.unwrap();
        }

        let page1 = customers.get_customers_with_pagination(1, 3).await.unwrap();
        assert_eq!(3, page1.len());

        let page2 = customers.get_customers_with_pagination(2, 3).await.unwrap();
        assert_eq!(3, page2.len());

        let page4 = customers.get_customers_with_pagination(4, 3).await.unwrap();
        assert_eq!(1, page4.len());

        let page5 = customers.get_customers_with_pagination(5, 3).await.unwrap();
        assert_eq!(0, page5.len());

        let all_customers = customers.get_all().await.unwrap();
        for customer in all_customers {
            customers.remove(customer.id).await.unwrap();
        }
    }).await;
}

#[tokio::test]
async fn error_handling_tests() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("error_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let non_existent_id = Uuid::new_v4();
        let get_result = customers.get(non_existent_id).await;
        assert!(get_result.is_err());

        let find_result = customers.find_by_email("nonexistent@example.com".to_string()).await;
        assert!(find_result.is_err());

        let new_customer = Customer {
            id: Default::default(),
            email: "error_test@example.com".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "ErrorTestPassword".to_string(),
            full_name: "Error Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();

        let duplicate_customer = Customer {
            id: Default::default(),
            email: "error_test@example.com".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "AnotherPassword".to_string(),
            full_name: "Duplicate User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let insert_result = customers.insert(&duplicate_customer).await;
        assert!(insert_result.is_err());

        let change_result = customers.change_password(created_customer.id, "WrongPassword", "NewPassword").await;
        assert!(change_result.is_err());

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn unique_email_constraint_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("unique_email_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let test_email = "unique@example.com";
        let new_customer = Customer {
            id: Default::default(),
            email: test_email.to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "UniquePassword".to_string(),
            full_name: "Unique Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();

        let duplicate_customer = Customer {
            id: Default::default(),
            email: test_email.to_string(),
            email_verified: false,
            encryption_mode: EncryptionModes::Argon2,
            password: "DifferentPassword".to_string(),
            full_name: "Another Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let insert_result = customers.insert(&duplicate_customer).await;
        assert!(insert_result.is_err());

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn update_customer_properties_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("update_properties_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let new_customer = Customer {
            id: Default::default(),
            email: "update_test@example.com".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "UpdatePassword".to_string(),
            full_name: "Update Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();

        let result = customers.update(created_customer.id, &CustomerProfile {
            email: "updated@example.com".to_string(),
            full_name: "Updated Full Name".to_string(),
            email_verified: false,
        }).await.unwrap();

        assert_eq!("updated@example.com", result.email);
        assert_eq!("Updated Full Name", result.full_name);
        assert_eq!(false, result.email_verified);

        let retrieved_customer = customers.get(created_customer.id).await.unwrap();
        assert_eq!("updated@example.com", retrieved_customer.email);
        assert_eq!("Updated Full Name", retrieved_customer.full_name);
        assert_eq!(false, retrieved_customer.email_verified);

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn password_hashing_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("password_hash_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let plain_password = "SimplePassword123";
        let new_customer = Customer {
            id: Default::default(),
            email: "hash_test@example.com".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: plain_password.to_string(),
            full_name: "Hash Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).await.unwrap();

        assert_ne!(plain_password, created_customer.password);
        assert!(created_customer.password.starts_with("$argon2"));

        let is_valid = customers.verify_password(created_customer.id, plain_password).await.unwrap();
        assert_eq!(true, is_valid);

        customers.remove(created_customer.id).await.unwrap();
    }).await;
}

#[tokio::test]
async fn get_all_empty_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("empty_get_all_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        let empty_customers = customers.get_all().await.unwrap();
        assert_eq!(0, empty_customers.len());
    }).await;
}

#[tokio::test]
async fn search_pagination_integration_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("search_pagination_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        for i in 1..=10 {
            let name = if i % 2 == 0 {
                format!("Searchable User {}", i)
            } else {
                format!("Regular User {}", i)
            };

            let new_customer = Customer {
                id: Default::default(),
                email: format!("user{}@example.com", i),
                email_verified: true,
                encryption_mode: EncryptionModes::Argon2,
                password: format!("Password{}", i),
                full_name: name,
                created_at: Default::default(),
                updated_at: None,
            };

            customers.insert(&new_customer).await.unwrap();
        }

        let search_results = customers.search_customers("Searchable").await.unwrap();
        assert_eq!(5, search_results.len());

        let search_results = customers.search_customers("Searchable").await.unwrap();
        let search_ids: Vec<Uuid> = search_results.iter().map(|c| c.id).collect();

        let page1 = customers.get_customers_with_pagination(1, 3).await.unwrap();

        let page1_search_results: Vec<&Customer> = page1.iter()
            .filter(|c| search_ids.contains(&c.id))
            .collect();

        assert!(page1_search_results.len() <= 3);

        let all_customers = customers.get_all().await.unwrap();
        for customer in all_customers {
            customers.remove(customer.id).await.unwrap();
        }
    }).await;
}

#[tokio::test]
async fn tenant_isolation_test() {
    test_harness_two_tenants(|tenet_connection_string, shopster_connection_string1, shopster_connection_string2| async move {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant1 = tenet.create_tenant("tenant1_isolation_test".to_string()).unwrap();
        let tenant2 = tenet.create_tenant("tenant2_isolation_test".to_string()).unwrap();

        let storage1 = Storage::new_postgresql_database(shopster_connection_string1.clone(), tenant1.id);
        let storage2 = Storage::new_postgresql_database(shopster_connection_string2.clone(), tenant2.id);

        tenant1.add_storage(&storage1).unwrap();
        tenant2.add_storage(&storage2).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers1 = shopster.customers(tenant1.id).unwrap();
        let customers2 = shopster.customers(tenant2.id).unwrap();

        let customer1 = Customer {
            id: Default::default(),
            email: "tenant1@example.com".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "Tenant1Password".to_string(),
            full_name: "Tenant 1 User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };
        customers1.insert(&customer1).await.unwrap();

        let customer2 = Customer {
            id: Default::default(),
            email: "tenant2@example.com".to_string(),
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "Tenant2Password".to_string(),
            full_name: "Tenant 2 User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };
        customers2.insert(&customer2).await.unwrap();

        let tenant1_customers = customers1.get_all().await.unwrap();
        let tenant2_customers = customers2.get_all().await.unwrap();

        assert_eq!(1, tenant1_customers.len());
        assert_eq!(1, tenant2_customers.len());

        assert_eq!("tenant1@example.com", tenant1_customers[0].email);
        assert_eq!("tenant2@example.com", tenant2_customers[0].email);

        customers1.remove(tenant1_customers[0].id).await.unwrap();
        customers2.remove(tenant2_customers[0].id).await.unwrap();
    }).await;
}
