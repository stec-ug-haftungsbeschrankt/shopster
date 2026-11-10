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
use argon2::{Config, ThreadMode, Variant, Version};
use std::sync::OnceLock;


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
        self.password = hash_password_argon2id(&self.password)?;
        Ok(())
    }

    /// `true` if the stored hash was made with weaker settings than the current policy
    /// (e.g. the legacy Argon2i parameters) and should be recomputed after a successful login.
    pub fn password_needs_rehash(&self) -> bool {
        needs_rehash(&self.password)
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


/// Current password hashing policy: Argon2id, v1.3, 19 MiB, 2 passes, 1 lane
/// (OWASP Password Storage Cheat Sheet minimum recommendation).
pub(crate) const ARGON2_MEM_COST_KIB: u32 = 19 * 1024;
pub(crate) const ARGON2_TIME_COST: u32 = 2;
pub(crate) const ARGON2_LANES: u32 = 1;

fn argon2_config() -> Config<'static> {
    Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: ARGON2_MEM_COST_KIB,
        time_cost: ARGON2_TIME_COST,
        lanes: ARGON2_LANES,
        thread_mode: ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 32,
    }
}

/// Hashes `password` with the current policy and a fresh random salt.
pub(crate) fn hash_password_argon2id(password: &str) -> Result<String, ShopsterError> {
    let salt: [u8; 32] = rand::random();
    Ok(argon2::hash_encoded(password.as_bytes(), &salt, &argon2_config())?)
}

/// Compares the parameters encoded in an Argon2 hash (`$argon2id$v=19$m=..,t=..,p=..$salt$hash`)
/// with the current policy. Unparseable hashes are reported as needing a rehash.
pub(crate) fn needs_rehash(encoded: &str) -> bool {
    let mut parts = encoded.split('$').skip(1);
    let (Some(variant), Some(version), Some(params)) = (parts.next(), parts.next(), parts.next()) else {
        return true;
    };

    if variant != "argon2id" || version != "v=19" {
        return true;
    }

    let (mut m, mut t, mut p) = (None, None, None);
    for kv in params.split(',') {
        match kv.split_once('=') {
            Some(("m", v)) => m = v.parse::<u32>().ok(),
            Some(("t", v)) => t = v.parse::<u32>().ok(),
            Some(("p", v)) => p = v.parse::<u32>().ok(),
            _ => {}
        }
    }

    match (m, t, p) {
        (Some(m), Some(t), Some(p)) => m < ARGON2_MEM_COST_KIB || t < ARGON2_TIME_COST || p < ARGON2_LANES,
        _ => true,
    }
}

/// Burns the same CPU/memory as a real verification. Used when the e-mail is unknown so
/// "no such user" and "wrong password" cannot be told apart by response time.
pub(crate) fn dummy_verify(password: &str) {
    static DUMMY_HASH: OnceLock<String> = OnceLock::new();
    let hash = DUMMY_HASH.get_or_init(|| {
        hash_password_argon2id("shopster-dummy-password").unwrap_or_default()
    });
    let _ = argon2::verify_encoded(hash, password.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_hashes_are_argon2id_with_policy_parameters() {
        let hash = hash_password_argon2id("correct horse").unwrap();
        assert!(hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"), "{hash}");
        assert!(!needs_rehash(&hash));
        assert!(argon2::verify_encoded(&hash, b"correct horse").unwrap());
        assert!(!argon2::verify_encoded(&hash, b"wrong").unwrap());
    }

    #[test]
    fn hashes_use_unique_salts() {
        assert_ne!(hash_password_argon2id("same").unwrap(), hash_password_argon2id("same").unwrap());
    }

    #[test]
    fn legacy_argon2i_hash_needs_rehash_but_still_verifies() {
        let salt = [7u8; 32];
        let legacy = argon2::hash_encoded(b"legacy", &salt, &Config::original()).unwrap();
        assert!(legacy.starts_with("$argon2i$"), "{legacy}");
        assert!(needs_rehash(&legacy));
        assert!(argon2::verify_encoded(&legacy, b"legacy").unwrap());
    }

    #[test]
    fn weaker_argon2id_parameters_need_rehash() {
        let config = Config { variant: Variant::Argon2id, mem_cost: 4096, time_cost: 2, lanes: 1, ..Config::default() };
        let weak = argon2::hash_encoded(b"pw", &[1u8; 16], &config).unwrap();
        assert!(needs_rehash(&weak));
    }

    #[test]
    fn stronger_parameters_do_not_need_rehash() {
        let config = Config { variant: Variant::Argon2id, mem_cost: 32 * 1024, time_cost: 3, lanes: 1, ..Config::default() };
        let strong = argon2::hash_encoded(b"pw", &[1u8; 16], &config).unwrap();
        assert!(!needs_rehash(&strong));
    }

    #[test]
    fn garbage_hashes_need_rehash() {
        for h in ["", "plaintext", "$argon2id$", "$argon2id$v=19$m=x,t=y,p=z$salt$hash"] {
            assert!(needs_rehash(h), "{h:?}");
        }
    }
}
