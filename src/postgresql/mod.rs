use diesel::result::ConnectionError;
use diesel::{Connection, sql_query, RunQueryDsl, PgConnection};

use crate::error::ShopsterError;

pub mod dbbasket;
pub mod dbcoupon;
pub mod dbpaymentattempt;
pub mod dbcustomer;
pub mod dbimage;
pub mod dborder;
pub mod dbproduct;
pub mod dbsettings;
pub mod dbtag;
pub mod dbwarehouse;

/// Maximum identifier length accepted by PostgreSQL (`NAMEDATALEN - 1`).
const MAX_DATABASE_NAME_LEN: usize = 63;

/// A connection string split into the part needed to reach the *server* (no database
/// path) and the database name taken from the path.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParsedDatabaseUrl {
    /// `scheme://userinfo@host:port` plus the original query string, if any.
    pub server_url: String,
    /// Validated database name (see [`validate_database_name`]).
    pub database_name: String,
}

/// Validates a database name that is about to be used as a SQL identifier.
///
/// Only ASCII letters, digits, `_` and `-` are accepted (1..=63 bytes). Anything else is
/// rejected instead of being escaped, so a hostile or malformed tenant connection string
/// can never smuggle SQL into `CREATE DATABASE` / `DROP DATABASE`.
pub(crate) fn validate_database_name(name: &str) -> Result<(), ShopsterError> {
    if name.is_empty() || name.len() > MAX_DATABASE_NAME_LEN {
        return Err(ShopsterError::InvalidOperationError(format!(
            "Invalid database name: must be 1-{} characters", MAX_DATABASE_NAME_LEN
        )));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(ShopsterError::InvalidOperationError(
            "Invalid database name: only letters, digits, '_' and '-' are allowed".to_string()
        ));
    }
    Ok(())
}

/// Splits `postgres://user:password@host:port/database?options` into a server URL
/// (without the database) and a validated database name. A query string is kept on the
/// server URL and never leaks into the database name.
pub(crate) fn parse_database_url(db_url: &str) -> Result<ParsedDatabaseUrl, ShopsterError> {
    let invalid = |msg: &str| ShopsterError::InvalidOperationError(format!("Invalid database url: {}", msg));

    let (main, query) = match db_url.split_once('?') {
        Some((main, query)) => (main, Some(query)),
        None => (db_url, None),
    };

    let (scheme, rest) = main.split_once("://").ok_or_else(|| invalid("missing scheme"))?;
    if scheme.is_empty() {
        return Err(invalid("missing scheme"));
    }

    // The database is the path after the last '/' of the authority part.
    let (authority, database_name) = rest.rsplit_once('/').ok_or_else(|| invalid("missing database name"))?;
    if authority.is_empty() {
        return Err(invalid("missing host"));
    }
    validate_database_name(database_name)?;

    let mut server_url = format!("{}://{}", scheme, authority);
    if let Some(query) = query {
        server_url.push('?');
        server_url.push_str(query);
    }

    Ok(ParsedDatabaseUrl { server_url, database_name: database_name.to_string() })
}

