use chrono::{NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    Queryable,
    Insertable
};
use diesel::prelude::*;
use uuid::Uuid;
use crate::ShopsterError;
use crate::schema::*;
use crate::aquire_database;
use argon2::Config;
use rand::Rng;


#[derive(Serialize, Deserialize, PartialEq, AsChangeset)]
#[diesel(table_name = customers)]
pub struct DbCustomerMessage {
    pub email: String,
    pub email_verified: bool,
    pub password: String,
    pub algorithm: String,
    pub full_name: String,
}


#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable)]
#[diesel(table_name = customers)]
pub struct DbCustomer {
    pub id: uuid::Uuid,
    pub email: String,
    pub email_verified: bool,
    pub algorithm: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub full_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}



impl From<&DbCustomerMessage> for DbCustomer {
    fn from(customer: &DbCustomerMessage) -> Self {
        DbCustomer {
            id: Uuid::new_v4(),
            email: customer.email.clone(),
            email_verified: customer.email_verified,
            password: customer.password.clone(),
            algorithm: customer.algorithm.clone(),
            full_name: customer.full_name.clone(),
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        }
    }
}

impl From<&DbCustomer> for DbCustomerMessage {
    fn from(customer: &DbCustomer) -> Self {
        DbCustomerMessage {
            email: customer.email.clone(),
            email_verified: customer.email_verified,
            password: customer.password.clone(),
            algorithm: customer.algorithm.clone(),
            full_name: customer.full_name.clone(),
        }
    }
}




impl DbCustomer {
    pub fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let customers = customers::table.load(&mut connection)?;
        Ok(customers)
    }

    pub fn find(tenant_id: Uuid, customer_id: Uuid) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        let customer = customers::table
            .filter(customers::id.eq(customer_id))
            .first(&mut connection)?;
        Ok(customer)
    }

    pub fn find_by_email(tenant_id: Uuid, email: String) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        let customer = customers::table
            .filter(customers::email.eq(email))
            .first(&mut connection)?;
        Ok(customer)
    }

    pub fn create(tenant_id: Uuid, customer: DbCustomerMessage) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let mut new_customer = DbCustomer::from(&customer);
        new_customer.hash_password()?;

        let db_customer = diesel::insert_into(customers::table)
            .values(new_customer)
            .get_result(&mut connection)?;
        Ok(db_customer)
    }

    pub fn update(tenant_id: Uuid, id: Uuid, customer: DbCustomerMessage) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        // FIXME Hash password if it has changed
        // How do I notice, if the password has changed?
        // Currently this is don at the Customer level, but it should be done at the database level.

        let customer = diesel::update(customers::table)
            .filter(customers::id.eq(id))
            .set(customer)
            .get_result(&mut connection)?;
        Ok(customer)
    }

    pub fn delete(tenant_id: Uuid, id: Uuid) -> Result<usize, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let res = diesel::delete(
            customers::table.filter(customers::id.eq(id))
            )
            .execute(&mut connection)?;
        Ok(res)
    }

    pub fn hash_password(&mut self) -> Result<(), ShopsterError> {
        let salt: [u8; 32] = rand::rng().random();
        // Alternative would be the low_memory variant. Can be time consuming.
        // See https://github.com/sru-systems/rust-argon2/issues/52
        let config = Config::original(); 

        self.password = argon2::hash_encoded(self.password.as_bytes(), &salt, &config)?;

        Ok(())
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, ShopsterError> {
        Ok(argon2::verify_encoded(&self.password, password.as_bytes())?)
    }

    pub fn count(tenant_id: Uuid) -> Result<i64, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let count: i64 = customers::table
            .count()
            .get_result(&mut connection)?;
        Ok(count)
    }

    pub fn search(tenant_id: Uuid, search_term: &str) -> Result<Vec<Self>, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let like_term = format!("%{}%", search_term);

        let customers = customers::table
            .filter(
                customers::email.like(&like_term)
                    .or(customers::full_name.like(&like_term))
            )
            .load(&mut connection)?;
        Ok(customers)
    }

    pub fn get_with_pagination(tenant_id: Uuid, page: i64, per_page: i64) -> Result<Vec<Self>, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let offset = (page - 1) * per_page;

        let customers = customers::table
            .offset(offset)
            .limit(per_page)
            .load(&mut connection)?;
        Ok(customers)
    }
}

