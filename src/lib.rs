extern crate diesel;
#[macro_use] extern crate diesel_migrations;

mod postgresql;
mod schema;
pub mod error;
pub mod baskets;
pub mod customers;
pub mod products;
pub mod orders;
pub mod settings;

use diesel::PgConnection;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::EmbeddedMigrations;

use error::ShopsterError;
use log::warn;
use crate::diesel_migrations::MigrationHarness;
use crate::postgresql::DatabaseHelper;
use log::info;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::Mutex;
use tenet::Tenet;
use uuid::Uuid;

use baskets::Baskets;
use customers::Customers;
use products::Products;
use orders::Orders;
use settings::Settings;


type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

const DATABASE_AQUISITION_ERROR: &str = "Unable to aquire Database";

#[derive(Debug)]
pub struct DatabaseSelector {
    tenants: Tenet,
    database_cache: HashMap<Uuid, Pool>
}

impl DatabaseSelector {
    pub fn new(tenet: Tenet) -> Self {      
        DatabaseSelector {
            tenants: tenet,
            database_cache: HashMap::new()
        }
    }

    pub fn add_default(&mut self, connection_string: String) -> Result<Uuid, ShopsterError> {
        info!("Initializing default Database");
        let manager = ConnectionManager::<PgConnection>::new(connection_string);
        let pool = Pool::new(manager)?;

        let mut database_connection = pool.get()?;
        database_connection.run_pending_migrations(MIGRATIONS).unwrap();

        let tenant_id = Uuid::new_v4();
        self.database_cache.insert(tenant_id, pool);

        Ok(tenant_id)
    }

    fn get_storage_for_tenant(&mut self, tenant_id: Uuid) -> Result<Pool, ShopsterError> {
        info!("Initializing Database");
    
        if !self.database_cache.contains_key(&tenant_id) {
            let tenant = self.tenants.get_tenant_by_id(tenant_id).ok_or(ShopsterError::TenantNotFoundError)?;
            let storages = tenant.get_storages();
            
            if storages.is_empty() {
                return Err(ShopsterError::TenantStorageNotFound);
            }
            
            let storage = &storages[0]; // FIXME What if we have multiple storages? Choose by storage type? 
            let connection_string = storage.connection_string.clone().unwrap(); // Option unwrap

            if !DatabaseHelper::is_database_exists(&connection_string) {
                info!("Database does not exit, creating it...");
                if let Err(e) = DatabaseHelper::create_database(&connection_string) {
                    warn!("{:?}", e);
                }
            }

            let manager = ConnectionManager::<PgConnection>::new(connection_string);
            let pool = Pool::new(manager)?;

            let mut database_connection = pool.get()?;
            database_connection.run_pending_migrations(MIGRATIONS).unwrap();

            self.database_cache.insert(tenant.id, pool);
        }

        let cached_pool = self.database_cache.get(&tenant_id).unwrap();
        Ok(cached_pool.clone())
    }
}

static DATABASE_SELECTOR: OnceLock<Mutex<DatabaseSelector>> = OnceLock::new();

fn aquire_database(tenant_id: Uuid) -> Result<DbConnection, ShopsterError> {
    let mut database_selector = DATABASE_SELECTOR.get().expect(DATABASE_AQUISITION_ERROR).lock().unwrap();
    let pool = database_selector.get_storage_for_tenant(tenant_id)?;
    let connection = pool.get()?;
    Ok(connection)
}


#[derive(Debug, Clone)]
pub struct Shopster { }

impl Shopster {
    pub fn new(database_selector: DatabaseSelector) -> Self {
        if let Err(e) = DATABASE_SELECTOR.set(Mutex::new(database_selector)) {
            warn!("{:?}", e);
        }
        Shopster { }
    }
    
    pub fn baskets(&self, tenant_id: Uuid) -> Result<Baskets, ShopsterError> {
        Ok(Baskets::new(tenant_id))
    }

    pub fn customers(&self, tenant_id: Uuid) -> Result<Customers, ShopsterError> {
        Ok(Customers::new(tenant_id))
    }

    pub fn products(&self, tenant_id: Uuid) -> Result<Products, ShopsterError> {     
        Ok(Products::new(tenant_id))
    }

