use chrono::{NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    Queryable,
    Insertable
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use crate::ShopsterError;
use crate::schema::*;
use crate::aquire_pool;
use argon2::Config;


/// Full message type used only for customer creation (includes password).
#[derive(Serialize, Deserialize, PartialEq)]
pub struct DbCustomerMessage {
    pub email: String,
    pub email_verified: bool,
    pub password: String,
    pub algorithm: String,
    pub full_name: String,
}

/// Partial update type for profile fields — never touches password/algorithm columns.
#[derive(Serialize, Deserialize, PartialEq, AsChangeset)]
#[diesel(table_name = customers)]
pub struct DbProfileMessage {
    pub email: String,
    pub email_verified: bool,
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
    pub async fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let customers = customers::table.load(&mut conn).await?;
        Ok(customers)
    }

    pub async fn find(tenant_id: Uuid, customer_id: Uuid) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let customer = customers::table
            .filter(customers::id.eq(customer_id))
            .first(&mut conn).await?;
        Ok(customer)
    }

    pub async fn find_by_email(tenant_id: Uuid, email: String) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let customer = customers::table
            .filter(customers::email.eq(email))
            .first(&mut conn).await?;
        Ok(customer)
    }

    pub async fn create(tenant_id: Uuid, customer: DbCustomerMessage) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let mut new_customer = DbCustomer::from(&customer);
        new_customer.hash_password()?;

        let db_customer = diesel::insert_into(customers::table)
            .values(new_customer)
            .get_result(&mut conn).await?;
        Ok(db_customer)
    }

    pub async fn update(tenant_id: Uuid, id: Uuid, profile: DbProfileMessage) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let customer = diesel::update(customers::table)
            .filter(customers::id.eq(id))
            .set(profile)
            .get_result(&mut conn).await?;
        Ok(customer)
    }

    /// Updates only the hashed password and algorithm columns. Always call
    /// `hash_password()` on the customer before invoking this.
    pub async fn update_password(tenant_id: Uuid, id: Uuid, password: &str, algorithm: &str) -> Result<(), ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        diesel::update(customers::table)
            .filter(customers::id.eq(id))
            .set((
                customers::password.eq(password),
                customers::algorithm.eq(algorithm),
            ))
            .execute(&mut conn).await?;
        Ok(())
    }

    pub async fn delete(tenant_id: Uuid, id: Uuid) -> Result<usize, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let res = diesel::delete(
            customers::table.filter(customers::id.eq(id))
            )
            .execute(&mut conn).await?;
        Ok(res)
    }

    pub fn hash_password(&mut self) -> Result<(), ShopsterError> {
        let salt: [u8; 32] = rand::random();
        let config = Config::original();

        self.password = argon2::hash_encoded(self.password.as_bytes(), &salt, &config)?;

        Ok(())
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, ShopsterError> {
        Ok(argon2::verify_encoded(&self.password, password.as_bytes())?)
    }

    pub async fn count(tenant_id: Uuid) -> Result<i64, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let count: i64 = customers::table
            .count()
            .get_result(&mut conn).await?;
        Ok(count)
    }

    pub async fn search(tenant_id: Uuid, search_term: &str) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let like_term = format!("%{}%", search_term);

        let customers = customers::table
            .filter(
                customers::email.like(&like_term)
                    .or(customers::full_name.like(&like_term))
            )
            .load(&mut conn).await?;
        Ok(customers)
    }

    pub async fn get_with_pagination(tenant_id: Uuid, page: i64, per_page: i64) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let offset = (page - 1) * per_page;

        let customers = customers::table
            .offset(offset)
            .limit(per_page)
            .load(&mut conn).await?;
        Ok(customers)
    }
}
