use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::{postgresql::dbbasket::DbBasket, error::ShopsterError};
use crate::postgresql::dbbasket::DbBasketProduct;


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

impl From<&DbBasket> for Basket {
    fn from(db_basket: &DbBasket) -> Self {
        Basket {
            id: db_basket.id,
            products: Vec::new(),
            created_at: db_basket.created_at,
            updated_at: db_basket.updated_at
        }
    }
}

impl From<&Basket> for DbBasket {
    fn from(basket: &Basket) -> Self {
        DbBasket {
            id: basket.id,
            created_at: basket.created_at,
            updated_at: basket.updated_at,
        }
    }
}


pub struct Baskets {
    tenant_id: Uuid
}

impl Baskets {
    pub fn new(tenant_id: Uuid) -> Self {
        Baskets { tenant_id }
    }
    
    pub fn get_basket(&self, basket_id: Uuid) -> Result<Basket, ShopsterError> {
        let db_basket = DbBasket::find(self.tenant_id, basket_id)?;
        let basket = Basket::from(&db_basket);
        Ok(basket)
    }
    
    pub fn add_basket(&self) -> Result<Uuid, ShopsterError> {
        let db_basket = DbBasket::create(self.tenant_id)?;
        Ok(db_basket.id)
    }
    
    pub fn delete_basket(&self, basket_id: Uuid) -> Result<bool, ShopsterError> {
        self.clear_basket(basket_id)?;
        let deleted_baskets = DbBasket::delete(self.tenant_id, basket_id)?;
        Ok(deleted_baskets > 0)
    }

    pub fn add_product_to_basket(&self, basket_id: Uuid, product_id: i64, quantity: i32) {
        todo!()
    }
    
    pub fn remove_product_from_basket(&self, basket_id: Uuid, product_id: i32, quanity: Option<i32>) {
        let amount = quanity.unwrap_or(1);
        
        todo!()
    }
    
    pub fn clear_basket(&self, basket_id: Uuid) -> Result<bool, ShopsterError> {
        let result = DbBasketProduct::delete_all_basket_items(self.tenant_id, basket_id)?;
        Ok(result > 0)
    }
}
