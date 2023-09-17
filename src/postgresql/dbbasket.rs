use chrono::{NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    Queryable,
    Insertable
};
use diesel::prelude::*;
use uuid::Uuid;
use crate::ShopsterError;
use crate::schema::*;
use crate::aquire_database;


#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = basketproducts)]
pub struct DbBasketProduct {
    pub id: i64,
    pub product_id: i64,
    pub quantity: i64,
    pub basket_id: Uuid
}

impl DbBasketProduct {
    pub fn find_basket_item(tenant_id: Uuid, basket_product_id: i64) -> Result<DbBasketProduct, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let basket_product = basketproducts::table
            .filter(basketproducts::id.eq(basket_product_id))
            .first(&mut connection)?;
        Ok(basket_product)
    }

    pub fn get_basket_items(tenant_id: Uuid, basket_id: Uuid) -> Result<Vec<DbBasketProduct>, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let basket_products = basketproducts::table
            .filter(basketproducts::basket_id.eq(basket_id))
            .get_results(&mut connection)?;
        Ok(basket_products)
    }

    pub fn add_or_update_item(tenant_id: Uuid, basket_product: DbBasketProduct) -> Result<DbBasketProduct, ShopsterError> {
        todo!()
    }

    pub fn delete_item(tenant_id: Uuid, basket_product_id: i64) -> Result<usize, ShopsterError>{
        let mut connection = aquire_database(tenant_id)?;

        let res = diesel::delete(
            basketproducts::table
                .filter(basketproducts::id.eq(basket_product_id))
        ).execute(&mut connection)?;
        Ok(res)
    }

    pub fn delete_all_basket_items(tenant_id: Uuid, basket_id: Uuid) -> Result<usize, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;

        let result = diesel::delete(
            basketproducts::table.filter(
                basketproducts::basket_id.eq(basket_id)
            )
        ).execute(&mut connection)?;

        Ok(result)
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
    pub fn find(tenant_id: Uuid, basket_id: Uuid) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let basket = baskets::table
            .filter(baskets::id.eq(basket_id))
            .first(&mut connection)?;
        Ok(basket)
    }

    pub fn create(tenant_id: Uuid) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let insertable = InsertableDbBasket {
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc())         
        };
        let basket = diesel::insert_into(baskets::table)
            .values(insertable)
            .get_result(&mut connection)?;
        Ok(basket)
    }

    pub fn delete(tenant_id: Uuid, basket_id: Uuid) -> Result<usize, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let res = diesel::delete(
                baskets::table
                    .filter(baskets::id.eq(basket_id))
            )
            .execute(&mut connection)?;
        Ok(res)
    }
}



