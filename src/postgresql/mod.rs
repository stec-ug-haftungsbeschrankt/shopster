use diesel::{Connection, sql_query, RunQueryDsl, PgConnection};

pub mod dbbasket;
pub mod dbcustomer;
pub mod dbimage;
pub mod dborder;
pub mod dbproduct;
pub mod dbsettings;
pub mod dbtag;

pub struct DatabaseHelper;

impl DatabaseHelper {

    /// database url format: postgres://user:password@host:port/database
    pub fn create_database(db_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (db_url, db_name) = db_url.rsplit_once('/').unwrap();
        let mut conn = PgConnection::establish(db_url)?;
        sql_query(&("CREATE DATABASE ".to_owned() + db_name + ";")).execute(&mut conn).unwrap();
        Ok(())
    }

    /// database url format: postgres://user:password@host:port/database
    pub fn is_database_exists(db_url: &str) -> bool {
        PgConnection::establish(db_url).is_ok()
    }

    /// database url format: postgres://user:password@host:port/database
    pub fn drop_database(db_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (db_url, db_name) = db_url.rsplit_once('/').unwrap();
        let mut conn = PgConnection::establish(db_url)?;
        sql_query(&("DROP DATABASE ".to_owned() + db_name + ";")).execute(&mut conn).unwrap();
        Ok(())
    }   

}