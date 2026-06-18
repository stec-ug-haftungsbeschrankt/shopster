//! Customer management and authentication.
//!
//! This module handles customer CRUD operations, authentication, password management,
//! email verification, and customer search functionality.
//!
//! # Example
//!
//! ```ignore
//! let customers = shopster.customers(tenant_id)?;
//! let customer = customers.insert(&Customer { ... })?;
//! let verified = customers.verify_email_password(email, password)?;
//! ```

use std::str::FromStr;
use std::convert::TryFrom;

use stec_tenet::encryption_modes::EncryptionModes;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};
use crate::error::ShopsterError;
use crate::postgresql::dbcustomer::{DbCustomer, DbCustomerMessage, DbProfileMessage};


/// A customer in the shop system.
///
/// Represents a registered customer with authentication credentials,
/// contact information, and account metadata.
#[derive(Clone)]
pub struct Customer {
    /// Unique customer identifier
    pub id: Uuid,
    /// Customer's email address
    pub email: String,
    /// Whether the email has been verified
    pub email_verified: bool,
    /// The password hashing algorithm used
    pub encryption_mode: EncryptionModes,
    /// Hashed password
    pub password: String,
    /// Customer's full name
    pub full_name: String,
    /// Account creation time
    pub created_at: NaiveDateTime,
    /// Last update time
    pub updated_at: Option<NaiveDateTime>,
}

/// Profile fields that can be changed via `Customers::update()`.
/// Password is intentionally absent — use `change_password` or `reset_password`.
pub struct CustomerProfile {
    pub email: String,
    pub email_verified: bool,
    pub full_name: String,
}

impl TryFrom<&DbCustomer> for Customer {
    type Error = ShopsterError;

    fn try_from(db_customer: &DbCustomer) -> Result<Self, Self::Error> {
        Ok(Customer {
            id: db_customer.id,
            email: db_customer.email.clone(),
            email_verified: db_customer.email_verified,
            encryption_mode: EncryptionModes::from_str(&db_customer.algorithm)
                .map_err(|_| ShopsterError::InvalidOperationError(
                    format!("Invalid encryption mode: {}", db_customer.algorithm)
                ))?,
            password: db_customer.password.clone(),
            full_name: db_customer.full_name.clone(),
            created_at: db_customer.created_at,
            updated_at: db_customer.updated_at,
        })
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

impl From<&CustomerProfile> for DbProfileMessage {
    fn from(profile: &CustomerProfile) -> Self {
        DbProfileMessage {
            email: profile.email.clone(),
            email_verified: profile.email_verified,
            full_name: profile.full_name.clone(),
        }
    }
}


/// Handler for customer management operations.
///
/// Provides CRUD operations, authentication, password management,
/// and search capabilities for customers within a tenant.
pub struct Customers {
    /// The tenant ID for tenant isolation
    tenant_id: Uuid
}

impl Customers {
    /// Creates a new Customers handler for a tenant.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    pub fn new(tenant_id: Uuid) -> Self {
        Customers { tenant_id }
    }

    /// Retrieves all customers for the tenant.
    ///
    /// # Returns
    ///
    /// `Ok(Vec<Customer>)` - All customers
    /// `Err(ShopsterError)` - If database error occurs
    pub fn get_all(&self) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::get_all(self.tenant_id)?;
        let customers = db_customers.iter().map(Customer::try_from).collect::<Result<Vec<_>, _>>()?;
        Ok(customers)
    }
    
