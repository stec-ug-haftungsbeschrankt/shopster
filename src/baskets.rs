//! Shopping basket management for customers.

use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel_async::AsyncConnection;

use crate::aquire_pool;
use crate::{postgresql::dbbasket::DbBasket, error::ShopsterError};
use crate::postgresql::dbbasket::DbBasketProduct;

/// A product within a shopping basket.
#[derive(Clone)]
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

/// A shopping basket.
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

/// A basket product with full product details.
pub struct BasketProductWithDetails {
    pub basket_product_id: i64,
    pub quantity: i64,
    pub product: crate::products::Product
}

/// Handler for shopping basket operations.
pub struct Baskets {
    tenant_id: Uuid
}

impl Baskets {
    pub fn new(tenant_id: Uuid) -> Self {
        Baskets { tenant_id }
    }

    pub async fn get_all_baskets(&self) -> Result<Vec<Basket>, ShopsterError> {
        let db_baskets = DbBasket::get_all(self.tenant_id).await?;
        let mut baskets = Vec::new();

        for db_basket in db_baskets {
            let mut basket = Basket::from(&db_basket);
            basket.products = self.get_products_from_basket(basket.id).await?;
            baskets.push(basket);
        }

        Ok(baskets)
    }

    pub async fn get_basket(&self, basket_id: Uuid) -> Result<Basket, ShopsterError> {
        let db_basket = DbBasket::find(self.tenant_id, basket_id).await?;
        let mut basket = Basket::from(&db_basket);
        basket.products = self.get_products_from_basket(basket.id).await?;
        Ok(basket)
    }

    pub async fn add_basket(&self) -> Result<Uuid, ShopsterError> {
        let db_basket = DbBasket::create(self.tenant_id).await?;
        Ok(db_basket.id)
    }

    pub async fn delete_basket(&self, basket_id: Uuid) -> Result<bool, ShopsterError> {
        self.clear_basket(basket_id).await?;
        let deleted_baskets = DbBasket::delete(self.tenant_id, basket_id).await?;
        Ok(deleted_baskets > 0)
    }

    pub async fn add_product_to_basket(&self, basket_id: Uuid, product_id: i64, quantity: i64) -> Result<i64, ShopsterError> {
        if quantity <= 0 {
            return Err(ShopsterError::InvalidOperationError(
                "Quantity must be positive".to_string(),
            ));
        }
        let items = DbBasketProduct::get_basket_items(self.tenant_id, basket_id).await?;

        if let Some(mut item) = items.into_iter().find(|x| x.product_id == product_id) {
            item.quantity = quantity;
            let updated_item = DbBasketProduct::update_basket_item(self.tenant_id, item.id, item).await?;
            Ok(updated_item.id)
        } else {
            let basket_product = DbBasketProduct { id: 0, product_id, quantity, basket_id };
            let new_item = DbBasketProduct::create_basket_item(self.tenant_id, basket_product).await?;
            Ok(new_item.id)
        }
    }

    pub async fn update_product_quantity(&self, basket_id: Uuid, basket_product_id: i64, quantity: i64) -> Result<BasketProduct, ShopsterError> {
        if quantity <= 0 {
            return Err(ShopsterError::InvalidOperationError(
                "Quantity must be positive".to_string(),
            ));
        }
        let basket_product = DbBasketProduct::find_basket_item(self.tenant_id, basket_product_id).await?;

        if basket_product.basket_id != basket_id {
            return Err(ShopsterError::InvalidOperationError("Produkt gehört nicht zu diesem Warenkorb".to_string()));
        }

        let mut updated_product = basket_product;
        updated_product.quantity = quantity;

        let result = DbBasketProduct::update_basket_item(self.tenant_id, basket_product_id, updated_product).await?;
        Ok(BasketProduct::from(&result))
    }

