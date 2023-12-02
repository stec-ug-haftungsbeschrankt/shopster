use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::{postgresql::dbbasket::DbBasket, error::ShopsterError};
use crate::postgresql::dbbasket::DbBasketProduct;


pub struct BasketProduct {
    pub id: i64,
    pub product_id: i64,
    pub quantity: i64
}

impl From<&DbBasketProduct> for BasketProduct {
    fn from(db_basket_product: &DbBasketProduct) -> Self {
        BasketProduct {
            id: db_basket_product.id,
            product_id: db_basket_product.product_id,
            quantity: db_basket_product.quantity
        }
    }
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

    pub fn set_product_to_basket(&self, basket_id: Uuid, product_id: i64, quantity: i64) -> Result<i64, ShopsterError> {
        let items = DbBasketProduct::get_basket_items(self.tenant_id, basket_id)?;

        if let Some(item) = items.into_iter().find(|x| x.product_id == product_id) {
            let updated_item = DbBasketProduct::update_basket_item(self.tenant_id, item.id, item)?;
            Ok(updated_item.id)
        } else {
            let basket_product = DbBasketProduct { id: 0, product_id, quantity, basket_id };
            let new_item = DbBasketProduct::create_basket_item(self.tenant_id, basket_product)?;
            Ok(new_item.id)
        }
    }

    pub fn get_products_from_basket(&self, basket_id: Uuid) -> Result<Vec<BasketProduct>, ShopsterError> {
        let db_items = DbBasketProduct::get_basket_items(self.tenant_id, basket_id)?;
        let items = db_items.iter().map(BasketProduct::from).collect();
        Ok(items)
    } 
    
    pub fn clear_basket(&self, basket_id: Uuid) -> Result<bool, ShopsterError> {
        let result = DbBasketProduct::delete_all_basket_items(self.tenant_id, basket_id)?;
        Ok(result > 0)
    }
}
