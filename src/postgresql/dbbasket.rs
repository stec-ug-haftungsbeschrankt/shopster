use chrono::{NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    Queryable,
    Insertable,
};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncPgConnection};
use uuid::Uuid;
use crate::ShopsterError;
use crate::schema::*;
use crate::TenantDb;


#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = basketproducts)]
pub struct DbBasketProduct {
    pub id: i64,
    pub product_id: i64,
    pub quantity: i64,
    pub basket_id: Uuid
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = basketproducts)]
pub struct InsertableDbBasketProduct {
    pub product_id: i64,
    pub quantity: i64,
    pub basket_id: Uuid
}

impl From<&DbBasketProduct> for InsertableDbBasketProduct {
    fn from(basket_product: &DbBasketProduct) -> Self {
        InsertableDbBasketProduct {
            product_id: basket_product.product_id,
            quantity: basket_product.quantity,
            basket_id: basket_product.basket_id
        }
    }
}

impl DbBasketProduct {
    pub async fn find_basket_item(tenant: &TenantDb, basket_product_id: i64) -> Result<DbBasketProduct, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let basket_product = basketproducts::table
            .filter(basketproducts::id.eq(basket_product_id))
            .first(&mut conn).await?;
        Ok(basket_product)
    }

    pub async fn get_basket_items(tenant: &TenantDb, basket_id: Uuid) -> Result<Vec<DbBasketProduct>, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let basket_products = basketproducts::table
            .filter(basketproducts::basket_id.eq(basket_id))
            .get_results(&mut conn).await?;
        Ok(basket_products)
    }

    pub async fn create_basket_item(tenant: &TenantDb, basket_product: DbBasketProduct) -> Result<Self, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let insertable = InsertableDbBasketProduct::from(&basket_product);
        let db_basket_product = diesel::insert_into(basketproducts::table)
            .values(insertable)
            .get_result(&mut conn).await?;
        Ok(db_basket_product)
    }

    pub async fn update_basket_item(tenant: &TenantDb, id: i64, basket_product: DbBasketProduct) -> Result<Self, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let db_basket_product = diesel::update(basketproducts::table)
            .filter(basketproducts::id.eq(id))
            .set(basket_product)
            .get_result(&mut conn).await?;
        Ok(db_basket_product)
    }

    pub async fn delete_basket_item(tenant: &TenantDb, basket_product_id: i64) -> Result<usize, ShopsterError>{
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let res = diesel::delete(
            basketproducts::table
                .filter(basketproducts::id.eq(basket_product_id))
        ).execute(&mut conn).await?;
        Ok(res)
    }

    pub async fn delete_all_basket_items(tenant: &TenantDb, basket_id: Uuid) -> Result<usize, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let result = diesel::delete(
            basketproducts::table.filter(
                basketproducts::basket_id.eq(basket_id)
            )
        ).execute(&mut conn).await?;

        Ok(result)
    }

    pub async fn delete_all_basket_items_conn(conn: &mut AsyncPgConnection, basket_id: Uuid) -> Result<usize, ShopsterError> {
        let result = diesel::delete(
            basketproducts::table.filter(
                basketproducts::basket_id.eq(basket_id)
            )
        ).execute(conn).await?;
        Ok(result)
    }

    pub async fn get_basket_items_conn(conn: &mut AsyncPgConnection, basket_id: Uuid) -> Result<Vec<DbBasketProduct>, ShopsterError> {
        let basket_products = basketproducts::table
            .filter(basketproducts::basket_id.eq(basket_id))
            .get_results(conn).await?;
        Ok(basket_products)
    }

    pub async fn create_basket_item_conn(conn: &mut AsyncPgConnection, basket_product: DbBasketProduct) -> Result<Self, ShopsterError> {
        let insertable = InsertableDbBasketProduct::from(&basket_product);
        let db_basket_product = diesel::insert_into(basketproducts::table)
            .values(insertable)
            .get_result(conn).await?;
        Ok(db_basket_product)
    }

    pub async fn update_basket_item_conn(conn: &mut AsyncPgConnection, id: i64, basket_product: DbBasketProduct) -> Result<Self, ShopsterError> {
        let db_basket_product = diesel::update(basketproducts::table)
            .filter(basketproducts::id.eq(id))
            .set(basket_product)
            .get_result(conn).await?;
        Ok(db_basket_product)
    }
}


#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = baskets)]
pub struct DbBasket {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = baskets)]
pub struct InsertableDbBasket {
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

impl From<&DbBasket> for InsertableDbBasket {
    fn from(basket: &DbBasket) -> Self {
        InsertableDbBasket {
            created_at: basket.created_at,
            updated_at: basket.updated_at
        }
    }
}


impl DbBasket {
    pub async fn touch(tenant: &TenantDb, basket_id: Uuid) -> Result<(), ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        Self::touch_conn(&mut conn, basket_id).await
    }

    pub async fn touch_conn(conn: &mut AsyncPgConnection, basket_id: Uuid) -> Result<(), ShopsterError> {
        diesel::update(baskets::table.filter(baskets::id.eq(basket_id)))
            .set(baskets::updated_at.eq(Some(Utc::now().naive_utc())))
            .execute(conn).await?;
        Ok(())
    }

    /// Deletes all baskets (and their products) where the last-activity timestamp is before
    /// `cutoff`. Last activity is `COALESCE(updated_at, created_at)`.
    pub async fn delete_older_than(tenant: &TenantDb, cutoff: NaiveDateTime) -> Result<usize, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        use diesel_async::AsyncConnection;
        conn.transaction(async |conn| {
            // Find basket ids to delete
            let stale_ids: Vec<Uuid> = baskets::table
                .select(baskets::id)
                .filter(
                    baskets::updated_at.assume_not_null().lt(cutoff)
                        .or(baskets::updated_at.is_null().and(baskets::created_at.lt(cutoff)))
                )
                .load(conn).await?;

            if stale_ids.is_empty() {
                return Ok(0);
            }

            diesel::delete(basketproducts::table.filter(basketproducts::basket_id.eq_any(&stale_ids)))
                .execute(conn).await?;

            let deleted = diesel::delete(baskets::table.filter(baskets::id.eq_any(&stale_ids)))
                .execute(conn).await?;

            Ok(deleted)
        }).await
    }

    pub async fn get_all(tenant: &TenantDb) -> Result<Vec<Self>, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let baskets = baskets::table
            .load(&mut conn).await?;
        Ok(baskets)
    }

    pub async fn find(tenant: &TenantDb, basket_id: Uuid) -> Result<Self, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let basket = baskets::table
            .filter(baskets::id.eq(basket_id))
            .first(&mut conn).await?;
        Ok(basket)
    }

    pub async fn create(tenant: &TenantDb) -> Result<Self, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let insertable = InsertableDbBasket {
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc())
        };
        let basket = diesel::insert_into(baskets::table)
            .values(insertable)
            .get_result(&mut conn).await?;
        Ok(basket)
    }

    pub async fn delete(tenant: &TenantDb, basket_id: Uuid) -> Result<usize, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let res = diesel::delete(
                baskets::table
                    .filter(baskets::id.eq(basket_id))
            )
            .execute(&mut conn).await?;
        Ok(res)
    }

    pub async fn delete_conn(conn: &mut AsyncPgConnection, basket_id: Uuid) -> Result<usize, ShopsterError> {
        let res = diesel::delete(
                baskets::table
                    .filter(baskets::id.eq(basket_id))
            )
            .execute(conn).await?;
        Ok(res)
    }
}
