use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::ShopsterError;
use crate::schema::*;
use crate::aquire_pool;

#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = coupons)]
pub struct DbCoupon {
    pub id: i64,
    pub code: String,
    pub percent_off: Option<i32>,
    pub amount_off: Option<i64>,
    pub currency: Option<String>,
    pub valid_from: Option<NaiveDateTime>,
    pub valid_until: Option<NaiveDateTime>,
    pub max_uses: Option<i32>,
    pub use_count: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = coupons)]
pub struct InsertableDbCoupon {
    pub code: String,
    pub percent_off: Option<i32>,
    pub amount_off: Option<i64>,
    pub currency: Option<String>,
    pub valid_from: Option<NaiveDateTime>,
    pub valid_until: Option<NaiveDateTime>,
    pub max_uses: Option<i32>,
    pub use_count: i32,
    pub created_at: NaiveDateTime,
}

impl DbCoupon {
    pub async fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        let all = coupons::table.order(coupons::created_at.desc()).load(&mut conn).await?;
        Ok(all)
    }

    pub async fn find_by_code(tenant_id: Uuid, code: &str) -> Result<Option<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        let coupon = coupons::table
            .filter(coupons::code.eq(code))
            .first(&mut conn).await
            .optional()?;
        Ok(coupon)
    }

    pub async fn create(tenant_id: Uuid, coupon: InsertableDbCoupon) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        let db_coupon = diesel::insert_into(coupons::table)
            .values(coupon)
            .get_result(&mut conn).await?;
        Ok(db_coupon)
    }

    pub async fn increment_use_count(tenant_id: Uuid, coupon_id: i64) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        let updated = diesel::update(coupons::table.filter(coupons::id.eq(coupon_id)))
            .set(coupons::use_count.eq(coupons::use_count + 1))
            .get_result(&mut conn).await?;
        Ok(updated)
    }

    pub async fn delete(tenant_id: Uuid, coupon_id: i64) -> Result<usize, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        let res = diesel::delete(coupons::table.filter(coupons::id.eq(coupon_id)))
            .execute(&mut conn).await?;
        Ok(res)
    }
}
