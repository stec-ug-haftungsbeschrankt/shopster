use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::postgresql::dbbasket::DbBasket;


pub struct BasketProduct {
    id: i64,
    product_id: i64,
    quantity: i32
}

pub struct Basket {
    pub id: Uuid,
    pub products: Vec<BasketProduct>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}


pub struct Baskets {
    tenant_id: Uuid
}

impl Baskets {
    pub fn new(tenant_id: Uuid) -> Self {
        Baskets { tenant_id }
    }
    
    pub fn get_basket(&self, basket_id: Uuid) -> Basket {
        todo!()
    }
    
    pub fn add_basket(&self) -> Uuid {
        todo!()
    }
    
    pub fn delete_basket(&self, basket_id: Uuid) {
        todo!()
    }
    
    pub fn add_product_to_basket(&self, basket_id: Uuid, product_id: i64, quantity: i32) {
        todo!()
    }
    
    pub fn remove_product_to_basket(&self, basket_id: Uuid, product_id: i32, quanity: Option<i32>) {
        let amount = quanity.unwrap_or(1);
        
        todo!()
    }
    
    pub fn clear_basket(&self, basket_id: Uuid) {
        todo!()
    }
}
