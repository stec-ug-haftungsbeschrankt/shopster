
use crate::DbConnection;
use uuid::Uuid;

pub struct Products {
    tenant_id: Uuid
}

impl Products {
    pub fn new(tenant_id: Uuid) -> Self {
        Products { tenant_id }
    }
}