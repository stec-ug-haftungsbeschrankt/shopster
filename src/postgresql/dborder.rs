use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    Queryable,
    Insertable
};
use diesel::prelude::*;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;

use crate::ShopsterError;
use crate::schema::*;
use crate::DbConnection;

#[derive(Debug, AsExpression, FromSqlRow, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
#[diesel(sql_type = crate::schema::sql_types::Orderstatus)]
pub enum OrderStatus {
    New,
    InProgress,
    ReadyToShip,
    Shipping,
    Done
}

impl ToSql<crate::schema::sql_types::Orderstatus, Pg> for OrderStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            OrderStatus::New => out.write_all(b"New")?,
            OrderStatus::InProgress => out.write_all(b"InProgress")?,
            OrderStatus::ReadyToShip => out.write_all(b"ReadyToShip")?,
            OrderStatus::Shipping => out.write_all(b"Shipping")?,
            OrderStatus::Done => out.write_all(b"Done")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::Orderstatus, Pg> for OrderStatus {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"New" => Ok(OrderStatus::New),
            b"InProgress" => Ok(OrderStatus::InProgress),
            b"ReadyToShip" => Ok(OrderStatus::ReadyToShip),
            b"Shipping" => Ok(OrderStatus::Shipping),
            b"Done" => Ok(OrderStatus::Done),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<&OrderStatus> for i32 {
    fn from(status: &OrderStatus) -> Self {
        match status {
            OrderStatus::New => 0,
            OrderStatus::InProgress => 1,
            OrderStatus::ReadyToShip => 2,
            OrderStatus::Shipping => 3,
            OrderStatus::Done => 4,
        }
    }
}

impl From<i32> for OrderStatus {
    fn from(status: i32) -> Self {
        match status {
            0 => OrderStatus::New,
            1 => OrderStatus::InProgress,
            2 => OrderStatus::ReadyToShip,
            3 => OrderStatus::Shipping,
            4 => OrderStatus::Done,
            _ => panic!()
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = orders)]
pub struct DbOrder {
    pub id: i64,
    pub status: OrderStatus,
    pub delivery_address: String,
    pub billing_address: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = orders)]
pub struct InsertableDbOrder {
    pub status: OrderStatus,
    pub delivery_address: String,
    pub billing_address: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

impl From<&DbOrder> for InsertableDbOrder {
    fn from(order: &DbOrder) -> Self {
        InsertableDbOrder {
            status: order.status.clone(),
            delivery_address: order.delivery_address.clone(),
            billing_address: order.billing_address.clone(),
            created_at: order.created_at,
            updated_at: order.updated_at
        }
    }
}



impl DbOrder {

    pub fn find(connection: &mut DbConnection, id: i64) -> Result<Self, ShopsterError> {
        let order = orders::table
            .filter(orders::id.eq(id))
            .first(connection)?;
        Ok(order)
    }

    pub fn get_all(connection: &mut DbConnection) -> Result<Vec<Self>, ShopsterError> {
        let orders = orders::table.load(connection)?;
        Ok(orders)
    }

    pub fn create(connection: &mut DbConnection, order: DbOrder) -> Result<Self, ShopsterError> {
        let insertable = InsertableDbOrder::from(&order);
        let db_order = diesel::insert_into(orders::table)
            .values(insertable)
            .get_result(connection)?;
        Ok(db_order)
    }

    pub fn update(connection: &mut DbConnection, id: i64, order: DbOrder) -> Result<Self, ShopsterError> {
        let db_order = diesel::update(orders::table)
            .filter(orders::id.eq(id))
            .set(order)
            .get_result(connection)?;
        Ok(db_order)
    }

    pub fn delete(connection: &mut DbConnection, id: i64) -> Result<usize, ShopsterError> {
        let res = diesel::delete(
                orders::table
                    .filter(orders::id.eq(id))
            )
            .execute(connection)?;
        Ok(res)
    }
}



