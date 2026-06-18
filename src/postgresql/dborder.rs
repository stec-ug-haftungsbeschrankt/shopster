use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    Queryable,
    Insertable,
};
use diesel::prelude::*;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel_async::{RunQueryDsl, AsyncPgConnection};
use std::fmt;
use std::io::Write;
use std::convert::TryFrom;
use uuid::Uuid;

use crate::ShopsterError;
use crate::schema::*;
use crate::aquire_pool;

#[derive(Debug, AsExpression, FromSqlRow, Serialize, Deserialize, PartialEq, PartialOrd, Copy, Clone)]
#[diesel(sql_type = crate::schema::sql_types::DbOrderStatus)]
pub enum DbOrderStatus {
    New,
    InProgress,
    ReadyToShip,
    Shipping,
    Done
}

impl fmt::Display for DbOrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ToSql<crate::schema::sql_types::DbOrderStatus, Pg> for DbOrderStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            DbOrderStatus::New => out.write_all(b"New")?,
            DbOrderStatus::InProgress => out.write_all(b"InProgress")?,
            DbOrderStatus::ReadyToShip => out.write_all(b"ReadyToShip")?,
            DbOrderStatus::Shipping => out.write_all(b"Shipping")?,
            DbOrderStatus::Done => out.write_all(b"Done")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::DbOrderStatus, Pg> for DbOrderStatus {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"New" => Ok(DbOrderStatus::New),
            b"InProgress" => Ok(DbOrderStatus::InProgress),
            b"ReadyToShip" => Ok(DbOrderStatus::ReadyToShip),
            b"Shipping" => Ok(DbOrderStatus::Shipping),
            b"Done" => Ok(DbOrderStatus::Done),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<&DbOrderStatus> for i32 {
    fn from(status: &DbOrderStatus) -> Self {
        match status {
            DbOrderStatus::New => 0,
            DbOrderStatus::InProgress => 1,
            DbOrderStatus::ReadyToShip => 2,
            DbOrderStatus::Shipping => 3,
            DbOrderStatus::Done => 4,
        }
    }
}

impl TryFrom<i32> for DbOrderStatus {
    type Error = String;

    fn try_from(status: i32) -> Result<Self, Self::Error> {
        match status {
            0 => Ok(DbOrderStatus::New),
            1 => Ok(DbOrderStatus::InProgress),
            2 => Ok(DbOrderStatus::ReadyToShip),
            3 => Ok(DbOrderStatus::Shipping),
            4 => Ok(DbOrderStatus::Done),
            _ => Err(format!("Unknown order status: {}", status))
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = orders)]
pub struct DbOrder {
    pub id: i64,
    pub customer_id: Option<Uuid>,
    pub status: DbOrderStatus,
    pub delivery_address: String,
    pub billing_address: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = orders)]
pub struct InsertableDbOrder {
    pub customer_id: Option<Uuid>,
    pub status: DbOrderStatus,
    pub delivery_address: String,
    pub billing_address: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

impl From<&DbOrder> for InsertableDbOrder {
    fn from(order: &DbOrder) -> Self {
        InsertableDbOrder {
            customer_id: order.customer_id,
            status: order.status,
            delivery_address: order.delivery_address.clone(),
            billing_address: order.billing_address.clone(),
            created_at: order.created_at,
            updated_at: order.updated_at
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = order_items)]
pub struct DbOrderItem {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub quantity: i64,
    pub article_number: String,
    pub gtin: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub tags: String,
    pub title_image: String,
    pub additional_images: String,
    pub price: i64,
    pub currency: String,
    pub weight: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = order_items)]
pub struct InsertableDbOrderItem {
    pub order_id: i64,
    pub product_id: i64,
    pub quantity: i64,
    pub article_number: String,
    pub gtin: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub tags: String,
    pub title_image: String,
    pub additional_images: String,
    pub price: i64,
    pub currency: String,
    pub weight: i32,
    pub created_at: NaiveDateTime,
}

impl From<&DbOrderItem> for InsertableDbOrderItem {
    fn from(item: &DbOrderItem) -> Self {
        InsertableDbOrderItem {
            order_id: item.order_id,
            product_id: item.product_id,
            quantity: item.quantity,
            article_number: item.article_number.clone(),
            gtin: item.gtin.clone(),
            title: item.title.clone(),
            short_description: item.short_description.clone(),
            description: item.description.clone(),
            tags: item.tags.clone(),
            title_image: item.title_image.clone(),
            additional_images: item.additional_images.clone(),
            price: item.price,
            currency: item.currency.clone(),
            weight: item.weight,
            created_at: item.created_at,
        }
    }
}

impl DbOrderItem {
    pub async fn get_for_order(tenant_id: Uuid, order_id: i64) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        Self::get_for_order_conn(&mut conn, order_id).await
    }

    pub async fn get_for_order_conn(conn: &mut AsyncPgConnection, order_id: i64) -> Result<Vec<Self>, ShopsterError> {
        let items = order_items::table
            .filter(order_items::order_id.eq(order_id))
            .get_results(conn).await?;
        Ok(items)
    }

    pub async fn create_for_order(tenant_id: Uuid, items: Vec<DbOrderItem>) -> Result<Vec<Self>, ShopsterError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        let insertables: Vec<InsertableDbOrderItem> = items.iter().map(InsertableDbOrderItem::from).collect();
        let db_items = diesel::insert_into(order_items::table)
            .values(insertables)
            .get_results(&mut conn).await?;
        Ok(db_items)
    }

    pub async fn create_for_order_conn(conn: &mut AsyncPgConnection, items: Vec<DbOrderItem>) -> Result<Vec<Self>, ShopsterError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let insertables: Vec<InsertableDbOrderItem> = items.iter().map(InsertableDbOrderItem::from).collect();
        let db_items = diesel::insert_into(order_items::table)
            .values(insertables)
            .get_results(conn).await?;
        Ok(db_items)
    }
}



