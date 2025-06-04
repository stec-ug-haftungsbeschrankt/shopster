use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::{postgresql::dbbasket::DbBasket, error::ShopsterError};
use crate::postgresql::dbbasket::DbBasketProduct;

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

pub struct BasketProductWithDetails {
    pub basket_product_id: i64,
    pub quantity: i64,
    pub product: crate::products::Product
}

pub struct Baskets {
    tenant_id: Uuid
}

impl Baskets {
    pub fn new(tenant_id: Uuid) -> Self {
        Baskets { tenant_id }
    }

    pub fn get_all_baskets(&self) -> Result<Vec<Basket>, ShopsterError> {
        let db_baskets = DbBasket::get_all(self.tenant_id)?;
        let mut baskets = Vec::new();

        for db_basket in db_baskets {
            let mut basket = Basket::from(&db_basket);
            // Optional: Lade direkt die Produkte für jeden Warenkorb
            basket.products = self.get_products_from_basket(basket.id)?;
            baskets.push(basket);
        }

        Ok(baskets)
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

    pub fn add_product_to_basket(&self, basket_id: Uuid, product_id: i64, quantity: i64) -> Result<i64, ShopsterError> {
        let items = DbBasketProduct::get_basket_items(self.tenant_id, basket_id)?;

        if let Some(mut item) = items.into_iter().find(|x| x.product_id == product_id) {
            item.quantity = quantity;
            let updated_item = DbBasketProduct::update_basket_item(self.tenant_id, item.id, item)?;
            Ok(updated_item.id)
        } else {
            let basket_product = DbBasketProduct { id: 0, product_id, quantity, basket_id };
            let new_item = DbBasketProduct::create_basket_item(self.tenant_id, basket_product)?;
            Ok(new_item.id)
        }
    }

    pub fn update_product_quantity(&self, basket_id: Uuid, basket_product_id: i64, quantity: i64) -> Result<BasketProduct, ShopsterError> {
        // Finde das Basket-Produkt
        let basket_product = DbBasketProduct::find_basket_item(self.tenant_id, basket_product_id)?;

        // Überprüfe, ob es zum richtigen Warenkorb gehört
        if basket_product.basket_id != basket_id {
            return Err(ShopsterError::InvalidOperationError("Produkt gehört nicht zu diesem Warenkorb".to_string()));
        }

        // Aktualisiere die Menge
        let mut updated_product = basket_product;
        updated_product.quantity = quantity;

        let result = DbBasketProduct::update_basket_item(self.tenant_id, basket_product_id, updated_product)?;
        Ok(BasketProduct::from(&result))
    }

    pub fn remove_product_from_basket(&self, basket_id: Uuid, basket_product_id: i64) -> Result<bool, ShopsterError> {
        // Überprüfen, ob das Produkt im angegebenen Warenkorb ist
        let basket_product = DbBasketProduct::find_basket_item(self.tenant_id, basket_product_id)?;

        if basket_product.basket_id != basket_id {
            return Err(ShopsterError::InvalidOperationError("Produkt gehört nicht zu diesem Warenkorb".to_string()));
        }

        // Produkt aus dem Warenkorb entfernen
        let result = DbBasketProduct::delete_basket_item(self.tenant_id, basket_product_id)?;
        Ok(result > 0)
    }



    pub fn get_products_from_basket(&self, basket_id: Uuid) -> Result<Vec<BasketProduct>, ShopsterError> {
        let db_items = DbBasketProduct::get_basket_items(self.tenant_id, basket_id)?;
        let items = db_items.iter().map(BasketProduct::from).collect();
        Ok(items)
    }
    
    pub fn get_products_with_details(&self, basket_id: Uuid) -> Result<Vec<BasketProductWithDetails>, ShopsterError> {
        // Hole alle Produkte im Warenkorb
        let basket_products = DbBasketProduct::get_basket_items(self.tenant_id, basket_id)?;
        let mut result = Vec::new();

        // Erstelle eine Instanz der Products-Struktur
        let products = crate::products::Products::new(self.tenant_id);

        // Für jedes Produkt im Warenkorb die vollständigen Details laden
        for basket_product in basket_products {
            let product = products.get(basket_product.product_id)?;

            result.push(BasketProductWithDetails {
                basket_product_id: basket_product.id,
                quantity: basket_product.quantity,
                product
            });
        }

        Ok(result)
    }

    pub fn clear_basket(&self, basket_id: Uuid) -> Result<bool, ShopsterError> {
        DbBasket::find(self.tenant_id, basket_id)?;
        let result = DbBasketProduct::delete_all_basket_items(self.tenant_id, basket_id)?;
        Ok(result > 0)
    }
    
    pub fn calculate_basket_total(&self, basket_id: Uuid) -> Result<(i64, String), ShopsterError> {
        let products_with_details = self.get_products_with_details(basket_id)?;

        if products_with_details.is_empty() {
            return Ok((0, "EUR".to_string())); // Standardwährung, falls der Warenkorb leer ist
        }

        // Annahme: Alle Produkte haben dieselbe Währung
        let currency = products_with_details[0].product.price.as_ref()
            .map(|p| p.currency.clone())
            .unwrap_or_else(|| "EUR".to_string());

        // Berechne den Gesamtpreis
        let total: i64 = products_with_details.iter()
            .filter_map(|bp| {
                bp.product.price.as_ref().map(|price| {
                    price.amount * bp.quantity
                })
            })
            .sum();

        Ok((total, currency))
    }


    pub fn merge_baskets(&self, source_basket_id: Uuid, target_basket_id: Uuid) -> Result<(), ShopsterError> {
        // Lade die Produkte aus dem Quell-Warenkorb
        let source_products = self.get_products_from_basket(source_basket_id)?;

        // Lade die Produkte aus dem Ziel-Warenkorb
        let target_products = self.get_products_from_basket(target_basket_id)?;

        // Füge alle Produkte aus dem Quell-Warenkorb zum Ziel-Warenkorb hinzu
        for source_product in source_products {
            // Prüfe, ob das Produkt bereits im Ziel-Warenkorb vorhanden ist
            let existing_product = target_products.iter()
                .find(|p| p.product_id == source_product.product_id);

            match existing_product {
                Some(product) => {
                    // Wenn das Produkt bereits im Ziel-Warenkorb existiert, aktualisiere die Menge
                    let new_quantity = product.quantity + source_product.quantity;
                    self.update_product_quantity(target_basket_id, product.id, new_quantity)?;
                },
                None => {
                    // Wenn das Produkt noch nicht im Ziel-Warenkorb existiert, füge es hinzu
                    self.add_product_to_basket(target_basket_id, source_product.product_id, source_product.quantity)?;
                }
            }
        }

        // Optional: Lösche den Quell-Warenkorb
        self.delete_basket(source_basket_id)?;

        Ok(())
    }
}
