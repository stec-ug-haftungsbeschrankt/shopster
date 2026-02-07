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
use std::fmt;
use std::io::Write;
use uuid::Uuid;

use crate::ShopsterError;
use crate::schema::*;
use crate::aquire_database;

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

impl From<i32> for DbOrderStatus {
    fn from(status: i32) -> Self {
        match status {
            0 => DbOrderStatus::New,
            1 => DbOrderStatus::InProgress,
            2 => DbOrderStatus::ReadyToShip,
            3 => DbOrderStatus::Shipping,
            4 => DbOrderStatus::Done,
            _ => panic!()
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = orders)]
pub struct DbOrder {
    pub id: i64,
    pub status: DbOrderStatus,
    pub delivery_address: String,
    pub billing_address: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = orders)]
pub struct InsertableDbOrder {
    pub status: DbOrderStatus,
    pub delivery_address: String,
    pub billing_address: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

impl From<&DbOrder> for InsertableDbOrder {
    fn from(order: &DbOrder) -> Self {
        InsertableDbOrder {
            status: order.status,
            delivery_address: order.delivery_address.clone(),
            billing_address: order.billing_address.clone(),
            created_at: order.created_at,
            updated_at: order.updated_at
        }
    }
}



impl DbOrder {

    pub fn find(tenant_id: Uuid, id: i64) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let order = orders::table
            .filter(orders::id.eq(id))
            .first(&mut connection)?;
        Ok(order)
    }

    pub fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let orders = orders::table.load(&mut connection)?;
        Ok(orders)
    }

    pub fn create(tenant_id: Uuid, order: DbOrder) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let insertable = InsertableDbOrder::from(&order);
        let db_order = diesel::insert_into(orders::table)
            .values(insertable)
            .get_result(&mut connection)?;
        Ok(db_order)
    }

    pub fn update(tenant_id: Uuid, id: i64, order: DbOrder) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let db_order = diesel::update(orders::table)
            .filter(orders::id.eq(id))
            .set(order)
            .get_result(&mut connection)?;
        Ok(db_order)
    }

    pub fn delete(tenant_id: Uuid, id: i64) -> Result<usize, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let res = diesel::delete(
                orders::table
                    .filter(orders::id.eq(id))
            )
            .execute(&mut connection)?;
        Ok(res)
    }
}