    pub fn orders(&self, tenant_id: Uuid) -> Result<Orders, ShopsterError> {
        Ok(Orders::new(tenant_id))
    }

    pub fn settings(&self, tenant_id: Uuid) -> Result<Settings, ShopsterError> {
        Ok(Settings::new(tenant_id))
    } 
}



#[cfg(test)]
mod tests {
    use chrono::Utc;

    use tenet::encryption_modes::EncryptionModes;
    use tenet::Storage;

    use super::*;

    use crate::products::Price;
    use crate::products::Product;
    use crate::DatabaseSelector;
    use crate::Uuid;

    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::SyncRunner;
    use crate::customers::Customer;
    use crate::orders::Order;
    use crate::postgresql::dborder::OrderStatus;

    fn test_harness(test_code: impl Fn(String, String)) {
        let tenet_node = Postgres::default().start().expect("Unable to create to tenet container");
        let tenet_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", tenet_node.get_host_port_ipv4(5432).unwrap());

        let shopster_node = Postgres::default().start().expect("Unable to create to shopster container");
        let shopster_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/stec_shopster_test", shopster_node.get_host_port_ipv4(5432).unwrap());

        test_code(tenet_connection_string, shopster_connection_string);

        shopster_node.stop().expect("Failed to stop shopster");
        tenet_node.stop().expect("Failed to stop tenet");
    }

