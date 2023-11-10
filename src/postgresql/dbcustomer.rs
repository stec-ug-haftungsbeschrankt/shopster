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



#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, Insertable)]
#[diesel(table_name = customers)]
pub struct DbCustomer {
    pub id: uuid::Uuid,
    pub email: String,
    pub email_verified: bool,
    pub algorithm: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub full_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

