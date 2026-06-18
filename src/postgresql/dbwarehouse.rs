use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::{
    self,
    Insertable,
    Queryable,
};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncPgConnection};
use uuid::Uuid;

use crate::ShopsterError;
use crate::schema::*;
use crate::aquire_pool;

#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = warehouse)]
pub struct DbWarehouse {
    pub id: i64,
    pub product_id: i64,
    pub in_stock: i64,
    pub reserved: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = warehouse)]
pub struct InsertableDbWarehouse {
    pub product_id: i64,
    pub in_stock: i64,
    pub reserved: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<&DbWarehouse> for InsertableDbWarehouse {
    fn from(item: &DbWarehouse) -> Self {
        InsertableDbWarehouse {
            product_id: item.product_id,
            in_stock: item.in_stock,
            reserved: item.reserved,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

impl DbWarehouse {
    pub async fn find_by_product_id(tenant_id: Uuid, product_id: i64) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let item = warehouse::table
            .filter(warehouse::product_id.eq(product_id))
            .first(&mut conn).await?;
        Ok(item)
    }

    pub async fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let items = warehouse::table.load(&mut conn).await?;
        Ok(items)
    }

    pub async fn create(tenant_id: Uuid, item: DbWarehouse) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let insertable = InsertableDbWarehouse::from(&item);
        let db_item = diesel::insert_into(warehouse::table)
            .values(insertable)
            .get_result(&mut conn).await?;
        Ok(db_item)
    }

    pub async fn update_by_product_id(tenant_id: Uuid, product_id: i64, item: DbWarehouse) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let db_item = diesel::update(warehouse::table)
            .filter(warehouse::product_id.eq(product_id))
            .set(item)
            .get_result(&mut conn).await?;
        Ok(db_item)
    }

    pub async fn delete_by_product_id(tenant_id: Uuid, product_id: i64) -> Result<usize, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let res = diesel::delete(
            warehouse::table
                .filter(warehouse::product_id.eq(product_id)),
        )
        .execute(&mut conn).await?;
        Ok(res)
    }

    pub async fn apply_reserved_delta(tenant_id: Uuid, product_id: i64, delta: i64) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        Self::apply_reserved_delta_conn(&mut conn, product_id, delta).await
    }

    pub async fn apply_reserved_delta_conn(conn: &mut AsyncPgConnection, product_id: i64, delta: i64) -> Result<Self, ShopsterError> {
        // Atomic UPDATE using SQL arithmetic — avoids the read-modify-write race condition
        // where two concurrent callers both read the same value and one update is lost.
        // The WHERE reserved + delta >= 0 guard is evaluated atomically with the SET.
        let result = diesel::update(warehouse::table)
            .filter(warehouse::product_id.eq(product_id))
            .filter((warehouse::reserved + delta).ge(0i64))
            .set((
                warehouse::reserved.eq(warehouse::reserved + delta),
                warehouse::updated_at.eq(Some(Utc::now().naive_utc())),
            ))
            .get_result::<DbWarehouse>(conn).await;

        match result {
            Ok(updated) => Ok(updated),
            Err(diesel::result::Error::NotFound) => {
                let exists: bool = diesel::select(diesel::dsl::exists(
                    warehouse::table.filter(warehouse::product_id.eq(product_id)),
                ))
                .get_result(conn).await?;

                if exists {
                    return Err(ShopsterError::InvalidOperationError(
                        "Reserved stock cannot be negative".to_string(),
                    ));
                }

                if delta < 0 {
                    return Err(ShopsterError::InvalidOperationError(
                        "Reserved stock cannot be negative".to_string(),
                    ));
                }

                let now = Utc::now().naive_utc();
                let insertable = InsertableDbWarehouse {
                    product_id,
                    in_stock: 0,
                    reserved: delta,
                    created_at: now,
                    updated_at: Some(now),
                };
                Ok(diesel::insert_into(warehouse::table)
                    .values(insertable)
                    .get_result(conn).await?)
            }
            Err(e) => Err(ShopsterError::DatabaseError(e)),
        }
    }
}