    pub async fn remove_product_from_basket(&self, basket_id: Uuid, basket_product_id: i64) -> Result<bool, ShopsterError> {
        let basket_product = DbBasketProduct::find_basket_item(self.tenant_id, basket_product_id).await?;

        if basket_product.basket_id != basket_id {
            return Err(ShopsterError::InvalidOperationError("Produkt gehört nicht zu diesem Warenkorb".to_string()));
        }

        let result = DbBasketProduct::delete_basket_item(self.tenant_id, basket_product_id).await?;
        Ok(result > 0)
    }

    pub async fn get_products_from_basket(&self, basket_id: Uuid) -> Result<Vec<BasketProduct>, ShopsterError> {
        let db_items = DbBasketProduct::get_basket_items(self.tenant_id, basket_id).await?;
        let items = db_items.iter().map(BasketProduct::from).collect();
        Ok(items)
    }

    pub async fn get_products_with_details(&self, basket_id: Uuid) -> Result<Vec<BasketProductWithDetails>, ShopsterError> {
        let basket_products = DbBasketProduct::get_basket_items(self.tenant_id, basket_id).await?;
        let mut result = Vec::new();

        let products = crate::products::Products::new(self.tenant_id);

        for basket_product in basket_products {
            let product = products.get(basket_product.product_id).await?;

            result.push(BasketProductWithDetails {
                basket_product_id: basket_product.id,
                quantity: basket_product.quantity,
                product
            });
        }

        Ok(result)
    }

    pub async fn clear_basket(&self, basket_id: Uuid) -> Result<bool, ShopsterError> {
        DbBasket::find(self.tenant_id, basket_id).await?;
        let result = DbBasketProduct::delete_all_basket_items(self.tenant_id, basket_id).await?;
        Ok(result > 0)
    }

    pub async fn calculate_basket_total(&self, basket_id: Uuid) -> Result<(i64, String), ShopsterError> {
        let products_with_details = self.get_products_with_details(basket_id).await?;

        if products_with_details.is_empty() {
            return Ok((0, "EUR".to_string()));
        }

        let first_currency = products_with_details[0]
            .product
            .price
            .as_ref()
            .ok_or_else(|| {
                ShopsterError::InvalidOperationError(
                    "Cannot calculate total: product has no price".to_string(),
                )
            })?
            .currency
            .clone();

        let mut total: i64 = 0;
        for bp in &products_with_details {
            let price = bp.product.price.as_ref().ok_or_else(|| {
                ShopsterError::InvalidOperationError(format!(
                    "Cannot calculate total: product {} has no price",
                    bp.product.id
                ))
            })?;
            if price.currency != first_currency {
                return Err(ShopsterError::InvalidOperationError(format!(
                    "Mixed currencies in basket: {} and {}",
                    first_currency, price.currency
                )));
            }
            total += price.amount * bp.quantity;
        }

        Ok((total, first_currency))
    }

    pub async fn merge_baskets(&self, source_basket_id: Uuid, target_basket_id: Uuid) -> Result<(), ShopsterError> {
        let pool = aquire_pool(self.tenant_id).await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        conn.transaction(async |conn| {
            let source_products = DbBasketProduct::get_basket_items_conn(conn, source_basket_id).await?;
            let target_products = DbBasketProduct::get_basket_items_conn(conn, target_basket_id).await?;

            for source_product in source_products {
                let existing_product = target_products.iter()
                    .find(|p| p.product_id == source_product.product_id);

                match existing_product {
                    Some(product) => {
                        let new_quantity = product.quantity + source_product.quantity;
                        let updated = DbBasketProduct {
                            id: product.id,
                            product_id: product.product_id,
                            quantity: new_quantity,
                            basket_id: target_basket_id,
                        };
                        DbBasketProduct::update_basket_item_conn(conn, product.id, updated).await?;
                    },
                    None => {
                        let new_item = DbBasketProduct {
                            id: 0,
                            product_id: source_product.product_id,
                            quantity: source_product.quantity,
                            basket_id: target_basket_id,
                        };
                        DbBasketProduct::create_basket_item_conn(conn, new_item).await?;
                    }
                }
            }

            DbBasketProduct::delete_all_basket_items_conn(conn, source_basket_id).await?;
            DbBasket::delete_conn(conn, source_basket_id).await?;

            Ok(())
        }).await
    }
}
