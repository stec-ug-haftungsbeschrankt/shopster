mod common;

use tenet::{Storage, Tenet};
use tenet::encryption_modes::EncryptionModes;
use shopster::{DatabaseSelector, Shopster};
use shopster::customers::Customer;
use uuid::Uuid;

use crate::common::test_harness;

#[test]
fn customer_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
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

        let customer = customers.insert(&new_customer).unwrap();

        let mut all_customers = customers.get_all().unwrap();
        assert_eq!(1, all_customers.len());

        let inserted_customer = all_customers.first().unwrap();
        assert_eq!(new_customer.email, inserted_customer.email);
        assert_eq!(new_customer.email_verified, inserted_customer.email_verified);
        assert_eq!(new_customer.encryption_mode, inserted_customer.encryption_mode);
        //assert_eq!(new_customer.password, inserted_customer.password); // Only Hash is returned
        assert_eq!(new_customer.full_name, inserted_customer.full_name);

        let updated_customer = all_customers.get_mut(0).unwrap();
        updated_customer.email_verified = false;
        updated_customer.email = "dummy@stecug.de".to_string();

        customers.update(updated_customer).unwrap();

        let success = customers.remove(all_customers.first().unwrap().id).unwrap();
        assert_eq!(true, success);
    });
}

#[test]
fn find_customer_by_email_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("find_by_email_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden
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

        customers.insert(&new_customer).unwrap();

        // Suchen des Kunden anhand der E-Mail
        let found_customer = customers.find_by_email(test_email).unwrap();

        // Überprüfen der Ergebnisse
        assert_eq!(new_customer.email, found_customer.email);
        assert_eq!(new_customer.full_name, found_customer.full_name);
        assert_eq!(new_customer.email_verified, found_customer.email_verified);

        // Aufräumen
        customers.remove(found_customer.id).unwrap();
    });
}

#[test]
fn verify_password_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("verify_password_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden
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

        let created_customer = customers.insert(&new_customer).unwrap();

        // Testen der Passwortüberprüfung mit korrektem Passwort
        let is_valid = customers.verify_password(created_customer.id, test_password).unwrap();
        assert_eq!(true, is_valid);

        // Testen der Passwortüberprüfung mit falschem Passwort
        let is_valid = customers.verify_password(created_customer.id, "WrongPassword123").unwrap();
        assert_eq!(false, is_valid);

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

#[test]
fn verify_email_password_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("verify_email_password_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden
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

        customers.insert(&new_customer).unwrap();

        // Testen der kombinierten E-Mail/Passwort-Überprüfung mit korrekten Daten
        let auth_customer = customers.verify_email_password(test_email.to_string(), test_password).unwrap();
        assert_eq!(new_customer.email, auth_customer.email);
        assert_eq!(new_customer.full_name, auth_customer.full_name);

        // Testen mit falschem Passwort - sollte einen Fehler zurückgeben
        let auth_result = customers.verify_email_password(test_email.to_string(), "WrongPassword123");
        assert!(auth_result.is_err());

        // Testen mit falscher E-Mail - sollte einen Fehler zurückgeben
        let auth_result = customers.verify_email_password("wrong@example.com".to_string(), test_password);
        assert!(auth_result.is_err());

        // Aufräumen
        customers.remove(auth_customer.id).unwrap();
    });
}

