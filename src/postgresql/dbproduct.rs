use chrono::NaiveDateTime;
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
use crate::DbConnection;
use crate::aquire_database;




#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = products)]
pub struct DbProduct {
    pub id: i64,
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
    pub updated_at: Option<NaiveDateTime>
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = products)]
pub struct InsertableDbProduct {
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
    pub updated_at: Option<NaiveDateTime>
}

impl From<&DbProduct> for InsertableDbProduct {
    fn from(product: &DbProduct) -> Self {
        InsertableDbProduct {
            title: product.title.clone(),
            gtin: product.gtin.clone(),
            article_number: product.article_number.clone(),
            short_description: product.short_description.clone(),
            description: product.description.clone(),
            price: product.price,
            currency: product.currency.clone(),
            tags: product.tags.clone(),
            title_image: product.title_image.clone(),
            additional_images: product.additional_images.clone(),
            weight: product.weight,
            created_at: product.created_at,
            updated_at: product.updated_at
        }
    }
}



impl DbProduct {

    pub fn find(tenant_id: Uuid, id: i64) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let product = products::table
            .filter(products::id.eq(id))
            .first(&mut connection)?;
        Ok(product)
    }

    pub fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let products = products::table.load(&mut connection)?;
        Ok(products)
    }

    pub fn create(tenant_id: Uuid, product: DbProduct) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let insertable = InsertableDbProduct::from(&product);
        let db_product = diesel::insert_into(products::table)
            .values(insertable)
            .get_result(&mut connection)?;
        Ok(db_product)
    }

    pub fn update(tenant_id: Uuid, id: i64, product: DbProduct) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let db_product = diesel::update(products::table)
            .filter(products::id.eq(id))
            .set(product)
            .get_result(&mut connection)?;
        Ok(db_product)
    }

    pub fn delete(tenant_id: Uuid, id: i64) -> Result<usize, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let res = diesel::delete(
                products::table
                    .filter(products::id.eq(id))
            )
            .execute(&mut connection)?;
        Ok(res)
    }
}



