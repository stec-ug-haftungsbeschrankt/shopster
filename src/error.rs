use tenet::TenetError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShopsterError {
    #[error("Tenant Error")]
    TenetError(#[from] TenetError),
    #[error("Serialization or Deserialization failed")]
    SerializationError(#[from] serde_json::Error),
    #[error("Database Connection Error")]
    DatabaseConnectionError(#[from] r2d2::Error),
    #[error("Database Error")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Tenant not found")]
    TenantNotFoundError
}