#[test]
fn change_password_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("change_password_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden
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

        let created_customer = customers.insert(&new_customer).unwrap();

        // Überprüfen, dass das alte Passwort funktioniert
        let is_valid = customers.verify_password(created_customer.id, old_password).unwrap();
        assert_eq!(true, is_valid);

        // Passwort ändern
        let change_result = customers.change_password(created_customer.id, old_password, new_password).unwrap();
        assert_eq!(true, change_result);

        // Überprüfen, dass das alte Passwort nicht mehr funktioniert
        let is_valid = customers.verify_password(created_customer.id, old_password).unwrap();
        assert_eq!(false, is_valid);

        // Überprüfen, dass das neue Passwort funktioniert
        let is_valid = customers.verify_password(created_customer.id, new_password).unwrap();
        assert_eq!(true, is_valid);

        // Testen mit falschem aktuellen Passwort - sollte einen Fehler zurückgeben
        let change_result = customers.change_password(created_customer.id, "WrongCurrentPassword", "AnotherNewPassword");
        assert!(change_result.is_err());

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

#[test]
fn reset_password_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("reset_password_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden
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

        let created_customer = customers.insert(&new_customer).unwrap();

        // Überprüfen, dass das alte Passwort funktioniert
        let is_valid = customers.verify_password(created_customer.id, old_password).unwrap();
        assert_eq!(true, is_valid);

        // Passwort zurücksetzen
        let reset_result = customers.reset_password(test_email.to_string(), reset_password).unwrap();
        assert_eq!(true, reset_result);

        // Überprüfen, dass das alte Passwort nicht mehr funktioniert
        let is_valid = customers.verify_password(created_customer.id, old_password).unwrap();
        assert_eq!(false, is_valid);

        // Überprüfen, dass das zurückgesetzte Passwort funktioniert
        let is_valid = customers.verify_password(created_customer.id, reset_password).unwrap();
        assert_eq!(true, is_valid);

        // Testen mit nicht existierender E-Mail - sollte einen Fehler zurückgeben
        let reset_result = customers.reset_password("nonexistent@example.com".to_string(), "AnyPassword");
        assert!(reset_result.is_err());

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

#[test]
fn request_password_reset_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("request_reset_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden
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

        let created_customer = customers.insert(&new_customer).unwrap();

        // Passwort-Reset anfordern
        let request_result = customers.request_password_reset(test_email.to_string()).unwrap();
        assert_eq!(true, request_result);

        // Testen mit nicht existierender E-Mail - sollte einen Fehler zurückgeben
        let request_result = customers.request_password_reset("nonexistent@example.com".to_string());
        assert!(request_result.is_err());

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

#[test]
fn verify_email_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("verify_email_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden mit nicht verifizierter E-Mail
        let new_customer = Customer {
            id: Default::default(),
            email: "verify_email@example.com".to_string(),
            email_verified: false,  // Nicht verifiziert
            encryption_mode: EncryptionModes::Argon2,
            password: "VerifyEmailPassword".to_string(),
            full_name: "Verify Email Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let created_customer = customers.insert(&new_customer).unwrap();
        assert_eq!(false, created_customer.email_verified);

        // E-Mail verifizieren
        let verified_customer = customers.verify_email(created_customer.id).unwrap();
        assert_eq!(true, verified_customer.email_verified);

        // Überprüfen, dass die Verifizierung in der Datenbank gespeichert wurde
        let retrieved_customer = customers.get(created_customer.id).unwrap();
        assert_eq!(true, retrieved_customer.email_verified);

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

#[test]
fn count_customers_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("count_customers_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Anfangs sollten keine Kunden vorhanden sein
        let initial_count = customers.count_customers().unwrap();
        assert_eq!(0, initial_count);

        // Erstellen von drei Test-Kunden
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

            customers.insert(&new_customer).unwrap();
        }

        // Es sollten jetzt drei Kunden sein
        let count_after_insert = customers.count_customers().unwrap();
        assert_eq!(3, count_after_insert);

        // Einen Kunden löschen
        let all_customers = customers.get_all().unwrap();
        customers.remove(all_customers[0].id).unwrap();

        // Es sollten jetzt zwei Kunden sein
        let count_after_delete = customers.count_customers().unwrap();
        assert_eq!(2, count_after_delete);

        // Aufräumen - restliche Kunden löschen
        let remaining_customers = customers.get_all().unwrap();
        for customer in remaining_customers {
            customers.remove(customer.id).unwrap();
        }
    });
}

#[test]
fn search_customers_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("search_customers_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen einiger Test-Kunden mit unterschiedlichen Namen und E-Mails
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
                email: "robert.johnson@example.com".to_string(),
                email_verified: true,
                encryption_mode: EncryptionModes::Argon2,
                password: "RobertPassword".to_string(),
                full_name: "Robert Johnson".to_string(),
                created_at: Default::default(),
                updated_at: None,
            },
        ];

        for customer in &test_customers {
            customers.insert(customer).unwrap();
        }

        // Suche nach "john" - sollte einen Treffer geben
        let john_results = customers.search_customers("john").unwrap();
        assert_eq!(1, john_results.len());
        assert_eq!("John Doe", john_results[0].full_name);

        // Suche nach "smith" - sollte einen Treffer geben
        let smith_results = customers.search_customers("smith").unwrap();
        assert_eq!(1, smith_results.len());
        assert_eq!("Jane Smith", smith_results[0].full_name);

        // Suche nach "@example.com" - sollte alle drei Treffer geben
        let example_results = customers.search_customers("@example.com").unwrap();
        assert_eq!(3, example_results.len());

        // Suche nach etwas, das nicht existiert
        let no_results = customers.search_customers("nonexistent").unwrap();
        assert_eq!(0, no_results.len());

        // Aufräumen - alle Kunden löschen
        let all_customers = customers.get_all().unwrap();
        for customer in all_customers {
            customers.remove(customer.id).unwrap();
        }
    });
}

