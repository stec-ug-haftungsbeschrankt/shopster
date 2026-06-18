//! Product catalog management.
//!
//! This module handles product CRUD operations, pricing, images, and tags.
//! Products are the core items available for purchase in the shop.
//!
//! # Example
//!
//! ```ignore
//! let products = shopster.products(tenant_id)?;
//! let product = products.insert(&Product { ... })?;
//! let all = products.get_all()?;
//! ```

use crate::error::ShopsterError;
use crate::postgresql::dbproduct::DbProduct;
use chrono::{NaiveDateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use uuid::Uuid;

/// Product pricing information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Price {
    /// Price amount in cents (to avoid floating point issues)
    pub amount: i64,
    /// Currency code (e.g., "EUR", "USD")
    pub currency: String
}

/// A product in the catalog.
///
/// Represents an item available for sale, including pricing, images, and metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Product {
    /// Unique product identifier
    pub id: i64,
    /// Internal article number/SKU
    pub article_number: String,
    /// Global Trade Item Number (barcode)
    pub gtin: String,
    /// Product display title
    pub title: String,
    /// Brief product description for listings
    pub short_description: String,
    /// Full product description
    pub description: String,
    /// Product tags/categories
    pub tags: Vec<String>,
    /// URL to the main product image
    pub image_url: String,
    /// URLs to additional product images
    pub additional_images: Vec<String>,
    /// Product pricing (if available)
    pub price: Option<Price>,
    /// Product weight in grams
    pub weight: i64,
    /// When the product was created
    pub created_at: NaiveDateTime,
    /// When the product was last updated
    pub updated_at: Option<NaiveDateTime>,
}

impl From<&DbProduct> for Product {
    fn from(db_product: &DbProduct) -> Self {
        let additional_images = db_product.additional_images.split('|').map(String::from).collect();
        let tags = db_product.tags.split('|').map(String::from).collect();

        Product {
            id: db_product.id,
            title: db_product.title.clone(),
            gtin: db_product.gtin.clone(),
            article_number: db_product.article_number.clone(),
            short_description: db_product.short_description.clone(),
            description: db_product.description.clone(),
            image_url: db_product.title_image.clone(),
            additional_images,
            tags,
            price: Some(Price {
                amount: db_product.price,
                currency: db_product.currency.clone()
            }),
            weight: db_product.weight as i64,
            created_at: db_product.created_at,
            updated_at: db_product.updated_at
        }
    }
}


impl TryFrom<&Product> for DbProduct {
    type Error = ShopsterError;

    fn try_from(product: &Product) -> Result<Self, ShopsterError> {
        // Price is required - products must have a price
        let price = product.price.as_ref()
            .ok_or_else(|| ShopsterError::InvalidOperationError(
                "Product price is required".to_string()
            ))?;

        Ok(DbProduct {
            id: product.id,
            title: product.title.clone(),
            gtin: product.gtin.clone(),
            article_number: product.article_number.clone(),
            short_description: product.short_description.clone(),
            description: product.description.clone(),
            price: price.amount,
            currency: price.currency.clone(),
            tags: product.tags.join("|"),
            title_image: product.image_url.clone(),
            additional_images: product.additional_images.join("|"),
            weight: product.weight as i32,
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc())
        })
    }
}


/// Handler for product management operations.
///
/// Provides CRUD operations and search capabilities for products within a tenant.
pub struct Products {
    /// The tenant ID for tenant isolation
    tenant_id: Uuid
}

impl Products {
    /// Creates a new Products handler for a tenant.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    pub fn new(tenant_id: Uuid) -> Self {
        Products { tenant_id }
    }
    
    /// Retrieves all products for the tenant.
    ///
    /// # Returns
    ///
    /// `Ok(Vec<Product>)` - All products
    /// `Err(ShopsterError)` - If database error occurs
    pub fn get_all(&self) -> Result<Vec<Product>, ShopsterError> {
        let db_products = DbProduct::get_all(self.tenant_id)?;
        let products = db_products.iter().map(Product::from).collect();
        Ok(products)
    }
    
    /// Retrieves a specific product by ID.
    ///
    /// # Arguments
    ///
    /// * `product_id` - The product's ID
    ///
    /// # Returns
    ///
    /// `Ok(Product)` - The product
    /// `Err(ShopsterError)` - If not found or database error
    pub fn get(&self, product_id: i64) -> Result<Product, ShopsterError> {
        let db_product = DbProduct::find(self.tenant_id, product_id)?;
        let product = Product::from(&db_product);
        Ok(product)
    }
    
    /// Creates a new product.
    ///
    /// # Arguments
    ///
    /// * `product` - The product to insert
    ///
    /// # Returns
    ///
    /// `Ok(Product)` - The created product
    /// `Err(ShopsterError)` - If creation fails
    pub fn insert(&self, product: &Product) -> Result<Product, ShopsterError> {
        if product.title.trim().is_empty() {
            return Err(ShopsterError::InvalidOperationError(
                "Product title cannot be empty".to_string(),
            ));
        }
        if let Some(price) = &product.price {
            if price.amount < 0 {
                return Err(ShopsterError::InvalidOperationError(
                    "Product price cannot be negative".to_string(),
                ));
            }
        }
        let db_product = DbProduct::try_from(product)?;
        let created_product = DbProduct::create(self.tenant_id, db_product)?;

        let reply = Product::from(&created_product);
        Ok(reply)
    }
    
    /// Updates an existing product.
    ///
    /// # Arguments
    ///
    /// * `product` - The product with updated data
    ///
    /// # Returns
    ///
    /// `Ok(Product)` - The updated product
    /// `Err(ShopsterError)` - If update fails
    pub fn update(&self, product: &Product) -> Result<Product, ShopsterError> {
        if product.title.trim().is_empty() {
            return Err(ShopsterError::InvalidOperationError(
                "Product title cannot be empty".to_string(),
            ));
        }
        if let Some(price) = &product.price {
            if price.amount < 0 {
                return Err(ShopsterError::InvalidOperationError(
                    "Product price cannot be negative".to_string(),
                ));
            }
        }
        let db_product = DbProduct::try_from(product)?;
        let updated_product = DbProduct::update(self.tenant_id, product.id, db_product)?;

        let reply = Product::from(&updated_product);
        Ok(reply)
    }
    
    pub fn remove(&self, product_id: i64) -> Result<bool, ShopsterError> {
        let result = DbProduct::delete(self.tenant_id, product_id)?;
        Ok(result > 0)
    }
}