/// Returns `connection_string` with any password replaced by `***`, safe to log.
///
/// Handles URL style (`postgres://user:secret@host/db?password=secret`) as well as
/// libpq keyword style (`host=x password=secret`).
pub(crate) fn redact_connection_string(connection_string: &str) -> String {
    if let Some((scheme, rest)) = connection_string.split_once("://") {
        let (main, query) = match rest.split_once('?') {
            Some((main, query)) => (main, Some(query)),
            None => (rest, None),
        };

        let main = match main.rfind('@') {
            Some(at) => {
                let (userinfo, host_and_path) = main.split_at(at);
                match userinfo.split_once(':') {
                    Some((user, _password)) => format!("{}:***{}", user, host_and_path),
                    None => main.to_string(),
                }
            }
            None => main.to_string(),
        };

        let mut redacted = format!("{}://{}", scheme, main);
        if let Some(query) = query {
            let params: Vec<String> = query
                .split('&')
                .map(|param| match param.split_once('=') {
                    Some((key, _)) if key.eq_ignore_ascii_case("password") => format!("{}=***", key),
                    _ => param.to_string(),
                })
                .collect();
            redacted.push('?');
            redacted.push_str(&params.join("&"));
        }
        return redacted;
    }

    connection_string
        .split_whitespace()
        .map(|token| match token.split_once('=') {
            Some((key, _)) if key.eq_ignore_ascii_case("password") => format!("{}=***", key),
            _ => token.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub struct DatabaseHelper;

impl DatabaseHelper {

    /// Creates the database named in the connection string.
    ///
    /// database url format: postgres://user:password@host:port/database
    pub fn create_database(db_url: &str) -> Result<(), ShopsterError> {
        let parsed = parse_database_url(db_url)?;
        let mut conn = PgConnection::establish(&parsed.server_url)
            .map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        // The name was validated above; quoting is defence in depth.
        sql_query(format!("CREATE DATABASE \"{}\";", parsed.database_name)).execute(&mut conn)?;
        Ok(())
    }

    /// Returns `Ok(true)` if the database can be connected to, `Ok(false)` if the server
    /// reports that it does not exist, and `Err` for every other failure (server
    /// unreachable, bad credentials, ...). Those must not be mistaken for "missing".
    ///
    /// database url format: postgres://user:password@host:port/database
    pub fn is_database_exists(db_url: &str) -> Result<bool, ShopsterError> {
        match PgConnection::establish(db_url) {
            Ok(_) => Ok(true),
            Err(ConnectionError::BadConnection(message)) if is_missing_database_message(&message) => Ok(false),
            Err(e) => Err(ShopsterError::DatabaseConnectionError(e.to_string())),
        }
    }

    /// Drops the database named in the connection string.
    ///
    /// database url format: postgres://user:password@host:port/database
    #[allow(dead_code)]
    pub fn drop_database(db_url: &str) -> Result<(), ShopsterError> {
        let parsed = parse_database_url(db_url)?;
        let mut conn = PgConnection::establish(&parsed.server_url)
            .map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;
        sql_query(format!("DROP DATABASE \"{}\";", parsed.database_name)).execute(&mut conn)?;
        Ok(())
    }

}

/// libpq reports a missing database as `database "name" does not exist` (SQLSTATE 3D000).
/// A missing *role* produces `role "name" does not exist`, which must not match.
fn is_missing_database_message(message: &str) -> bool {
    message.contains("database \"") && message.contains("does not exist")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_url() {
        let parsed = parse_database_url("postgres://user:pw@localhost:5432/shop_db").unwrap();
        assert_eq!(parsed.server_url, "postgres://user:pw@localhost:5432");
        assert_eq!(parsed.database_name, "shop_db");
    }

    #[test]
    fn query_string_is_not_part_of_database_name() {
        let parsed = parse_database_url("postgres://u:p@h/shop?sslmode=require").unwrap();
        assert_eq!(parsed.database_name, "shop");
        assert_eq!(parsed.server_url, "postgres://u:p@h?sslmode=require");
    }

    #[test]
    fn rejects_malformed_urls_without_panicking() {
        for url in ["", "shop_db", "postgres://", "postgres://host", "postgres:///db", "://h/db", "postgres://h/"] {
            assert!(parse_database_url(url).is_err(), "{url:?} should be rejected");
        }
    }

    #[test]
    fn rejects_injection_attempts_in_database_name() {
        for name in [
            "x\"; DROP DATABASE postgres; --",
            "a b",
            "db;drop",
            "quo\"te",
            "semi;colon",
            "ünïcode",
            "",
        ] {
            assert!(validate_database_name(name).is_err(), "{name:?} should be rejected");
        }
        assert!(parse_database_url("postgres://h/x\"; DROP DATABASE postgres; --").is_err());
        assert!(validate_database_name(&"a".repeat(64)).is_err());
    }

    #[test]
    fn accepts_reasonable_database_names() {
        for name in ["shop", "stec_shopster_test_1a2b", "tenant-42", "A_b-9"] {
            assert!(validate_database_name(name).is_ok(), "{name:?} should be accepted");
        }
        assert!(validate_database_name(&"a".repeat(63)).is_ok());
    }

    #[test]
    fn create_and_drop_reject_invalid_input_before_connecting() {
        assert!(matches!(DatabaseHelper::create_database("garbage"), Err(ShopsterError::InvalidOperationError(_))));
        assert!(matches!(DatabaseHelper::drop_database("postgres://h/bad;name"), Err(ShopsterError::InvalidOperationError(_))));
    }

    #[test]
    fn missing_database_is_distinguished_from_missing_role() {
        assert!(is_missing_database_message("FATAL:  database \"nope\" does not exist\n"));
        assert!(!is_missing_database_message("FATAL:  role \"nope\" does not exist\n"));
        assert!(!is_missing_database_message("connection refused"));
    }

    #[test]
    fn redacts_url_password() {
        assert_eq!(
            redact_connection_string("postgres://admin:s3cret@db.internal:5432/shop"),
            "postgres://admin:***@db.internal:5432/shop"
        );
    }

    #[test]
    fn redacts_password_containing_special_characters() {
        let redacted = redact_connection_string("postgres://admin:p@ss:w/rd@db.internal/shop");
        assert!(!redacted.contains("p@ss"), "{redacted}");
        assert!(!redacted.contains("w/rd"), "{redacted}");
        assert!(redacted.contains("db.internal/shop"), "{redacted}");
    }

    #[test]
    fn redacts_query_password_and_keeps_other_params() {
        assert_eq!(
            redact_connection_string("postgres://admin@h/shop?sslmode=require&password=hunter2"),
            "postgres://admin@h/shop?sslmode=require&password=***"
        );
    }

    #[test]
    fn leaves_urls_without_credentials_untouched() {
        assert_eq!(redact_connection_string("postgres://localhost/shop"), "postgres://localhost/shop");
        assert_eq!(redact_connection_string("postgres://user@localhost/shop"), "postgres://user@localhost/shop");
    }

    #[test]
    fn redacts_keyword_style_connection_strings() {
        assert_eq!(
            redact_connection_string("host=db user=admin password=hunter2 dbname=shop"),
            "host=db user=admin password=*** dbname=shop"
        );
    }

    #[test]
    fn redaction_never_panics_on_odd_input() {
        for input in ["", "://", "postgres://@", "postgres://:@/?&=", "no-url-at-all"] {
            let _ = redact_connection_string(input);
        }
    }
}