#[test]
fn pagination_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("pagination_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen von 10 Test-Kunden
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

            customers.insert(&new_customer).unwrap();
        }

        // Erste Seite mit 3 Elementen abrufen
        let page1 = customers.get_customers_with_pagination(1, 3).unwrap();
        assert_eq!(3, page1.len());

        // Zweite Seite mit 3 Elementen abrufen
        let page2 = customers.get_customers_with_pagination(2, 3).unwrap();
        assert_eq!(3, page2.len());

        // Vierte Seite mit 3 Elementen abrufen (sollte nur 1 Element enthalten)
        let page4 = customers.get_customers_with_pagination(4, 3).unwrap();
        assert_eq!(1, page4.len());

        // Fünfte Seite sollte leer sein
        let page5 = customers.get_customers_with_pagination(5, 3).unwrap();
        assert_eq!(0, page5.len());

        // Aufräumen - alle Kunden löschen
        let all_customers = customers.get_all().unwrap();
        for customer in all_customers {
            customers.remove(customer.id).unwrap();
        }
    });
}

#[test]
fn error_handling_tests() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("error_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Versuche, einen nicht existierenden Kunden zu finden
        let non_existent_id = Uuid::new_v4();
        let get_result = customers.get(non_existent_id);
        assert!(get_result.is_err());

        // Versuche, einen Kunden mit einer nicht existierenden E-Mail zu finden
        let find_result = customers.find_by_email("nonexistent@example.com".to_string());
        assert!(find_result.is_err());

        // Erstelle einen Test-Kunden
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

        let created_customer = customers.insert(&new_customer).unwrap();

        // Versuche, einen Kunden mit einer bereits vorhandenen E-Mail einzufügen (sollte Fehler geben)
        let duplicate_customer = Customer {
            id: Default::default(),
            email: "error_test@example.com".to_string(), // Gleiche E-Mail
            email_verified: true,
            encryption_mode: EncryptionModes::Argon2,
            password: "AnotherPassword".to_string(),
            full_name: "Duplicate User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let insert_result = customers.insert(&duplicate_customer);
        assert!(insert_result.is_err());

        // Versuche, das Passwort mit falschen Zugangsdaten zu ändern
        let change_result = customers.change_password(created_customer.id, "WrongPassword", "NewPassword");
        assert!(change_result.is_err());

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

#[test]
fn unique_email_constraint_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("unique_email_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden
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

        let created_customer = customers.insert(&new_customer).unwrap();

        // Versuche, einen anderen Kunden mit der gleichen E-Mail-Adresse zu erstellen
        let duplicate_customer = Customer {
            id: Default::default(),
            email: test_email.to_string(),  // Gleiche E-Mail
            email_verified: false,
            encryption_mode: EncryptionModes::Argon2,
            password: "DifferentPassword".to_string(),
            full_name: "Another Test User".to_string(),
            created_at: Default::default(),
            updated_at: None,
        };

        let insert_result = customers.insert(&duplicate_customer);

        // Sollte fehlschlagen, da die E-Mail-Adresse eindeutig sein muss
        assert!(insert_result.is_err());

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

#[test]
fn update_customer_properties_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("update_properties_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden
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

        let created_customer = customers.insert(&new_customer).unwrap();

        // Aktualisieren verschiedener Eigenschaften
        let mut updated_customer = created_customer.clone();
        updated_customer.email = "updated@example.com".to_string();
        updated_customer.full_name = "Updated Full Name".to_string();
        updated_customer.email_verified = false;

        let result = customers.update(&updated_customer).unwrap();

        // Überprüfen der aktualisierten Eigenschaften
        assert_eq!("updated@example.com", result.email);
        assert_eq!("Updated Full Name", result.full_name);
        assert_eq!(false, result.email_verified);

        // Abrufen des Kunden, um zu überprüfen, dass die Änderungen gespeichert wurden
        let retrieved_customer = customers.get(created_customer.id).unwrap();
        assert_eq!("updated@example.com", retrieved_customer.email);
        assert_eq!("Updated Full Name", retrieved_customer.full_name);
        assert_eq!(false, retrieved_customer.email_verified);

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

// Test für Passwort-Hash-Überprüfung
#[test]
fn password_hashing_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("password_hash_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen eines Test-Kunden mit einem einfachen Passwort
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

        let created_customer = customers.insert(&new_customer).unwrap();

        // Das gespeicherte Passwort sollte ein Hash sein, nicht der Klartext
        assert_ne!(plain_password, created_customer.password);
        assert!(created_customer.password.starts_with("$argon2")); // Typischer Anfang eines Argon2-Hashes

        // Trotzdem sollte die Passwortüberprüfung funktionieren
        let is_valid = customers.verify_password(created_customer.id, plain_password).unwrap();
        assert_eq!(true, is_valid);

        // Aufräumen
        customers.remove(created_customer.id).unwrap();
    });
}

#[test]
fn get_all_empty_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("empty_get_all_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Überprüfen, dass get_all eine leere Liste zurückgibt, wenn keine Kunden existieren
        let empty_customers = customers.get_all().unwrap();
        assert_eq!(0, empty_customers.len());
    });
}