impl DbOrder {

    pub async fn find(tenant_id: Uuid, id: i64) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let order = orders::table
            .filter(orders::id.eq(id))
            .first(&mut conn).await?;
        Ok(order)
    }

    pub async fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let orders = orders::table.load(&mut conn).await?;
        Ok(orders)
    }

    pub async fn get_by_customer_id(tenant_id: Uuid, customer_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let orders = orders::table
            .filter(orders::customer_id.eq(customer_id))
            .load(&mut conn).await?;
        Ok(orders)
    }

    pub async fn get_without_customer_id(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let orders = orders::table
            .filter(orders::customer_id.is_null())
            .load(&mut conn).await?;
        Ok(orders)
    }

    pub async fn create(tenant_id: Uuid, order: DbOrder) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let insertable = InsertableDbOrder::from(&order);
        let db_order = diesel::insert_into(orders::table)
            .values(insertable)
            .get_result(&mut conn).await?;
        Ok(db_order)
    }

    pub async fn create_conn(conn: &mut AsyncPgConnection, order: DbOrder) -> Result<Self, ShopsterError> {
        let insertable = InsertableDbOrder::from(&order);
        let db_order = diesel::insert_into(orders::table)
            .values(insertable)
            .get_result(conn).await?;
        Ok(db_order)
    }

    pub async fn update(tenant_id: Uuid, id: i64, order: DbOrder) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let db_order = diesel::update(orders::table)
            .filter(orders::id.eq(id))
            .set(order)
            .get_result(&mut conn).await?;
        Ok(db_order)
    }

    pub async fn update_conn(conn: &mut AsyncPgConnection, id: i64, order: DbOrder) -> Result<Self, ShopsterError> {
        let db_order = diesel::update(orders::table)
            .filter(orders::id.eq(id))
            .set(order)
            .get_result(conn).await?;
        Ok(db_order)
    }

    pub async fn delete(tenant_id: Uuid, id: i64) -> Result<usize, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let res = diesel::delete(
                orders::table
                    .filter(orders::id.eq(id))
            )
            .execute(&mut conn).await?;
        Ok(res)
    }
}
