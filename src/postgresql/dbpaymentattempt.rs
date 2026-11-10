use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::ShopsterError;
use crate::schema::*;
use crate::TenantDb;

#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = payment_attempts)]
pub struct DbPaymentAttempt {
    pub id: i64,
    pub payment_intent_id: String,
    pub reason: String,
    pub basket_id: Option<Uuid>,
    pub amount: i64,
    pub currency: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = payment_attempts)]
pub struct InsertableDbPaymentAttempt {
    pub payment_intent_id: String,
    pub reason: String,
    pub basket_id: Option<Uuid>,
    pub amount: i64,
    pub currency: String,
    pub created_at: NaiveDateTime,
}

impl DbPaymentAttempt {
    pub async fn get_all(tenant: &TenantDb) -> Result<Vec<Self>, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let attempts = payment_attempts::table
            .order(payment_attempts::created_at.desc())
            .load(&mut conn).await?;
        Ok(attempts)
    }

    pub async fn find_by_payment_intent_id(tenant: &TenantDb, payment_intent_id: &str) -> Result<Option<Self>, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let attempt = payment_attempts::table
            .filter(payment_attempts::payment_intent_id.eq(payment_intent_id))
            .first(&mut conn).await
            .optional()?;
        Ok(attempt)
    }

    pub async fn create(tenant: &TenantDb, attempt: InsertableDbPaymentAttempt) -> Result<Self, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let db_attempt = diesel::insert_into(payment_attempts::table)
            .values(attempt)
            .get_result(&mut conn).await?;
        Ok(db_attempt)
    }
}