    /// Retrieves a specific customer by ID.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The customer's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Customer)` - The customer
    /// `Err(ShopsterError)` - If not found or database error
    pub fn get(&self, customer_id: Uuid) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find(self.tenant_id, customer_id)?;
        let customer = Customer::try_from(&db_customer)?;
        Ok(customer)
    }

    /// Finds a customer by email address.
    ///
    /// # Arguments
    ///
    /// * `email` - The customer's email
    ///
    /// # Returns
    ///
    /// `Ok(Customer)` - The customer
    /// `Err(ShopsterError)` - If not found or database error
    pub fn find_by_email(&self, email: String) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find_by_email(self.tenant_id, email)?;
        let customer = Customer::try_from(&db_customer)?;
        Ok(customer)
    }

    /// Creates a new customer.
    ///
    /// # Arguments
    ///
    /// * `customer` - The customer to insert
    ///
    /// # Returns
    ///
    /// `Ok(Customer)` - The created customer
    /// `Err(ShopsterError)` - If creation fails
    pub fn insert(&self, customer: &Customer) -> Result<Customer, ShopsterError> {
        if !is_valid_email(&customer.email) {
            return Err(ShopsterError::InvalidOperationError(
                "Invalid email format".to_string(),
            ));
        }
        let db_customer = DbCustomerMessage::from(customer);
        let created_customer = DbCustomer::create(self.tenant_id, db_customer)?;

        let reply = Customer::try_from(&created_customer)?;
        Ok(reply)
    }
    
    /// Updates the profile fields of an existing customer.
    ///
    /// Password is never modified by this method. Use `change_password` or
    /// `reset_password` for password changes.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The customer to update
    /// * `profile` - The new profile values
    ///
    /// # Returns
    ///
    /// `Ok(Customer)` - The updated customer
    /// `Err(ShopsterError)` - If update fails
    pub fn update(&self, customer_id: Uuid, profile: &CustomerProfile) -> Result<Customer, ShopsterError> {
        if !is_valid_email(&profile.email) {
            return Err(ShopsterError::InvalidOperationError(
                "Invalid email format".to_string(),
            ));
        }
        let db_profile = DbProfileMessage::from(profile);
        let updated_customer = DbCustomer::update(self.tenant_id, customer_id, db_profile)?;
        Ok(Customer::try_from(&updated_customer)?)
    }

    /// Deletes a customer.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The customer to delete
    ///
    /// # Returns
    ///
    /// `Ok(bool)` - True if deleted, false if not found
    /// `Err(ShopsterError)` - If deletion fails
    pub fn remove(&self, customer_id: Uuid) -> Result<bool, ShopsterError> {
        let result = DbCustomer::delete(self.tenant_id, customer_id)?;
        Ok(result > 0)
    }

    /// Verifies whether a password is correct for a customer.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The customer's UUID
    /// * `password` - The password to verify
    ///
    /// # Returns
    ///
    /// `Ok(bool)` - True if password matches, false otherwise
    /// `Err(ShopsterError)` - If verification fails
    pub fn verify_password(&self, customer_id: Uuid, password: &str) -> Result<bool, ShopsterError> {
        let db_customer = DbCustomer::find(self.tenant_id, customer_id)?;
        db_customer.verify_password(password)
    }

    /// Authenticates a customer by email and password.
    ///
    /// # Arguments
    ///
    /// * `email` - The customer's email
    /// * `password` - The password to verify
    ///
    /// # Returns
    ///
    /// `Ok(Customer)` - The authenticated customer
    /// `Err(ShopsterError)` - If authentication fails
    pub fn verify_email_password(&self, email: String, password: &str) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find_by_email(self.tenant_id, email)?;

        // Überprüfe das Passwort
        let is_valid = db_customer.verify_password(password)?;

        if !is_valid {
            return Err(ShopsterError::AuthenticationError("Ungültiges Passwort".to_string()));
        }

        // Wenn das Passwort korrekt ist, gib den Kunden zurück
        let customer = Customer::try_from(&db_customer)?;
        Ok(customer)
    }


    /// Changes a customer's password.
    ///
    /// Requires verification of the current password before allowing the change.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The customer's UUID
    /// * `current_password` - The current password (for verification)
    /// * `new_password` - The new password
    ///
    /// # Returns
    ///
    /// `Ok(bool)` - True if successful
    /// `Err(ShopsterError)` - If current password is wrong or update fails
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

        DbCustomer::update_password(self.tenant_id, customer_id, &db_customer.password, &db_customer.algorithm)?;

        Ok(true)
    }

    /// Resets a customer's password to a new value (typically used with recovery tokens).
    ///
    /// # Arguments
    ///
    /// * `email` - Customer's email address
    /// * `new_password` - The new password
    ///
    /// # Returns
    ///
    /// `Ok(bool)` - True if successful
    /// `Err(ShopsterError)` - If customer not found or update fails
    pub fn reset_password(&self, email: String, new_password: &str) -> Result<bool, ShopsterError> {
        // Finde den Kunden anhand der E-Mail-Adresse
        let mut db_customer = DbCustomer::find_by_email(self.tenant_id, email)?;

        db_customer.password = new_password.to_string();
        db_customer.hash_password()?;

        DbCustomer::update_password(self.tenant_id, db_customer.id, &db_customer.password, &db_customer.algorithm)?;

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


    /// Marks a customer's email as verified.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The customer's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Customer)` - The updated customer
    /// `Err(ShopsterError)` - If operation fails
    pub fn verify_email(&self, customer_id: Uuid) -> Result<Customer, ShopsterError> {
        // Finde den Kunden
        let db_customer = DbCustomer::find(self.tenant_id, customer_id)?;

        let profile = DbProfileMessage {
            email: db_customer.email.clone(),
            email_verified: true,
            full_name: db_customer.full_name.clone(),
        };

        let updated_db_customer = DbCustomer::update(self.tenant_id, customer_id, profile)?;

        // Konvertiere den aktualisierten Kunden und gib ihn zurück
        let customer = Customer::try_from(&updated_db_customer)?;
        Ok(customer)
    }

    /// Returns the total number of customers for this tenant.
    ///
    /// # Returns
    ///
    /// `Ok(i64)` - Number of customers
    /// `Err(ShopsterError)` - If query fails
    pub fn count_customers(&self) -> Result<i64, ShopsterError> {
        let count = DbCustomer::count(self.tenant_id)?;
        Ok(count)
    }

    /// Searches for customers by name or email.
    ///
    /// # Arguments
    ///
    /// * `search_term` - The search query (name or email fragment)
    ///
    /// # Returns
    ///
    /// `Ok(Vec<Customer>)` - Matching customers
    /// `Err(ShopsterError)` - If search fails
    pub fn search_customers(&self, search_term: &str) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::search(self.tenant_id, search_term)?;
        let customers = db_customers.iter().map(Customer::try_from).collect::<Result<Vec<_>, _>>()?;
        Ok(customers)
    }

    pub fn get_customers_with_pagination(&self, page: i64, per_page: i64) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::get_with_pagination(self.tenant_id, page, per_page)?;
        let customers = db_customers.iter().map(Customer::try_from).collect::<Result<Vec<_>, _>>()?;
        Ok(customers)
    }

}

fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

