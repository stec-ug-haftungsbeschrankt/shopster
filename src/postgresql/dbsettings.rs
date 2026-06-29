use crate::ShopsterError;
use crate::schema::settings;
use crate::aquire_pool;

use uuid::Uuid;
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    prelude::*,
    Queryable,
    Insertable, Identifiable, AsChangeset
};
use diesel_async::RunQueryDsl;

#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = settings)]
pub struct DbSetting {
    pub id: i32,
    pub title: String,
    pub datatype: String,
    pub value: String
}


impl DbSetting {

    pub async fn find(tenant_id: Uuid, id: i32) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let setting = settings::table.filter(settings::id.eq(id)).first(&mut conn).await?;
        Ok(setting)
    }

    pub async fn find_by_title(tenant_id: Uuid, title: String) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let setting = settings::table.filter(settings::title.eq(title)).first(&mut conn).await?;
        Ok(setting)
    }

    pub async fn get_all(tenant_id: Uuid) -> Result<Vec<Self>, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let settings = settings::table.load(&mut conn).await?;
        Ok(settings)
    }

    pub async fn create(tenant_id: Uuid, setting: DbSetting) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let db_setting = diesel::insert_into(settings::table)
            .values(setting)
            .get_result(&mut conn).await?;
        Ok(db_setting)
    }

    pub async fn update(tenant_id: Uuid, id: i32, setting: DbSetting) -> Result<Self, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let db_setting = diesel::update(settings::table)
            .filter(settings::id.eq(id))
            .set(setting)
            .get_result(&mut conn).await?;
        Ok(db_setting)
    }

    pub async fn delete(tenant_id: Uuid, id: i32) -> Result<usize, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let res = diesel::delete(
                settings::table.filter(settings::id.eq(id))
            )
            .execute(&mut conn).await?;
        Ok(res)
    }

    pub async fn delete_by_title(tenant_id: Uuid, title: &str) -> Result<usize, ShopsterError> {
        let pool = aquire_pool(tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let res = diesel::delete(
                settings::table.filter(settings::title.eq(title))
            )
            .execute(&mut conn).await?;
        Ok(res)
    }
}
