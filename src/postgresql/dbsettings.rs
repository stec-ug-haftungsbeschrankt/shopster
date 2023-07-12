use crate::ShopsterError;
use crate::schema::settings;
use crate::aquire_database;

use uuid::Uuid;
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    prelude::*,
    Queryable,
    Insertable, Identifiable, AsChangeset
};

#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = settings)]
pub struct DbSetting {
    pub id: i32,
    pub title: String,
    pub datatype: String,
    pub value: String
}


impl DbSetting {

    pub fn find(tenant_id: Uuid, id: i32) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let settings = settings::table.filter(settings::id.eq(id)).first(&mut connection)?;
        Ok(settings)
    }

    pub fn find_by_title(tenant_id: Uuid, title: String) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let settings = settings::table.filter(settings::title.eq(title)).first(&mut connection)?;
        Ok(settings)
    }

    pub fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let settings = settings::table.load(&mut connection)?;
        Ok(settings)
    }

    pub fn create(tenant_id: Uuid, settings: DbSetting) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let db_settings = diesel::insert_into(settings::table)
            .values(settings)
            .get_result(&mut connection)?;
        Ok(db_settings)
    }

    pub fn update(tenant_id: Uuid, id: i32, settings: DbSetting) -> Result<Self, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let db_settings = diesel::update(settings::table)
            .filter(settings::id.eq(id))
            .set(settings)
            .get_result(&mut connection)?;
        Ok(db_settings)
    }

    pub fn delete(tenant_id: Uuid, id: i32) -> Result<usize, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let res = diesel::delete(
                settings::table.filter(settings::id.eq(id))
            )
            .execute(&mut connection)?;
        Ok(res)
    }

    pub fn delete_by_title(tenant_id: Uuid, title: &str) -> Result<usize, ShopsterError> {
        let mut connection = aquire_database(tenant_id)?;
        
        let res = diesel::delete(
                settings::table.filter(settings::title.eq(title))
            )
            .execute(&mut connection)?;
        Ok(res)
    }
}