    #[test]
    fn tenant_not_found_test() {
        test_harness(|tenet_connection_string, _shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);
            let mut database_selector = DatabaseSelector::new(tenet);
            let tenant = database_selector.get_storage_for_tenant(Uuid::new_v4());
            
            assert!(tenant.is_err());
        });
    }

    #[test]
    fn settings_get_all() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("settings_get_all_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);

            let shopster = Shopster::new(database_selector);
            let settings = shopster.settings(tenant.id).unwrap().get_all();

            assert!(settings.is_ok());
            assert_eq!(13, settings.unwrap().len());
        });
    }

    #[test]
    fn basket_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
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
            let product = products.insert(&new_product).unwrap();

            let baskets = shopster.baskets(tenant.id).unwrap();
            let basket_id = baskets.add_basket().unwrap();
            let basket = baskets.get_basket(basket_id).unwrap();

            baskets.add_product_to_basket(basket_id, product.id, 2).unwrap();
            let products = baskets.get_products_from_basket(basket_id).unwrap();
            assert_eq!(1, products.len());
        });
    }

    #[test]
    fn update_product_quantity_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("update_product_quantity_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            // Erstellen eines Produkts
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
            let product = products.insert(&new_product).unwrap();

            // Erstellen eines Warenkorbs und Hinzufügen des Produkts
            let baskets = shopster.baskets(tenant.id).unwrap();
            let basket_id = baskets.add_basket().unwrap();

            // Produkt zum Warenkorb hinzufügen mit Menge 2
            let basket_product_id = baskets.add_product_to_basket(basket_id, product.id, 2).unwrap();

            // Menge auf 5 aktualisieren
            let updated_product = baskets.update_product_quantity(basket_id, basket_product_id, 5).unwrap();
            assert_eq!(5, updated_product.quantity);

            // Überprüfen, ob die Menge korrekt aktualisiert wurde
            let basket_products = baskets.get_products_from_basket(basket_id).unwrap();
            assert_eq!(1, basket_products.len());
            assert_eq!(5, basket_products[0].quantity);
        });
    }

    #[test]
    fn remove_product_from_basket_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("remove_product_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            // Erstellen eines Produkts
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
            let product = products.insert(&new_product).unwrap();

            // Erstellen eines Warenkorbs und Hinzufügen des Produkts
            let baskets = shopster.baskets(tenant.id).unwrap();
            let basket_id = baskets.add_basket().unwrap();

            // Produkt zum Warenkorb hinzufügen
            let basket_product_id = baskets.add_product_to_basket(basket_id, product.id, 3).unwrap();

            // Produkt aus dem Warenkorb entfernen
            let removal_success = baskets.remove_product_from_basket(basket_id, basket_product_id).unwrap();
            assert_eq!(true, removal_success);

            // Überprüfen, ob der Warenkorb jetzt leer ist
            let basket_products = baskets.get_products_from_basket(basket_id).unwrap();
            assert_eq!(0, basket_products.len());
        });
    }

    #[test]
    fn get_all_baskets_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("get_all_baskets_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);
            let baskets = shopster.baskets(tenant.id).unwrap();

            // Überprüfen, dass am Anfang keine Warenkörbe vorhanden sind
            let initial_baskets = baskets.get_all_baskets().unwrap();
            assert_eq!(0, initial_baskets.len());

            // Drei Warenkörbe erstellen
            let basket_id1 = baskets.add_basket().unwrap();
            let basket_id2 = baskets.add_basket().unwrap();
            let basket_id3 = baskets.add_basket().unwrap();

            // Überprüfen, dass jetzt drei Warenkörbe vorhanden sind
            let all_baskets = baskets.get_all_baskets().unwrap();
            assert_eq!(3, all_baskets.len());

            // Überprüfen, dass die IDs korrekt sind
            let basket_ids: Vec<Uuid> = all_baskets.iter().map(|b| b.id).collect();
            assert!(basket_ids.contains(&basket_id1));
            assert!(basket_ids.contains(&basket_id2));
            assert!(basket_ids.contains(&basket_id3));
        });
    }

    #[test]
    fn get_products_with_details_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("product_details_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            // Erstellen eines Produkts
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
            let product = products.insert(&new_product).unwrap();

            // Erstellen eines Warenkorbs und Hinzufügen des Produkts
            let baskets = shopster.baskets(tenant.id).unwrap();
            let basket_id = baskets.add_basket().unwrap();

            // Produkt zum Warenkorb hinzufügen
            baskets.add_product_to_basket(basket_id, product.id, 2).unwrap();

            // Abrufen der Produkte mit Details
            let products_with_details = baskets.get_products_with_details(basket_id).unwrap();

            // Überprüfen der Ergebnisse
            assert_eq!(1, products_with_details.len());
            let product_with_details = &products_with_details[0];
            assert_eq!(2, product_with_details.quantity);
            assert_eq!("Test Product with Details", product_with_details.product.title);
            assert_eq!("ART-44444", product_with_details.product.article_number);
            assert_eq!(499, product_with_details.product.price.as_ref().unwrap().amount);
            assert_eq!("EUR", product_with_details.product.price.as_ref().unwrap().currency);
        });
    }

    #[test]
    fn calculate_basket_total_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("basket_total_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            // Erstellen von zwei Produkten
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
                    amount: 100,  // 1 EUR
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
                    amount: 200,  // 2 EUR
                    currency: "EUR".to_string()
                }),
                weight: 200,
                tags: Vec::new(),
                created_at: Utc::now().naive_utc().to_owned(),
                updated_at: None
            };

            let product1 = products.insert(&product1).unwrap();
            let product2 = products.insert(&product2).unwrap();

            // Erstellen eines Warenkorbs und Hinzufügen der Produkte
            let baskets = shopster.baskets(tenant.id).unwrap();
            let basket_id = baskets.add_basket().unwrap();

            // Produkte zum Warenkorb hinzufügen: 3x Produkt 1 und 2x Produkt 2
            baskets.add_product_to_basket(basket_id, product1.id, 3).unwrap();
            baskets.add_product_to_basket(basket_id, product2.id, 2).unwrap();

            // Berechnen des Gesamtpreises: (3 * 100) + (2 * 200) = 300 + 400 = 700
            let (total, currency) = baskets.calculate_basket_total(basket_id).unwrap();

            assert_eq!(700, total);
            assert_eq!("EUR", currency);
        });
    }

    #[test]
    fn merge_baskets_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("merge_baskets_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            // Erstellen von drei Produkten
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
                article_number: "ART-88888".to_string(),
                title: "Product B".to_string(),
                gtin: "4444555566".to_string(),
                short_description: "Product B Description".to_string(),
                description: "Full Description for Product B".to_string(),
                image_url: "/images/ART-8888/IMG_8808.png".to_string(),
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

            let product3 = Product {
                id: 0,
                article_number: "ART-99999".to_string(),
                title: "Product C".to_string(),
                gtin: "7777888899".to_string(),
                short_description: "Product C Description".to_string(),
                description: "Full Description for Product C".to_string(),
                image_url: "/images/ART-9999/IMG_9909.png".to_string(),
                additional_images: Vec::new(),
                price: Some(Price {
                    amount: 300,
                    currency: "EUR".to_string()
                }),
                weight: 300,
                tags: Vec::new(),
                created_at: Utc::now().naive_utc().to_owned(),
                updated_at: None
            };

            let product1 = products.insert(&product1).unwrap();
            let product2 = products.insert(&product2).unwrap();
            let product3 = products.insert(&product3).unwrap();

            // Erstellen von zwei Warenkörben
            let baskets = shopster.baskets(tenant.id).unwrap();
            let source_basket_id = baskets.add_basket().unwrap();
            let target_basket_id = baskets.add_basket().unwrap();

            // Produkte zum Quell-Warenkorb hinzufügen: 2x Produkt A und 1x Produkt B
            baskets.add_product_to_basket(source_basket_id, product1.id, 2).unwrap();
            baskets.add_product_to_basket(source_basket_id, product2.id, 1).unwrap();

            // Produkte zum Ziel-Warenkorb hinzufügen: 1x Produkt A und 3x Produkt C
            baskets.add_product_to_basket(target_basket_id, product1.id, 1).unwrap();
            baskets.add_product_to_basket(target_basket_id, product3.id, 3).unwrap();

            // Warenkörbe zusammenführen
            baskets.merge_baskets(source_basket_id, target_basket_id).unwrap();

            // Überprüfen des Ergebnisses:
            // 1. Der Quell-Warenkorb sollte gelöscht sein
            let source_basket_result = baskets.get_basket(source_basket_id);
            assert!(source_basket_result.is_err());

            // 2. Der Ziel-Warenkorb sollte alle Produkte enthalten
            let target_products = baskets.get_products_from_basket(target_basket_id).unwrap();

            // Sortieren nach Produkt-ID für einfacheren Vergleich
            let mut sorted_products = target_products.clone();
            sorted_products.sort_by_key(|p| p.product_id);

            // Es sollten 3 Produkte sein
            assert_eq!(3, sorted_products.len());

            // Produkt A sollte 3x vorhanden sein (2 aus source + 1 aus target)
            assert_eq!(product1.id, sorted_products[0].product_id);
            assert_eq!(3, sorted_products[0].quantity);

            // Produkt B sollte 1x vorhanden sein (1 aus source)
            assert_eq!(product2.id, sorted_products[1].product_id);
            assert_eq!(1, sorted_products[1].quantity);

            // Produkt C sollte 3x vorhanden sein (3 aus target)
            assert_eq!(product3.id, sorted_products[2].product_id);
            assert_eq!(3, sorted_products[2].quantity);
        });
    }

    #[test]
    fn delete_basket_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("delete_basket_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);
            let baskets = shopster.baskets(tenant.id).unwrap();

            // Erstellen eines Warenkorbs
            let basket_id = baskets.add_basket().unwrap();

            // Überprüfen, dass der Warenkorb existiert
            let basket = baskets.get_basket(basket_id);
            assert!(basket.is_ok());

            // Warenkorb löschen
            let delete_result = baskets.delete_basket(basket_id).unwrap();
            assert_eq!(true, delete_result);

            // Überprüfen, dass der Warenkorb nicht mehr existiert
            let deleted_basket = baskets.get_basket(basket_id);
            assert!(deleted_basket.is_err());
        });
    }

    #[test]
    fn clear_basket_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("clear_basket_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            // Erstellen eines Produkts
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
                price: Some(Price {
                    amount: 199,
                    currency: "EUR".to_string()
                }),
                weight: 100,
                tags: Vec::new(),
                created_at: Utc::now().naive_utc().to_owned(),
                updated_at: None
            };
            let product = products.insert(&new_product).unwrap();

            // Erstellen eines Warenkorbs und Hinzufügen des Produkts
            let baskets = shopster.baskets(tenant.id).unwrap();
            let basket_id = baskets.add_basket().unwrap();

            // Produkt zum Warenkorb hinzufügen
            baskets.add_product_to_basket(basket_id, product.id, 3).unwrap();

            // Überprüfen, dass Produkt im Warenkorb ist
            let products_before = baskets.get_products_from_basket(basket_id).unwrap();
            assert_eq!(1, products_before.len());

            // Warenkorb leeren
            let clear_result = baskets.clear_basket(basket_id).unwrap();
            assert_eq!(true, clear_result);

            // Überprüfen, dass der Warenkorb leer ist, aber noch existiert
            let products_after = baskets.get_products_from_basket(basket_id).unwrap();
            assert_eq!(0, products_after.len());

            // Warenkorb sollte noch existieren
            let basket = baskets.get_basket(basket_id);
            assert!(basket.is_ok());
        });
    }

    #[test]
    fn add_existing_product_to_basket_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("add_existing_product_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            // Erstellen eines Produkts
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
                price: Some(Price {
                    amount: 299,
                    currency: "EUR".to_string()
                }),
                weight: 150,
                tags: Vec::new(),
                created_at: Utc::now().naive_utc().to_owned(),
                updated_at: None
            };
            let product = products.insert(&new_product).unwrap();

            // Erstellen eines Warenkorbs
            let baskets = shopster.baskets(tenant.id).unwrap();
            let basket_id = baskets.add_basket().unwrap();

            // Produkt zum Warenkorb hinzufügen mit Menge 2
            let basket_product_id1 = baskets.add_product_to_basket(basket_id, product.id, 2).unwrap();

            // Überprüfen, dass das Produkt im Warenkorb ist
            let products_before = baskets.get_products_from_basket(basket_id).unwrap();
            assert_eq!(1, products_before.len());
            assert_eq!(2, products_before[0].quantity);

            // Dasselbe Produkt erneut hinzufügen mit Menge 3
            let basket_product_id2 = baskets.add_product_to_basket(basket_id, product.id, 3).unwrap();

            // Die Basket-Produkt-ID sollte dieselbe sein
            assert_eq!(basket_product_id1, basket_product_id2);

            // Überprüfen, dass die Menge aktualisiert wurde
            let products_after = baskets.get_products_from_basket(basket_id).unwrap();
            assert_eq!(1, products_after.len());
            assert_eq!(3, products_after[0].quantity); // Es sollte jetzt Menge 3 sein
        });
    }

    #[test]
    fn non_existent_basket_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("non_existent_basket_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);
            let baskets = shopster.baskets(tenant.id).unwrap();

            // Generiere eine zufällige UUID für einen nicht existierenden Warenkorb
            let non_existent_id = Uuid::new_v4();

            // Versuche, den nicht existierenden Warenkorb abzurufen
            let get_result = baskets.get_basket(non_existent_id);
            assert!(get_result.is_err());

            // Versuche, ein Produkt zu einem nicht existierenden Warenkorb hinzuzufügen
            let add_result = baskets.add_product_to_basket(non_existent_id, 1, 1);
            assert!(add_result.is_err());

            // Versuche, einen nicht existierenden Warenkorb zu leeren
            let clear_result = baskets.clear_basket(non_existent_id);
            assert!(clear_result.is_err());

            // Versuche, einen nicht existierenden Warenkorb zu löschen
            let delete_result = baskets.delete_basket(non_existent_id);
            assert!(delete_result.is_err());
        });
    }

    #[test]
    fn multiple_products_in_basket_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("multiple_products_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            // Erstellen von drei verschiedenen Produkten
            let products = shopster.products(tenant.id).unwrap();

            let product1 = Product {
                id: 0,
                article_number: "ART-MULTI-1".to_string(),
                title: "Product Multi 1".to_string(),
                gtin: "1111222233".to_string(),
                short_description: "Multi Product 1".to_string(),
                description: "Description 1".to_string(),
                image_url: "/images/ART-MULTI-1/IMG_M1.png".to_string(),
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
                article_number: "ART-MULTI-2".to_string(),
                title: "Product Multi 2".to_string(),
                gtin: "4444555566".to_string(),
                short_description: "Multi Product 2".to_string(),
                description: "Description 2".to_string(),
                image_url: "/images/ART-MULTI-2/IMG_M2.png".to_string(),
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

            let product3 = Product {
                id: 0,
                article_number: "ART-MULTI-3".to_string(),
                title: "Product Multi 3".to_string(),
                gtin: "7777888899".to_string(),
                short_description: "Multi Product 3".to_string(),
                description: "Description 3".to_string(),
                image_url: "/images/ART-MULTI-3/IMG_M3.png".to_string(),
                additional_images: Vec::new(),
                price: Some(Price {
                    amount: 300,
                    currency: "EUR".to_string()
                }),
                weight: 300,
                tags: Vec::new(),
                created_at: Utc::now().naive_utc().to_owned(),
                updated_at: None
            };

            let product1 = products.insert(&product1).unwrap();
            let product2 = products.insert(&product2).unwrap();
            let product3 = products.insert(&product3).unwrap();

            // Erstellen eines Warenkorbs
            let baskets = shopster.baskets(tenant.id).unwrap();
            let basket_id = baskets.add_basket().unwrap();

            // Hinzufügen aller drei Produkte mit unterschiedlichen Mengen
            baskets.add_product_to_basket(basket_id, product1.id, 2).unwrap();
            baskets.add_product_to_basket(basket_id, product2.id, 1).unwrap();
            baskets.add_product_to_basket(basket_id, product3.id, 3).unwrap();

            // Abrufen aller Produkte im Warenkorb
            let basket_products = baskets.get_products_from_basket(basket_id).unwrap();

            // Es sollten 3 Produkte im Warenkorb sein
            assert_eq!(3, basket_products.len());

            // Sortieren nach Produkt-ID für einfacheren Vergleich
            let mut sorted_products = basket_products.clone();
            sorted_products.sort_by_key(|p| p.product_id);

            // Überprüfen der einzelnen Produkte und Mengen
            assert_eq!(product1.id, sorted_products[0].product_id);
            assert_eq!(2, sorted_products[0].quantity);

            assert_eq!(product2.id, sorted_products[1].product_id);
            assert_eq!(1, sorted_products[1].quantity);

            assert_eq!(product3.id, sorted_products[2].product_id);
            assert_eq!(3, sorted_products[2].quantity);

            // Ein Produkt aus dem Warenkorb entfernen
            let remove_result = baskets.remove_product_from_basket(basket_id, sorted_products[1].id).unwrap();
            assert_eq!(true, remove_result);

            // Überprüfen, dass nur noch 2 Produkte im Warenkorb sind
            let updated_products = baskets.get_products_from_basket(basket_id).unwrap();
            assert_eq!(2, updated_products.len());

            // Überprüfen, dass das richtige Produkt entfernt wurde
            let product_ids: Vec<i64> = updated_products.iter().map(|p| p.product_id).collect();
            assert!(product_ids.contains(&product1.id));
            assert!(!product_ids.contains(&product2.id)); // Sollte nicht mehr enthalten sein
            assert!(product_ids.contains(&product3.id));
        });
    }

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
    fn order_test() {
        test_harness(|tenet_connection_string, shopster_connection_string| {
            let tenet = Tenet::new(tenet_connection_string);

            let tenant = tenet.create_tenant("basket_test".to_string()).unwrap();
            let storage = Storage::new_postgresql_database(shopster_connection_string, tenant.id);
            tenant.add_storage(&storage).unwrap();

            let database_selector = DatabaseSelector::new(tenet);
            let shopster = Shopster::new(database_selector);

            let orders = shopster.orders(tenant.id).unwrap();
            let new_order = Order {
                id: 0,
                status: OrderStatus::New,
                delivery_address: "Duffy Duck, Duck road 22, 44444 Duckhousen".to_string(),
                billing_address: "Duffy Duck, Duck road 22, 44444 Duckhousen".to_string(),
                created_at: Default::default(),
                updated_at: None,
            };

            let order = orders.insert(&new_order).unwrap();

            let mut all_orders = orders.get_all().unwrap();
            assert_eq!(1, all_orders.len());

            let inserted_order = all_orders.first().unwrap();
            assert_eq!(new_order.status, inserted_order.status);
            assert_eq!(new_order.billing_address, inserted_order.billing_address);
            assert_eq!(new_order.delivery_address, inserted_order.delivery_address);

            let updated_order = all_orders.get_mut(0).unwrap();
            updated_order.status = OrderStatus::ReadyToShip;
            updated_order.delivery_address = "Bugs Bunny, Bunny road 44, 55555 Bunnycity".to_string();

            orders.update(updated_order).unwrap();

            let success = orders.remove(all_orders.first().unwrap().id).unwrap();
            assert_eq!(true, success);
        });
    }
}