#[test]
fn search_pagination_integration_test() {
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        let tenant = tenet.create_tenant("search_pagination_test".to_string()).unwrap();
        let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
        tenant.add_storage(&storage).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers = shopster.customers(tenant.id).unwrap();

        // Erstellen von 10 Test-Kunden, wobei 5 davon "searchable" im Namen haben
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

            customers.insert(&new_customer).unwrap();
        }

        // Suchen nach "Searchable" und Überprüfen, dass 5 Ergebnisse zurückgegeben werden
        let search_results = customers.search_customers("Searchable").unwrap();
        assert_eq!(5, search_results.len());

        // Abrufen der ersten Seite mit 3 Elementen aus der Suche
        // Hierfür kombinieren wir die Suche mit Paginierung
        let search_results = customers.search_customers("Searchable").unwrap();
        let search_ids: Vec<Uuid> = search_results.iter().map(|c| c.id).collect();

        // Abrufen aller Kunden mit Paginierung
        let page1 = customers.get_customers_with_pagination(1, 3).unwrap();

        // Filtern der ersten Seite nach den IDs aus der Suche
        let page1_search_results: Vec<&Customer> = page1.iter()
            .filter(|c| search_ids.contains(&c.id))
            .collect();

        // Je nach Sortierung könnten unterschiedlich viele Ergebnisse zurückgegeben werden
        // Wir überprüfen nur, dass die Anzahl <= 3 ist (die Seitengröße)
        assert!(page1_search_results.len() <= 3);

        // Aufräumen - alle Kunden löschen
        let all_customers = customers.get_all().unwrap();
        for customer in all_customers {
            customers.remove(customer.id).unwrap();
        }
    });
}

#[test]
fn tenant_isolation_test() {
    // Test, um sicherzustellen, dass Kunden zwischen Mandanten isoliert sind
    test_harness(|tenet_connection_string, shopster_connection_string| {
        let tenet = Tenet::new(tenet_connection_string);

        // Erstellen von zwei Mandanten
        let tenant1 = tenet.create_tenant("tenant1_isolation_test".to_string()).unwrap();
        let tenant2 = tenet.create_tenant("tenant2_isolation_test".to_string()).unwrap();

        let storage1 = Storage::new_postgresql_database(shopster_connection_string.clone(), tenant1.id);
        let storage2 = Storage::new_postgresql_database(shopster_connection_string.clone(), tenant2.id);

        tenant1.add_storage(&storage1).unwrap();
        tenant2.add_storage(&storage2).unwrap();

        let database_selector = DatabaseSelector::new(tenet);
        let shopster = Shopster::new(database_selector);

        let customers1 = shopster.customers(tenant1.id).unwrap();
        let customers2 = shopster.customers(tenant2.id).unwrap();

        // Erstellen eines Kunden für Mandant 1
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
        customers1.insert(&customer1).unwrap();

        // Erstellen eines Kunden für Mandant 2
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
        customers2.insert(&customer2).unwrap();

        // Überprüfen, dass jeder Mandant nur seinen eigenen Kunden sieht
        let tenant1_customers = customers1.get_all().unwrap();
        let tenant2_customers = customers2.get_all().unwrap();

        assert_eq!(1, tenant1_customers.len());
        assert_eq!(1, tenant2_customers.len());

        assert_eq!("tenant1@example.com", tenant1_customers[0].email);
        assert_eq!("tenant2@example.com", tenant2_customers[0].email);

        // Aufräumen
        customers1.remove(tenant1_customers[0].id).unwrap();
        customers2.remove(tenant2_customers[0].id).unwrap();
    });
}
