use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::{
    self,
    Insertable,
    Queryable,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::ShopsterError;
use crate::schema::*;
use crate::aquire_database;

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
    pub fn find_by_product_id(tenant_id: Uuid, product_id: i64) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let item = warehouse::table
            .filter(warehouse::product_id.eq(product_id))
            .first(&mut connection)?;
        Ok(item)
    }

    pub fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let items = warehouse::table.load(&mut connection)?;
        Ok(items)
    }

    pub fn create(tenant_id: Uuid, item: DbWarehouse) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let insertable = InsertableDbWarehouse::from(&item);
        let db_item = diesel::insert_into(warehouse::table)
            .values(insertable)
            .get_result(&mut connection)?;
        Ok(db_item)
    }

    pub fn update_by_product_id(tenant_id: Uuid, product_id: i64, item: DbWarehouse) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let db_item = diesel::update(warehouse::table)
            .filter(warehouse::product_id.eq(product_id))
            .set(item)
            .get_result(&mut connection)?;
        Ok(db_item)
    }

    pub fn delete_by_product_id(tenant_id: Uuid, product_id: i64) -> Result<usize, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let res = diesel::delete(
            warehouse::table
                .filter(warehouse::product_id.eq(product_id)),
        )
        .execute(&mut connection)?;
        Ok(res)
    }

    pub fn apply_reserved_delta(tenant_id: Uuid, product_id: i64, delta: i64) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

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
            .get_result::<DbWarehouse>(&mut connection);

        match result {
            Ok(updated) => Ok(updated),
            Err(diesel::result::Error::NotFound) => {
                // Either the row doesn't exist yet, or reserved + delta < 0 — distinguish them.
                let exists: bool = diesel::select(diesel::dsl::exists(
                    warehouse::table.filter(warehouse::product_id.eq(product_id)),
                ))
                .get_result(&mut connection)?;

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
                    .get_result(&mut connection)?)
            }
            Err(e) => Err(ShopsterError::DatabaseError(e)),
        }
    }
}
