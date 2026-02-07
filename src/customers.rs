use std::str::FromStr;

use stec_tenet::encryption_modes::EncryptionModes;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};
use crate::error::ShopsterError;
use crate::postgresql::dbcustomer::{DbCustomer, DbCustomerMessage};


#[derive(Clone)]
pub struct Customer {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub encryption_mode: EncryptionModes,
    pub password: String,
    pub full_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}


impl From<&DbCustomer> for Customer {
    fn from(db_customer: &DbCustomer) -> Self {
        Customer {
            id: db_customer.id,
            email: db_customer.email.clone(),
            email_verified: db_customer.email_verified,
            encryption_mode: EncryptionModes::from_str(&db_customer.algorithm).unwrap(),
            password: db_customer.password.clone(),
            full_name: db_customer.full_name.clone(),
            created_at: db_customer.created_at,
            updated_at: db_customer.updated_at
        }
    }
}

impl From<&Customer> for DbCustomer {
    fn from(customer: &Customer) -> Self {
        DbCustomer {
            id: customer.id,
            email: customer.email.clone(),
            email_verified: customer.email_verified,
            algorithm: customer.encryption_mode.to_string(),
            password: customer.password.clone(),
            full_name: customer.full_name.clone(),
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc())
        }
    }
}

impl From<&Customer> for DbCustomerMessage {
    fn from(customer: &Customer) -> Self {
        DbCustomerMessage {
            email: customer.email.clone(),
            email_verified: customer.email_verified,
            algorithm: customer.encryption_mode.to_string(),
            password: customer.password.clone(),
            full_name: customer.full_name.clone(),
        }
    }
}




pub struct Customers { 
    tenant_id: Uuid
}

impl Customers {
    pub fn new(tenant_id: Uuid) -> Self {
        Customers { tenant_id }
    }

    pub fn get_all(&self) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::get_all(self.tenant_id)?;
        let customers = db_customers.iter().map(Customer::from).collect();
        Ok(customers)
    }
    
    pub fn get(&self, customer_id: Uuid) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find(self.tenant_id, customer_id)?;
        let customer = Customer::from(&db_customer);
        Ok(customer)
    }

    pub fn find_by_email(&self, email: String) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find_by_email(self.tenant_id, email)?;
        let customer = Customer::from(&db_customer);
        Ok(customer)
    }

    pub fn insert(&self, customer: &Customer) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomerMessage::from(customer);
        let created_customer = DbCustomer::create(self.tenant_id, db_customer)?;

        let reply = Customer::from(&created_customer);
        Ok(reply)
    }
    
    pub fn update(&self, customer: &Customer) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomerMessage::from(customer);
        let updated_customer = DbCustomer::update(self.tenant_id, customer.id, db_customer)?;

        let reply = Customer::from(&updated_customer);
        Ok(reply)
    }
    
    pub fn remove(&self, customer_id: Uuid) -> Result<bool, ShopsterError> {
        let result = DbCustomer::delete(self.tenant_id, customer_id)?;
        Ok(result > 0)
    }

    pub fn verify_password(&self, customer_id: Uuid, password: &str) -> Result<bool, ShopsterError> {
        let db_customer = DbCustomer::find(self.tenant_id, customer_id)?;
        db_customer.verify_password(password)
    }

    pub fn verify_email_password(&self, email: String, password: &str) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find_by_email(self.tenant_id, email)?;

        // Überprüfe das Passwort
        let is_valid = db_customer.verify_password(password)?;

        if !is_valid {
            return Err(ShopsterError::AuthenticationError("Ungültiges Passwort".to_string()));
        }

        // Wenn das Passwort korrekt ist, gib den Kunden zurück
        let customer = Customer::from(&db_customer);
        Ok(customer)
    }


    pub fn change_password(&self, customer_id: Uuid, current_password: &str, new_password: &str) -> Result<bool, ShopsterError> {
        // Finde den Kunden
        let mut db_customer = DbCustomer::find(self.tenant_id, customer_id)?;

        // Überprüfe das aktuelle Passwort
        let is_valid = db_customer.verify_password(current_password)?;

        if !is_valid {
            return Err(ShopsterError::AuthenticationError("Aktuelles Passwort ist ungültig".to_string()));
        }

        db_customer.password = new_password.to_string();
        db_customer.hash_password()?;

        // Erstelle eine aktualisierte Version des Kunden mit dem neuen Passwort
        let customer_message = DbCustomerMessage::from(&db_customer);

        // Aktualisiere den Kunden in der Datenbank
        DbCustomer::update(self.tenant_id, customer_id, customer_message)?;

        Ok(true)
    }

    pub fn reset_password(&self, email: String, new_password: &str) -> Result<bool, ShopsterError> {
        // Finde den Kunden anhand der E-Mail-Adresse
        let mut db_customer = DbCustomer::find_by_email(self.tenant_id, email)?;

        db_customer.password = new_password.to_string();
        db_customer.hash_password()?;

        // Erstelle eine aktualisierte Version des Kunden mit dem neuen Passwort
        let customer_message = DbCustomerMessage::from(&db_customer);

        // Aktualisiere den Kunden in der Datenbank
        DbCustomer::update(self.tenant_id, db_customer.id, customer_message)?;

        Ok(true)
    }

    // In einer realen Anwendung würde hier eine Methode zum Senden eines Passwort-Reset-Links hinzugefügt:
    pub fn request_password_reset(&self, email: String) -> Result<bool, ShopsterError> {
        // Finde den Kunden anhand der E-Mail-Adresse
        let db_customer = DbCustomer::find_by_email(self.tenant_id, email)?;

        // Hier würde in einer realen Anwendung:
        // 1. Ein einmaliger Token generiert werden
        // 2. Der Token in der Datenbank gespeichert werden (mit Ablaufzeit)
        // 3. Eine E-Mail mit einem Reset-Link an den Kunden gesendet werden

        // Da dies nur ein Beispiel ist, geben wir einfach true zurück
        Ok(true)
    }


    pub fn verify_email(&self, customer_id: Uuid) -> Result<Customer, ShopsterError> {
        // Finde den Kunden
        let db_customer = DbCustomer::find(self.tenant_id, customer_id)?;

        // Erstelle eine aktualisierte Version des Kunden mit verifizierter E-Mail
        let mut customer_message = DbCustomerMessage::from(&db_customer);
        customer_message.email_verified = true;

        // Aktualisiere den Kunden in der Datenbank
        let updated_db_customer = DbCustomer::update(self.tenant_id, customer_id, customer_message)?;

        // Konvertiere den aktualisierten Kunden und gib ihn zurück
        let customer = Customer::from(&updated_db_customer);
        Ok(customer)
    }

    pub fn count_customers(&self) -> Result<i64, ShopsterError> {
        let count = DbCustomer::count(self.tenant_id)?;
        Ok(count)
    }

    pub fn search_customers(&self, search_term: &str) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::search(self.tenant_id, search_term)?;
        let customers = db_customers.iter().map(Customer::from).collect();
        Ok(customers)
    }

    pub fn get_customers_with_pagination(&self, page: i64, per_page: i64) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::get_with_pagination(self.tenant_id, page, per_page)?;
        let customers = db_customers.iter().map(Customer::from).collect();
        Ok(customers)
    }

}

