//! Customer management and authentication.

use std::str::FromStr;
use std::convert::TryFrom;

use stec_tenet::encryption_modes::EncryptionModes;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};
use crate::error::ShopsterError;
use crate::postgresql::dbcustomer::{DbCustomer, DbCustomerMessage, DbProfileMessage};


/// A customer in the shop system.
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
pub struct Customers {
    tenant_id: Uuid
}

impl Customers {
    pub fn new(tenant_id: Uuid) -> Self {
        Customers { tenant_id }
    }

    pub async fn get_all(&self) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::get_all(self.tenant_id).await?;
        let customers = db_customers.iter().map(Customer::try_from).collect::<Result<Vec<_>, _>>()?;
        Ok(customers)
    }

    pub async fn get(&self, customer_id: Uuid) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find(self.tenant_id, customer_id).await?;
        let customer = Customer::try_from(&db_customer)?;
        Ok(customer)
    }

    pub async fn find_by_email(&self, email: String) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find_by_email(self.tenant_id, email).await?;
        let customer = Customer::try_from(&db_customer)?;
        Ok(customer)
    }

    pub async fn insert(&self, customer: &Customer) -> Result<Customer, ShopsterError> {
        if !is_valid_email(&customer.email) {
            return Err(ShopsterError::InvalidOperationError(
                "Invalid email format".to_string(),
            ));
        }
        let db_customer = DbCustomerMessage::from(customer);
        let created_customer = DbCustomer::create(self.tenant_id, db_customer).await?;

        let reply = Customer::try_from(&created_customer)?;
        Ok(reply)
    }

    pub async fn update(&self, customer_id: Uuid, profile: &CustomerProfile) -> Result<Customer, ShopsterError> {
        if !is_valid_email(&profile.email) {
            return Err(ShopsterError::InvalidOperationError(
                "Invalid email format".to_string(),
            ));
        }
        let db_profile = DbProfileMessage::from(profile);
        let updated_customer = DbCustomer::update(self.tenant_id, customer_id, db_profile).await?;
        Ok(Customer::try_from(&updated_customer)?)
    }

    pub async fn remove(&self, customer_id: Uuid) -> Result<bool, ShopsterError> {
        let result = DbCustomer::delete(self.tenant_id, customer_id).await?;
        Ok(result > 0)
    }

    pub async fn verify_password(&self, customer_id: Uuid, password: &str) -> Result<bool, ShopsterError> {
        let db_customer = DbCustomer::find(self.tenant_id, customer_id).await?;
        db_customer.verify_password(password)
    }

    pub async fn verify_email_password(&self, email: String, password: &str) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find_by_email(self.tenant_id, email).await?;

        let is_valid = db_customer.verify_password(password)?;

        if !is_valid {
            return Err(ShopsterError::AuthenticationError("Ungültiges Passwort".to_string()));
        }

        let customer = Customer::try_from(&db_customer)?;
        Ok(customer)
    }

    pub async fn change_password(&self, customer_id: Uuid, current_password: &str, new_password: &str) -> Result<bool, ShopsterError> {
        let mut db_customer = DbCustomer::find(self.tenant_id, customer_id).await?;

        let is_valid = db_customer.verify_password(current_password)?;

        if !is_valid {
            return Err(ShopsterError::AuthenticationError("Aktuelles Passwort ist ungültig".to_string()));
        }

        db_customer.password = new_password.to_string();
        db_customer.hash_password()?;

        DbCustomer::update_password(self.tenant_id, customer_id, &db_customer.password, &db_customer.algorithm).await?;

        Ok(true)
    }

    pub async fn reset_password(&self, email: String, new_password: &str) -> Result<bool, ShopsterError> {
        let mut db_customer = DbCustomer::find_by_email(self.tenant_id, email).await?;

        db_customer.password = new_password.to_string();
        db_customer.hash_password()?;

        DbCustomer::update_password(self.tenant_id, db_customer.id, &db_customer.password, &db_customer.algorithm).await?;

        Ok(true)
    }

    pub async fn request_password_reset(&self, email: String) -> Result<bool, ShopsterError> {
        let _db_customer = DbCustomer::find_by_email(self.tenant_id, email).await?;
        Ok(true)
    }

    pub async fn verify_email(&self, customer_id: Uuid) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomer::find(self.tenant_id, customer_id).await?;

        let profile = DbProfileMessage {
            email: db_customer.email.clone(),
            email_verified: true,
            full_name: db_customer.full_name.clone(),
        };

        let updated_db_customer = DbCustomer::update(self.tenant_id, customer_id, profile).await?;

        let customer = Customer::try_from(&updated_db_customer)?;
        Ok(customer)
    }

    pub async fn count_customers(&self) -> Result<i64, ShopsterError> {
        let count = DbCustomer::count(self.tenant_id).await?;
        Ok(count)
    }

    pub async fn search_customers(&self, search_term: &str) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::search(self.tenant_id, search_term).await?;
        let customers = db_customers.iter().map(Customer::try_from).collect::<Result<Vec<_>, _>>()?;
        Ok(customers)
    }

    pub async fn get_customers_with_pagination(&self, page: i64, per_page: i64) -> Result<Vec<Customer>, ShopsterError> {
        let db_customers = DbCustomer::get_with_pagination(self.tenant_id, page, per_page).await?;
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
