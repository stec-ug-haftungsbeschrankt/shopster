//! Shopping basket management for customers.
//!
//! This module provides functionality for managing shopping baskets, including:
//! - Creating and deleting baskets
//! - Adding and removing products from baskets
//! - Calculating basket totals
//! - Merging baskets
//!
//! # Example
//!
//! ```ignore
//! let baskets = shopster.baskets(tenant_id)?;
//! let basket_id = baskets.add_basket()?;
//! baskets.add_product_to_basket(basket_id, product_id, 2)?;
//! let basket = baskets.get_basket(basket_id)?;
//! let (total, currency) = baskets.calculate_basket_total(basket_id)?;
//! ```

use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::{postgresql::dbbasket::DbBasket, error::ShopsterError};
use crate::postgresql::dbbasket::DbBasketProduct;

/// A product within a shopping basket.
///
/// Represents a product line item in a basket with quantity information.
#[derive(Clone)]
pub struct BasketProduct {
    /// Unique identifier for this basket-product entry
    pub id: i64,
    /// The product's ID
    pub product_id: i64,
    /// Quantity of the product in the basket
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
///
/// Contains the ID, product list, and timestamps for a customer's shopping cart.
pub struct Basket {
    /// Unique identifier for the basket
    pub id: Uuid,
    /// Products in this basket
    pub products: Vec<BasketProduct>,
    /// When the basket was created
    pub created_at: NaiveDateTime,
    /// When the basket was last updated (if applicable)
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
///
/// Used when you need to display complete product information alongside
/// basket quantity information.
pub struct BasketProductWithDetails {
    /// The ID of the basket-product entry
    pub basket_product_id: i64,
    /// Quantity of this product in the basket
    pub quantity: i64,
    /// Full product details
    pub product: crate::products::Product
}

/// Handler for shopping basket operations.
///
/// Manages all basket-related operations for a tenant, including CRUD operations
/// on baskets and the products within them.
pub struct Baskets {
    /// The tenant ID for tenant isolation
    tenant_id: Uuid
}

impl Baskets {
    /// Creates a new Baskets handler for a tenant.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    pub fn new(tenant_id: Uuid) -> Self {
        Baskets { tenant_id }
    }

    /// Retrieves all baskets for the tenant.
    ///
    /// # Returns
    ///
    /// `Ok(Vec<Basket>)` - All baskets with their products loaded
    /// `Err(ShopsterError)` - If the database operation fails
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

    /// Gets a specific basket by ID.
    ///
    /// # Arguments
    ///
    /// * `basket_id` - The basket's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Basket)` - The basket with products loaded
    /// `Err(ShopsterError)` - If basket not found or database operation fails
    pub fn get_basket(&self, basket_id: Uuid) -> Result<Basket, ShopsterError> {
        let db_basket = DbBasket::find(self.tenant_id, basket_id)?;
        let mut basket = Basket::from(&db_basket);
        basket.products = self.get_products_from_basket(basket.id)?;
        Ok(basket)
    }
    
    /// Creates a new empty basket.
    ///
    /// # Returns
    ///
    /// `Ok(Uuid)` - The ID of the newly created basket
    /// `Err(ShopsterError)` - If creation fails
    pub fn add_basket(&self) -> Result<Uuid, ShopsterError> {
        let db_basket = DbBasket::create(self.tenant_id)?;
        Ok(db_basket.id)
    }
    
    /// Deletes a basket and all its products.
    ///
    /// # Arguments
    ///
    /// * `basket_id` - The basket's UUID
    ///
    /// # Returns
    ///
    /// `Ok(bool)` - True if basket was deleted, false if not found
    /// `Err(ShopsterError)` - If operation fails
    pub fn delete_basket(&self, basket_id: Uuid) -> Result<bool, ShopsterError> {
        self.clear_basket(basket_id)?;
        let deleted_baskets = DbBasket::delete(self.tenant_id, basket_id)?;
        Ok(deleted_baskets > 0)
    }

    /// Adds a product to a basket or updates quantity if already present.
    ///
    /// # Arguments
    ///
    /// * `basket_id` - The basket's UUID
    /// * `product_id` - The product to add
    /// * `quantity` - Quantity to add (replaces if product already exists)
    ///
    /// # Returns
    ///
    /// `Ok(i64)` - The ID of the basket-product entry
    /// `Err(ShopsterError)` - If operation fails
    pub fn add_product_to_basket(&self, basket_id: Uuid, product_id: i64, quantity: i64) -> Result<i64, ShopsterError> {
        if quantity <= 0 {
            return Err(ShopsterError::InvalidOperationError(
                "Quantity must be positive".to_string(),
            ));
        }
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
        if quantity <= 0 {
            return Err(ShopsterError::InvalidOperationError(
                "Quantity must be positive".to_string(),
            ));
        }
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
    
    /// Gets all basket products with full product details.
    ///
    /// Loads complete product information for each item in the basket,
    /// useful for rendering checkout pages.
    ///
    /// # Arguments
    ///
    /// * `basket_id` - The basket's UUID
    ///
    /// # Returns
    ///
    /// `Ok(Vec<BasketProductWithDetails>)` - Products with full details
    /// `Err(ShopsterError)` - If operation fails
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

    /// Removes all products from a basket without deleting the basket itself.
    ///
    /// # Arguments
    ///
    /// * `basket_id` - The basket's UUID
    ///
    /// # Returns
    ///
    /// `Ok(bool)` - True if any items were cleared, false if basket was empty
    /// `Err(ShopsterError)` - If operation fails
    pub fn clear_basket(&self, basket_id: Uuid) -> Result<bool, ShopsterError> {
        DbBasket::find(self.tenant_id, basket_id)?;
        let result = DbBasketProduct::delete_all_basket_items(self.tenant_id, basket_id)?;
        Ok(result > 0)
    }
    
    /// Calculates the total cost of all products in a basket.
    ///
    /// Returns both the total amount (in cents) and the currency code.
    ///
    /// # Arguments
    ///
    /// * `basket_id` - The basket's UUID
    ///
    /// # Returns
    ///
    /// `Ok((i64, String))` - Tuple of (amount in cents, currency code)
    /// `Err(ShopsterError)` - If operation fails
    pub fn calculate_basket_total(&self, basket_id: Uuid) -> Result<(i64, String), ShopsterError> {
        let products_with_details = self.get_products_with_details(basket_id)?;

        if products_with_details.is_empty() {
            return Ok((0, "EUR".to_string())); // Standardwährung, falls der Warenkorb leer ist
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


    /// Merges the contents of a source basket into a target basket.
    ///
    /// All products from the source basket are moved to the target basket.
    /// If a product already exists in the target, quantities are summed.
    /// The source basket is deleted after merging.
    ///
    /// # Arguments
    ///
    /// * `source_basket_id` - The basket to merge from
    /// * `target_basket_id` - The basket to merge into
    ///
    /// # Returns
    ///
    /// `Ok(())` - If merge succeeds
    /// `Err(ShopsterError)` - If operation fails
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
