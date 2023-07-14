use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

use crate::error::ShopsterError;
use crate::postgresql::dborder::DbOrder;
use crate::postgresql::dborder::OrderStatus;


pub struct Order {
    pub id: i64,
    pub status: OrderStatus,
    pub delivery_address: String,
    pub billing_address: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}


impl From<&DbOrder> for Order {
    fn from(db_order: &DbOrder) -> Self {
        Order {
            id: db_order.id,
            status: db_order.status,
            delivery_address: db_order.delivery_address.clone(),
            billing_address: db_order.billing_address.clone(),
            created_at: db_order.created_at,
            updated_at: db_order.updated_at
        }
    }
}

impl From<&Order> for DbOrder {
    fn from(order: &Order) -> Self {
        DbOrder {
            id: order.id,
            status: order.status.into(),
            delivery_address: order.delivery_address.clone(),
            billing_address: order.billing_address.clone(),
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc())
        }
    }
}


pub struct Orders { 
    tenant_id: Uuid
}

impl Orders {
    pub fn new(tenant_id: Uuid) -> Self {
        Orders { tenant_id }
    }
    
    pub fn get_all(&self) -> Result<Vec<Order>, ShopsterError> {
        let db_orders = DbOrder::get_all(self.tenant_id)?;

        let orders = db_orders.iter().map(Order::from).collect();
        Ok(orders)
    }
    
    pub fn get_by_id(&self, order_id: i64) -> Result<Order, ShopsterError> {
        let db_order = DbOrder::find(self.tenant_id, order_id)?;

        let reply = Order::from(&db_order);
        Ok(reply)
    }
    
    pub fn insert(&self, order: Order) -> Result<Order, ShopsterError> {
        let db_order = DbOrder::from(&order);
        let created_order = DbOrder::create(self.tenant_id, db_order)?;

        let reply = Order::from(&created_order);
        Ok(reply)
    }
    
    pub fn update(&self, order: Order) -> Result<Order, ShopsterError> {
        let db_order = DbOrder::from(&order);
        let updated_order = DbOrder::update(self.tenant_id, order.id, db_order)?;

        let reply = Order::from(&updated_order);
        Ok(reply)
    }
    
    pub fn remove(&self, order_id: i64) -> Result<bool, ShopsterError> {
        let result = DbOrder::delete(self.tenant_id, order_id)?;
        Ok(result > 0)
    }
}
