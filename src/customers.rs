use std::str::FromStr;

use tenet::encryption_modes::EncryptionModes;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

use crate::error::ShopsterError;
use crate::postgresql::dbcustomer::{DbCustomer, DbCustomerMessage};


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
    
    pub fn insert(&self, customer: Customer) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomerMessage::from(&customer);
        let created_customer = DbCustomer::create(self.tenant_id, db_customer)?;

        let reply = Customer::from(&created_customer);
        Ok(reply)
    }
    
    pub fn update(&self, customer: Customer) -> Result<Customer, ShopsterError> {
        let db_customer = DbCustomerMessage::from(&customer);
        let updated_customer = DbCustomer::update(self.tenant_id, customer.id, db_customer)?;

        let reply = Customer::from(&updated_customer);
        Ok(reply)
    }
    
    pub fn remove(&self, customer_id: Uuid) -> Result<bool, ShopsterError> {
        let result = DbCustomer::delete(self.tenant_id, customer_id)?;
        Ok(result > 0)
    }
}

