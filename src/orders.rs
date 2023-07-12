use crate::DbConnection;
use uuid::Uuid;

pub struct Orders { 
    tenant_id: Uuid
}

impl Orders {
    pub fn new(tenant_id: Uuid) -> Self {
        Orders { tenant_id }
    }
}