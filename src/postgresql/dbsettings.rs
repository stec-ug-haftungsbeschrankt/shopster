use crate::ShopsterError;
use crate::schema::settings;
use crate::DbConnection;
use crate::DATABASE_SELECTOR;

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

    pub fn find(connection: &mut DbConnection, id: i32) -> Result<Self, ShopsterError> {
        let settings = settings::table.filter(settings::id.eq(id)).first(connection)?;
        Ok(settings)
    }

    pub fn find_by_title(connection: &mut DbConnection, title: String) -> Result<Self, ShopsterError> {
        let settings = settings::table.filter(settings::title.eq(title)).first(connection)?;
        Ok(settings)
    }

    pub fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        //let mut connection = database::connection()?;
        let mut database_selector = DATABASE_SELECTOR.get().expect("Test").lock().unwrap();
        let pool = database_selector.get_storage_for_tenant(tenant_id)?;
        let mut connection = pool.get()?;
        let settings = settings::table.load(&mut connection)?;
        Ok(settings)
    }

    pub fn create(connection: &mut DbConnection, settings: DbSetting) -> Result<Self, ShopsterError> {
        let db_settings = diesel::insert_into(settings::table)
            .values(settings)
            .get_result(connection)?;
        Ok(db_settings)
    }

    pub fn update(connection: &mut DbConnection, id: i32, settings: DbSetting) -> Result<Self, ShopsterError> {
        let db_settings = diesel::update(settings::table)
            .filter(settings::id.eq(id))
            .set(settings)
            .get_result(connection)?;
        Ok(db_settings)
    }

    pub fn delete(connection: &mut DbConnection, id: i32) -> Result<usize, ShopsterError> {
        let res = diesel::delete(
                settings::table.filter(settings::id.eq(id))
            )
            .execute(connection)?;
        Ok(res)
    }

    pub fn delete_by_title(connection: &mut DbConnection, title: &str) -> Result<usize, ShopsterError> {
        let res = diesel::delete(
                settings::table.filter(settings::title.eq(title))
            )
            .execute(connection)?;
        Ok(res)
    }
}