//! Error types for Shopster operations.
//!
//! This module defines all error types that can occur during Shopster operations.
//! Errors are built using the `thiserror` crate for ergonomic error handling.

use stec_tenet::TenetError;
use thiserror::Error;

/// Error types that can occur during Shopster operations.
///
/// All Shopster operations return `Result<T, ShopsterError>`. This enum covers
/// database errors, authentication issues, validation failures, and other
/// operational errors.
#[derive(Error, Debug)]
pub enum ShopsterError {
    /// Error from the tenant management service (Tenet).
    #[error("Tenet Error")]
    TenetError(#[from] TenetError),

    /// JSON serialization or deserialization failed.
    #[error("Serialization or Deserialization failed")]
    SerializationError(#[from] serde_json::Error),

    /// Error acquiring a database connection from the pool.
    #[error("Database Connection Error: {0}")]
    DatabaseConnectionError(String),

    /// Error from Diesel ORM operations.
    #[error("Database Error")]
    DatabaseError(#[from] diesel::result::Error),

    /// The specified tenant was not found in the system.
    #[error("Tenant not found")]
    TenantNotFoundError,

    /// The tenant has no configured storage (database).
    #[error("Tenant Storage not found")]
    TenantStorageNotFound,

    /// Password hashing operation failed.
    #[error("Password hashing Error")]
    PasswordHashingError(#[from] argon2::Error),

    /// An operation was attempted in an invalid state or with invalid parameters.
    #[error("Invalid Operation")]
    InvalidOperationError(String),

    /// Authentication failed (invalid credentials, expired session, etc.).
    #[error("Authentication Error")]
    AuthenticationError(String),

    /// Database migration failed.
    #[error("Database Migration Error: {0}")]
    DatabaseMigrationError(String),

    /// An internal error occurred (e.g., mutex poisoned).
    #[error("Internal Error: {0}")]
    InternalError(String),
}