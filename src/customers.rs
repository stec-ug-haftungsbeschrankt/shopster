use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

use crate::error::ShopsterError;
use crate::postgresql::dbcustomer::DbCustomer;


pub struct Customer {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub algorithm: String,
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
            algorithm: db_customer.algorithm.clone(),
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
            algorithm: customer.algorithm.clone(),
            password: customer.password.clone(),
            full_name: customer.full_name.clone(),
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc())
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
}