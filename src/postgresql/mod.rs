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
    pub fn create_database<Conn: Connection>(db_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (db_url, db_name) = db_url.rsplit_once('/').unwrap();
        let mut conn = Conn::establish(db_url)?;
        conn.batch_execute(&("CREATE DATABASE ".to_owned() + db_name + ";"));
        //conn.execute(&("CREATE DATABASE ".to_owned() + db_name + ";"))?;
        Ok(())
    }

    /// database url format: postgres://user:password@host:port/database
    pub fn is_database_exists<Conn: Connection>(db_url: &str) -> bool {
        Conn::establish(db_url).is_ok()
    }

    /// database url format: postgres://user:password@host:port/database
    pub fn drop_database<Conn: Connection>(db_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (db_url, db_name) = db_url.rsplit_once('/').unwrap();
        let mut conn = Conn::establish(db_url)?;
        conn.batch_execute(&("DROP DATABASE ".to_owned() + db_name + ";"));
        Ok(())
    }   

}